//! CLI command parsing and execution.

use crate::controller::{BuildRequest, CheckRequest, Controller, VerifyRequest};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The Prism Platform Model CLI.
#[derive(Parser, Debug)]
#[command(
    name = "prismpm",
    version = "0.1.0",
    about = "Prism Platform Model Framework"
)]
pub struct Cli {
    /// Optional project path (defaults to current directory).
    #[arg(short, long, global = true)]
    pub project: Option<PathBuf>,

    /// Emit machine-readable canonical JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check project static validity without filesystem modification.
    Check,
    /// Build project artifacts and publish under .prism/build/<id>.
    Build,
    /// Verify model against Lean 4.32.1 and lean4-prod execution oracle.
    Verify,
}

/// Run CLI arguments.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let root = cli.project.unwrap_or_else(|| PathBuf::from("."));
    let controller = Controller::load(&root)?;

    match cli.command {
        Commands::Check => {
            let res = controller.check(CheckRequest { config_path: None })?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("check passed (semantic ID: {})", res.semantic_id);
            }
        }
        Commands::Build => {
            let res = controller.build(BuildRequest { config_path: None })?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("build published: {}", res.build_id);
            }
        }
        Commands::Verify => {
            let res = controller.verify(VerifyRequest { config_path: None })?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("verified attestation: {}", res.attestation_id);
            }
        }
    }
    Ok(())
}
