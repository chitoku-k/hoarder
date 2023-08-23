use clap::{crate_version, Parser};

#[derive(Debug, Eq, Parser, PartialEq)]
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
}

pub fn get() -> Config {
    Config::parse()
}

fn version() -> &'static str {
    crate_version!()
}
