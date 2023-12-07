use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use serde::{Serialize, Deserialize};
use anyhow::{Result, Error as AnyhowError};
use log::{info, error};
use shiplift::{Docker, Error as DockerError};
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use shiplift::rep::Container;

use crate::utils;
use crate::config::{self, read_or_create_config};

#[derive(Serialize, Deserialize, Clone)]
pub struct Instance {
    pub uuid: String,
    pub status: crate::InstanceStatus,
    pub containers: HashMap<String, InstanceContainer>,
    pub nginx_port: u32,
    pub adminer_port: u32,
    pub wordpress_data: Option<InstanceData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceData {
    pub admin_user: String,
    pub admin_password: String,
    pub admin_email: String,
    pub site_title: String,
    pub site_url: String,
    pub adminer_url: String,
    pub adminer_user: String,
    pub adminer_password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceContainer {
    pub container_id: String,
    pub container_image: crate::ContainerImage,
    pub container_status: crate::ContainerStatus,
}

pub enum InstanceSelection {
    All,
    One(String)
}

impl InstanceContainer {
    pub async fn new(
        docker: &Docker,
        options: ContainerOptions,
        container_type: &str,
        container_ids: &mut Vec<String>,
        ) -> Result<(String, crate::ContainerStatus), AnyhowError> {
        match docker.containers().create(&options).await {
            Ok(container) => {
                container_ids.push(container.id.clone());
                log::info!("{} container successfully created: {:?}", container_type, container);

                match Self::get_status(docker, &container.id).await {
                    Ok(status) => Ok((container.id, status.unwrap_or(crate::ContainerStatus::Unknown))),
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

    pub async fn get_status(
        docker: &Docker,
        container_id: &str,
        ) -> Result<Option<crate::ContainerStatus>, DockerError> {
        match docker.containers().get(container_id).inspect().await {
            Ok(container_info) => {
                let status = match container_info.state.status.as_str() {
                    "running" => crate::ContainerStatus::Running,
                    "exited" => crate::ContainerStatus::Stopped,
                    "paused" => crate::ContainerStatus::Paused,
                    "dead" => crate::ContainerStatus::Dead,
                    "restarting" => crate::ContainerStatus::Restarting,
                    _ => crate::ContainerStatus::Unknown,
                };
                Ok(Some(status))
            },
            Err(DockerError::Fault { code, .. }) if code.as_u16() == 404 => {
                Ok(None)
            },
            Err(e) => Err(e)
        }

    }
}

pub async fn configure_wordpress_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars
) -> Result<ContainerOptions> {

    let wordpress_config_dir = instance_path.join("wordpress");
    let wordpress_path = utils::create_path(&wordpress_config_dir).await?;
    let wordpress_labels = utils::create_labels(crate::ContainerImage::Wordpress, labels.clone());
    let wordpress_labels_view: HashMap<_, _> = wordpress_labels.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let wordpress_options = ContainerOptions::builder(crate::WORDPRESS_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.wordpress.clone())
        .labels(&wordpress_labels_view)
        .user("1000:1000")
        .name(&format!("{}-{}", instance_label, crate::ContainerImage::Wordpress.to_string()))
        .volumes(vec![
                 &format!("{}:/var/www/html/", wordpress_path.to_str().ok_or_else(|| AnyhowError::msg("Instance directory not found"))?),
        ])
        .build();
    Ok(wordpress_options)
}

pub async fn configure_mysql_container(
    instance_label: &str,
    instance_path: &PathBuf,
    labels: &HashMap<String, String>,
    env_vars: &crate::EnvVars
) -> Result<ContainerOptions> {

    let mysql_config_dir = instance_path.join("mysql");
    let mysql_socket_path = utils::create_path(&mysql_config_dir).await?;
    let mysql_labels = utils::create_labels(crate::ContainerImage::MySQL, labels.clone());
    let mysql_labels_view: HashMap<_, _> = mysql_labels.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let mysql_options = ContainerOptions::builder(crate::MYSQL_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.mysql.clone())
        .labels(&mysql_labels_view)
        .user("1000:1000")
        .name(&format!("{}-{}", instance_label, crate::ContainerImage::MySQL.to_string()))
        .volumes(vec![
                 &format!("{}:/var/run/mysqld", mysql_socket_path.to_str().ok_or_else(|| AnyhowError::msg("Instance directory not found"))?)
        ])
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
            &format!("{}-{}", &instance_label, crate::ContainerImage::Adminer.to_string()),
            &format!("{}-{}", &instance_label, crate::ContainerImage::Wordpress.to_string()),
            instance_path,
            ).await?;
    let nginx_labels = utils::create_labels(crate::ContainerImage::Nginx, labels.clone());
    let nginx_labels_view: HashMap<_, _> = nginx_labels.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let nginx_options = ContainerOptions::builder(crate::NGINX_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .labels(&nginx_labels_view)
        .name(&format!("{}-{}", instance_label, crate::ContainerImage::Nginx.to_string()))
        .volumes(vec![&format!("{}:/etc/nginx/conf.d/default.conf", nginx_config_path.to_str().ok_or_else(|| AnyhowError::msg("Instance directory not found"))?)])
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
    let adminer_labels_view: HashMap<_, _> = adminer_labels.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let adminer_options = ContainerOptions::builder(crate::ADMINER_IMAGE)
        .network_mode(crate::NETWORK_NAME)
        .env(env_vars.adminer.clone())
        .labels(&adminer_labels_view)
        .name(&format!("{}-{}", instance_label, crate::ContainerImage::Adminer.to_string()))
        .expose(8080, "tcp", adminer_port)
        .build();
    Ok(adminer_options)
}

impl Instance {
    pub async fn default(
        instance_label: &str,
        labels: &HashMap<String, String>
        ) -> Result<Self, AnyhowError> {
        let config = config::read_or_create_config().await?;
        let home_dir = dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;
        let instance_dir = home_dir.join(format!("{}/{}/instance.toml", &config.custom_root, instance_label));

        // Ensure the file exists
        if !instance_dir.exists() {
            return Err(AnyhowError::msg(format!("Instance file not found at {:?}", instance_dir)));
        }

        let contents = fs::read_to_string(&instance_dir).await?;
        let instance_data: InstanceData = toml::from_str(&contents)
            .map_err(|err| AnyhowError::msg(format!("Error parsing TOML: {}", err)))?;

        let nginx_port = utils::parse_port(labels.get("nginx_port"))?;
        let adminer_port = utils::parse_port(labels.get("adminer_port"))?;

        let instance = Self {
            uuid: instance_label.to_string(),
            status: crate::InstanceStatus::default(),
            containers: HashMap::new(),
            nginx_port,
            adminer_port,
            wordpress_data: Some(instance_data),
        };

        Ok(instance)
    }

    async fn parse(
        env_vars: &crate::EnvVars,
        nginx_port: &u32,
        adminer_port: &u32,
        config: &crate::AppConfig,
        home_dir: &PathBuf,
        instance_label: &str,
        ) -> Result<InstanceData> {
        let instance_dir = home_dir.join(format!("{}/{}/instance.toml", &config.custom_root, instance_label));

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
            site_url: format!("{}:{}", config.site_url, nginx_port),
            adminer_url: format!("{}:{}", config.adminer_url, adminer_port),
            adminer_user: extract_value(&env_vars.adminer, "ADMINER_DEFAULT_USERNAME"),
            adminer_password: extract_value(&env_vars.adminer, "ADMINER_DEFAULT_PASSWORD"),
        };

        fs::write(&instance_dir, toml::to_string(&instance_data)?).await?;
        info!("Instance data written to {:?}", instance_dir);

        Ok(instance_data)

    }

    pub async fn new(
        docker: &Docker,
        network_name: &str,
        instance_label: &str,
        user_env_vars: crate::ContainerEnvVars,
        ) -> Result<Self> {

        let config = config::read_or_create_config().await?;
        let mut container_ids = Vec::new();
        let home_dir = dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;

        let env_vars = config::initialize_env_vars(instance_label, &user_env_vars).await?;

        config::create_network_if_not_exists(&docker, &network_name).await?;

        let nginx_port = utils::find_free_port().await?;
        let adminer_port = utils::find_free_port().await?;

        let mut labels = HashMap::new();
        let instance_label_str = instance_label.to_string();
        let nginx_port_str = nginx_port.to_string();
        let adminer_port_str = adminer_port.to_string();
        labels.insert("instance".to_string(), instance_label_str);
        labels.insert("nginx_port".to_string(), nginx_port_str);
        labels.insert("adminer_port".to_string(), adminer_port_str);



        let instance_path = home_dir.join(PathBuf::from(format!("{}/{}", &config.custom_root, instance_label)));

        let mysql_options = configure_mysql_container(
            instance_label,
            &instance_path,
            &labels,
            &env_vars,
            ).await?;

        let wordpress_options = configure_wordpress_container(
            instance_label,
            &instance_path,
            &labels,
            &env_vars,
            ).await?;

        let nginx_options = configure_nginx_container(
            &instance_path,
            instance_label,
            &labels,
            nginx_port,
            ).await?;

        let adminer_options = configure_adminer_container(
            instance_label,
            &labels,
            &env_vars,
            adminer_port,
            ).await?;

        let wordpress_data = Self::parse(
            &env_vars,
            &nginx_port,
            &adminer_port,
            &config,
            &home_dir,
            &instance_label,
            ).await?;

        let mut instance = Instance {
            uuid: instance_label.to_string(),
            status: crate::InstanceStatus::default(),
            containers: HashMap::new(),
            nginx_port,
            adminer_port,
            wordpress_data: Some(wordpress_data),
        };

        config::generate_wpcli_config(
            &config,
            instance_label,
            &home_dir,
            ).await?;

        let containers_to_create: Vec<crate::ContainerInfo> = vec![
            (mysql_options, "MySQL"),
            (wordpress_options, "Wordpress"),
            (nginx_options, "Nginx"),
            (adminer_options, "Adminer"),
        ];

        for (options, container_type) in containers_to_create {
            let (container_id, container_status) = InstanceContainer::new(docker, options, container_type, &mut container_ids).await?;
            let container = InstanceContainer {
                container_id: container_id.clone(),
                container_status,
                container_image: match container_type {
                    "MySQL" => crate::ContainerImage::MySQL,
                    "Wordpress" => crate::ContainerImage::Wordpress,
                    "Nginx" => crate::ContainerImage::Nginx,
                    "Adminer" => crate::ContainerImage::Adminer,
                    _ => crate::ContainerImage::Unknown,
                }
            };
            instance.containers.insert(container_id, container);
        }

        // Determine overall instance status based on container statuses
        instance.status = Self::get_status(&instance.containers);

        Ok(instance)

    }

    async fn list(
        docker: &Docker,
        network_name: &str,
        containers: Vec<Container>
        ) -> Result<HashMap<String, Instance>, AnyhowError> {
        let mut instances: HashMap<String, Instance> = HashMap::new();

        info!("Starting to list instances");

        for container in containers {
            match docker.containers().get(&container.id).inspect().await {
                Ok(details) => {
                    if details.network_settings.networks.contains_key(network_name) {
                        if let Some(labels) = &details.config.labels {
                            if let Some(instance_label) = labels.get("instance") {
                                let instance = instances.entry(instance_label.to_string())
                                    .or_insert(Instance::default(instance_label, labels).await.unwrap());

                                let container_image = match labels.get("image") {
                                    Some(image) => crate::ContainerImage::from_string(image).unwrap_or(crate::ContainerImage::Unknown),
                                    None => crate::ContainerImage::Unknown,
                                };

                                let container_status = match container.state.as_str() {
                                    "running" => crate::ContainerStatus::Running,
                                    "exited" => crate::ContainerStatus::Stopped,
                                    "paused" => crate::ContainerStatus::Paused,
                                    "dead" => crate::ContainerStatus::Dead,
                                    "restarting" => crate::ContainerStatus::Restarting,
                                    _ => crate::ContainerStatus::Unknown,
                                };

                                instance.containers.insert(container.id.clone(), InstanceContainer {
                                    container_id: container.id.clone(),
                                    container_image,
                                    container_status,
                                });

                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Error inspecting container {}: {}", container.id, e);
                }
            }
        }
        info!("Successfully listed instances");

        Ok(instances)
    }


    pub async fn list_all(
        docker: &Docker,
        network_name: &str,
        ) -> Result<HashMap<String, Instance>, AnyhowError> {
        let containers = docker
            .containers()
            .list(&ContainerListOptions::builder()
                  .all()
                  .build())
            .await?;
        Ok(Self::list(docker, network_name, containers).await?)
    }

    pub async fn list_running(
        docker: &Docker,
        network_name: &str,
        ) -> Result<HashMap<String, Instance>, AnyhowError> {
        let containers = docker
            .containers()
            .list(&ContainerListOptions::default())
            .await?;
        Ok(Self::list(docker, network_name, containers).await?)
    }

    pub fn get_status(
        containers: &HashMap<String, InstanceContainer>
        ) -> crate::InstanceStatus {
        let all_running = containers.values().all(|container| container.container_status == crate::ContainerStatus::Running);
        let any_running = containers.values().any(|container| container.container_status == crate::ContainerStatus::Running);

        match (all_running, any_running) {
            (true, _) => crate::InstanceStatus::Running,
            (false, true) => crate::InstanceStatus::PartiallyRunning,
            (false, false) => crate::InstanceStatus::Stopped,
        }
    }
}

pub async fn handle_instance(
    docker: &Docker,
    network_name: &str,
    instance_uuid: &str,
    operation: crate::ContainerOperation,
) -> Result<Instance, AnyhowError> {
    let mut instances = Instance::list_all(docker, network_name).await?;

    if let Some(mut instance) = instances.remove(instance_uuid) {
        let mut containers_to_delete = Vec::new();
        for (container_id, container) in instance.containers.iter_mut() {
            let current_container_status = &container.container_status;
            match operation {
                crate::ContainerOperation::Start => {
                    if *current_container_status != crate::ContainerStatus::Running {
                        docker.containers().get(container_id).start().await?;
                        info!("{} container successfully started", container_id);
                    } else {
                        info!("{} container is already running, skipping start operation", container_id);
                    }
                }
                crate::ContainerOperation::Stop => {
                    if *current_container_status == crate::ContainerStatus::Running {
                        docker.containers().get(container_id).stop(None).await?;
                        info!("{} container successfully stopped", container_id);
                    } else {
                        info!("{} container is already stopped, skipping stop operation", container_id);
                    }
                }
                crate::ContainerOperation::Restart => {
                    if *current_container_status == crate::ContainerStatus::Running {
                        docker.containers().get(container_id).restart(None).await?;
                        info!("{} container successfully restarted", container_id);
                    } else {
                        info!("{} container is not running, skipping restart operation", container_id);
                    }
                }
                crate::ContainerOperation::Delete => {
                    if *current_container_status == crate::ContainerStatus::Running {
                        // First, stop the container if it's running
                        docker.containers().get(container_id).stop(None).await?;
                        info!("{} container successfully stopped before deletion", container_id);
                    }
                    // Then delete the container
                    docker.containers().get(container_id).delete().await?;
                    containers_to_delete.push(container_id.clone());
                    info!("{} container successfully deleted", container_id);
                }
                crate::ContainerOperation::Inspect => {
                    docker.containers().get(container_id).inspect().await?;
                    info!("{} container successfully inspected", container_id);
                }
            }


        }

        // Remove the deleted containers after iterating
        for container_id in containers_to_delete {
            instance.containers.remove(&container_id);
        }

        instance.status = Instance::get_status(&instance.containers);
        // Return the modified instance
        Ok(instance)

    } else {
        Err(AnyhowError::msg(format!("Instance with UUID {} not found", instance_uuid)))
    }
}

async fn handle_all_instances(
    docker: &Docker,
    network_name: &str,
    operation: crate::ContainerOperation,
) -> Result<Vec<Instance>, AnyhowError> {
    let instances = Instance::list_all(docker, network_name).await?;

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
    operation: crate::ContainerOperation,
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
    let config = read_or_create_config().await?;
    let home_dir = dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;
    let config_dir = home_dir.join(&config.custom_root);


    match instance {
        InstanceSelection::All => {
            let p = &config_dir;
            let path = p.to_str().ok_or_else(|| AnyhowError::msg("Config directory not found"))?;
            fs::remove_dir_all(&path).await
                .map_err(|err| AnyhowError::msg(format!("Error removing directory: {}: {}", path, err)))?;
            Ok(())
        }
        InstanceSelection::One(instance_uuid) => {
            let p = &config_dir.join(&instance_uuid);
            let path = p.to_str().ok_or_else(|| AnyhowError::msg("Config directory not found"))?;
            fs::remove_dir_all(&path).await
                .map_err(|err| AnyhowError::msg(format!("Error removing directory: {}: {}", path, err)))?;
            Ok(())
        }
    }

}



