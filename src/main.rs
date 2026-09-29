use std::{path::PathBuf, process::ExitCode, str::FromStr, time::Duration};

use clap::{Parser, Subcommand, ValueEnum};
use rfc_agent_cli::{ClientOptions, DocumentId, Error, RfcClient};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "rfc",
    version,
    about = "Agent-friendly RFC CLI and MCP server",
    long_about = None
)]
struct Cli {
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    output: OutputFormat,

    #[arg(long, global = true, conflicts_with = "output")]
    json: bool,

    #[arg(long, global = true)]
    offline: bool,

    #[arg(long, global = true)]
    refresh: bool,

    #[arg(long, global = true, env = "RFC_CACHE_DIR")]
    cache_dir: Option<PathBuf>,

    #[arg(long, global = true, default_value_t = 20)]
    timeout: u64,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Jsonl,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the authoritative plain-text RFC or Internet-Draft.
    Show {
        /// RFC number (`9110` or `RFC9110`) or full Internet-Draft name.
        identifier: String,

        /// Retained for compatibility. Paging is performed only by an explicit caller.
        #[arg(long, hide = true)]
        pager: bool,
    },

    /// Print structured RFC Editor metadata.
    Info {
        /// RFC number (`9110` or `RFC9110`).
        identifier: String,
    },

    /// Manage the persistent cache.
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },

    /// Start an MCP server over standard input/output.
    Mcp,
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    /// Show the cache directory.
    Status,
    /// Remove all cached documents and metadata.
    Clear,
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    schema_version: &'static str,
    kind: &'static str,
    error: ErrorBody<'a>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'static str,
    message: &'a str,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = if cli.json {
        OutputFormat::Json
    } else {
        cli.output
    };
    match run(cli, output).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if matches!(output, OutputFormat::Json | OutputFormat::Jsonl) {
                let message = error.to_string();
                let envelope = ErrorEnvelope {
                    schema_version: "1",
                    kind: "error",
                    error: ErrorBody {
                        code: error.code(),
                        message: &message,
                    },
                };
                eprintln!("{}", serde_json::to_string(&envelope).unwrap_or(message));
            } else {
                eprintln!("error: {error}");
            }
            ExitCode::from(error.exit_code())
        }
    }
}

async fn run(cli: Cli, output: OutputFormat) -> Result<(), Error> {
    let client = RfcClient::new(ClientOptions {
        cache_dir: cli.cache_dir,
        offline: cli.offline,
        refresh: cli.refresh,
        timeout: Duration::from_secs(cli.timeout),
    })?;

    match cli.command {
        Command::Show { identifier, .. } => {
            let id = DocumentId::from_str(&identifier)?;
            let response = client.document(&id).await?;
            if output == OutputFormat::Human {
                print!("{}", response.content);
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
        }
        Command::Info { identifier } => {
            let id = DocumentId::from_str(&identifier)?;
            let response = client.metadata(&id).await?;
            if output == OutputFormat::Human {
                println!("{} — {}", response.metadata.doc_id, response.metadata.title);
                if !response.metadata.authors.is_empty() {
                    println!("Authors: {}", response.metadata.authors.join(", "));
                }
                if let Some(status) = response.metadata.status {
                    println!("Status: {status}");
                }
                if let Some(date) = response.metadata.pub_date {
                    println!("Published: {date}");
                }
                println!("Source: {}", response.source_url);
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
        }
        Command::Cache { command } => match command {
            CacheCommand::Status => {
                if output == OutputFormat::Human {
                    println!("{}", client.cache().root().display());
                } else {
                    println!(
                        "{}",
                        serde_json::json!({
                            "schema_version": "1",
                            "kind": "cache_status",
                            "path": client.cache().root(),
                        })
                    );
                }
            }
            CacheCommand::Clear => {
                client.cache().clear().await?;
                if output == OutputFormat::Human {
                    println!("Cache cleared");
                } else {
                    println!(
                        "{}",
                        serde_json::json!({
                            "schema_version": "1",
                            "kind": "cache_cleared"
                        })
                    );
                }
            }
        },
        Command::Mcp => rfc_agent_cli::mcp::serve(client).await?,
    }
    Ok(())
}
