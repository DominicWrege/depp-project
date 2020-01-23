tonic::include_proto!("deep_project");

use std::path::PathBuf;
use std::ffi::OsString;

pub type AssignmentId = uuid::Uuid;

#[cfg(any(unix))]
impl Script {
    pub fn commandline(&self) -> (&'static str, Vec<PathBuf>) {
        match self {
            Script::PowerShell => ("pwsh", vec![]),
            Script::Shell => ("sh", vec![]),
            Script::Batch => ("wine", vec!["cmd.exe".into(), "/C".into()]),
            Script::Python3 => ("python3", vec![]),
            Script::Bash | Script::Awk | Script::Sed | _ => ("bash", vec![]),
        }
    }
}

impl From<i32> for Script {
    fn from(n: i32) -> Self {
        match n {
            0 => Script::PowerShell,
            1 => Script::Batch,
            2 => Script::Python3,
            3 => Script::Shell,
            5 => Script::Awk,
            6 => Script::Sed,
            4 | _ => Script::Bash,
        }
    }
}

#[cfg(target_os = "windows")]
impl Script {
    pub fn commandline(&self) -> (&'static str, Vec<OsString>) {
        match self {
            Script::PowerShell => ("powershell.exe", vec![]),
            Script::Batch => ("cmd.exe", vec!["/C".into()]),
            Script::Python3 => ("python", vec![]),
            Script::Shell | Script::Bash | Script::Awk | Script::Sed => ("bash", vec![]),
        }
    }
}

impl Script {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Script::Batch => ".bat",
            Script::PowerShell => ".ps1",
            Script::Python3 => ".py",
            Script::Shell | Script::Bash | Script::Sed | Script::Awk  => ".sh",
        }
    }
}
