use spinners::{Spinner, Spinners};
use std::future::Future;
use std::{thread, time::Duration};
use std::io::{self, Write};
use prettytable::{Table, Row, Cell, format};
use prettytable::row;
use serde_json::Value;

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
    table.add_row(row!["UUID", "Status", "Adminer Port", "Nginx Port", "Container"]);

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
                    statuses.iter()
                        .map(|(_, status)| {
                            format!("{}", status.as_str().unwrap_or("Unknown"))
                        })
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
