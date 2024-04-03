use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Serve API [default]
    Serve(ServeCommand),

    /// Manage GraphQL schema
    Schema(SchemaCommand),

    /// Execute database migration
    Migration(MigrationCommand),
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommands {
    /// Print schema in SDL (Schema Definition Language)
    Print(SchemaPrintCommand),
}

#[derive(Debug, Parser)]
pub struct ServeCommand {
    /// Port number
    #[arg(long, env)]
    pub port: u16,

    /// Root directory for media
    #[arg(long, env)]
    pub media_root_dir: String,

    /// Root URL for media
    #[arg(long, env)]
    pub media_root_url: Option<String>,

    /// Path to TLS certificate (if not specified, application is served over HTTP)
    #[arg(long, env, requires = "tls_key")]
    pub tls_cert: Option<String>,

    /// Path to TLS private key (if not specified, application is served over HTTP)
    #[arg(long, env, requires = "tls_cert")]
    pub tls_key: Option<String>,
}

#[derive(Debug, Parser)]
pub struct SchemaCommand {
    #[command(subcommand)]
    pub command: SchemaCommands,
}

#[derive(Debug, Parser)]
pub struct SchemaPrintCommand;

#[derive(Debug, Parser)]
#[group(skip)]
pub struct MigrationCommand {
    #[command(flatten)]
    pub command: postgres::MigrationCommand,
}
