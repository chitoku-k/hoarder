use clap::{crate_version, Parser};
use icu::locid::Locale;
use log::LevelFilter;

#[derive(Debug, Parser)]
#[command(version = version())]
pub struct Config {
    /// Print schema in SDL (Schema Definition Language)
    #[arg(long)]
    pub print_schema: bool,

    /// Port number
    #[arg(long, env)]
    pub port: u16,

    /// Root directory for media
    #[arg(long, env)]
    pub media_root_dir: String,

    /// Root URL for media
    #[arg(long, env)]
    pub media_root_url: Option<String>,

    /// Locale
    #[arg(long, env, default_value_t)]
    pub locale: Locale,

    /// Path to TLS certificate (if not specified, application is served over HTTP)
    #[arg(long, env, requires = "tls_key")]
    pub tls_cert: Option<String>,

    /// Path to TLS private key (if not specified, application is served over HTTP)
    #[arg(long, env, requires = "tls_cert")]
    pub tls_key: Option<String>,

    /// Log level
    #[arg(long, env, default_value = "info")]
    pub log_level: String,
}

pub fn init() -> Config {
    let config = Config::parse();
    config.init();
    config
}

const fn version() -> &'static str {
    crate_version!()
}

impl Config {
    pub fn init(&self) {
        env_logger::builder()
            .format_target(true)
            .format_timestamp_secs()
            .format_indent(None)
            .filter(None, LevelFilter::Info)
            .parse_filters(&self.log_level)
            .init();
    }
}
