use std::{io::ErrorKind, path::PathBuf, pin::pin};

use anyhow::Context;
use sqlx::PgPool;
use tokio::{fs, io::AsyncRead};

use crate::file::{FileId, FileInfo};

/// Persists stored files.
pub struct FileDb {
    db: PgPool,
    store: FileStore,
}

impl FileDb {
    pub fn new(db: PgPool, store: FileStore) -> Self {
        Self { db, store }
    }

    pub async fn create(
        &self,
        file_name: String,
        content: impl AsyncRead,
    ) -> anyhow::Result<FileInfo> {
        let mut tx = self.db.begin().await?;

        let file_id = loop {
            let id = FileId::random();

            let exists = sqlx::query!(
                "select exists(select * from files where file_id = $1)",
                id.raw() as i64
            )
            .fetch_one(&mut *tx)
            .await?
            .exists
            .unwrap_or(false);

            if !exists {
                break id;
            }
        };

        let file_size = self.store.put(file_id, content).await?;
        let file_size_db: i64 = file_size.try_into().context("invalid file size")?;

        // todo: correctly store who uploaded the file
        let row = sqlx::query!(
            r#"
            insert into files(file_id, file_name, file_size, upload_date, uploader_id)
                values(
                    $1,
                    $2,
                    $3,
                    now(),
                    (select user_id from users order by created_at asc limit 1)
                )
                returning upload_date
            "#,
            file_id.raw() as i64,
            file_name,
            file_size_db,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(FileInfo {
            file_id,
            file_name,
            upload_date: row.upload_date,
            file_size,
        })
    }

    pub async fn file_info(&self, file_id: FileId) -> anyhow::Result<Option<FileInfo>> {
        Ok(sqlx::query!(
            "select file_name, upload_date, file_size from files where file_id=$1",
            file_id.raw() as i64
        )
        .fetch_optional(&self.db)
        .await?
        .map(|row| FileInfo {
            file_id,
            file_name: row.file_name,
            upload_date: row.upload_date,
            file_size: row.file_size.try_into().unwrap_or_default(),
        }))
    }

    pub async fn content(
        &self,
        file_id: FileId,
        file_name: &str,
    ) -> anyhow::Result<Option<impl AsyncRead>> {
        let exists = sqlx::query!(
            "select exists(select * from files where file_id=$1 and file_name=$2)",
            file_id.raw() as i64,
            file_name
        )
        .fetch_one(&self.db)
        .await?
        .exists
        .unwrap_or(false);

        if !exists {
            return Ok(None);
        }

        self.store.get(file_id).await
    }
}

/// Responsible for keeping track of file contents.
pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    pub async fn new(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = root.into();
        fs::create_dir_all(root.as_path())
            .await
            .context("could not create root directory")?;
        Ok(Self { root })
    }

    pub async fn get(&self, id: FileId) -> anyhow::Result<Option<impl AsyncRead>> {
        let path = self.root.join(id.to_string());
        let file = match fs::File::open(path).await {
            Ok(file) => file,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        Ok(Some(file))
    }

    /// Writes a file to the file store, returning how many bytes were written.
    pub async fn put(&self, id: FileId, data: impl AsyncRead) -> anyhow::Result<u64> {
        let path = self.root.join(id.to_string());

        let mut file = fs::File::create(path)
            .await
            .context("could not create file")?;

        let mut pinned_data = pin!(data);
        let size = tokio::io::copy(&mut pinned_data, &mut file)
            .await
            .context("while writing to file")?;
        Ok(size)
    }
}
