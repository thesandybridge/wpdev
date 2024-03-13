use anyhow::{Context, Error as AnyhowError, Result};
use bollard::container::ListContainersOptions;
use bollard::Docker;
use dirs;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use crate::config::{self, read_or_create_config};
use crate::docker::config::{
    configure_adminer_container, configure_mysql_container, configure_nginx_container,
    configure_wordpress_container,
};
use crate::docker::container::{
    ContainerEnvVars, ContainerImage, ContainerStatus, InstanceContainer,
};
use crate::utils;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Instance {
    pub uuid: String,
    pub status: InstanceStatus,
    pub containers: Vec<InstanceContainer>,
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
    pub network_name: String,
    pub nginx_port: u32,
    pub adminer_port: u32,
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

impl InstanceStatus {
    pub async fn default(docker: &Docker, containers: &Vec<InstanceContainer>) -> Result<Self> {
        let mut all_running = true;
        let mut any_running = false;

        for container in containers {
            match InstanceContainer::get_status(docker, &container.container_id).await? {
                ContainerStatus::Running => {
                    any_running = true;
                }
                ContainerStatus::Stopped | ContainerStatus::Unknown => {
                    all_running = false;
                }
                _ => {}
            }
        }

        let overall_status = if all_running {
            Self::Running
        } else if any_running {
            Self::PartiallyRunning
        } else {
            Self::Stopped
        };

        Ok(overall_status)
    }
}

pub enum InstanceSelection {
    All,
    One(String),
}

#[derive(Serialize, Deserialize)]
pub struct InstanceInfo {
    uuid: String,
    status: String,
}

impl Instance {
    pub async fn new(
        docker: &Docker,
        instance_label: &str,
        user_env_vars: ContainerEnvVars,
    ) -> Result<Self> {
        let config = config::read_or_create_config().await?;
        let home_dir =
            dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;

        let env_vars = config::initialize_env_vars(instance_label, &user_env_vars).await?;
        config::create_network_if_not_exists(docker, crate::NETWORK_NAME, instance_label).await?;

        let nginx_port = utils::find_free_port()
            .await
            .context("Failed to find free port")?;
        let adminer_port = utils::find_free_port()
            .await
            .context("Failed to find free port")?;

        let mut labels = HashMap::new();
        let instance_label_str = instance_label.to_string();
        let nginx_port_str = nginx_port.to_string();
        let adminer_port_str = adminer_port.to_string();
        labels.insert("instance".to_string(), instance_label_str);
        labels.insert("nginx_port".to_string(), nginx_port_str);
        labels.insert("adminer_port".to_string(), adminer_port_str);

        let instance_path = home_dir.join(PathBuf::from(format!(
            "{}/{}-{}",
            &config.custom_root,
            crate::NETWORK_NAME,
            instance_label
        )));

        let mysql_options =
            configure_mysql_container(instance_label, &instance_path, &labels, &env_vars).await?;

        let wordpress_options =
            configure_wordpress_container(instance_label, &instance_path, &labels, &env_vars)
                .await?;

        let nginx_options =
            configure_nginx_container(&instance_path, instance_label, &labels, nginx_port).await?;

        let adminer_options = configure_adminer_container(
            instance_label,
            &instance_path,
            &labels,
            &env_vars,
            adminer_port,
        )
        .await?;

        let wordpress_data = config::parse_instance_data(
            &env_vars,
            &nginx_port,
            &adminer_port,
            &config,
            &home_dir,
            &instance_label,
        )
        .await?;

        let mut instance = Instance {
            uuid: format!("{}-{}", crate::NETWORK_NAME, instance_label.to_string()),
            status: InstanceStatus::default(&docker, &vec![])
                .await
                .context("Failed to get default status for instance containers")?,
            containers: Vec::new(),
            nginx_port,
            adminer_port,
            wordpress_data: Some(wordpress_data),
        };

        config::generate_wpcli_config(&config, instance_label, &home_dir).await?;

        let containers = vec![
            (mysql_options, "mysql"),
            (wordpress_options, "wordpress"),
            (nginx_options, "nginx"),
            (adminer_options, "adminer"),
        ];

        for (container, container_type_str) in containers {
            let container_image = match container_type_str {
                "mysql" => ContainerImage::MySQL,
                "wordpress" => ContainerImage::Wordpress,
                "nginx" => ContainerImage::Nginx,
                "adminer" => ContainerImage::Adminer,
                _ => ContainerImage::Unknown,
            };

            let (container_id, container_status) = container;

            let instance_container = InstanceContainer {
                container_id: container_id.clone(),
                container_status,
                container_image,
            };

            instance.containers.push(instance_container);
        }

        instance.status = InstanceStatus::default(&docker, &instance.containers)
            .await
            .context("Failed to get default status for instance containers")?;

        Ok(instance)
    }

    pub async fn list(docker: &Docker, network_name: &str) -> Result<Instance> {
        info!("Starting to list instances for network: {}", network_name);

        let instance_data = crate::config::read_instance_data_from_toml(network_name)
            .await
            .context(format!(
                "Failed to read instance data from TOML file for network: {}",
                network_name
            ))?;

        let mut filters = HashMap::new();
        filters.insert("network".to_string(), vec![network_name.to_string()]);
        let containers = docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
            .context("Failed to list containers")?;

        let instance_containers = containers
            .into_iter()
            .map(|container| {
                let container_status =
                    ContainerStatus::from_str(&container.state.unwrap_or_default());
                InstanceContainer {
                    container_id: container.id.unwrap_or_default(),
                    container_status,
                    container_image: ContainerImage::from_str(&container.image.unwrap_or_default()),
                }
            })
            .collect();

        let instance = Instance {
            uuid: network_name.to_string(),
            status: InstanceStatus::default(&docker, &instance_containers)
                .await
                .context("Failed to get default status for instance containers")?,
            containers: instance_containers,
            nginx_port: instance_data.nginx_port,
            adminer_port: instance_data.adminer_port,
            wordpress_data: Some(instance_data),
        };

        info!("Successfully listed instance for network: {}", network_name);
        Ok(instance)
    }

    pub async fn list_all(
        docker: &Docker,
        network_prefix: &str,
    ) -> Result<HashMap<String, Instance>> {
        info!(
            "Starting to list all instances for network prefix: {}",
            network_prefix
        );

        let networks = docker
            .list_networks::<String>(None)
            .await
            .context("Failed to list networks")?;

        let mut instances = HashMap::new();
        for network in networks.into_iter().filter(|n| {
            n.name
                .as_ref()
                .map_or(false, |name| name.starts_with(network_prefix))
        }) {
            let full_network_name = network.name.unwrap_or_default();

            match Self::list(docker, &full_network_name).await {
                Ok(instance) => {
                    instances.insert(full_network_name.clone(), instance);
                    info!("Successfully processed network: {}", full_network_name);
                }
                Err(e) => {
                    info!("Failed to process network: {}", full_network_name);
                    info!("Error: {}", e);
                    println!("Error: {}", e);
                }
            }
        }

        info!(
            "Successfully listed all instances for network prefix: {}",
            network_prefix
        );
        Ok(instances)
    }

    pub async fn start(docker: &Docker, instance_id: &str) -> Result<InstanceInfo> {
        let mut instance = Self::list(docker, &instance_id)
            .await
            .context("Failed to list instance")?;
        for container in &instance.containers {
            InstanceContainer::start(docker, &container.container_id)
                .await
                .context(format!(
                    "Failed to start container {}",
                    &container.container_id
                ))?;
        }
        instance.status = InstanceStatus::default(docker, &instance.containers)
            .await
            .context("Failed to get default status for instance containers")?;
        Ok(InstanceInfo {
            uuid: instance.uuid.clone(),
            status: format!("{:?}", instance.status),
        })
    }

    pub async fn start_all(docker: &Docker, network_prefix: &str) -> Result<()> {
        let instances = Self::list_all(docker, network_prefix)
            .await
            .context("Failed to list instances")?;
        for (instance_id, _) in instances {
            Self::start(docker, &instance_id).await?;
        }
        Ok(())
    }

    pub async fn stop(docker: &Docker, instance_id: &str) -> Result<InstanceInfo> {
        let mut instance = Self::list(docker, &instance_id)
            .await
            .context("Failed to list instance")?;
        for container in &instance.containers {
            InstanceContainer::stop(docker, &container.container_id)
                .await
                .context(format!(
                    "Failed to stop container {}",
                    &container.container_id
                ))?;
        }
        instance.status = InstanceStatus::Stopped;
        Ok(InstanceInfo {
            uuid: instance.uuid.clone(),
            status: format!("{:?}", instance.status),
        })
    }

    pub async fn stop_all(docker: &Docker, network_prefix: &str) -> Result<()> {
        let instances = Self::list_all(docker, network_prefix)
            .await
            .context("Failed to list instances")?;

        for (_, instance) in instances {
            Self::stop(docker, &instance.uuid).await?;
        }

        Ok(())
    }

    pub async fn restart(docker: &Docker, instance_id: &str) -> Result<InstanceInfo> {
        let mut instance = Self::list(docker, &instance_id)
            .await
            .context("Failed to list instance")?;
        for container in &instance.containers {
            InstanceContainer::restart(docker, &container.container_id)
                .await
                .context(format!(
                    "Failed to restart container {}",
                    &container.container_id
                ))?;
        }
        instance.status = InstanceStatus::default(docker, &instance.containers)
            .await
            .context("Failed to get default status for instance containers")?;
        Ok(InstanceInfo {
            uuid: instance.uuid.clone(),
            status: format!("{:?}", instance.status),
        })
    }

    pub async fn restart_all(docker: &Docker, network_prefix: &str) -> Result<()> {
        let instances = Self::list_all(docker, network_prefix)
            .await
            .context("Failed to list instances")?;

        for (_, instance) in instances {
            Self::restart(docker, &instance.uuid).await?;
        }

        Ok(())
    }

    pub async fn delete(
        docker: &Docker,
        instance_id: &str,
        delete_all: bool,
    ) -> Result<InstanceInfo> {
        let mut instance = Self::list(docker, &instance_id)
            .await
            .context("Failed to list instance")?;
        for container in &instance.containers {
            InstanceContainer::delete(docker, &container.container_id)
                .await
                .context(format!(
                    "Failed to delete container {}",
                    &container.container_id
                ))?;
        }
        instance.status = InstanceStatus::Stopped;
        match delete_all {
            true => {
                purge_instances(InstanceSelection::All).await?;
            }
            false => {
                purge_instances(InstanceSelection::One(instance_id.to_string())).await?;
            }
        }
        Ok(InstanceInfo {
            uuid: instance.uuid.clone(),
            status: format!("{:?}", instance.status),
        })
    }

    pub async fn delete_all(docker: &Docker, network_prefix: &str) -> Result<()> {
        let instances = Self::list_all(docker, network_prefix)
            .await
            .context("Failed to list instances")?;

        for (instance_id, _) in instances {
            Self::delete(docker, &instance_id, true).await?;
        }

        Ok(())
    }

    pub async fn inspect(
        docker: &Docker,
        network_prefix: &str,
        instance_id: &str,
    ) -> Result<Instance> {
        let instance_name = format!("{}-{}", network_prefix, instance_id);
        let instance = Self::list(docker, &instance_name)
            .await
            .context("Failed to list instance")?;
        Ok(instance)
    }

    pub async fn inspect_all(docker: &Docker, network_prefix: &str) -> Result<Vec<Instance>> {
        let instances = Self::list_all(docker, network_prefix)
            .await
            .context("Failed to list instances")?;
        Ok(instances.values().cloned().collect())
    }
}

pub async fn purge_instances(instance: InstanceSelection) -> Result<()> {
    let config = read_or_create_config()
        .await
        .context("Failed to read config")?;
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let config_dir = home_dir.join(&config.custom_root);
    let docker = Docker::connect_with_defaults().context("Failed to connect to Docker")?;

    match instance {
        InstanceSelection::All => {
            let p = &config_dir;
            let path = p.to_str().context("Instance directory not found")?;
            let networks = docker
                .list_networks::<String>(None)
                .await
                .context("Failed to list networks")?;

            for network in networks.into_iter().filter(|n| {
                n.name
                    .as_ref()
                    .map_or(false, |name| name.starts_with(crate::NETWORK_NAME))
            }) {
                let full_network_name = network.name.unwrap_or_default();
                docker
                    .remove_network(&full_network_name)
                    .await
                    .context(format!("Failed to remove network {}", full_network_name))?;
            }
            fs::remove_dir_all(&path)
                .await
                .context(format!("Error removing directory: {}", path))?;
            Ok(())
        }
        InstanceSelection::One(instance_uuid) => {
            let p = &config_dir.join(&instance_uuid);
            let path = p.to_str().context("Instance directory not found")?;
            let network_name = format!("{}-{}", crate::NETWORK_NAME, instance_uuid);
            docker
                .remove_network(&network_name)
                .await
                .context(format!("Failed to remove network {}", network_name))?;
            fs::remove_dir_all(&path)
                .await
                .context(format!("Error removing directory: {}", path))?;
            Ok(())
        }
    }
}
