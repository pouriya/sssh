use crate::{
    error::AppError,
    subcommands::sample::{Samples, DEFAULT_CONFIGURATION, DEFAULT_SCRIPT},
};
use anyhow::{Context, Result};
use clap::Parser;
use dirs::config_dir;
use faccess::PathExt;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};
use tracing::{debug, info};

const DEFAULT_CONFIGURATION_FILENAME: &str = "sssh.toml";
const DEFAULT_SCRIPT_FILENAME: &str = "sssh.sh";
const DEFAULT_USERNAME: &str = "root";
pub const DEFAULT_PORT_NUMBER: u16 = 22;
const EDITOR_COMMAND_NOT_FOUND: &str = "<not found>";
const DEFAULT_EDITOR_ARGUMENTS: &str = "{FILENAME}";
#[cfg(target_family = "unix")]
const TO_BE_SEARCHED_EDITOR_LIST: &[(&str, &[&str])] = &[
    ("vim", &["{FILENAME}"]),
    ("nano", &["-l", "{FILENAME}"]),
    ("vi", &["{FILENAME}"]),
];
#[cfg(not(target_family = "unix"))]
const TO_BE_SEARCHED_EDITOR_LIST: &[(&str, &[&str])] = &[
    ("notepad++", &["-nosession", "-notabbar", "{FILENAME}"]),
    ("notepad", &["{FILENAME}"]),
];

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Settings {
    /// Increase verbosity.
    #[arg(short, long, global = true, env = "SSSH_VERBOSE")]
    pub verbose: bool,
    /// Disable logging
    #[arg(short, long, global = true, env = "SSSH_QUIET")]
    pub quiet: bool,
    /// TOML Configuration file.
    #[arg(
        name = "config-file",
        short,
        long,
        global = true,
        env = "SSSH_CONFIG_FILE",
        default_value = default_configuration_filename(),
    )]
    pub configuration_file: PathBuf,
    /// An executable file that will accept SSH info to connect to chosen server.
    #[arg(
        name = "script-file",
        short,
        long,
        global = true,
        env = "SSSH_SCRIPT_FILE",
        default_value = default_script_filename(),
    )]
    pub script_file: PathBuf,
    #[arg(skip)]
    pub script: String,
    /// Skip running final script.
    #[arg(short = 'S', long, global = true, env = "SSSH_SKIP_SELECT")]
    pub skip_select: bool,
    /// Editor command for editing configuration file.
    #[arg(
        name = "editor-command",
        short='e',
        long,
        global = true,
        default_value = default_editor_command(),
        env = "SSSH_EDITOR_COMMAND"
    )]
    pub editor_command: PathBuf,
    /// List of arguments passed to --editor-command
    #[arg(
        name = "editor-argument",
        short='E',
        long,
        default_value = default_editor_argument_list(),
        global = true,
        env = "SSSH_EDITOR_ARGUMENTS"
    )]
    pub editor_argument_list: Vec<PathBuf>,
    #[command(subcommand)]
    maybe_subcommand: Option<SubCommand>,
    #[arg(skip)]
    pub subcommand: SubCommand,
    #[arg(skip)]
    pub configuration: Config,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub enum SubCommand {
    /// Select a server to connect from the terminal UI.
    Select,
    /// Edit configuration file to add/remove servers.
    Edit,
    /// Prints current configuration file contents.
    Config,
    /// Prints current script file contents.
    Script,
    /// Samples for configuration and script.
    #[command(subcommand)]
    Sample(Samples),
}

