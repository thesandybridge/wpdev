use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::network::{CreateNetworkOptions, ListNetworksOptions};
use bollard::Docker;
use futures::stream::StreamExt;
use log::{error, info};
use std::collections::HashMap;
use std::path::PathBuf;

use dirs;

use anyhow::{Error as AnyhowError, Result};
use tokio::fs::{self};

use crate::utils;

pub async fn read_or_create_config() -> Result<crate::AppConfig> {
    let config_dir = dirs::config_dir().unwrap().join("wpdev");
    fs::create_dir_all(&config_dir).await?;

    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        let contents = fs::read_to_string(&config_path).await?;
        let config: crate::AppConfig = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let config = crate::AppConfig::default();
        let toml = toml::to_string(&config)?;
        fs::write(&config_path, toml).await?;
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
pub async fn image_exists(image_name: &str) -> Result<bool> {
    let docker = Docker::connect_with_defaults()?;
    let options = Some(ListImagesOptions::<String> {
        ..Default::default()
    });
    let images = docker.list_images(options).await?;
    Ok(images.iter().any(|image| {
        image
            .repo_tags
            .iter()
            .any(|tag| tag.contains(&image_name.to_string()))
    }))
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
/// pull_docker_image_if_not_exists("wordpress:latest").await?;
/// ```
async fn pull_docker_image_if_not_exists(image_name: &str) -> Result<()> {
    let image = image_exists(image_name).await?;
    if !image {
        let docker = Docker::connect_with_defaults()?;
        let options = CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        };
        let mut stream = docker.create_image(Some(options), None, None);

        let mut success = false;
        let mut error_message = None;

        // Process each event in the pull stream
        while let Some(result) = stream.next().await {
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

/// Creates a Docker Network if it doesn't already exist.
///
/// # Arguments
///
/// * `docker` - &Docker
/// * `network_name` - name of the network
pub async fn create_network_if_not_exists(
    docker: &Docker,
    network_name: Option<&str>,
) -> Result<()> {
    let network_list_options = Some(ListNetworksOptions::<String> {
        ..Default::default()
    });
    let networks = docker.list_networks(network_list_options).await?;
    if !networks.iter().any(|network| {
        network
            .name
            .as_ref()
            .map(|name| name == network_name.unwrap_or("wpdev"))
            .unwrap_or(false)
    }) {
        let options = CreateNetworkOptions {
            name: network_name.unwrap_or("wpdev"),
            ..Default::default()
        };
        docker.create_network(options).await?;
    }
    Ok(())
}

fn merge_env_vars(
    defaults: HashMap<String, String>,
    overrides: &Option<HashMap<String, String>>,
) -> Vec<String> {
    let mut env_vars = defaults;

    if let Some(overrides) = overrides {
        for (key, value) in overrides.iter() {
            env_vars.insert(key.clone(), value.clone());
        }
    }

    env_vars
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect()
}

pub async fn initialize_env_vars(
    instance_label: &str,
    user_env_vars: &crate::ContainerEnvVars,
) -> Result<crate::EnvVars, AnyhowError> {
    let default_adminer_vars = HashMap::from([
        ("ADMINER_DESIGN".to_string(), "nette".to_string()),
        (
            "ADMINER_PLUGINS".to_string(),
            "tables-filter tinymce".to_string(),
        ),
        ("MYSQL_PORT".to_string(), "3306".to_string()),
        (
            "ADMINER_DEFAULT_SERVER".to_string(),
            format!("{}-mysql", instance_label).to_string(),
        ),
        (
            "ADMINER_DEFAULT_USERNAME".to_string(),
            "wordpress".to_string(),
        ),
        (
            "ADMINER_DEFAULT_PASSWORD".to_string(),
            "password".to_string(),
        ),
        (
            "ADMINER_DEFAULT_DATABASE".to_string(),
            "wordpress".to_string(),
        ),
    ]);

    let default_mysql_vars = HashMap::from([
        ("MYSQL_ROOT_PASSWORD".to_string(), "password".to_string()),
        ("MYSQL_DATABASE".to_string(), "wordpress".to_string()),
        ("MYSQL_USER".to_string(), "wordpress".to_string()),
        ("MYSQL_PASSWORD".to_string(), "password".to_string()),
    ]);

    let default_wordpress_vars = HashMap::from([
        (
            "WORDPRESS_DB_HOST".to_string(),
            format!("{}-mysql", instance_label).to_string(),
        ),
        ("WORDPRESS_DB_USER".to_string(), "wordpress".to_string()),
        ("WORDPRESS_DB_PASSWORD".to_string(), "password".to_string()),
        ("WORDPRESS_DB_NAME".to_string(), "wordpress".to_string()),
        ("WORDPRESS_TABLE_PREFIX".to_string(), "wp_".to_string()),
        ("WORDPRESS_DEBUG".to_string(), "1".to_string()),
        ("WORDPRESS_CONFIG_EXTRA".to_string(), "".to_string()),
    ]);

    let adminer_env_vars = merge_env_vars(default_adminer_vars, &None);
    let mysql_env_vars = merge_env_vars(default_mysql_vars, &None);
    let wordpress_env_vars = merge_env_vars(default_wordpress_vars, &user_env_vars.wordpress);

    Ok(crate::EnvVars {
        adminer: adminer_env_vars,
        mysql: mysql_env_vars,
        wordpress: wordpress_env_vars,
    })
}

pub async fn generate_nginx_config(
    instance_label: &str,
    nginx_port: u32,
    adminer_name: &str,
    wordpress_name: &str,
    instance_dir: &PathBuf,
) -> Result<PathBuf, AnyhowError> {
    let nginx_config = format!(
        r#"
server {{
    listen {nginx_port};
    server_name localhost;

    location / {{
        proxy_pass http://{wordpress_name}:80/;
        proxy_set_header Host $host:$server_port;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}

server {{
    listen 8080;
    server_name localhost;

    location / {{
        proxy_pass http://{adminer_name}:8080/;
        proxy_set_header Host $host:$server_port;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}
        "#,
        nginx_port = nginx_port,
        wordpress_name = wordpress_name,
        adminer_name = adminer_name,
    );

    let instance_path = instance_dir.join("nginx");
    utils::create_path(&instance_path).await?;
    let nginx_config_path = instance_path.join(format!("{}-nginx.conf", instance_label));
    fs::write(&nginx_config_path, nginx_config).await?;

    Ok(nginx_config_path)
}

pub async fn generate_wpcli_config(
    config: &crate::AppConfig,
    instance_label: &str,
    home_dir: &PathBuf,
) -> Result<(), AnyhowError> {
    let instance_dir = home_dir.join(format!("{}/{}/", &config.custom_root, instance_label));
    let wpcli_yml = format!(
        r#"path: app
require:
  - wp-cli.local.php
        "#,
    );

    let wpcli_php = format!(
        r#"<?php

define('DB_HOST', 'localhost:{instance_dir}mysql/mysqld.sock');
define('DB_NAME', 'wordpress');
define('DB_USER', 'wordpress');
define('DB_PASSWORD', 'password');

// disables errors when using wp-cli
error_reporting(E_ERROR);
define('WP_DEBUG', false);
        "#,
        instance_dir = instance_dir
            .to_str()
            .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?,
    );

    let instance_dir = home_dir.join(format!("{}/{}/", &config.custom_root, instance_label));
    utils::create_path(&instance_dir).await?;
    let wpcli_yml_path = instance_dir.join("wp-cli.local.yml");
    let wpcli_php_path = instance_dir.join("wp-cli.local.php");
    fs::write(&wpcli_yml_path, wpcli_yml).await?;
    fs::write(&wpcli_php_path, wpcli_php).await?;

    Ok(())
}
