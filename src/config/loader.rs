use serde::{Serialize, Deserialize};
use dirs;
use anyhow::Result;
use std::path::Path;
use tokio::fs::{self, OpenOptions};
use tokio::io::{self, AsyncWriteExt, BufReader, AsyncBufReadExt};
use log::info;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub custom_root: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            custom_root: String::from(".config/wpdev/instances"),
        }
    }
}

pub async fn read_or_create_config() -> Result<AppConfig> {
    let config_dir = dirs::config_dir().unwrap().join("wpdev");
    fs::create_dir_all(&config_dir).await?;

    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        let contents = fs::read_to_string(&config_path).await?;
        let config: AppConfig = toml::from_str(&contents)?;
        info!("Loaded config from {:?}", config_path);
        Ok(config)
    } else {
        let config = AppConfig::default();
        let toml = toml::to_string(&config)?;
        fs::write(&config_path, toml).await?;
        info!("Created config file at {:?}", config_path);
        Ok(config)
    }
}

#[derive(PartialEq)]
pub enum HostsFileAction {
    Add,
    Remove,
    Purge,
}

pub async fn update_hosts_file(instance_label: &str, action: HostsFileAction) -> io::Result<()> {
    let hosts_path = Path::new("/etc/hosts"); // Adjust for Windows if necessary
    let temp_path = hosts_path.with_extension("tmp");

    let reader = BufReader::new(fs::File::open(&hosts_path).await?);
    let mut writer = OpenOptions::new().write(true).create(true).open(&temp_path).await?;

    let start_marker = "# START WP_DEV";
    let end_marker = "# END WP_DEV";
    let entry = format!("127.0.0.1 {}.local", instance_label);

    let mut in_custom_block = false;
    let mut entry_handled = false;

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim() == start_marker {
            in_custom_block = true;

            if action == HostsFileAction::Add {
                writer.write_all(line.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.write_all(entry.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                entry_handled = true;
                info!("Added entry for instance '{}'", instance_label);
            } else if action != HostsFileAction::Purge {
                writer.write_all(line.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
        } else if line.trim() == end_marker {
            in_custom_block = false;
            if action != HostsFileAction::Purge {
                writer.write_all(line.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
            if action == HostsFileAction::Remove {
                info!("Removed entry for instance '{}'", instance_label);
            }
        } else if !in_custom_block || (action == HostsFileAction::Remove && line.trim() != entry) {
            writer.write_all(line.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }
    }

    if action == HostsFileAction::Add && !entry_handled {
        writer.write_all(format!("\n{}\n{}\n{}", start_marker, entry, end_marker).as_bytes()).await?;
        info!("Added new block for instance '{}'", instance_label);
    }

    if action == HostsFileAction::Purge {
        info!("Purged WP_DEV block from hosts file");
    }

    fs::rename(temp_path, hosts_path).await?;
    Ok(())
}

