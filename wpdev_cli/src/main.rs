use anyhow::Result;
use tokio;
mod commands;
use wpdev_core::config;
use wpdev_core::utils;

use anyhow::Context;
use bat::PrettyPrinter;
use clap::{Args, Parser, Subcommand};
use env_logger;
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
    /// List instances. If an ID is provided, details for that instance are shown. If -a is provided, lists all instances.
    List(InstanceArgs),
    /// Create a new instance
    Create {
        #[clap(value_parser, group = "listing")]
        options: Option<String>,
    },
    /// Start instances. If an ID is provided, starts that instance. If -a is provided, starts all instances.
    Start(InstanceArgs),
    /// Stop instances. If an ID is provided, stops that instance. If -a is provided, stops all instances.
    Stop(InstanceArgs),
    /// Restart instances. If an ID is provided, restarts that instance. If -a is provided, restarts all instances.
    Restart(InstanceArgs),
    /// Prune instances. If an ID is provided, prune that instance. If -a is provided, prune all instances.
    Prune(InstanceArgs),
    /// Get the status of an instance
    Status {
        #[clap(long)]
        id: String,
    },
}

#[derive(Args, Debug)]
struct InstanceArgs {
    /// Instance ID
    #[clap(value_parser, required_unless_present = "all")]
    id: Option<String>,

    /// Operate on all instances
    #[clap(short = 'a', long, action = clap::ArgAction::SetTrue, conflicts_with = "id")]
    all: bool,
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
async fn main() -> Result<()> {
    let config = config::read_or_create_config()
        .await
        .context("Failed to read or create config")?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config.log_level))
        .init();
    config::pull_docker_images_from_config().await?;
    let cli = Cli::parse();
    match cli.command {
        Commands::List(args) => {
            if args.all {
                let instances =
                    utils::with_spinner(commands::inspect_all_instances(), "Listing instances")
                        .await?;
                println!("\n");
                let instances_str = serde_json::to_string_pretty(&instances)?;
                pretty_print("json", &instances_str).await?;
            } else if let Some(id) = args.id {
                let instance = utils::with_spinner(
                    commands::inspect_instance(&id),
                    "Getting instance details",
                )
                .await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
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
        Commands::Start(args) => {
            if args.all {
                let instance =
                    utils::with_spinner(commands::start_all_instances(), "Starting all instances")
                        .await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            } else if let Some(id) = args.id {
                let instance =
                    utils::with_spinner(commands::start_instance(&id), "Starting instance").await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            }
        }
        Commands::Stop(args) => {
            if args.all {
                let instance =
                    utils::with_spinner(commands::stop_all_instances(), "Stopping all instances")
                        .await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            } else if let Some(id) = args.id {
                let instance =
                    utils::with_spinner(commands::stop_instance(&id), "Stopping instance").await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            }
        }
        Commands::Restart(args) => {
            if args.all {
                let instance = utils::with_spinner(
                    commands::restart_all_instances(),
                    "Restarting all instances",
                )
                .await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            } else if let Some(id) = args.id {
                let instance =
                    utils::with_spinner(commands::restart_instance(&id), "Restarting instance")
                        .await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            }
        }
        Commands::Prune(args) => {
            if args.all {
                let instance =
                    utils::with_spinner(commands::delete_all_instances(), "Pruning all instances")
                        .await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            } else if let Some(id) = args.id {
                let instance =
                    utils::with_spinner(commands::delete_instance(&id), "Pruning instance").await?;
                println!("\n");
                let instance_str = serde_json::to_string_pretty(&instance)?;
                pretty_print("json", &instance_str).await?;
            }
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
