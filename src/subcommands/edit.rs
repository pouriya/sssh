use crate::utils::run_command;
use crate::{error::AppError, settings::Settings};
use anyhow::Result;
use std::time::Instant;
use tracing::debug;

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    settings.maybe_try_create_configuration_file()?;
    settings.ensure_editor_command()?;
    let command = settings.editor_command.clone();
    let argument_list = settings.editor_argument_list.clone();
    let start_time = Instant::now();
    let _ = run_command("Editor", command, argument_list, Vec::new())?;
    let duration = start_time.elapsed().as_secs();
    debug!(edit_duration = duration);
    if duration < 2 {
        return Err(AppError::EditorFastStop);
    }
    settings.try_load_and_set_configuration()?;
    Ok(())
}
