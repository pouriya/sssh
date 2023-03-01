use crate::utils::run_command;
use crate::{error::AppError, settings::Settings};
use anyhow::Result;
use std::path::PathBuf;

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    settings.ensure_configuration_file()?;
    let command = PathBuf::from(settings.editor_command.clone());
    let argument_list = [settings.configuration_file.clone()].to_vec();
    let _ = run_command("Editor", command, argument_list, Vec::new())?;
    settings.ensure_configuration()?;
    Ok(())
}
