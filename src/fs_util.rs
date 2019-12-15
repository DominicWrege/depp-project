use std::io::Write;

use crate::base64::Base64;
use crate::config::Script;
use crate::fs_util::Error::TmpFileCreate;
use tempfile::{Builder, NamedTempFile};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Error while creating a new tmp file")]
    TmpFileCreate(std::io::Error),
    #[error(display = "Error while writing bytes into tmp file")]
    TmpFileWrite(std::io::Error),
}

pub fn new_tmp_script_file(script_type: Script, content: Base64) -> Result<NamedTempFile, Error> {
    let mut file = Builder::new()
        .suffix(script_type.fs_extension())
        .tempfile()
        .map_err(TmpFileCreate)?;
    write_into(&mut file, content)?;

    Ok(file)
}

fn write_into(file: &mut NamedTempFile, content: Base64) -> Result<usize, Error> {
    file.write(&content.0.as_bytes())
        .map_err(Error::TmpFileWrite)
}

impl Script {
    pub fn fs_extension(&self) -> &'static str {
        match self {
            Script::Batch => "bat",
            Script::Powershell => "ps1",
            Script::Python3 => "py",
            Script::Shell | Script::Bash | Script::Sed => "sh",
            Script::Awk => "awk",
        }
    }
}
