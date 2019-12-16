use std::io::Write;

use crate::base64::Base64;
use crate::config::Script;
use tempfile::{Builder, NamedTempFile};

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

impl Script {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Script::Batch => ".bat",
            Script::PowerShell => ".ps1",
            Script::Python3 => ".py",
            Script::Shell | Script::Bash | Script::Sed | Script::Awk => ".sh",
        }
    }
}
