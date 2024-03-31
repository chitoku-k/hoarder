use clap::{crate_version, Parser};
use icu_locid::Locale;

pub mod commands;
use commands::{Commands, ServeCommand};

#[derive(Debug, Parser)]
#[command(version = version())]
#[command(arg_required_else_help = false)]
#[command(subcommand_required = false)]
pub struct Config {
    #[command(flatten)]
    pub global: Global,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
struct ServeConfig {
    #[command(flatten)]
    pub global: Global,

    #[command(flatten)]
    pub command: ServeCommand,
}

#[derive(Debug, Clone, Parser)]
pub struct Global {
    /// Locale
    #[arg(long, env, default_value_t)]
    #[arg(global = true)]
    pub locale: Locale,

    /// Log level
    #[arg(long, env, default_value = "info")]
    #[arg(global = true)]
    pub log_level: String,
}

pub fn init() -> Config {
    let config = Config::get();
    config.init();
    config
}

const fn version() -> &'static str {
    crate_version!()
}

impl Config {
    fn get() -> Self {
        use clap::error::{ContextKind, ErrorKind};
        match Self::try_parse() {
            Ok(config) => config,
            Err(e) if matches!(e.kind(), ErrorKind::MissingSubcommand | ErrorKind::UnknownArgument) && e.get(ContextKind::InvalidSubcommand).is_none() => {
                ServeConfig::parse().into()
            },
            Err(e) => {
                e.exit();
            },
        }
    }

    pub fn init(&self) {
        env_logger::builder()
            .format_target(true)
            .format_timestamp_secs()
            .format_indent(None)
            .parse_filters(&self.global.log_level)
            .init();
    }
}

impl From<ServeConfig> for Config {
    fn from(config: ServeConfig) -> Self {
        Self {
            command: Commands::Serve(config.command),
            global: config.global,
        }
    }
}
