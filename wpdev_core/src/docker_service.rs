use std::net::{TcpListener, SocketAddr};
use std::collections::HashMap;
use std::path::PathBuf;

use shiplift::{Docker, NetworkCreateOptions, Error as DockerError};
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use shiplift::rep::Container;

use dirs;

use log::{info, error};

use anyhow::{Result, Error as AnyhowError};

use tokio::fs;

use serde::{Serialize, Deserialize};

use crate::config::{self, AppConfig};

#[derive(Serialize, Deserialize, Clone)]
pub struct Instance {
    pub container_ids: Vec<String>,
    pub uuid: String,
    pub status: InstanceStatus,
    pub container_statuses: HashMap<String, ContainerStatus>,
    pub nginx_port: u32,
    pub adminer_port: u32,
}

#[derive(Deserialize)]
pub struct ContainerEnvVars {
    wordpress: Option<HashMap<String, String>>,
}

impl Default for ContainerEnvVars {
    fn default() -> Self {
        ContainerEnvVars {
            wordpress: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerOperation {
    Start,
    Stop,
    Restart,
    Delete,
    Inspect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
    NotFound,
    Deleted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstanceStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
    PartiallyRunning,
}

pub enum InstanceSelection {
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
        info!("Network '{}' already exists, skipping...", network_name);
        Ok(())
    } else {
        // Create network
        let network_options = NetworkCreateOptions::builder(network_name).build();

        match docker.networks().create(&network_options).await {
            Ok(container) => {
                info!("Wordpress network successfully created: {:?}", container);
                Ok(())
            }
            Err(err) => {
                error!("Error creating network '{}': {:?}", network_name, err);
                Err(err)
            }
        }
    }
}


struct EnvVars {
    adminer: Vec<String>,
    mysql: Vec<String>,
    wordpress: Vec<String>,
}

async fn initialize_env_vars(
    instance_label: &str,
    user_env_vars: &ContainerEnvVars,
) -> Result<EnvVars, AnyhowError> {

    let default_adminer_vars = HashMap::from([
        ("ADMINER_DESIGN".to_string(), "nette".to_string()),
        ("ADMINER_PLUGINS".to_string(), "tables-filter tinymce".to_string()),
        ("MYSQL_PORT".to_string(), "3306".to_string()),
        ("ADMINER_DEFAULT_SERVER".to_string(), format!("{}-mysql", instance_label).to_string()),
        ("ADMINER_DEFAULT_USERNAME".to_string(), "wordpress".to_string()),
        ("ADMINER_DEFAULT_PASSWORD".to_string(), "password".to_string()),
        ("ADMINER_DEFAULT_DATABASE".to_string(), "wordpress".to_string()),
    ]);

    let default_mysql_vars = HashMap::from([
        ("MYSQL_ROOT_PASSWORD".to_string(),"password".to_string()),
        ("MYSQL_DATABASE".to_string(),"wordpress".to_string()),
        ("MYSQL_USER".to_string(),"wordpress".to_string()),
        ("MYSQL_PASSWORD".to_string(),"password".to_string()),
    ]);

    let default_wordpress_vars = HashMap::from([
        ("WORDPRESS_DB_HOST".to_string(), format!("{}-mysql", instance_label).to_string()),
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

async fn create_container(
    docker: &Docker,
    options: ContainerOptions,
    container_type: &str,
    container_ids: &mut Vec<String>,
) -> Result<(String, ContainerStatus), AnyhowError> {
    match docker.containers().create(&options).await {
        Ok(container) => {
            container_ids.push(container.id.clone());
            log::info!("{} container successfully created: {:?}", container_type, container);

            match fetch_container_status(docker, &container.id).await {
                Ok(status) => Ok((container.id, status.unwrap_or(ContainerStatus::Unknown))),
                Err(err) => {
                    log::error!("Failed to fetch status for container {}: {:?}", container.id, err);
                    Err(err.into())
                }
            }
        }
        Err(err) => {
            log::error!("Error creating {} container: {:?}", container_type, err);
            Err(err.into())
        }
    }
}

async fn find_free_port() -> Result<u32, AnyhowError> {
    // Bind to port 0; the OS will assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let socket_addr: SocketAddr = listener.local_addr()?;
    let port = socket_addr.port();

    Ok(u32::from(port))
}

async fn generate_nginx_config(
    config: &AppConfig,
    instance_label: &str,
    nginx_port: u32,
    adminer_name: &str,
    wordpress_name: &str,
    home_dir: &PathBuf
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

    let nginx_config_dir = home_dir.join(format!("{}/{}/nginx", &config.custom_root, instance_label));
    tokio::fs::create_dir_all(&nginx_config_dir).await?;
    let nginx_config_path = nginx_config_dir.join(format!("{}-nginx.conf", instance_label));
    tokio::fs::write(&nginx_config_path, nginx_config).await?;

    Ok(nginx_config_path)
}

async fn generate_wpcli_config(
    config: &AppConfig,
    instance_label: &str,
    home_dir: &PathBuf
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
        instance_dir = instance_dir.to_str().unwrap(),
    );

    let instance_dir = home_dir.join(format!("{}/{}/", &config.custom_root, instance_label));
    tokio::fs::create_dir_all(&instance_dir).await?;
    let wpcli_yml_path = instance_dir.join("wp-cli.local.yml");
    let wpcli_php_path  = instance_dir.join("wp-cli.local.php");
    tokio::fs::write(&wpcli_yml_path, wpcli_yml).await?;
    tokio::fs::write(&wpcli_php_path, wpcli_php).await?;

    Ok(())
}

type ContainerInfo = (ContainerOptions, &'static str);

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
) -> Result<Instance, AnyhowError> {
    let config = config::read_or_create_config().await?;
    let mut container_ids = Vec::new();
    let home_dir = dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;

    let env_vars = initialize_env_vars(instance_label, &user_env_vars).await?;

    create_network_if_not_exists(&docker, &network_name).await?;

    let nginx_port = find_free_port().await?;
    let adminer_port = find_free_port().await?;

    let mut labels = HashMap::new();
    let instance_label_str = instance_label.to_string();
    let nginx_port_str = nginx_port.to_string();
    let adminer_port_str = adminer_port.to_string();
    labels.insert("instance", instance_label_str.as_str());
    labels.insert("nginx_port", nginx_port_str.as_str());
    labels.insert("adminer_port", adminer_port_str.as_str());

    let mysql_config_dir = home_dir.join(format!("{}/{}/mysql", &config.custom_root, instance_label));
    fs::create_dir_all(&mysql_config_dir).await?;
    let mysql_socket_path = mysql_config_dir;

    let mysql_options = ContainerOptions::builder(crate::MYSQL_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.mysql)
        .labels(&labels)
        .user("1000:1000")
        .name(&format!("{}-mysql", &instance_label))
        .volumes(vec![
                 &format!("{}:/var/run/mysqld", mysql_socket_path.to_str().unwrap())
        ])
        .build();

    let instance_path = home_dir.join(PathBuf::from(format!("{}/{}/app", &config.custom_root, instance_label)));
    fs::create_dir_all(&instance_path).await?;
    let wordpress_path = instance_path;


    let nginx_config_path = generate_nginx_config(
        &config,
        instance_label,
        nginx_port,
        &format!("{}-adminer", &instance_label),
        &format!("{}-wordpress", &instance_label),
        &home_dir,
    ).await?;


    let wordpress_options = ContainerOptions::builder(crate::WORDPRESS_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.wordpress)
        .labels(&labels)
        .user("1000:1000")
        .name(&format!("{}-wordpress", &instance_label))
        .volumes(vec![
                 &format!("{}:/var/www/html/", wordpress_path.to_str().unwrap()),
        ])
        .build();


    let nginx_options = ContainerOptions::builder(crate::NGINX_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .labels(&labels)
        .name(&format!("{}-nginx", instance_label))
        .volumes(vec![&format!("{}:/etc/nginx/conf.d/default.conf", nginx_config_path.to_str().unwrap())])
        .expose(nginx_port, "tcp", nginx_port)
        .build();

    let adminer_options = ContainerOptions::builder(crate::ADMINER_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.adminer)
        .labels(&labels)
        .name(&format!("{}-adminer", instance_label))
        .expose(8080, "tcp", adminer_port)
        .build();

    let mut instance = Instance {
        container_ids: Vec::new(),
        uuid: instance_label.to_string(),
        status: InstanceStatus::Stopped,
        container_statuses: HashMap::new(),
        nginx_port,
        adminer_port,
    };

    generate_wpcli_config(
        &config,
        instance_label,
        &home_dir,
    ).await?;

    let containers_to_create: Vec<ContainerInfo> = vec![
        (mysql_options, "MySQL"),
        (wordpress_options, "Wordpress"),
        (nginx_options, "Nginx"),
        (adminer_options, "Adminer"),
    ];

    for (options, container_type) in containers_to_create {
        let (container_id, container_status) = create_container(docker, options, container_type, &mut container_ids).await?;
        instance.container_statuses.insert(container_id, container_status);
    }

    // Determine overall instance status based on container statuses
    instance.status = determine_instance_status(&instance.container_statuses);

    Ok(instance)
}

async fn list_instances(
    docker: &Docker,
    network_name: &str,
    containers: Vec<Container>
) -> Result<HashMap<String, Instance>, shiplift::Error> {
    let mut instances: HashMap<String, Instance> = HashMap::new();

    for container in containers {
        match docker.containers().get(&container.id).inspect().await {
            Ok(details) => {
                if details.network_settings.networks.contains_key(network_name) {
                    if let Some(labels) = &details.config.labels {
                        if let Some(instance_label) = labels.get("instance") {
                            let instance = instances.entry(instance_label.to_string())
                                .or_insert_with(|| create_new_instance(instance_label, labels));

                            instance.container_ids.push(container.id);
                        }
                    }
                }
            },
            Err(e) => {
                // Log the error or handle it appropriately
                eprintln!("Error inspecting container {}: {}", container.id, e);
            }
        }
    }

    Ok(instances)
}

/// Creates a new instance with initial settings based on provided labels.
fn create_new_instance(instance_label: &str, labels: &HashMap<String, String>) -> Instance {
    let nginx_port = parse_port(labels.get("nginx_port"));
    let adminer_port = parse_port(labels.get("adminer_port"));

    Instance {
        container_ids: Vec::new(),
        uuid: instance_label.to_string(),
        status: InstanceStatus::Unknown,
        container_statuses: HashMap::new(),
        nginx_port,
        adminer_port,
    }
}

/// Parses a port from a label, providing a default value if necessary.
fn parse_port(port_label: Option<&String>) -> u32 {
    port_label
        .and_then(|port| port.parse::<u32>().ok())
        .unwrap_or(0)
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
) -> Result<HashMap<String, Instance>, shiplift::Error> {
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
) -> Result<HashMap<String, Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::default())
        .await?;
   list_instances(docker, network_name, containers).await
}

#[derive(Debug)]
struct CustomError(String);

impl std::error::Error for CustomError {}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn fetch_container_status(
    docker: &Docker,
    container_id: &str,
) -> Result<Option<ContainerStatus>, DockerError> {
    match docker.containers().get(container_id).inspect().await {
        Ok(container_info) => {
            let status = match container_info.state.status.as_str() {
                "running" => ContainerStatus::Running,
                "exited" => ContainerStatus::Stopped,
                _ => ContainerStatus::Unknown,
            };
            Ok(Some(status))
        },
        Err(DockerError::Fault { code, .. }) if code.as_u16() == 404 => {
            Ok(None)
        },
        Err(e) => Err(e)
    }
}

pub fn determine_instance_status(container_statuses: &HashMap<String, ContainerStatus>) -> InstanceStatus {
    let all_running = container_statuses.values().all(|status| *status == ContainerStatus::Running);
    let any_running = container_statuses.values().any(|status| *status == ContainerStatus::Running);

    match (all_running, any_running) {
        (true, _) => InstanceStatus::Running,
        (false, true) => InstanceStatus::PartiallyRunning,
        (false, false) => InstanceStatus::Stopped,
    }
}

pub async fn handle_instance(
    docker: &Docker,
    network_name: &str,
    instance_uuid: &str,
    operation: ContainerOperation,
) -> Result<Instance, AnyhowError> {
    let instances = list_all_instances(docker, network_name).await?;

    if let Some(mut instance) = instances.get(instance_uuid).cloned() {
        let mut container_statuses = HashMap::new();
        for container_id in &instance.container_ids {
            let current_container_status = fetch_container_status(docker, container_id).await?;
            match operation {
                ContainerOperation::Start => {
                    if current_container_status != Some(ContainerStatus::Running) {
                        docker.containers().get(container_id).start().await?;
                        info!("{} container successfully started", container_id);
                    } else {
                        info!("{} container is already running, skipping start operation", container_id);
                    }
                }
                ContainerOperation::Stop => {
                    if current_container_status == Some(ContainerStatus::Running) {
                        docker.containers().get(container_id).stop(None).await?;
                        info!("{} container successfully stopped", container_id);
                    } else {
                        info!("{} container is already stopped, skipping stop operation", container_id);
                    }
                }
                ContainerOperation::Restart => {
                    if current_container_status == Some(ContainerStatus::Running) {
                        docker.containers().get(container_id).restart(None).await?;
                        info!("{} container successfully restarted", container_id);
                    } else {
                        info!("{} container is not running, skipping restart operation", container_id);
                    }
                }
                ContainerOperation::Delete => {
                    if current_container_status == Some(ContainerStatus::Running) {
                        // First, stop the container if it's running
                        docker.containers().get(container_id).stop(None).await?;
                        info!("{} container successfully stopped before deletion", container_id);
                    }
                    // Then delete the container
                    docker.containers().get(container_id).delete().await?;
                    container_statuses.insert(container_id.clone(), ContainerStatus::Deleted);
                    info!("{} container successfully deleted", container_id);
                }
                ContainerOperation::Inspect => {
                    docker.containers().get(container_id).inspect().await?;
                    info!("{} container successfully inspected", container_id);
                }
            }


            let updated_status = fetch_container_status(docker, container_id).await?;

            if let Some(status) = updated_status {
                container_statuses.insert(container_id.clone(), status);
            } else {
                container_statuses.insert(container_id.clone(), ContainerStatus::NotFound);
            }
        }

        let instance_status = determine_instance_status(&container_statuses);
        instance.status = instance_status;
        instance.container_statuses = container_statuses;

        // Return the modified instance
        Ok(instance)

    } else {
        Err(AnyhowError::msg(format!("Instance with UUID {} not found", instance_uuid)))
    }
}

async fn handle_all_instances(
    docker: &Docker,
    network_name: &str,
    operation: ContainerOperation,
) -> Result<Vec<Instance>, AnyhowError> {
    let instances = list_all_instances(docker, network_name).await?;

    let mut instances_group = Vec::new();

    for (uuid, _) in instances.iter() {
        let instance = handle_instance(
            docker,
            network_name,
            uuid,
            operation.clone(),
        ).await?;

        instances_group.push(instance);
    }

    Ok(instances_group)
}



pub async fn instance_handler(
    docker: &Docker,
    network_name: &str,
    instance_selection: InstanceSelection,
    operation: ContainerOperation,
) -> Result<Vec<Instance>, AnyhowError> {
    match instance_selection {
        InstanceSelection::All => {
            Ok(handle_all_instances(docker, network_name, operation).await?)
        }
        InstanceSelection::One(instance_uuid) => {
            let instance = handle_instance(docker, network_name, &instance_uuid, operation).await?;
            Ok(vec![instance])
        }
    }
}


pub async fn purge_instances(instance: InstanceSelection) -> Result<(), AnyhowError> {
    let config_dir = dirs::config_dir().unwrap().join("wpdev");

    match instance {
        InstanceSelection::All => {
            let p = &config_dir.join(PathBuf::from("instances"));
            let path = p.to_str().unwrap();
            fs::remove_dir_all(&path).await
                .map_err(|err| AnyhowError::msg(format!("Error removing directory: {}: {}", path, err)))?;
            Ok(())
        }
        InstanceSelection::One(instance_uuid) => {
            let p = &config_dir.join(PathBuf::from("instances").join(&instance_uuid));
            let path = p.to_str().unwrap();
            fs::remove_dir_all(&path).await
                .map_err(|err| AnyhowError::msg(format!("Error removing directory: {}: {}", path, err)))?;
            Ok(())
        }
    }

}
