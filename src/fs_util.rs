use crate::base64::Base64;
use crate::script::Script;
use std::io::Write;

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
