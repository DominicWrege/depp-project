use grpc_api::Script;
use serde::Deserialize;
use uuid::Uuid;

/*pub type AssignmentsMap = HashMap<AssignmentId, Assignment>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub name: String,
    pub assignments: HashMap<Uuid, Assignment>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Assignment {
    pub name: String,
    //#[serde(deserialize_with = "into_absolute_path")]
    pub solution: String,
    #[serde(default)]
    pub include_files: Vec<PathBuf>,
    #[serde(default)]
    #[serde(rename = "type")]
    pub script_type: Script,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum Error {
    #[from]
    #[error(display = "can`t find config file, {}", _0)]
    ConfigFile(std::io::Error),
    #[from]
    #[error(display = "wrong toml format, {}", _0)]
    Toml(toml::de::Error),
}

pub fn parse_config(path: &Path) -> Result<Config, Error> {
    let file_content = fs::read_to_string(path)?;
    let conf = toml::from_str::<Config>(&file_content)?;
    //check_config(&conf)?;
    Ok(conf)
}*/

// // can panic
// fn check_config(conf: &Config) -> Result<(), std::io::Error> {
//     for assignment in conf.assignments.values() {
//         for path in &assignment.include_files {
//             path_exists_and_is_file(&path);
//         }
//         path_exists_and_is_file(&assignment.solution_path);
//     }
//     Ok(())
// }

// fn path_exists_and_is_file(p: &Path) {
//     if !p.exists() || p.is_dir() {
//         panic!(
//             "Config error: path to {:#?} does not exists or is not a file.",
//             p
//         )
//     }
// }

// delete stupid windows newlines
pub fn fix_win_ln(s: &str) -> String {
    s.chars().filter(|&c| c != '\r').collect::<_>()
}
