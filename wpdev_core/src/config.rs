use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::network::CreateNetworkOptions;
use bollard::Docker;
use futures::stream::StreamExt;
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;

use dirs;

use anyhow::{Context, Error as AnyhowError, Result};
use tokio::fs::{self};

use crate::docker::container::{ContainerEnvVars, ContainerImage, EnvVars};
use crate::docker::instance::InstanceData;
use crate::utils;
use crate::AppConfig;

pub async fn read_or_create_config() -> Result<crate::AppConfig> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Failed to find config directory"))?;
    let config_dir = config_dir.join("wpdev");
    fs::create_dir_all(&config_dir)
        .await
        .context("Failed to create config directory")?;

    let config_path = config_dir.join("config.toml");

    match fs::read_to_string(&config_path).await {
        Ok(contents) => {
            let config: AppConfig = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file at {:?}", config_path))?;
            Ok(config)
        }
        Err(_) => Ok(AppConfig::default()),
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

        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => {
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
    let config = read_or_create_config()
        .await
        .context("Failed to read config")?;

    if config.docker_images.is_empty() {
        return Ok(());
    }

    for image_name in config.docker_images {
        pull_docker_image_if_not_exists(&image_name)
            .await
            .context(format!("Failed to pull image {}", image_name))?;
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
    network_prefix: &str,
    id: &str,
) -> Result<()> {
    let network_name = format!("{}-{}", network_prefix, id);
    let options = CreateNetworkOptions {
        name: network_name,
        driver: "bridge".to_string(),
        check_duplicate: true,
        ..Default::default()
    };
    docker
        .create_network(options)
        .await
        .context("Failed to create network")?;
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
    user_env_vars: &ContainerEnvVars,
) -> Result<EnvVars, AnyhowError> {
    let default_adminer_vars = HashMap::from([
        ("ADMINER_DESIGN".to_string(), "nette".to_string()),
        (
            "ADMINER_PLUGINS".to_string(),
            "tables-filter tinymce".to_string(),
        ),
        ("MYSQL_PORT".to_string(), "3306".to_string()),
        (
            "ADMINER_DEFAULT_SERVER".to_string(),
            format!("{}-{}", instance_label, ContainerImage::MySQL.to_string()).to_string(),
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
            format!("{}-{}", instance_label, ContainerImage::MySQL.to_string()).to_string(),
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

    Ok(EnvVars {
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
    utils::create_path(&instance_path)
        .await
        .context("Failed to create nginx directory")?;
    let nginx_config_path = instance_path.join(format!("{}-nginx.conf", instance_label));
    fs::write(&nginx_config_path, nginx_config)
        .await
        .context(format!(
            "Failed to write nginx config to {:?}",
            nginx_config_path
        ))?;

    Ok(nginx_config_path)
}

pub async fn generate_wpcli_config(
    config: &crate::AppConfig,
    instance_label: &str,
    home_dir: &PathBuf,
) -> Result<(), AnyhowError> {
    let instance_dir = home_dir.join(format!(
        "{}/{}-{}/",
        &config.custom_root,
        crate::NETWORK_NAME,
        instance_label
    ));
    let wpcli_yml = format!(
        r#"path: wordpress
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

    utils::create_path(&instance_dir)
        .await
        .context("Failed to create instance directory")?;
    let wpcli_yml_path = instance_dir.join("wp-cli.local.yml");
    let wpcli_php_path = instance_dir.join("wp-cli.local.php");
    fs::write(&wpcli_yml_path, wpcli_yml)
        .await
        .context(format!(
            "Failed to write wp-cli config to {:?}",
            wpcli_yml_path
        ))?;
    fs::write(&wpcli_php_path, wpcli_php)
        .await
        .context(format!(
            "Failed to write wp-cli config to {:?}",
            wpcli_php_path
        ))?;

    Ok(())
}

pub async fn read_instance_data_from_toml(instance_label: &str) -> Result<InstanceData> {
    let config = read_or_create_config()
        .await
        .context("Failed to read config")?;
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let instance_dir = home_dir
        .join(&config.custom_root)
        .join(format!("{}/instance.toml", instance_label));

    if !instance_dir.exists() {
        return Err(AnyhowError::msg(format!(
            "Instance file not found at {:?}",
            instance_dir
        )));
    }

    let contents = fs::read_to_string(&instance_dir).await.context(format!(
        "Failed to read instance file at {:?}",
        instance_dir
    ))?;

    let instance_data: InstanceData = toml::from_str(&contents).context(format!(
        "Failed to parse instance data from file at {:?}",
        instance_dir
    ))?;

    Ok(instance_data)
}

pub async fn parse_instance_data(
    env_vars: &EnvVars,
    nginx_port: &u32,
    adminer_port: &u32,
    config: &crate::AppConfig,
    home_dir: &PathBuf,
    instance_label: &str,
) -> Result<InstanceData> {
    let instance_dir = home_dir.join(format!(
        "{}/{}-{}/instance.toml",
        &config.custom_root,
        crate::NETWORK_NAME,
        instance_label
    ));

    fn extract_value(vars: &Vec<String>, key: &str) -> String {
        vars.iter()
            .find_map(|s| {
                let parts: Vec<&str> = s.splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == key {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "defaultValue".to_string())
    }

    let instance_data = InstanceData {
        admin_user: extract_value(&env_vars.wordpress, "WORDPRESS_DB_USER"),
        admin_password: extract_value(&env_vars.wordpress, "WORDPRESS_DB_PASSWORD"),
        admin_email: "admin@example.com".to_string(),
        site_title: "My Wordpress Site".to_string(),
        site_url: format!("{}:{}", config.site_url, &nginx_port),
        adminer_url: format!("{}:{}", config.adminer_url, &adminer_port),
        adminer_user: extract_value(&env_vars.adminer, "ADMINER_DEFAULT_USERNAME"),
        adminer_password: extract_value(&env_vars.adminer, "ADMINER_DEFAULT_PASSWORD"),
        network_name: format!("{}-{}", crate::NETWORK_NAME, instance_label),
        nginx_port: *nginx_port,
        adminer_port: *adminer_port,
    };

    fs::write(&instance_dir, toml::to_string(&instance_data)?)
        .await
        .context(format!(
            "Failed to write instance data to {:?}",
            instance_dir
        ))?;
    info!("Instance data written to {:?}", instance_dir);

    Ok(instance_data)
}
