use shiplift::{Docker, NetworkCreateOptions, Error as DockerError};
use std::net::{TcpListener, SocketAddr};
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use shiplift::rep::Container;
use std::error::Error;
use log::{info, error};
use std::collections::HashMap;
use rocket::http::Status;
use rocket::response::status::Custom;
use anyhow::Result;
use std::path::PathBuf;
use dirs;
use tokio::fs;
use crate::config::loader;
use serde::{Serialize, Deserialize};
use crate::config::loader::AppConfig;

#[derive(Serialize, Deserialize)]
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


async fn create_container(
    docker: &Docker,
    options: ContainerOptions,
    container_type: &str,
    container_ids: &mut Vec<String>,
) -> Result<(String, ContainerStatus), Box<dyn std::error::Error>> {
    match docker.containers().create(&options).await {
        Ok(container) => {
            container_ids.push(container.id.clone());
            log::info!("{} container successfully created: {:?}", container_type, container);

            match fetch_container_status(docker, &container.id).await {
                Ok(status) => Ok((container.id, status.unwrap_or(ContainerStatus::Unknown))),
                Err(err) => {
                    log::error!("Failed to fetch status for container {}: {:?}", container.id, err);
                    Err(err)
                }
            }
        }
        Err(err) => {
            log::error!("Error creating {} container: {:?}", container_type, err);
            Err(err.into())
        }
    }
}

async fn find_free_port() -> Result<u32, Box<dyn std::error::Error>> {
    // Bind to port 0; the OS will assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let socket_addr: SocketAddr = listener.local_addr()?;
    let port = socket_addr.port();

    Ok(u32::from(port))
}

