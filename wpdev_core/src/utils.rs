use anyhow::{Context, Result};
use prettytable::row;
use prettytable::{format, Cell, Row, Table};
use serde_json::Value;
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::future::Future;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpListener};
use std::{thread, time::Duration};

use crate::docker::container::ContainerImage;
use std::path::PathBuf;
use tokio::fs;

pub async fn with_spinner<F, T, E>(future: F, message: &str) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    let _ = io::stdout().flush();
    let mut sp = Spinner::new(Spinners::Dots9, message.into());
    let result = future.await;
    sp.stop();

    thread::sleep(Duration::from_millis(100));

    result
}

pub fn print_instances_table(json_data: &Value) {
    let mut table = Table::new();

    table.set_format(*format::consts::FORMAT_BOX_CHARS);

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

            let display_uuid = uuid.get(..8).unwrap_or(uuid);

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

    table.printstd();
}

pub async fn create_path(path: &PathBuf) -> Result<&PathBuf> {
    fs::create_dir_all(&path).await.context(format!(
        "Failed to create directory at path: {}",
        path.to_string_lossy()
    ))?;
    Ok(path)
}

pub fn parse_port(port_label: Option<&String>) -> Result<u32> {
    let port = port_label
        .and_then(|port| port.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(port)
}

pub async fn find_free_port() -> Result<u32> {
    // Bind to port 0; the OS will assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let socket_addr: SocketAddr = listener.local_addr()?;
    let port = socket_addr.port();

    Ok(u32::from(port))
}

pub fn create_labels(
    image: ContainerImage,
    hashmap: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut new_labels = hashmap.clone();
    new_labels.insert("image".to_string(), image.to_string());
    new_labels
}
