use serde::{Serialize, Deserialize};
use shiplift::{Docker, PullOptions, ImageListOptions};
use futures::stream::StreamExt;

use dirs;

use anyhow::{Result, Error as AnyhowError};
use tokio::fs::{self};

use log::info;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub custom_root: String,
    pub docker_images: Vec<String>,
    pub enable_logging: bool,
    pub enable_frontend: bool,
    pub site_url: String,
    pub adminer_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            custom_root: String::from(".config/wpdev/instances"),
            docker_images: vec![
                "wordpress:latest".into(),
                "nginx:latest".into(),
                "mysql:latest".into(),
                "adminer:latest".into(),
                "wordpress:cli".into(),
            ],
            enable_logging: true,
            enable_frontend: false,
            site_url: String::from("http://localhost"),
            adminer_url: String::from("http://localhost"),
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

/// Check if a Docker image exists locally.
///
/// # Arguments
///
/// * `image_name` - The name of the image to check for.
///
/// # Examples
///
/// ```
/// let image_exists = image_exists("wordpress:latest").await;
/// ```
pub async fn image_exists(image_name: &str) -> bool {
    let docker = Docker::new();
    let options = ImageListOptions::default();
    let images = docker.images().list(&options).await.unwrap();
    images.iter().any(|image| {
        image.repo_tags.iter().any(|tag| tag.contains(&image_name.to_string())) // Convert image_name to String
    })
}


/// Pull a Docker image if it does not already exist locally.
///
/// # Arguments
///
/// * `image_name` - The name of the image to pull.
///
/// # Errors
///
/// * If the image fails to pull.
/// * If the image is not found.
/// * If the image is not valid.
/// * If the image is not authorized.
/// * If the image is not available.
/// * If the image is not ready.
/// * If the image is not a Docker image.
///
///
/// # Examples
///
/// ```
/// pull_docker_image_if_not_exists("wordpress:latest").await;
/// ```
async fn pull_docker_image_if_not_exists(image_name: &str) -> Result<(), shiplift::errors::Error> {
    if !image_exists(image_name).await {
        let docker = Docker::new();
        let mut pull_options = PullOptions::builder();
        pull_options.image(image_name);
        let mut pull_stream = docker.images().pull(&pull_options.build());

        let mut success = false;
        let mut error_message = None;

        // Process each event in the pull stream
        while let Some(result) = pull_stream.next().await {
            match result {
                Ok(_) => {
                    // Image successfully pulled
                    success = true;
                }
                Err(err) => {
                    error_message = Some(format!("Error pulling image: {:?}", err));
                }
            }
        }

        if success {
            println!("Image {} is now available locally.", image_name);
        } else {
            if let Some(message) = error_message {
                eprintln!("{}", message);
            } else {
                eprintln!("Failed to pull image {}.", image_name);
            }
        }
    }

    Ok(())
}

pub async fn pull_docker_images_from_config() -> Result<(), AnyhowError> {
    let config = read_or_create_config().await?;

    if config.docker_images.is_empty() {
        return Ok(());
    }

    for image_name in config.docker_images {
        pull_docker_image_if_not_exists(&image_name).await?;
    }

    Ok(())
}