async fn generate_nginx_config(
    config: AppConfig,
    instance_label: &str,
    nginx_port: u32,
    adminer_name: &str,
    wordpress_name: &str,
    home_dir: &PathBuf
) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
) -> Result<Instance, Box<dyn std::error::Error>> {
    let config = loader::read_or_create_config().await?;
    let mut container_ids = Vec::new();
    let home_dir = dirs::home_dir().ok_or("Home directory not found")?;

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

    let mysql_env_vars = merge_env_vars(default_mysql_vars, &None);
    let wordpress_env_vars = merge_env_vars(default_wordpress_vars, &user_env_vars.wordpress);
    let adminer_env_vars = merge_env_vars(default_adminer_vars, &None);

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

    let mysql_options = ContainerOptions::builder(crate::MYSQL_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(mysql_env_vars)
        .labels(&labels)
        .name(&format!("{}-mysql", &instance_label))
        .build();

    let instance_path = home_dir.join(PathBuf::from(format!("{}/{}/app", &config.custom_root, instance_label)));
    fs::create_dir_all(&instance_path).await?;
    let wordpress_path = instance_path;


    let nginx_config_path = generate_nginx_config(
        config,
        instance_label,
        nginx_port,
        &format!("{}-adminer", &instance_label),
        &format!("{}-wordpress", &instance_label),
        &home_dir,
    ).await?;


    let wordpress_options = ContainerOptions::builder(crate::WORDPRESS_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(wordpress_env_vars)
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
        .env(adminer_env_vars)
        .labels(&labels)
        .name(&format!("{}-adminer", instance_label))
        .expose(8080, "tcp", adminer_port)
        .build();

    let mut instance = Instance {
        container_ids: Vec::new(),
        uuid: instance_label.to_string(),
        status: InstanceStatus::Unknown,
        container_statuses: HashMap::new(),
        nginx_port,
        adminer_port,
    };

    let (mysql_id, mysql_status) = create_container(docker, mysql_options, "MySQL", &mut container_ids).await?;
    instance.container_statuses.insert(mysql_id, mysql_status);

    let (wordpress_id, wordpress_status) = create_container(docker, wordpress_options, "Wordpress", &mut container_ids).await?;
    instance.container_statuses.insert(wordpress_id, wordpress_status);

    let (nginx_id, nginx_status) = create_container(docker, nginx_options, "Nginx", &mut container_ids).await?;
    instance.container_statuses.insert(nginx_id, nginx_status);

    let (adminer_id, adminer_status) = create_container(docker, adminer_options, "Adminer", &mut container_ids).await?;
    instance.container_statuses.insert(adminer_id, adminer_status);

    // Determine overall instance status based on container statuses
    instance.status = determine_instance_status(&instance.container_statuses);

    Ok(instance)
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
) -> Result<HashMap<String, Instance>, shiplift::Error> {
    let mut instances: HashMap<String, Instance> = HashMap::new();

    for container in containers {
        let details = docker.containers().get(&container.id).inspect().await?;
        let network_settings = &details.network_settings;

        if let Some(labels) = &details.config.labels {
            if network_settings.networks.contains_key(network_name) {
                if let Some(instance_label) = labels.get("instance") {
                    let nginx_port = labels.get("nginx_port")
                               .and_then(|port| port.parse::<u32>().ok())
                               .unwrap_or(0);

                    let adminer_port = labels.get("adminer_port")
                               .and_then(|port| port.parse::<u32>().ok())
                               .unwrap_or(0);

                    instances.entry(instance_label.to_string())
                        .or_insert_with(|| Instance {
                            container_ids: Vec::new(),
                            uuid: instance_label.to_string(),
                            status: InstanceStatus::Stopped,
                            container_statuses: HashMap::new(),
                            nginx_port,
                            adminer_port,
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
    container_id: &str
    ) ->
Result<Option<ContainerStatus>, Box<dyn Error + Send>> {
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
            // Container not found, treat as a valid case
            Ok(None)
        },
        Err(e) => Err(Box::new(e))
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
    status: Option<InstanceStatus>,
) -> Result<InstanceStatus, Custom<String>> {
    let instances = list_all_instances(docker, network_name).await
        .map_err(|e| {
            error!("Error listing instances: {}", e);
            Custom(Status::InternalServerError, format!("Error listing instances: {}", e))
        })?;

    let running_instances = list_running_instances(docker, network_name).await
        .map_err(|e| {
            error!("Error listing running instances: {}", e);
            Custom(Status::InternalServerError, format!("Error listing running instances: {}", e))
        })?;

    let target_instances = match status {
        Some(InstanceStatus::Running) => &running_instances,
        _ => &instances,
    };

    if let Some(instance) = target_instances.get(instance_uuid) {
        let mut container_statuses = HashMap::new();
        for container_id in &instance.container_ids {
            match operation {
                ContainerOperation::Start if status != Some(InstanceStatus::Running) => {
                    docker.containers().get(container_id).start().await
                        .map_err(|err| {
                            error!("Error starting container {}: {}", container_id, err);
                            Custom(Status::InternalServerError, format!("Error starting container {}: {}", container_id, err))
                        })?;
                    info!("{} container successfully started", container_id);
                }
                ContainerOperation::Stop | ContainerOperation::Restart if status == Some(InstanceStatus::Running) => {
                    if operation == ContainerOperation::Stop {
                        docker.containers().get(container_id).stop(None).await
                            .map_err(|err| {
                                error!("Error stopping container {}: {}", container_id, err);
                                Custom(Status::InternalServerError, format!("Error stopping container {}: {}", container_id, err))
                            })?;
                        info!("{} container successfully stopped", container_id);
                    } else {
                        docker.containers().get(container_id).restart(None).await
                            .map_err(|err| {
                                error!("Error restarting container {}: {}", container_id, err);
                                Custom(Status::InternalServerError, format!("Error restarting container {}: {}", container_id, err))
                            })?;
                        info!("{} container successfully restarted", container_id);
                    }
                }
                ContainerOperation::Delete if status != Some(InstanceStatus::Running) => {
                    docker.containers().get(container_id).delete().await
                        .map_err(|err| {
                            error!("Error deleting container {}: {}", container_id, err);
                            Custom(Status::InternalServerError, format!("Error deleting container {}: {}", container_id, err))
                        })?;
                    container_statuses.insert(container_id.clone(), ContainerStatus::Deleted);
                    info!("{} container successfully deleted", container_id);
                }
                ContainerOperation::Inspect => {
                    docker.containers().get(container_id).inspect().await
                        .map_err(|err| {
                            error!("Error inspecting container {}: {}", container_id, err);
                            Custom(Status::InternalServerError, format!("Error inspecting container {}: {}", container_id, err))
                        })?;
                    info!("{} container successfully inspected", container_id);
                }
                _ => {
                    info!("Operation {:?} is not valid for instance {} with status {:?}", operation, instance_uuid, status);
                }
            }

            let container_status = fetch_container_status(docker, container_id).await
                .map_err(|err| Custom(Status::InternalServerError, format!("Error fetching status for container {}: {}", container_id, err)))?;

            if let Some(status) = container_status {
                container_statuses.insert(container_id.clone(), status);
            } else {
                container_statuses.insert(container_id.clone(), ContainerStatus::NotFound);
            }
        }

        let instance_status = determine_instance_status(&container_statuses);
        Ok(instance_status)

    } else {
        Err(Custom(Status::NotFound, format!("Instance with UUID {} not found", instance_uuid)))
    }
}

async fn handle_all_instances(
    docker: &Docker,
    network_name: &str,
    operation: ContainerOperation,
    status: Option<InstanceStatus>,
) -> Result<Vec<(String, InstanceStatus)>, Custom<String>> {
    let instances = list_all_instances(docker, network_name).await
        .map_err(|e| Custom(Status::InternalServerError, format!("Error listing instances: {}", e)))?;

    let mut statuses = Vec::new();

    for (uuid, _) in instances.iter() {
        let instance_status = handle_instance(
            docker,
            network_name,
            uuid,
            operation.clone(),
            status.clone(),
        ).await
        .map_err(|e| Custom(Status::InternalServerError, format!("Error handling instance {}: {:?}", uuid, e)))?;

        statuses.push((uuid.clone(), instance_status));
    }

    Ok(statuses)
}



pub async fn instance_handler(
    docker: &Docker,
    network_name: &str,
    instance_selection: InstanceSelection,
    operation: ContainerOperation,
    status: Option<InstanceStatus>,
) -> Result<Vec<(String, InstanceStatus)>, Custom<String>> {
    match instance_selection {
        InstanceSelection::All => {
            handle_all_instances(docker, network_name, operation, status).await
        }
        InstanceSelection::One(instance_uuid) => {
            let instance_status = handle_instance(docker, network_name, &instance_uuid, operation, status).await?;
            Ok(vec![(instance_uuid.to_string(), instance_status)])
        }
    }
}


pub async fn purge_instances(instance: InstanceSelection) -> Result<(), Custom<String>> {
    let config_dir = dirs::config_dir().unwrap().join("wpdev");

    match instance {
        InstanceSelection::All => {
            let p = &config_dir.join(PathBuf::from("instances"));
            let path = p.to_str().unwrap();
            fs::remove_dir_all(&path).await
                .map_err(|err| Custom(
                        Status::InternalServerError,
                        format!("Error removing directory {}: {}", path, err)))?;
            Ok(())
        }
        InstanceSelection::One(instance_uuid) => {
            let p = &config_dir.join(PathBuf::from("instances").join(&instance_uuid));
            let path = p.to_str().unwrap();
            fs::remove_dir_all(&path).await
                .map_err(|err| Custom(
                        Status::InternalServerError,
                        format!("Error removing directory {}: {}", path, err)))?;
            Ok(())
        }
    }

}
