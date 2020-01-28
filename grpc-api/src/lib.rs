tonic::include_proto!("deep_project");

pub type AssignmentId = uuid::Uuid;

#[cfg(target_family = "unix")]
impl Script {
    pub fn command_line(&self) -> (&'static str, Vec<std::path::PathBuf>) {
        match self {
            Script::PowerShell => ("pwsh", vec![]), // maybe ln -s pwsh -> powershell.exe
            Script::Batch => ("cmd.exe", vec!["/C".into()]), // only works inside wsl
            Script::Python3 => ("python3", vec![]),
            Script::Awk => ("awk", vec![]),
            Script::Sed => ("sed", vec![]),
            Script::Bash => ("bash", vec![]),
            Script::Shell => ("sh", vec![]),
        }
    }
}

#[cfg(target_family = "windows")]
impl Script {
    pub fn command_line(&self) -> (&'static str, Vec<std::ffi::OsString>) {
        match self {
            Script::PowerShell => ("powershell.exe", vec![]),
            Script::Batch => ("cmd.exe", vec!["/C".into()]),
            Script::Python3 => ("python3", vec![]),
            Script::Awk => ("awk", vec![]),
            Script::Sed => ("sed", vec![]),
            Script::Bash => ("bash", vec![]), // bash -c are forwarded to the WSL process without modification.
            Script::Shell => ("sh", vec![]),
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

impl Script {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Script::Batch => ".bat",
            Script::PowerShell => ".ps1",
            Script::Python3 => ".py",
            Script::Shell | Script::Bash | Script::Sed | Script::Awk => ".sh",
        }
    }
    pub fn target_os(&self) -> TargetOs {
        match self {
            Script::PowerShell | Script::Batch => TargetOs::Windows,
            _ => TargetOs::Unix,
        }
    }
}

#[derive(Debug)]
pub enum TargetOs {
    Windows,
    Unix,
}
