mod error;
mod settings;
mod subcommands;
mod utils;

use std::io;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use anyhow::{bail, Result};

use lazy_static::lazy_static;

use crate::error::AppError;
use crate::settings::{Settings, SubCommand};
use subcommands::sample::Samples;

type SubCommandFunction = fn(&mut Settings) -> Result<(), AppError>;

lazy_static! {
    static ref CALLBACK_LIST: Vec<(SubCommand, SubCommandFunction)> = [
        (
            SubCommand::Select,
            subcommands::select::run as SubCommandFunction
        ),
        (
            SubCommand::Edit,
            subcommands::edit::run as SubCommandFunction
        ),
        (
            SubCommand::Config,
            subcommands::config::run as SubCommandFunction
        ),
        (
            SubCommand::Script,
            subcommands::script::run as SubCommandFunction
        ),
        (
            SubCommand::Sample(Samples::Config),
            subcommands::sample::run as SubCommandFunction
        ),
        (
            SubCommand::Sample(Samples::Script),
            subcommands::sample::run as SubCommandFunction
        ),
    ]
    .to_vec();
}

fn main() -> Result<()> {
    let (logging_filter, logging_reload_handle) =
        tracing_subscriber::reload::Layer::new(tracing_subscriber::filter::LevelFilter::OFF);
    tracing_subscriber::registry()
        .with(logging_filter)
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .pretty()
                .with_line_number(true)
                .with_file(true)
                .with_target(false)
                .with_writer(io::stderr),
        )
        .init();
    let mut settings = settings::Settings::new()?;
    if !settings.quiet {
        // Update log level from `settings` and reload it again!
        let new_logging_level = if settings.verbose {
            tracing_subscriber::filter::LevelFilter::DEBUG
        } else {
            tracing_subscriber::filter::LevelFilter::INFO
        };
        logging_reload_handle
            .modify(|filter| *filter = new_logging_level)
            .unwrap();
        settings = settings::Settings::new()?;
    };
    for (subcommand, function) in CALLBACK_LIST.iter().cloned() {
        if subcommand == settings.subcommand {
            if subcommand == SubCommand::Select {
                if settings.verbose {
                    if atty::is(atty::Stream::Stderr) {
                        bail!("If you enable verbose mode (-v or --verbose) and use `select` subcommand (which is the default subcommand), you have to forward `stderr` to somewhere else")
                    }
                } else {
                    logging_reload_handle
                        .modify(|filter| {
                            *filter = tracing_subscriber::filter::LevelFilter::OFF;
                        })
                        .unwrap();
                }
            }
            function(&mut settings)?;
            return Ok(());
        }
    }
    bail!(
        "Could not found callback function for subcommand `{}`",
        format!("{:?}", settings.subcommand).to_lowercase()
    );
}
