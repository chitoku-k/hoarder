use clap::{crate_version, Parser};
use icu::locid::Locale;

#[derive(Debug, Parser)]
#[command(version = version())]
pub struct Config {
    /// Print schema in SDL (Schema Definition Language)
    #[arg(long)]
    pub print_schema: bool,

    /// Port number
    #[arg(long, env)]
    pub port: u16,

    /// ICU Locale
    #[arg(long, env)]
    pub icu_locale: Locale,

    /// Path to TLS certificate (if not specified, application is served over HTTP)
    #[arg(long, env)]
    pub tls_cert: Option<String>,

    /// Path to TLS private key (if not specified, application is served over HTTP)
    #[arg(long, env)]
    pub tls_key: Option<String>,

    /// Root directory for media
    #[arg(long, env)]
    pub media_root_dir: String,

    /// Root URL for media
    #[arg(long, env)]
    pub media_root_url: Option<String>,
}

pub fn get() -> Config {
    Config::parse()
}

const fn version() -> &'static str {
    crate_version!()
}
