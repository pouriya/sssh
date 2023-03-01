use crate::error::AppError;
use crate::settings::Settings;
use anyhow::Result;
use clap::crate_name;

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    settings.ensure_configuration()?;
    println!("# file: {:?}", settings.configuration_file.clone());
    println!("# Use `{} edit` to edit this file.", crate_name!());
    println!();
    print!("{}", settings.configuration.raw);
    Ok(())
}
