use serde::{Serialize, Deserialize};
use dirs;
use tokio::fs;
use anyhow::Result;

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
        Ok(config)
    } else {
        let config = AppConfig::default();
        let toml = toml::to_string(&config)?;
        fs::write(&config_path, toml).await?;
        Ok(config)
    }
}
