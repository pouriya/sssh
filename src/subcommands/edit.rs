use crate::utils::run_command;
use crate::{error::AppError, settings::Settings};
use anyhow::Result;
use std::path::PathBuf;

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    settings.ensure_configuration_file()?;
    settings.ensure_editor_command()?;
    let command = PathBuf::from(settings.editor_command.clone());
    let argument_list = settings.editor_argument_list.clone();
    let _ = run_command("Editor", command, argument_list, Vec::new())?;
    settings.ensure_configuration()?;
    Ok(())
}
