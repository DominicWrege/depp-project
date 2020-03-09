use tokio_postgres::Row;
tonic::include_proto!("deep_project");

pub type AssignmentId = uuid::Uuid;

#[cfg(target_family = "unix")]
impl Script {
    pub fn command_line(&self) -> Vec<&str> {
        match self {
            Script::PowerShell => (vec!["pwsh".into()]), // maybe ln -s pwsh -> powershell.exe
            Script::Batch => (vec!["cmd.exe".into(), "/C".into()]), // only works inside wsl
            Script::Python3 => (vec!["python3".into()]),
            Script::Awk => (vec!["awk".into()]),
            Script::Sed => (vec!["sed".into()]),
            Script::Bash => (vec!["bash".into()]),
            Script::Shell => (vec!["sh".into()]),
        }
    }
}

#[cfg(target_family = "windows")]
impl Script {
    pub fn command_line(&self) -> Vec<&str> {
        match self {
            Script::PowerShell => (vec!["powershell.exe".into()]),
            Script::Batch => (vec!["cmd.exe".into(), "/C".into()]),
            Script::Python3 => (vec!["python3".into()]),
            Script::Awk => (vec!["awk".into()]),
            Script::Sed => (vec!["sed".into()]),
            Script::Bash => (vec!["bash".into()]), // bash -c are forwarded to the WSL process without modification.
            Script::Shell => (vec!["sh".into()]),
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

impl From<&tokio_postgres::row::Row> for Assignment {
    fn from(r: &Row) -> Self {
        let s: Script = r.get("script_type");
        Assignment {
            name: r.get("assignment_name"),
            solution: r.get("solution"),
            include_files: r.get("include_files"),
            script_type: s.into(),
            args: r.get("args"),
        }
    }
}
