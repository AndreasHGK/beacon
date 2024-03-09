use std::path::PathBuf;

use clap::Parser;

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
async fn main() {
    let _command = Command::parse();

    todo!()
}
