use std::{io::ErrorKind, path::PathBuf, pin::pin};

use anyhow::Context;
use beacon_panel_shared::file::FileId;
use tokio::{fs, io::AsyncRead};

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

    pub async fn put(&self, id: FileId, data: impl AsyncRead) -> anyhow::Result<()> {
        let path = self.root.join(id.to_string());

        let mut file = fs::File::create(path)
            .await
            .context("could not create file")?;

        let mut pinned_data = pin!(data);
        tokio::io::copy(&mut pinned_data, &mut file)
            .await
            .context("while writing to file")?;
        Ok(())
    }
}
