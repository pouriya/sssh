use crate::error::AppError;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error};

pub fn run_command(
    title: &'static str,
    command: PathBuf,
    argument_list: Vec<PathBuf>,
    env_list: Vec<(&str, &str)>,
) -> Result<(String, String), AppError> {
    debug!(command = ?command, arguments = ?argument_list, "Attempt to start {} process", title);
    let process = Command::new(command.clone())
        .args(argument_list.clone())
        .envs(env_list)
        .spawn()
        .map_err(|source| AppError::ProcessStart {
            command: command.clone(),
            argument_list: argument_list.clone(),
            title,
            source,
        })?;
    let output = process
        .wait_with_output()
        .map_err(|source| AppError::ProcessWait {
            command: command.clone(),
            argument_list: argument_list.clone(),
            title,
            source,
        })?;
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    let stdout = String::from_utf8(output.stdout).unwrap_or_default();
    if output.status.success() {
        debug!(
            stdout = stdout,
            stderr = stderr,
            command = ?command,
            argument = ?argument_list,
            "{} process exited successfully", title
        )
    } else {
        let output = format!("{}{}", stderr, stdout);
        error!(
            output = output,
            command = ?command,
            argument = ?argument_list,
            "{} process failed", title
        );
        let error = if output.trim().is_empty() {
            io::Error::new(io::ErrorKind::Other, format!("{} process failed", title))
        } else {
            io::Error::new(io::ErrorKind::Other, output)
        };
        return Err(AppError::ProcessFailed {
            command,
            argument_list,
            source: error,
            title,
        });
    }
    Ok((stdout, stderr))
}
