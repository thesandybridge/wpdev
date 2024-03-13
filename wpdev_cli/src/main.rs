use anyhow::{Error as AnyhowError, Result};
use tokio;
mod commands;
use wpdev_core::config;
use wpdev_core::utils;

use clap::{arg, Command};
use serde_json;

fn cli() -> Command {
    Command::new("wpdev")
        .about("A CLI for managing WordPress development environments.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("instances")
                .about("Manage instances")
                .after_help("Use 'wpdev instances <SUBCOMMAND> --help' for more information")
                .subcommand(
                    Command::new("list")
                        .about("List instances")
                        .arg(arg!(<ID> "Instance ID").required(false)),
                )
                .subcommand(
                    Command::new("create")
                        .about("Create a new instance")
                        .arg(arg!(<OPTIONS> "WordPress Options").required(false)),
                )
                .subcommand(
                    Command::new("start")
                        .about("Start an instance")
                        .arg(arg!(<ID> "Instance ID").required(true)),
                )
                .subcommand(
                    Command::new("stop")
                        .about("Stop an instance")
                        .arg(arg!(<ID> "Instance ID").required(true)),
                )
                .subcommand(
                    Command::new("restart")
                        .about("Restart an instance")
                        .arg(arg!(<ID> "Instance ID").required(true)),
                )
                .subcommand(
                    Command::new("delete")
                        .about("Delete an instance")
                        .arg(arg!(<ID> "Instance ID").required(true)),
                )
                .subcommand(Command::new("start_all").about("Start all instances"))
                .subcommand(Command::new("stop_all").about("Stop all instances"))
                .subcommand(Command::new("restart_all").about("Restart all instances"))
                .subcommand(Command::new("purge").about("Delete all instances"))
                .subcommand(
                    Command::new("status")
                        .about("Get the status of an instance")
                        .arg(arg!(<ID> "Instance ID").required(true)),
                ),
        )
}

#[tokio::main]
async fn main() -> Result<(), AnyhowError> {
    config::pull_docker_images_from_config().await?;

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("instances", sites_matches)) => match sites_matches.subcommand() {
            Some(("create", create_matches)) => {
                let options = create_matches.get_one("OPTIONS");
                let instance =
                    utils::with_spinner(commands::create_instance(options), "Creating instance")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("start", start_matches)) => {
                let id = start_matches.get_one("ID").unwrap();
                let instance =
                    utils::with_spinner(commands::start_instance(id), "Starting instance").await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("stop", stop_matches)) => {
                let id = stop_matches.get_one("ID").unwrap();
                let instance =
                    utils::with_spinner(commands::stop_instance(id), "Stopping instance").await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("restart", restart_matches)) => {
                let id = restart_matches.get_one("ID").unwrap();
                let instance =
                    utils::with_spinner(commands::restart_instance(id), "Restarting instance")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("delete", delete_matches)) => {
                let id = delete_matches.get_one("ID").unwrap();
                let instance =
                    utils::with_spinner(commands::delete_instance(id), "Deleting instance").await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("start_all", _)) => {
                let instance =
                    utils::with_spinner(commands::start_all_instances(), "Starting all instances")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("stop_all", _)) => {
                let instance =
                    utils::with_spinner(commands::stop_all_instances(), "Stopping all instances")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("restart_all", _)) => {
                let instance = utils::with_spinner(
                    commands::restart_all_instances(),
                    "Restarting all instances",
                )
                .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("purge", _)) => {
                let instance =
                    utils::with_spinner(commands::delete_all_instances(), "Purging all instances")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("status", status_matches)) => {
                let id = status_matches.get_one("ID").unwrap();
                let instance =
                    utils::with_spinner(commands::inspect_instance(id), "Getting instance status")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            Some(("list", _)) => {
                let instance =
                    utils::with_spinner(commands::inspect_all_instances(), "Getting instances")
                        .await?;
                println!("\n{}", serde_json::to_string_pretty(&instance)?);
            }
            _ => println!("Invalid command. Please use <help> to a see full list of commands."),
        },
        _ => println!("Invalid command. Please use <help> to a see full list of commands."),
    }
    Ok(())
}
