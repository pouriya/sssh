use crate::error::AppError;
use crate::{settings, settings::Settings};
use anyhow::Result;
use clap::Parser;

pub const DEFAULT_SCRIPT: &str = include_str!("sssh.sh");
pub const DEFAULT_CONFIGURATION: &str = include_str!("sssh.toml");

#[derive(Debug, Clone, PartialEq, Parser)]
pub enum Samples {
    /// Default TOML configuration sample.
    Config,
    /// Default script sample.
    Script,
}

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    if let settings::SubCommand::Sample(ref sample) = settings.subcommand {
        match sample {
            Samples::Script => sample_script(settings),
            Samples::Config => sample_config(settings),
        }
    } else {
        // It's already checked in main.rs
        unreachable!()
    }
}

fn sample_config(_settings: &mut Settings) -> Result<(), AppError> {
    print!("{}", DEFAULT_CONFIGURATION);
    Ok(())
}

fn sample_script(_settings: &mut Settings) -> Result<(), AppError> {
    print!("{}", DEFAULT_SCRIPT);
    Ok(())
}
