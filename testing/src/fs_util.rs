use crate::config::fix_win_ln;
use crate::crash_test::Error;
use async_stream::try_stream;
use futures::stream::Stream;
use futures::try_join;
use grpc_api::Script;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile, TempDir};
use tokio::{fs, io};

const TEMP_DIR: &str = "/tmp/scripts";

pub fn new_tmp_dir() -> Result<TempDir, std::io::Error> {
    Builder::new().tempdir_in(TEMP_DIR)
}
pub async fn copy_items_include(files: &[PathBuf]) -> Result<TempDir, Error> {
    let dir = new_tmp_dir()?;
    cp_files(&files, &dir).await?;
    Ok(dir)
}

pub fn new_tmp_script_file(
    script_type: Script,
    content: &str,
) -> Result<NamedTempFile, std::io::Error> {
    #[cfg(target_family = "unix")]
    let mut file = Builder::new()
        .suffix(script_type.file_extension())
        .tempfile_in(TEMP_DIR)?;
    #[cfg(target_family = "windows")]
    let mut file = Builder::new()
        .suffix(script_type.file_extension())
        .tempfile()?;
    let bytes = if script_type != Script::PowerShell
        && script_type != Script::Batch
        && content.contains(r"\r\n")
    {
        fix_win_ln(&content).as_bytes().to_owned()
    } else {
        content.as_bytes().to_owned()
    };
    file.write(&bytes)?;
    Ok(file)
}
pub async fn cp_files(files: &[PathBuf], dir: &TempDir) -> Result<(), Error> {
    for path in files {
        // TODO fix unwrap
        let file_name = path.file_name().unwrap();
        let dest_path = dir.path().join(&file_name);
        let _ = copy(&path, &dest_path).await?;
    }
    Ok(())
}

pub fn ls_dir_content(root: PathBuf) -> impl Stream<Item = Result<PathBuf, Error>> {
    try_stream! {
        if !root.is_file() {
            let mut stack = vec![root.to_path_buf()];
            while let Some(dir) = stack.pop() {
                let mut dir_entry = fs::read_dir(&dir).await.map_err(|_e| Error::ListDir(dir))?;
                while let Ok(Some(entry)) = dir_entry.next_entry().await {
                    if entry.file_type().await?.is_dir() {
                        stack.push(entry.path());
                    }
                    yield entry.path();
                }
            }
        }
    }
}

async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64, std::io::Error> {
    let (from, to) = try_join!(fs::File::open(from), fs::File::create(to))?;
    let (mut from, mut to) = (io::BufReader::new(from), io::BufWriter::new(to));
    io::copy(&mut from, &mut to).await
}
