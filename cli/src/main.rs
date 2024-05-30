mod auth;
mod byte_stream;
mod config;

use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context};
use arboard::Clipboard;
use auth::{create_session, get_private_key};
use clap::Parser;
use config::Config;
use flate2::{write::GzEncoder, Compression};
use indicatif::ProgressBar;
use reqwest::{Body, StatusCode};
use ssh_key::PrivateKey;
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
    let cli_theme = dialoguer::theme::ColorfulTheme::default();

    let config = Config::read().await?;
    // let config = Config::read().await.context("could not read config")?;
    let command = Command::parse();

    if command.files.is_empty() {
        return Err(anyhow!("please provide at least one file to share"));
    }

    // Determine if we are uploading a single non-directory file. This is used to determine whether
    // to archieve the input files or not.
    let is_single_file = if command.files.len() == 1 {
        let is_dir = match fs::metadata(&command.files[0])
            .await
            .context("trying to read file")?
            .file_type()
        {
            e if e.is_dir() => true,
            e if e.is_file() => false,
            other => return Err(anyhow!("unsupported file type: {other:?}")),
        };

        !is_dir
    } else {
        false
    };

    // Determine the name of the file to be uploaded.
    let mut file_name = if command.files.len() == 1 {
        let mut file_name = command.files[0]
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("file")
            .to_string();
        if !is_single_file {
            file_name += ".tar.gz";
        }
        file_name
    } else {
        "files.tar.gz".to_string()
    };

    let default_settings = dialoguer::Confirm::with_theme(&cli_theme)
        .with_prompt("Do you want to use the default settings?")
        .interact()?;

    if !default_settings {
        file_name = dialoguer::Input::with_theme(&cli_theme)
            .with_prompt("What name should the uploaded file have?")
            .default(file_name)
            .interact_text()?;
    }

    let (mut writer, reader) = byte_stream::byte_stream(4096);

    // Write all the files into a compressed tar archive.
    let write_task = task::spawn_blocking(move || -> anyhow::Result<()> {
        use std::fs;

        if !is_single_file {
            let enc = GzEncoder::new(writer, Compression::default());
            let mut archive = tar::Builder::new(enc);

            for file in command.files {
                let is_dir = std::fs::metadata(&file)
                    .context("could not get metadata for file")?
                    .is_dir();

                let file_name = Path::new(
                    file.file_name()
                        .and_then(|v| v.to_str())
                        .unwrap_or("unnamed"),
                );
                if is_dir {
                    archive
                        .append_dir_all(file_name, file.clone())
                        .context("could not add directory to archive")?;
                } else {
                    archive
                        .append_path_with_name(file.clone(), file_name)
                        .context("could not add directory to archive")?;
                }
            }
            archive.into_inner()?.try_finish()?;
        } else {
            let mut reader =
                fs::File::open(&command.files[0]).context("could not open file for reading")?;
            io::copy(&mut reader, &mut writer)?;
        }
        Ok(())
    });

    let mut client = reqwest::ClientBuilder::new()
        // Sessions are stored in the cookie jar.
        .cookie_store(true)
        // Sessions rely on HTTPS being active.
        .https_only({
            #[cfg(debug_assertions)]
            let x = false;
            #[cfg(not(debug_assertions))]
            let x = true;
            x
        })
        .build()
        .context("could not build http client")?;

    let priv_key = match &config.ssh_key {
        // If the user has configured a path to an SSH key, use that.
        Some(ssh_key_path) => {
            let priv_key_data = fs::read_to_string(Path::new(ssh_key_path))
                .await
                .context("could not read ssh key file")?;
            PrivateKey::from_openssh(priv_key_data)
                .context("could not parse openssh private key")?
        }
        // Otherwise, try to find the private key.
        None => match get_private_key().await {
            Some(priv_key) => priv_key,
            None => {
                return Err(anyhow!("could not find private ssh key"));
            }
        },
    };

    create_session(&mut client, &config.host, &config.username, priv_key)
        .await
        .context("could not create session")?;

    let progress = ProgressBar::new_spinner().with_message("Uploading file");
    progress.enable_steady_tick(Duration::from_millis(100));

    // Upload the file while it is being written.
    let upload_task: JoinHandle<anyhow::Result<_>> = tokio::spawn(async move {
        let resp = client
            .post(format!("{}/api/files", config.host))
            .header("file_name", file_name)
            .body(Body::wrap_stream(ReaderStream::new(reader)))
            .send()
            .await
            .context("could make request")?;
        if resp.status() != StatusCode::OK {
            eprintln!("Server returned with status code {}", resp.status());
            eprintln!(
                "Error body: {}",
                resp.text().await.context("could not error response")?,
            );
            return Ok(());
        }
        let url = resp.text().await.context("could not read response")?;
        progress.finish_with_message("File has been uploaded!");
        println!(
            "Use the following link to share it: {} (copied to clipboard)",
            &url
        );

        let mut clipboard = Clipboard::new().context("failed to get clipboard")?;
        #[allow(unused_mut)]
        let mut set = clipboard.set();
        #[cfg(target_os = "linux")]
        {
            use arboard::SetExtLinux;
            println!("Detected you are on linux. This application will keep running to persist the clipboard.");
            set = set.wait();
        }
        set.text(url).context("could not write to clipboard")?;
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
