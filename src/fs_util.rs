use crate::base64::Base64;
use crate::crash_test::Error;
use crate::script::Script;
use fs_extra::dir;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile, TempDir};
use tokio::fs::read_dir;
pub fn new_tmp_script_file(
    script_type: Script,
    content: &Base64,
) -> Result<NamedTempFile, std::io::Error> {
    let mut file = Builder::new()
        .suffix(script_type.file_extension())
        .tempfile()?;
    file.write(&content.0.as_bytes())?;
    Ok(file)
}
pub fn cp_include_into(
    files: &Vec<PathBuf>,
    dir_solution: &TempDir,
    dir_to_test: &TempDir,
) -> Result<(), Error> {
    let opt = dir::CopyOptions::new();
    fs_extra::copy_items(&files, &dir_solution.path(), &opt)?;
    fs_extra::copy_items(&files, &dir_to_test.path(), &opt)?;
    Ok(())
}

pub async fn ls_dir(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = vec![];
    if root.is_file() {
        return Ok(vec![]);
    }
    let mut stack = vec![root.to_path_buf()];
    while !stack.is_empty() {
        let dir = stack.pop().unwrap();
        let mut dir_entrys = read_dir(&dir).await.map_err(|_e| Error::ListDir(dir))?;
        while let Ok(Some(entry)) = dir_entrys.next_entry().await {
            paths.push(entry.path());
            if entry.file_type().await?.is_dir() {
                stack.push(entry.path());
            }
        }
    }

    Ok(paths)
}
