use shiplift::{Docker, NetworkCreateOptions};
use rocket::serde::json::Json;
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use shiplift::rep::Container;
use log::{info, error};
use std::collections::HashMap;
use rocket::http::Status;
use rocket::response::status::Custom;
use serde::Deserialize;
use anyhow::Result;
use std::path::PathBuf;
use dirs;
use tokio::fs;
use crate::config::loader;

#[derive(Deserialize)]
pub struct ContainerEnvVars {
    mysql: Option<HashMap<String, String>>,
    wordpress: Option<HashMap<String, String>>,
}

#[derive(Clone)]
pub enum ContainerOperation {
    Start,
    Stop,
    Restart,
    Delete
}

pub enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
}

pub enum InstanceStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
}

pub enum InstanceOperation {
    Start,
    Stop,
    Restart,
    Delete
}

pub enum Instance {
    All,
    One(String)
}

fn merge_env_vars(defaults: HashMap<String, String>, overrides: &Option<HashMap<String, String>>) -> Vec<String> {
    let mut env_vars = defaults;

    if let Some(overrides) = overrides {
        for (key, value) in overrides.iter() {
            env_vars.insert(key.clone(), value.clone());
        }
    }

    env_vars.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect()
}

async fn generate_nginx_config(instance_label: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let wordpress_container_name = format!("{}-wordpress", instance_label);
    let nginx_config = format!(
        r#"
        server {{
            listen 80;
            server_name {instance_label}.local;

            location / {{
                proxy_pass http://{wordpress_container_name}:80;
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_set_header X-Forwarded-Proto $scheme;
            }}
        }}
        "#,
        instance_label = instance_label,
    );

    let nginx_config_dir = PathBuf::from(format!(".local/wpdev/instances/{}/nginx", instance_label));
    fs::create_dir_all(&nginx_config_dir).await?;
    let nginx_config_path = nginx_config_dir.join(format!("{}-nginx.conf", instance_label));
    fs::write(&nginx_config_path, nginx_config).await?;

    Ok(nginx_config_path)
}

/// Creates a Docker Network if it doesn't already exist.
///
/// # Arguments
///
/// * `docker` - &Docker
/// * `network_name` - name of the network
pub async fn create_network_if_not_exists(
    docker: &Docker,
    network_name: &str
) -> Result<(), shiplift::Error> {
    let networks = docker.networks().list(&Default::default()).await?;
    if networks.iter().any(|network| network.name == network_name) {
        // Network already exists
        info!("Network already exists, skipping...");
        Ok(())
    } else {
        // Create network
        let network_options = NetworkCreateOptions::builder(network_name).build();
        docker.networks().create(&network_options).await?;

        match docker.networks().create(&network_options).await {
            Ok(container) => {
                info!("Wordpress network successfully created: {:?}", container);
                Ok(())
            }
            Err(err) => {
                error!("Error creating network: {:?}", err);
                Err(err)
            }
        }
    }
}

/// Create docker docker containers that are grouped by a unique
/// identifier.
///
/// # Arguments
///
/// * `docker` - Docker interface
/// * `network_name` - Docker network name
/// * `instance_label` - UUID
/// * `user_env_vars` - User defined environment variables
pub async fn create_instance(
    docker: &Docker,
    network_name: &str,
    instance_label: &str,
    user_env_vars: ContainerEnvVars,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let config = loader::read_or_create_config().await?;
    let mut container_ids = Vec::new();

    let default_mysql_vars = HashMap::from([
        ("MYSQL_ROOT_PASSWORD".to_string(),"password".to_string()),
        ("MYSQL_DATABASE".to_string(),"wordpress".to_string()),
        ("MYSQL_USER".to_string(),"wordpress".to_string()),
        ("MYSQL_PASSWORD".to_string(),"password".to_string()),
    ]);

    let default_wordpress_vars = HashMap::from([
        ("WORDPRESS_DB_HOST".to_string(), "mysql".to_string()),
        ("WORDPRESS_DB_USER".to_string(), "wordpress".to_string()),
        ("WORDPRESS_DB_PASSWORD".to_string(), "password".to_string()),
        ("WORDPRESS_DB_NAME".to_string(), "wordpress".to_string()),
        ("WORDPRESS_TABLE_PREFIX".to_string(), "wp_".to_string()),
        ("WORDPRESS_DEBUG".to_string(), "1".to_string()),
        ("WORDPRESS_CONFIG_EXTRA".to_string(), "".to_string()),
    ]);

    let mysql_env_vars = merge_env_vars(default_mysql_vars, &user_env_vars.mysql);
    let wordpress_env_vars = merge_env_vars(default_wordpress_vars, &user_env_vars.wordpress);

    create_network_if_not_exists(&docker, &network_name).await?;

    let mut labels = HashMap::new();
    labels.insert("instance", instance_label);

    let mysql_options = ContainerOptions::builder(crate::MYSQL_IMAGE)
        .env(mysql_env_vars)
        .labels(&labels)
        .name(&format!("{}-mysql", &instance_label))
        .build();

    let wordpress_path = PathBuf::from(&config.wordpress_instance_path).join(instance_label);
    fs::create_dir_all(&wordpress_path).await?;

    let nginx_config_path = generate_nginx_config(instance_label).await?;


    let wordpress_options = ContainerOptions::builder(crate::WORDPRESS_IMAGE)
        .env(wordpress_env_vars)
        .labels(&labels)
        .name(&format!("{}-wordpress", &instance_label))
        .volumes(vec![&format!("{}:/var/www/html/wp-content", wordpress_path.to_str().unwrap())])
        .build();

    let nginx_options = ContainerOptions::builder(crate::NGINX_IMAGE)
        .labels(&labels)
        .name(&format!("{}-nginx", instance_label))
        .volumes(vec![&format!("{}:/etc/nginx/conf.d/default.conf", nginx_config_path.to_str().unwrap())])
        .build();

    crate::create_container!(docker, mysql_options, "MySQL", container_ids)?;
    crate::create_container!(docker, wordpress_options, "Wordpress", container_ids)?;
    crate::create_container!(docker, nginx_options, "Nginx", container_ids)?;

    Ok(container_ids)
}

