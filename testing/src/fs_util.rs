use crate::crash_test::Error;
use crate::deep_project::Script;
use async_stream::try_stream;
use futures::stream::Stream;
use futures::try_join;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile, TempDir};
use tokio::{fs, io};

pub fn new_tmp_script_file(
    script_type: Script,
    content: &str,
) -> Result<NamedTempFile, std::io::Error> {
    let mut file = Builder::new()
        .suffix(script_type.file_extension())
        .tempfile()?;
    file.write(&content.as_bytes())?;
    Ok(file)
}
pub async fn cp_files(files: &Vec<PathBuf>, dir: &TempDir) -> Result<(), Error> {
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
