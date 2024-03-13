use crate::config;
use crate::docker::container;
use crate::utils;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::docker::container::{ContainerImage, ContainerStatus, EnvVars};

pub async fn configure_wordpress_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &EnvVars,
) -> Result<(String, ContainerStatus)> {
    let wordpress_config_dir = instance_path.join("wordpress");
    let wordpress_path = utils::create_path(&wordpress_config_dir)
        .await
        .context("Failed to create wordpress directory")?;
    let (ids, status) = container::InstanceContainer::new(
        instance_label,
        instance_path,
        ContainerImage::Wordpress,
        labels,
        env_vars.wordpress.clone(),
        Some("1000:1000".to_string()),
        Some((Some(wordpress_path.to_path_buf()), "/var/www/html/")),
        None,
    )
    .await?;
    Ok((ids, status))
}

pub async fn configure_mysql_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &EnvVars,
) -> Result<(String, ContainerStatus)> {
    let mysql_config_dir = instance_path.join("mysql");
    let mysql_socket_path = utils::create_path(&mysql_config_dir)
        .await
        .context("Failed to create mysql directory")?;
    let (ids, status) = container::InstanceContainer::new(
        instance_label,
        instance_path,
        ContainerImage::MySQL,
        labels,
        env_vars.mysql.clone(),
        Some("1000:1000".to_string()),
        Some((Some(mysql_socket_path.to_path_buf()), "/var/run/mysqld")),
        None,
    )
    .await?;
    Ok((ids, status))
}

pub async fn configure_adminer_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &EnvVars,
    adminer_port: u32,
) -> Result<(String, ContainerStatus)> {
    let (ids, status) = container::InstanceContainer::new(
        instance_label,
        instance_path,
        ContainerImage::Adminer,
        labels,
        env_vars.adminer.clone(),
        None,
        None,
        Some((adminer_port, 8080)),
    )
    .await?;
    Ok((ids, status))
}

pub async fn configure_nginx_container(
    instance_path: &PathBuf,
    instance_label: &str,
    labels: &HashMap<String, String>,
    nginx_port: u32,
) -> Result<(String, ContainerStatus)> {
    let nginx_config_path = config::generate_nginx_config(
        instance_label,
        nginx_port,
        &format!("{}-{}", instance_label, ContainerImage::Adminer.to_string()),
        &format!(
            "{}-{}",
            instance_label,
            ContainerImage::Wordpress.to_string()
        ),
        instance_path,
    )
    .await?;
    let (ids, status) = container::InstanceContainer::new(
        instance_label,
        instance_path,
        ContainerImage::Nginx,
        labels,
        Vec::new(),
        None,
        Some((Some(nginx_config_path), "/etc/nginx/conf.d/default.conf")),
        Some((nginx_port, nginx_port)),
    )
    .await?;

    Ok((ids, status))
}
