use crate::config;
use crate::docker::container;
use crate::utils;
use anyhow::{Error as AnyhowError, Result};
use bollard::container::{Config, CreateContainerOptions};
use bollard::models::HostConfig;
use bollard::Docker;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn configure_container(
    instance_label: &str,
    instance_path: &PathBuf,
    container_image: crate::ContainerImage,
    labels: &HashMap<String, String>,
    env_vars: Vec<String>,
    user: Option<String>,
    volume_binding: Option<(Option<PathBuf>, &str)>,
    port: Option<u32>,
) -> Result<(String, crate::ContainerStatus)> {
    let docker = Docker::connect_with_defaults()?;
    let config_dir = instance_path.join(&container_image.to_string());
    let path = utils::create_path(&config_dir).await?;
    let path_str = path
        .to_str()
        .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?;

    let container_labels = utils::create_labels(container_image.clone(), labels.clone());
    let labels_view = container_labels.into_iter().collect();

    let host_config = match volume_binding {
        Some((Some(config_path), container_path)) => {
            let config_path_str = config_path
                .to_str()
                .ok_or_else(|| AnyhowError::msg("Configuration directory not found"))?;
            HostConfig {
                binds: Some(vec![format!("{}:{}", config_path_str, container_path)]),
                ..Default::default()
            }
        }
        Some((None, container_path)) => HostConfig {
            binds: Some(vec![format!("{}:{}", path_str, container_path)]),
            ..Default::default()
        },
        None => HostConfig::default(),
    };

    let mut container_config = Config {
        image: Some(container_image.to_string()),
        env: Some(env_vars),
        labels: Some(labels_view),
        user,
        host_config: Some(host_config),
        ..Default::default()
    };

    if let Some(port) = port {
        let port_key = format!("{}/tcp", port);
        container_config.exposed_ports = Some(HashMap::from([(
            port_key.to_string(),
            HashMap::<(), ()>::new(),
        )]));
    }

    let options = CreateContainerOptions {
        name: format!("{}-{}", instance_label, container_image),
        platform: None,
    };

    let container_ids = &mut Vec::new();

    match docker
        .create_container(Some(options), container_config)
        .await
    {
        Ok(response) => {
            let container_id = response.id;
            container_ids.push(container_id.clone());
            println!(
                "{} container successfully created: {:?}",
                container_image.to_string(),
                container_id
            );

            match container::InstanceContainer::get_status(&docker, &container_id).await {
                Ok(status) => Ok((
                    container_id,
                    status.unwrap_or(crate::ContainerStatus::Unknown),
                )),
                Err(err) => {
                    println!(
                        "Failed to fetch status for container {}: {:?}",
                        container_id, err
                    );
                    Err(err.into())
                }
            }
        }
        Err(err) => {
            println!(
                "Error creating {} container: {:?}",
                container_image.to_string(),
                err
            );
            Err(err.into())
        }
    }
}

pub async fn configure_wordpress_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
) -> Result<(String, crate::ContainerStatus)> {
    let (ids, status) = configure_container(
        instance_label,
        instance_path,
        crate::ContainerImage::Wordpress,
        labels,
        env_vars.wordpress.clone(),
        Some("1000:1000".to_string()),
        Some((None, "/var/www/html/")),
        None,
    )
    .await?;
    Ok((ids, status))
}

pub async fn configure_mysql_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
) -> Result<(String, crate::ContainerStatus)> {
    let (ids, status) = configure_container(
        instance_label,
        instance_path,
        crate::ContainerImage::MySQL,
        labels,
        env_vars.mysql.clone(),
        Some("1000:1000".to_string()),
        Some((None, "/var/run/mysqld/")),
        None,
    )
    .await?;
    Ok((ids, status))
}

pub async fn configure_adminer_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
    adminer_port: u32,
) -> Result<(String, crate::ContainerStatus)> {
    let (ids, status) = configure_container(
        instance_label,
        instance_path,
        crate::ContainerImage::Adminer,
        labels,
        env_vars.adminer.clone(),
        None,
        Some((None, "/var/www/html/")),
        Some(adminer_port),
    )
    .await?;
    Ok((ids, status))
}

pub async fn configure_nginx_container(
    instance_path: &PathBuf,
    instance_label: &str,
    labels: &HashMap<String, String>,
    nginx_port: u32,
) -> Result<(String, crate::ContainerStatus)> {
    let nginx_config_path = config::generate_nginx_config(
        instance_label,
        nginx_port,
        &format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::Adminer.to_string()
        ),
        &format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::Wordpress.to_string()
        ),
        instance_path,
    )
    .await?;
    let (ids, status) = configure_container(
        instance_label,
        instance_path,
        crate::ContainerImage::Nginx,
        labels,
        Vec::new(),
        None,
        Some((Some(nginx_config_path), "/etc/nginx/conf.d/default.conf")),
        Some(nginx_port),
    )
    .await?;

    Ok((ids, status))
}
