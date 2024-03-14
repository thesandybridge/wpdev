use anyhow::{Error as AnyhowError, Result};
use tokio;
mod commands;
use wpdev_core::config;
use wpdev_core::utils;

use bat::PrettyPrinter;
use clap::{Parser, Subcommand};
use serde_json;

/// A CLI for managing WordPress development environments.
#[derive(Parser, Debug)]
#[clap(name = "wpdev")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List instances
    List {
        #[clap(value_parser)]
        id: Option<String>,
    },
    /// Create a new instance
    Create {
        #[clap(value_parser)]
        options: Option<String>,
    },
    /// Start an instance
    Start {
        #[clap(value_parser)]
        id: String,
    },
    /// Stop an instance
    Stop {
        #[clap(value_parser)]
        id: String,
    },
    /// Restart an instance
    Restart {
        #[clap(value_parser)]
        id: String,
    },
    /// Delete an instance
    Delete {
        #[clap(value_parser)]
        id: String,
    },
    /// Start all instances
    StartAll,
    /// Stop all instances
    StopAll,
    /// Restart all instances
    RestartAll,
    /// Delete all instances
    Prune,
    /// Get the status of an instance
    Status {
        #[clap(long)]
        id: String,
    },
}

async fn pretty_print(language: &str, input: &str) -> Result<()> {
    let config = config::read_or_create_config().await?;
    let color = config.cli_colored_output;
    let theme = config.cli_theme;
    let mut printer = PrettyPrinter::new();
    printer.input_from_bytes(input.as_bytes());
    printer.language(language);
    printer.colored_output(color);
    if let Some(theme) = theme {
        printer.theme(theme);
    }
    printer.print()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AnyhowError> {
    config::pull_docker_images_from_config().await?;
    let cli = Cli::parse();
    match cli.command {
        Commands::List { id } => {
            match id {
                Some(ref id) => {
                    // If an ID is provided, inspect that specific instance
                    let instance = utils::with_spinner(
                        commands::inspect_instance(id),
                        &format!("Getting status for instance {}", id),
                    )
                    .await?;
                    println!("\n");
                    let instance_str = serde_json::to_string_pretty(&instance)?;
                    pretty_print("json", &instance_str).await?;
                }
                None => {
                    // If no ID is provided, inspect all instances
                    let instances = utils::with_spinner(
                        commands::inspect_all_instances(),
                        "Getting statuses for all instances",
                    )
                    .await?;
                    println!("\n");
                    let instances_str = serde_json::to_string_pretty(&instances)?;
                    pretty_print("json", &instances_str).await?;
                }
            }
        }
        Commands::Create { options } => {
            let instance = utils::with_spinner(
                commands::create_instance(options.as_ref()),
                "Creating instance",
            )
            .await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::Start { id } => {
            let instance =
                utils::with_spinner(commands::start_instance(&id), "Starting instance").await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::Stop { id } => {
            let instance =
                utils::with_spinner(commands::stop_instance(&id), "Stopping instance").await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::Restart { id } => {
            let instance =
                utils::with_spinner(commands::restart_instance(&id), "Restarting instance").await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::Delete { id } => {
            let instance =
                utils::with_spinner(commands::delete_instance(&id), "Deleting instance").await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::StartAll => {
            let instance =
                utils::with_spinner(commands::start_all_instances(), "Starting all instances")
                    .await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::StopAll => {
            let instance =
                utils::with_spinner(commands::stop_all_instances(), "Stopping all instances")
                    .await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::RestartAll => {
            let instance = utils::with_spinner(
                commands::restart_all_instances(),
                "Restarting all instances",
            )
            .await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::Prune => {
            let instance =
                utils::with_spinner(commands::delete_all_instances(), "Purging all instances")
                    .await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
        Commands::Status { id } => {
            let instance =
                utils::with_spinner(commands::inspect_instance(&id), "Getting instance status")
                    .await?;
            println!("\n");
            let instance_str = serde_json::to_string_pretty(&instance)?;
            pretty_print("json", &instance_str).await?;
        }
    }

    Ok(())
}
