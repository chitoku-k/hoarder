use clap::{crate_version, Parser};
use media::Regex;

#[derive(Debug, Parser)]
#[command(version = version())]
pub struct Config {
    /// Print schema in SDL (Schema Definition Language)
    #[arg(long)]
    pub print_schema: bool,

    /// Port number
    #[arg(long, env)]
    pub port: u16,

    /// Path to TLS certificate (if not specified, application is served over HTTP)
    #[arg(long, env)]
    pub tls_cert: Option<String>,

    /// Path to TLS private key (if not specified, application is served over HTTP)
    #[arg(long, env)]
    pub tls_key: Option<String>,

    /// Regex pattern from which media URLs are rewritten
    #[arg(long, env)]
    pub rewrite_original_url_from: Option<Regex>,

    /// Replacement target to which media URLs are rewritten
    #[arg(long, env)]
    pub rewrite_original_url_to: Option<String>,
}

pub fn get() -> Config {
    Config::parse()
}

const fn version() -> &'static str {
    crate_version!()
}
