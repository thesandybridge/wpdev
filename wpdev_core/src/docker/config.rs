use crate::config;
use crate::utils;
use anyhow::{Error as AnyhowError, Result};
use bollard::container::{Config, CreateContainerOptions};
use bollard::models::HostConfig;
use bollard::Docker;
use std::collections::HashMap;
use std::path::PathBuf;

async fn configure_container(
    instance_label: &str,
    container_image: crate::ContainerImage,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: Vec<String>,
    user: Option<String>,
    volume_binding: &str,
) -> Result<()> {
    let docker = Docker::connect_with_defaults()?;
    let config_dir = instance_path.join(&container_image.to_string());
    let path = utils::create_path(&config_dir).await?;
    let path_str = path
        .to_str()
        .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?;

    let container_labels = utils::create_labels(container_image.clone(), labels.clone());
    let labels_view: HashMap<String, String> = container_labels
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let host_config = HostConfig {
        binds: Some(vec![format!("{}:{}", path_str, volume_binding)]),
        ..Default::default()
    };

    let container_config = Config {
        image: Some(container_image.to_string()),
        env: Some(env_vars),
        labels: Some(labels_view),
        user,
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: format!("{:?}-{:?}", instance_label, container_image),
        ..Default::default()
    };

    docker
        .create_container(Some(options), container_config)
        .await?;
    Ok(())
}

pub async fn configure_wordpress_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
) -> Result<()> {
    configure_container(
        instance_label,
        crate::ContainerImage::Wordpress,
        instance_path,
        labels,
        env_vars.wordpress.clone(),
        Some("1000:1000".to_string()),
        "/var/www/html/",
    )
    .await?;
    Ok(())
}

pub async fn configure_mysql_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
) -> Result<()> {
    configure_container(
        instance_label,
        crate::ContainerImage::MySQL,
        instance_path,
        labels,
        env_vars.mysql.clone(),
        Some("1000:1000".to_string()),
        "/var/run/mysqld/",
    )
    .await?;
    Ok(())
}

pub async fn configure_adminer_container(
    instance_label: &str,
    labels: &HashMap<String, String>,
    instance_path: &PathBuf,
    env_vars: &crate::EnvVars,
) -> Result<()> {
    configure_container(
        instance_label,
        crate::ContainerImage::Adminer,
        instance_path,
        labels,
        env_vars.adminer.clone(),
        None,
        "/var/www/html/",
    )
    .await?;
    Ok(())
}

pub async fn configure_nginx_container(
    instance_path: &PathBuf,
    instance_label: &str,
    labels: &HashMap<String, String>,
    nginx_port: u32,
) -> Result<()> {
    let docker = Docker::connect_with_defaults()?;
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
    let nginx_labels = utils::create_labels(crate::ContainerImage::Nginx, labels.clone());
    let nginx_labels_view: HashMap<_, _> = nginx_labels
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let nginx_path_str = nginx_config_path
        .to_str()
        .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?;
    let host_config = HostConfig {
        binds: Some(vec![format!(
            "{}:/etc/nginx/conf.d/default.conf",
            nginx_path_str
        )]),
        ..Default::default()
    };
    let nginx_options = Some(CreateContainerOptions {
        name: format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::Nginx.to_string()
        ),
        platform: None,
    });
    let exposed_port_key = format!("{}/tcp", nginx_port);
    let nginx_config = Config {
        image: Some(crate::NGINX_IMAGE),
        labels: Some(nginx_labels_view),
        host_config: Some(host_config),
        exposed_ports: Some(HashMap::from([(
            exposed_port_key.as_str(),
            HashMap::<(), ()>::new(),
        )])),
        ..Default::default()
    };

    docker.create_container(nginx_options, nginx_config).await?;
    Ok(())
}
