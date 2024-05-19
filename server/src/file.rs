use std::{
    fmt::{self, Display},
    io::ErrorKind,
    num::ParseIntError,
    path::PathBuf,
    pin::pin,
    str::FromStr,
};

use anyhow::Context;
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{de::Visitor, Deserialize, Serialize};
use sqlx::PgPool;
use tokio::{fs, io::AsyncRead};

/// Information for a file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileInfo {
    pub file_id: FileId,
    pub file_name: String,
    #[serde(with = "ts_milliseconds")]
    pub upload_date: DateTime<Utc>,
    pub file_size: u64,
}

/// A unique identifier for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, sqlx::Type)]
#[sqlx(transparent)]
pub struct FileId(i64);

impl FileId {
    /// Generate a new random file id.
    pub fn random() -> Self {
        Self(rand::random())
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0 as u64)
    }
}

impl FromStr for FileId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(|v| v as i64).map(FileId)
    }
}

impl<'de> Deserialize<'de> for FileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = FileId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a hex-encoded FileId")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FileId::from_str(v).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"a 64-bit hex-encoded integer",
                    )
                })
            }
        }
        deserializer.deserialize_str(V)
    }
}

impl Serialize for FileId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

/// Persists stored files.
pub struct FileDb {
    db: PgPool,
    store: FileStore,
}

impl FileDb {
    pub fn new(db: PgPool, store: FileStore) -> Self {
        Self { db, store }
    }

    pub fn file_store(&self) -> &FileStore {
        &self.store
    }

    pub fn db(&self) -> &PgPool {
        &self.db
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
                r#"select exists(select * from files where file_id = $1)"#,
                id as FileId
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
            file_id as FileId,
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
            file_id as FileId
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
            file_id as FileId,
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

    /// Permanently remove a file from the file store.
    pub async fn remove(&self, id: FileId) -> anyhow::Result<()> {
        let path = self.root.join(id.to_string());
        fs::remove_file(path)
            .await
            .context("could not remove file")?;
        Ok(())
    }
}
