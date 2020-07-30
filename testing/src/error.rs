//! Error handling using [failure](https://docs.rs/crate/failure) as error library.
use crate::docker_api::DockerError;
use std::path::PathBuf;
use std::{fmt, time};
use zip::result::ZipError;

#[derive(Debug, failure::Fail)]
pub enum IOError {
    #[fail(display = "Failed to read file: {}", _0)]
    ReadFile(std::io::Error),
    #[fail(display = "Could not copy included files for testing {}", _0)]
    Copy(std::io::Error),
    #[fail(display = "Can't create temp file. {}", _0)]
    CreateFile(std::io::Error),
    #[fail(display = "Failed to run the custom scrip. Error: {}", _0)]
    FailedRunCustomScript(tokio::io::Error),
    #[fail(display = "Zip error: {}", _0)]
    Zip(zip::result::ZipError),
    #[fail(display = "IO error while reading the dir {:?}", _0)]
    ListDir(PathBuf),
}
#[derive(Debug, failure::Fail)]
pub enum SystemError {
    #[fail(display = "IO error: {}", _0)]
    IO(IOError),
    #[fail(display = "Docker error {}", _0)] // this should be some docker_err
    Docker(DockerError),
    #[fail(display = "Error in sample solution.")]
    BadSampleSolution,
}

impl From<zip::result::ZipError> for IOError {
    fn from(z_err: ZipError) -> Self {
        IOError::Zip(z_err)
    }
}

impl From<DockerError> for Error {
    fn from(d_err: DockerError) -> Self {
        Error::InvalidTest(SystemError::Docker(d_err))
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::InvalidTest(SystemError::IO(err))
    }
}

#[derive(Debug, failure::Fail, derive_more::From)]
pub enum Error {
    #[from]
    #[fail(display = "Invalid. {}", _0)]
    InvalidTest(SystemError),
    #[fail(display = "Time out reached! Script took more than {}.", _1)]
    Timeout(tokio::time::Elapsed, DurationDisplay),
    #[from]
    #[fail(display = "Script produced invalid UTF8.")]
    NoUTF8(std::string::FromUtf8Error),
    #[fail(display = "Does not contains expected output. {}", _0)]
    WrongOutput(String),
    #[fail(display = "Solution dir and tested dir have not the same content")]
    ExpectedDirNotSame,
    #[fail(display = "Script finished with exit code 1 stderr: {}", _0)]
    ExitCode(String),
    #[fail(
        display = "This content of this file {:#?} does not match with the solution. expected({}) result({})",
        _0, _1, _2
    )]
    ExpectedFileNotSame(PathBuf, String, String),
    #[fail(display = "Regex error {}", _0)]
    InvalidRegex(String),
    #[fail(display = "No Regex match found in '{}' for regex: '{}'", _0, _1)]
    NoRegexMatch(String, regex::Regex),
    #[fail(display = "Stdout is not sorted. stdout: {:#?}", _0)]
    NoSorted(String),
    #[fail(
        display = "Custom script wrote to some to the stderr or exit is not 0. Stderr: {}",
        _0
    )]
    CustomScript(String),
}
#[derive(Debug, derive_more::From)]
pub struct DurationDisplay(time::Duration);

impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0.as_secs())
    }
}
