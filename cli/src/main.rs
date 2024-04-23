mod byte_stream;

use std::{io, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use flate2::{write::GzEncoder, Compression};
use reqwest::Body;
use tokio::{
    fs,
    task::{self, JoinHandle},
};
use tokio_util::io::ReaderStream;

/// Upload files and generate a link to share them.
#[derive(Parser)]
struct Command {
    /// One or more files to share.
    ///
    /// When multiple files are provided, they will be combined into an archive. When a single
    /// directory is provided, its contents will be combined into an archive.
    files: Vec<PathBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let command = Command::parse();

    if command.files.is_empty() {
        return Err(anyhow!("please provide at least one file to share"));
    }

    if command.files.len() > 1 {
        return Err(anyhow!(
            "uploading more than one file is currently not supported"
        ));
    }

    // todo: support uploading of multiple files at once
    let file = command.files[0].clone();
    let is_dir = match fs::metadata(&file)
        .await
        .context("trying to read file")?
        .file_type()
    {
        e if e.is_dir() => true,
        e if e.is_file() => false,
        other => return Err(anyhow!("unsupported file type: {other:?}")),
    };

    // Determine the name of the file to be uploaded.
    let file_name = if command.files.len() == 1 {
        let mut file_name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("file")
            .to_string();
        if is_dir {
            file_name += ".tar.gz";
        }
        file_name
    } else {
        "files.tar.gz".to_string()
    };

    let (mut writer, reader) = byte_stream::byte_stream(4096);

    // Write all the files into a compressed tar archive.
    let write_task = task::spawn_blocking(move || -> anyhow::Result<()> {
        use std::fs;

        if is_dir {
            let enc = GzEncoder::new(writer, Compression::default());
            let mut archive = tar::Builder::new(enc);
            archive
                .append_dir_all(".", file.clone())
                .context("could not add directory to archive")?;
            archive.into_inner()?.try_finish()?;
        } else {
            let mut reader = fs::File::open(file).context("could not open file for reading")?;
            io::copy(&mut reader, &mut writer)?;
        }
        Ok(())
    });

    // Upload the file while it is being written.
    let upload_task: JoinHandle<anyhow::Result<_>> = tokio::spawn(async {
        println!("Uploading file...");
        let resp = reqwest::Client::new()
            .post("http://127.0.0.1:3000/files")
            .header("file_name", file_name)
            .body(Body::wrap_stream(ReaderStream::new(reader)))
            .send()
            .await
            .context("could make request")?;
        println!(
            "File was uploaded successfully!\nUse the following link to share it: {}",
            resp.text().await.context("could not read response")?
        );
        Ok(())
    });

    write_task
        .await
        .context("writer task crashed")?
        .context("while writing files")?;
    upload_task
        .await
        .context("upload task crashed")?
        .context("could not upload file")?;

    Ok(())
}
