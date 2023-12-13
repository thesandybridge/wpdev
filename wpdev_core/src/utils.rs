use anyhow::{Error as AnyhowError, Result};
use prettytable::row;
use prettytable::{format, Cell, Row, Table};
use serde_json::Value;
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::future::Future;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpListener};
use std::{thread, time::Duration};

use std::path::PathBuf;
use tokio::fs;

pub async fn with_spinner<F, T, E>(future: F, message: &str) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    // Flush stdout before starting the spinner
    io::stdout().flush().unwrap();

    let mut sp = Spinner::new(Spinners::Dots9, message.into());
    let result = future.await;
    sp.stop();

    // Short delay to ensure the spinner is cleared
    thread::sleep(Duration::from_millis(100));

    result
}

pub fn print_instances_table(json_data: &Value) {
    let mut table = Table::new();

    // Set table format
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    // Add a title row
    table.add_row(row![
        "UUID",
        "Status",
        "Adminer Port",
        "Nginx Port",
        "Container"
    ]);

    if let Some(instances) = json_data.as_object() {
        for (uuid, details) in instances {
            let status = details["status"].as_str().unwrap_or("Unknown");
            let adminer_port = details["adminer_port"].as_u64().unwrap_or(0);
            let nginx_port = details["nginx_port"].as_u64().unwrap_or(0);

            // Truncate UUID for display
            let display_uuid = uuid.get(..8).unwrap_or(uuid);

            // Summarize container statuses
            let status_summary = details["container_statuses"]
                .as_object()
                .map(|statuses| {
                    statuses
                        .iter()
                        .map(|(_, status)| format!("{}", status.as_str().unwrap_or("Unknown")))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_else(|| "No Data".to_string());

            table.add_row(Row::new(vec![
                Cell::new(format!("{}..", display_uuid).as_str()),
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

pub async fn create_path(path: &PathBuf) -> Result<&PathBuf, AnyhowError> {
    fs::create_dir_all(&path).await?;
    Ok(path)
}

/// Parses a port from a label, providing a default value if necessary.
pub fn parse_port(port_label: Option<&String>) -> Result<u32> {
    let port = port_label
        .and_then(|port| port.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(port)
}

pub async fn find_free_port() -> Result<u32, AnyhowError> {
    // Bind to port 0; the OS will assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let socket_addr: SocketAddr = listener.local_addr()?;
    let port = socket_addr.port();

    Ok(u32::from(port))
}

pub fn create_labels(
    image: crate::ContainerImage,
    hashmap: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut new_labels = hashmap.clone();
    new_labels.insert("image".to_string(), image.to_string());
    new_labels
}
