use crate::config;
use crate::utils;
use anyhow::{Error as AnyhowError, Result};
use shiplift::builder::ContainerOptions;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn configure_wordpress_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
) -> Result<ContainerOptions> {
    let wordpress_config_dir = instance_path.join("wordpress");
    let wordpress_path = utils::create_path(&wordpress_config_dir).await?;
    let wordpress_labels = utils::create_labels(crate::ContainerImage::Wordpress, labels.clone());
    let wordpress_labels_view: HashMap<_, _> = wordpress_labels
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let wordpress_options = ContainerOptions::builder(crate::WORDPRESS_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.wordpress.clone())
        .labels(&wordpress_labels_view)
        .user("1000:1000")
        .name(&format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::Wordpress.to_string()
        ))
        .volumes(vec![&format!(
            "{}:/var/www/html/",
            wordpress_path
                .to_str()
                .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?
        )])
        .build();
    Ok(wordpress_options)
}

pub async fn configure_mysql_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
) -> Result<ContainerOptions> {
    let mysql_config_dir = instance_path.join("mysql");
    let mysql_socket_path = utils::create_path(&mysql_config_dir).await?;
    let mysql_labels = utils::create_labels(crate::ContainerImage::MySQL, labels.clone());
    let mysql_labels_view: HashMap<_, _> = mysql_labels
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let mysql_options = ContainerOptions::builder(crate::MYSQL_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.mysql.clone())
        .labels(&mysql_labels_view)
        .user("1000:1000")
        .name(&format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::MySQL.to_string()
        ))
        .volumes(vec![&format!(
            "{}:/var/run/mysqld",
            mysql_socket_path
                .to_str()
                .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?
        )])
        .build();
    Ok(mysql_options)
}

pub async fn configure_nginx_container(
    instance_path: &PathBuf,
    instance_label: &str,
    labels: &HashMap<String, String>,
    nginx_port: u32,
) -> Result<ContainerOptions> {
    let nginx_config_path = config::generate_nginx_config(
        instance_label,
        nginx_port,
        &format!(
            "{}-{}",
            &instance_label,
            crate::ContainerImage::Adminer.to_string()
        ),
        &format!(
            "{}-{}",
            &instance_label,
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
    let nginx_options = ContainerOptions::builder(crate::NGINX_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .labels(&nginx_labels_view)
        .name(&format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::Nginx.to_string()
        ))
        .volumes(vec![&format!(
            "{}:/etc/nginx/conf.d/default.conf",
            nginx_config_path
                .to_str()
                .ok_or_else(|| AnyhowError::msg("Instance directory not found"))?
        )])
        .expose(nginx_port, "tcp", nginx_port)
        .build();
    Ok(nginx_options)
}

pub async fn configure_adminer_container(
    instance_label: &str,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars,
    adminer_port: u32,
) -> Result<ContainerOptions> {
    let adminer_labels = utils::create_labels(crate::ContainerImage::Adminer, labels.clone());
    let adminer_labels_view: HashMap<_, _> = adminer_labels
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let adminer_options = ContainerOptions::builder(crate::ADMINER_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.adminer.clone())
        .labels(&adminer_labels_view)
        .name(&format!(
            "{}-{}",
            instance_label,
            crate::ContainerImage::Adminer.to_string()
        ))
        .expose(8080, "tcp", adminer_port)
        .build();
    Ok(adminer_options)
}