/// List all instances that are currently running.
///
/// # Arguments
///
/// * `docker` -
/// * `network_name` - [TODO:description]
/// * `containers` - [TODO:description]
async fn list_instances(
    docker: &Docker,
    network_name: &str,
    containers: Vec<Container>
) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let mut instances: HashMap<String, crate::Instance> = HashMap::new();

    for container in containers {
        let details = docker.containers().get(&container.id).inspect().await?;
        let network_settings = &details.network_settings;

        if let Some(labels) = &details.config.labels {
            if network_settings.networks.contains_key(network_name) {
                if let Some(instance_label) = labels.get("instance") {
                    instances.entry(instance_label.to_string())
                        .or_insert_with(|| crate::Instance {
                            container_ids: Vec::new(),
                            uuid: instance_label.to_string()
                        })
                        .container_ids.push(container.id);
                }
            }
        }
    }

    Ok(instances)
}

/// List all instances.
///
/// # Arguments
///
/// * `docker` - [TODO:description]
/// * `network_name` - [TODO:description]
pub async fn list_all_instances(
    docker: &Docker,
    network_name: &str
) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::builder()
              .all()
              .build())
        .await?;
    list_instances(docker, network_name, containers).await
}

/// List all instances that are currently running.
///
/// # Arguments
///
/// * `docker` - [TODO:description]
/// * `network_name` - [TODO:description]
pub async fn list_running_instances(
    docker: &Docker,
    network_name: &str
) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::default())
        .await?;
   list_instances(docker, network_name, containers).await
}

async fn handle_instance(
    docker: &Docker,
    network_name: &str,
    instance_uuid: &str,
    operation: ContainerOperation,
    success_message: &str,
) -> Result<(), Custom<String>> {
    let instances = list_all_instances(docker, network_name).await
        .map_err(|e| Custom(Status::InternalServerError, format!("Error listing instances: {}", e)))?;

    if let Some(instance) = instances.get(instance_uuid) {
        for container_id in &instance.container_ids {
            match operation {
                ContainerOperation::Start => {
                    docker.containers().get(container_id).start().await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error starting container {}: {}", container_id, err)))?;
                }
                ContainerOperation::Stop => {
                    docker.containers().get(container_id).stop(None).await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error stopping container {}: {}", container_id, err)))?;
                }
                ContainerOperation::Restart => {
                    docker.containers().get(container_id).restart(None).await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error restarting container {}: {}", container_id, err)))?;
                }
                ContainerOperation::Delete => {
                    docker.containers().get(container_id).delete().await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error restarting container {}: {}", container_id, err)))?;
                }
            }
            info!("{} container successfully {}", container_id, success_message);
        }
        Ok(())
    } else {
        Err(Custom(Status::NotFound, format!("Instance with UUID {} not found", instance_uuid)))
    }
}

async fn handle_all_instances(
    docker: &Docker,
    network_name: &str,
    operation: ContainerOperation,
    success_message: &str,
) -> Result<(), Custom<String>> {
    let instances = list_all_instances(docker, network_name).await
        .map_err(|e| Custom(Status::InternalServerError, format!("Error listing instances: {}", e)))?;

    for (_, instance) in instances.iter() {
        handle_instance(
            docker,
            network_name,
            &instance.uuid,
            operation.clone(),
            success_message
        ).await?;
    }

    Ok(())
}

pub async fn instance_handler(
    docker: &Docker,
    network_name: &str,
    instance: Instance,
    operation: ContainerOperation,
    success_message: &str,
) -> Result<(), Custom<String>> {
    match instance {
        Instance::All => {
            handle_all_instances(docker, network_name, operation, success_message).await
        }
        Instance::One(instance_uuid) => {
            handle_instance(docker, network_name, &instance_uuid, operation, success_message).await
        }
    }
}
