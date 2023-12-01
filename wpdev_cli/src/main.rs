use anyhow::Result;
use tokio;
mod commands;
use prettytable::{Table, Row, Cell, format};
use prettytable::row;

use clap::{arg, Command};
use serde_json;
use serde_json::Value;

fn print_instances_table(json_data: &Value) {
    let mut table = Table::new();

    // Set table format
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add a title row
    table.add_row(row!["UUID", "Status", "Adminer Port", "Nginx Port", "Container Statuses"]);

    if let Some(instances) = json_data.as_object() {
        for (uuid, details) in instances {
            let status = details["status"].as_str().unwrap_or("Unknown");
            let adminer_port = details["adminer_port"].as_u64().unwrap_or(0);
            let nginx_port = details["nginx_port"].as_u64().unwrap_or(0);

            // Truncate UUID for display
            let display_uuid = uuid.get(..8).unwrap_or(uuid);

            // Summarize container statuses
            let status_summary = details["container_statuses"].as_object()
                .map(|statuses| {
                    let total = statuses.len();
                    let unknown_count = statuses.values()
                        .filter(|status| status.as_str().unwrap_or("") == "Unknown")
                        .count();
                    format!("{}/{} Unknown", unknown_count, total)
                })
                .unwrap_or_else(|| "No Data".to_string());

            table.add_row(Row::new(vec![
                Cell::new(display_uuid),
                Cell::new(status),
                Cell::new(&adminer_port.to_string()),
                Cell::new(&nginx_port.to_string()),
                Cell::new(&status_summary),
            ]));
        }
    }

    // Print the table
    table.printstd();
}

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
                        .arg(arg!(<ID> "Instance ID").required(false))
                )
                .subcommand(
                    Command::new("create")
                        .about("Create a new instance")
                        .arg(arg!(<OPTIONS> "WordPress Options").required(false))
                )
                .subcommand(
                    Command::new("start")
                        .about("Start an instance")
                        .arg(arg!(<ID> "Instance ID").required(true))
                )
                .subcommand(
                    Command::new("stop")
                        .about("Stop an instance")
                        .arg(arg!(<ID> "Instance ID").required(true))
                )
                .subcommand(
                    Command::new("restart")
                        .about("Restart an instance")
                        .arg(arg!(<ID> "Instance ID").required(true))
                )
                .subcommand(
                    Command::new("delete")
                        .about("Delete an instance")
                        .arg(arg!(<ID> "Instance ID").required(true))
                )
                .subcommand(
                    Command::new("start_all")
                        .about("Start all instances")
                )
                .subcommand(
                    Command::new("stop_all")
                        .about("Stop all instances")
                )
                .subcommand(
                    Command::new("restart_all")
                        .about("Restart all instances")
                )
                .subcommand(
                    Command::new("purge")
                        .about("Delete all instances")
                )
                .subcommand(
                    Command::new("status")
                        .about("Get the status of an instance")
                        .arg(arg!(<ID> "Instance ID").required(true))
                )

        )
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("instances", sites_matches)) => {
            match sites_matches.subcommand() {
                Some(("list", _)) => {
                    let instance = commands::list_instances().await?;
                    print_instances_table(&instance);
                },
                Some(("create", create_matches)) => {
                    let options = create_matches.get_one("OPTIONS");
                    let instance = commands::create_instance(options).await?;
                    print_instances_table(&instance);
                },
                Some(("start", start_matches)) => {
                    let id = start_matches.get_one("ID").unwrap();
                    let instance = commands::start_instance(id).await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("stop", stop_matches)) => {
                    let id = stop_matches.get_one("ID").unwrap();
                    let instance = commands::stop_instance(id).await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("restart", restart_matches)) => {
                    let id = restart_matches.get_one("ID").unwrap();
                    let instance = commands::restart_instance(id).await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("delete", delete_matches)) => {
                    let id = delete_matches.get_one("ID").unwrap();
                    let instance = commands::delete_instance(id).await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("start_all", _)) => {
                    let instance = commands::start_all_instances().await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("stop_all", _)) => {
                    let instance = commands::stop_all_instances().await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("restart_all", _)) => {
                    let instance = commands::restart_all_instances().await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("purge", _)) => {
                    let instance = commands::delete_all_instances().await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                Some(("status", status_matches)) => {
                    let id = status_matches.get_one("ID").unwrap();
                    let instance = commands::inspect_instance(id).await?;
                    println!("{}", serde_json::to_string_pretty(&instance)?);
                },
                _ => println!("Invalid command. Please use <help> to a see full list of commands.")
            }
        },
        _ => println!("Invalid command. Please use <help> to a see full list of commands.")
    }
    Ok(())
}
