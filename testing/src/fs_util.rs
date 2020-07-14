//! FS utils for creating temporary files folder and doing FS work.
use crate::error::{Error, IOError};
use async_stream::try_stream;
use futures::stream::Stream;
use grpc_api::{Script, TargetOs};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile, TempDir};
use tokio::fs;

const TEMP_DIR: &str = "/tmp/scripts";

pub async fn new_tmp_dir() -> Result<TempDir, std::io::Error> {
    if cfg!(target_family = "unix") {
        let p = Path::new(TEMP_DIR);
        if !p.exists() {
            tokio::fs::create_dir(p).await?
        }
        Builder::new().tempdir_in(TEMP_DIR)
    } else {
        tempfile::tempdir()
    }
}
pub async fn extract_files_include(zip: &[u8]) -> Result<TempDir, Error> {
    let dir = new_tmp_dir().await.map_err(|e| IOError::CreateFile(e))?;
    let c_dir = dir.path().clone();
    tokio::task::block_in_place(move || unzip_into_dir(c_dir, &zip))?;
    Ok(dir)
}

pub fn new_tmp_script_file(
    script_type: Script,
    content: &str,
) -> Result<NamedTempFile, std::io::Error> {
    let mut file = if cfg!(target_family = "unix") {
        Builder::new()
            .suffix(script_type.file_extension())
            .tempfile_in(TEMP_DIR)?
    } else {
        Builder::new()
            .suffix(script_type.file_extension())
            .tempfile()?
    };
    let content = if script_type.target_os() == TargetOs::Unix && content.contains("\r\n") {
        content.replace("\r\n", "\n")
    } else {
        content.to_string()
    };
    file.write(content.as_bytes())?;
    Ok(file)
}

pub fn ls_dir_content(root: PathBuf) -> impl Stream<Item = Result<PathBuf, IOError>> {
    try_stream! {
        if !root.is_file() {
            let mut stack = vec![root.to_path_buf()];
            while let Some(dir) = stack.pop() {
                let mut dir_entry = fs::read_dir(&dir).await.map_err(|_e| IOError::ListDir(dir.clone()))?;
                while let Ok(Some(entry)) = dir_entry.next_entry().await {
                    if entry.file_type().await.map_err(|_e| IOError::ListDir(dir.clone()))?.is_dir() {
                        stack.push(entry.path());
                    }
                    yield entry.path();
                }
            }
        }
    }
}

fn unzip_into_dir(outdir: &Path, zip_buf: &[u8]) -> Result<(), IOError> {
    if !zip_buf.is_empty() {
        let reader = std::io::Cursor::new(zip_buf);
        let mut zip = zip::ZipArchive::new(reader)?;
        use std::fs;
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let outpath = outdir.join(file.sanitized_name());
            if file.is_dir() {
                fs::create_dir_all(&outpath).map_err(|e| IOError::CreateFile(e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).map_err(|e| IOError::Copy(e))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| IOError::CreateFile(e))?;
                io::copy(&mut file, &mut outfile).map_err(|e| IOError::Copy(e))?;
            }
        }
    }
    Ok(())
}