impl Default for SubCommand {
    fn default() -> Self {
        Self::Select
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub sssh: ConfigApp,
    #[serde(skip)]
    pub raw: String,
    #[serde(flatten)]
    pub servers: HashMap<String, ConfigServer>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigApp {}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigServer {
    #[serde(skip)]
    pub name: String,
    #[serde(alias = "users", default)]
    pub username_list: Vec<String>,
    pub hostname: String,
    #[serde(default = "default_port_number")]
    pub port: u16,
    #[serde(default)]
    pub description: String,
}

impl Settings {
    pub fn new() -> Result<Self, AppError> {
        let mut settings = Settings::parse();
        settings.subcommand = settings
            .maybe_subcommand
            .clone()
            .or_else(|| {
                let subcommand = SubCommand::default();
                debug!(
                    subcommand = format!("{:?}", subcommand).to_lowercase(),
                    "Set default subcommand"
                );
                Some(subcommand)
            })
            .unwrap();
        debug!(settings = ?settings);
        Ok(settings)
    }

    pub fn try_load_and_set_configuration(&mut self) -> Result<Config, AppError> {
        self.ensure_configuration_file()?;
        let config = Config::try_from(self.configuration_file.clone())?;
        self.configuration = config.clone();
        Ok(config)
    }

    pub fn ensure_configuration(&mut self) -> Result<(), AppError> {
        self.try_load_and_set_configuration().and(Ok(()))
    }

    pub fn ensure_configuration_file(&mut self) -> Result<(), AppError> {
        let configuration_filename = self.configuration_file.clone();
        if !configuration_filename.exists() {
            let _ = self.try_create_default_configuration_file()?;
        }
        Ok(())
    }

    pub fn try_create_default_script_file(&mut self) -> Result<String, AppError> {
        let script_filename = self.script_file.clone();
        let script = DEFAULT_SCRIPT.to_string();
        fs::write(script_filename.clone(), script.clone()).map_err(|error| {
            AppError::FileWrite {
                title: "script",
                filename: script_filename.clone(),
                source: error,
            }
        })?;
        info!(
            filename = format!("{:?}", script_filename),
            "Created script file"
        );
        self.script = script.clone();
        Ok(script)
    }

    pub fn ensure_script_file(&mut self) -> Result<(), AppError> {
        let script_filename = self.script_file.clone();
        if !script_filename.exists() {
            let _ = self.try_create_default_script_file()?;
        }
        let script =
            fs::read_to_string(script_filename.clone()).map_err(|error| AppError::FileRead {
                title: "script",
                filename: script_filename.clone(),
                source: error,
            })?;
        self.script = script;
        if !script_filename.executable() {
            set_permissions(script_filename)?
        }
        Ok(())
    }

    pub fn try_create_default_configuration_file(&mut self) -> Result<Config, AppError> {
        self.configuration = Config::new(self.configuration_file.clone())?;
        Ok(self.configuration.clone())
    }

    pub fn is_default_servers(&self) -> bool {
        self.configuration.is_default_servers()
    }

    pub fn ensure_editor_command(&mut self) -> Result<(), AppError> {
        if self.editor_command.to_str().unwrap() == EDITOR_COMMAND_NOT_FOUND {}
        if self.editor_argument_list == [PathBuf::from(DEFAULT_EDITOR_ARGUMENTS)].to_vec() {
            for (command_name, argument_list) in TO_BE_SEARCHED_EDITOR_LIST.iter().cloned() {
                if command_name == self.editor_command.to_str().unwrap() {
                    self.editor_argument_list =
                        argument_list.iter().cloned().map(PathBuf::from).collect();
                    break;
                }
            }
        }
        let mut append_filename = true;
        self.editor_argument_list = self
            .editor_argument_list
            .iter()
            .map(|argument| {
                let new_argument = PathBuf::from(
                    argument
                        .to_str()
                        .unwrap()
                        .replace("{FILENAME}", self.configuration_file.to_str().unwrap()),
                );
                if &new_argument != argument {
                    debug!("Replaced configuration file in editor arguments");
                    append_filename = false
                };
                if argument == &self.configuration_file {
                    debug!("Configuration file already exists in editor arguments");
                    append_filename = false
                };
                new_argument
            })
            .collect();
        if append_filename {
            debug!("Appended configuration file to editor arguments");
            self.editor_argument_list
                .push(self.configuration_file.clone())
        }
        Ok(())
    }
}

#[cfg(target_family = "unix")]
fn set_permissions(filename: PathBuf) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(filename.clone(), fs::Permissions::from_mode(0o775)).map_err(|source| {
        AppError::FileSetPermission {
            title: "script",
            filename,
            source,
        }
    })
}

#[cfg(not(target_family = "unix"))]
fn set_permissions(_filename: PathBuf) -> Result<(), AppError> {
    Ok(())
}

impl TryFrom<PathBuf> for Config {
    type Error = AppError;

    fn try_from(filename: PathBuf) -> Result<Self, Self::Error> {
        let configuration =
            fs::read_to_string(filename.clone()).map_err(|error| AppError::FileRead {
                title: "configuration",
                filename: filename.clone(),
                source: error,
            })?;
        let mut config: Config =
            toml::from_str(&configuration).map_err(|error| AppError::ConfigSyntax {
                filename: filename.clone(),
                source: error,
            })?;
        config.servers.iter_mut().for_each(|(name, server)| {
            server.name = name.clone();
            if server.username_list.is_empty() {
                debug!(
                    server_name = name,
                    "Use default username `{}` for server", DEFAULT_USERNAME
                );
                server.username_list.push(DEFAULT_USERNAME.to_string());
            }
        });
        config.raw = configuration;
        Ok(config)
    }
}

impl Config {
    pub fn new(filename: PathBuf) -> Result<Self, AppError> {
        if filename.exists() {
            return Err(AppError::FileAlreadyExists {
                title: "Configuration file",
                filename,
            });
        }
        fs::write(filename.clone(), DEFAULT_CONFIGURATION).map_err(|error| {
            AppError::FileWrite {
                title: "configuration",
                filename: filename.clone(),
                source: error,
            }
        })?;
        info!(
            configuration_file = ?filename,
            "Created configuration file"
        );
        Self::try_from(filename)
    }

    pub fn is_default_servers(&self) -> bool {
        self.raw == DEFAULT_CONFIGURATION
    }
}

fn default_configuration_filename() -> &'static str {
    Box::leak(
        try_join_to_user_configuration_directory(DEFAULT_CONFIGURATION_FILENAME)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .into_boxed_str(),
    )
}

fn default_script_filename() -> &'static str {
    Box::leak(
        try_join_to_user_configuration_directory(DEFAULT_SCRIPT_FILENAME)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .into_boxed_str(),
    )
}

fn default_port_number() -> u16 {
    DEFAULT_PORT_NUMBER
}

fn default_editor_command() -> &'static str {
    for (command, _) in TO_BE_SEARCHED_EDITOR_LIST.into_iter().cloned() {
        if let Some(_) = pathsearch::find_executable_in_path(command) {
            return Box::leak(command.to_string().into_boxed_str());
        }
    }
    return EDITOR_COMMAND_NOT_FOUND;
}

fn default_editor_argument_list() -> &'static str {
    let default_editor_command = default_editor_command();
    for (command_name, argument_list) in TO_BE_SEARCHED_EDITOR_LIST.into_iter().cloned() {
        if command_name == default_editor_command {
            return Box::leak(argument_list.join(" ").into_boxed_str());
        }
    }
    DEFAULT_EDITOR_ARGUMENTS
}

fn try_join_to_user_configuration_directory(filename: &'static str) -> Result<PathBuf> {
    // e.g. Linux: ~/.config/<filename>
    Ok(config_dir()
        .context("Could not get user's configuration directory")?
        .join(filename))
}
