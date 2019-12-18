use crate::base64::Base64;
use crate::crash_test::Error;
use crate::script::Script;
use fs_extra::dir;
use std::io::Write;
use std::path::PathBuf;
use tempfile::{Builder, NamedTempFile, TempDir};
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
