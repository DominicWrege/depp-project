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
            Script::Awk => (vec!["awk".into(), "-f".into()]),
            Script::Sed => (vec!["sed".into(), "-f".into()]),
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

impl From<i32> for RegexMode {
    fn from(n: i32) -> Self {
        match n {
            1 => RegexMode::Stdout,
            2 => RegexMode::ScriptContent,
            _ => RegexMode::UnknownRegex,
        }
    }
}

impl From<i32> for SortStdoutBy {
    fn from(n: i32) -> Self {
        match n {
            1 => SortStdoutBy::Asc,
            2 => SortStdoutBy::Desc,
            _ => SortStdoutBy::UnknownSort,
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

#[derive(Debug, PartialEq)]
pub enum TargetOs {
    Windows,
    Unix,
}

impl From<&tokio_postgres::row::Row> for Assignment {
    fn from(r: &Row) -> Self {
        Assignment {
            name: r.get("assignment_name"),
            solution: r.get("solution"),
            include_files: r.get("include_files"),
            script_type: r.get::<_, Script>("script_type") as i32,
            args: r.get("args"),
            compare_fs_solution: r.get("compare_fs_solution"),
            compare_stdout_solution: r.get("compare_stdout_solution"),
            custom_script: r.get::<_, Option<String>>("custom_script"),
            regex: r.get::<_, Option<String>>("regex"),
            regex_mode: r.get::<_, RegexMode>("regex_check_mode") as i32,
            sort_stdout: r.get::<_, SortStdoutBy>("sort_stdout") as i32,
        }
    }
}

impl From<Option<&String>> for RegexMode {
    fn from(str: Option<&String>) -> Self {
        match str {
            Some(s) if s == "Stdout" => RegexMode::Stdout,
            Some(s) if s == "ScriptContent" => RegexMode::ScriptContent,
            _ => RegexMode::UnknownRegex,
        }
    }
}

impl From<Option<&String>> for SortStdoutBy {
    fn from(str: Option<&String>) -> Self {
        match str {
            Some(s) if s == "Asc" => SortStdoutBy::Asc,
            Some(s) if s == "Desc" => SortStdoutBy::Desc,
            _ => SortStdoutBy::UnknownSort,
        }
    }
}

impl From<String> for SortStdoutBy {
    fn from(str: String) -> Self {
        match str {
            _ => SortStdoutBy::UnknownSort,
        }
    }
}
