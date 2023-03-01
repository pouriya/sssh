use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // settings:
    #[error("Could not write to {title:} file {filename:?}")]
    FileWrite {
        title: &'static str,
        filename: PathBuf,
        source: io::Error,
    },
    #[error("Could not read {title:} file {filename:?}")]
    FileRead {
        title: &'static str,
        filename: PathBuf,
        source: io::Error,
    },
    #[error("Could not set execute permissions for {title:} file {filename:?}")]
    FileSetPermission {
        title: &'static str,
        filename: PathBuf,
        source: io::Error,
    },
    #[error("Could not decode configuration from {filename:?}")]
    ConfigSyntax {
        filename: PathBuf,
        source: toml::de::Error,
    },
    #[error("{title:} {filename:?} already exists.")]
    FileAlreadyExists {
        title: &'static str,
        filename: PathBuf,
    },
    // Edit subcommand:
    #[error("Could not run start {title:} process with command {command:?} and arguments {argument_list:?}")]
    ProcessStart {
        title: &'static str,
        command: PathBuf,
        argument_list: Vec<PathBuf>,
        source: io::Error,
    },
    #[error("Could not wait for {title:} process running command {command:?} with argument {argument_list:?}")]
    ProcessWait {
        title: &'static str,
        command: PathBuf,
        argument_list: Vec<PathBuf>,
        source: io::Error,
    },
    #[error("{title:} process running command {command:?} with argument {argument_list:?} failed")]
    ProcessFailed {
        title: &'static str,
        command: PathBuf,
        argument_list: Vec<PathBuf>,
        source: io::Error,
    },
    // Select subcommand:
    #[error("UI error")]
    UI { source: io::Error },
}
