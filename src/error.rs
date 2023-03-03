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
    #[cfg(target_family = "unix")]
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
    #[error("{title:} {filename:?} already exists")]
    FileAlreadyExists {
        title: &'static str,
        filename: PathBuf,
    },
    #[error("{title:} command {command:?} not found.")]
    CommandNotFound {
        title: &'static str,
        command: PathBuf,
    },
    // Edit subcommand:
    #[error("Editor process was running for less than 2 seconds!\nMaybe your editor opened the edit tab inside another session.")]
    EditorFastStop,
    #[error(
        "Could not start {title:} process with command {command:?} and arguments {argument_list:?}"
    )]
    // Utils:
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
