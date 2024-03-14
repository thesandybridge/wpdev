use anyhow::{Context, Result};
use log::info;
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

pub async fn create_path(path: &PathBuf) -> Result<&PathBuf> {
    info!("Creating directory at path: {}", path.to_string_lossy());
    fs::create_dir_all(&path).await.context(format!(
        "Failed to create directory at path: {}",
        path.to_string_lossy()
    ))?;
    Ok(path)
}

pub fn parse_port(port_label: Option<&String>) -> Result<u32> {
    info!("Parsing port from label: {:?}", port_label);
    let port = port_label
        .and_then(|port| port.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(port)
}

pub async fn find_free_port() -> Result<u32> {
    info!("Finding a free port");
    let listener = TcpListener::bind("127.0.0.1:0").context("Failed to bind to port")?;
    let socket_addr: SocketAddr = listener
        .local_addr()
        .context("Failed to get local address")?;
    let port = socket_addr.port();

    Ok(u32::from(port))
}

pub fn create_labels(
    image: ContainerImage,
    hashmap: HashMap<String, String>,
) -> HashMap<String, String> {
    info!("Creating labels for image: {:?}", image);
    let mut new_labels = hashmap.clone();
    new_labels.insert("image".to_string(), image.to_string());
    new_labels
}
