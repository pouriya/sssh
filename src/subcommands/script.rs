use crate::error::AppError;
use crate::settings::Settings;
use anyhow::Result;

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    settings.ensure_script_file()?;
    print!("{}", settings.script.clone());
    println!();
    println!("# file: {:?}", settings.script_file.clone());
    Ok(())
}
