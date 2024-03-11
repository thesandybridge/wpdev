use anyhow::{Error as AnyhowError, Result};
use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::Docker;
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
use crate::docker::container::InstanceContainer;
use crate::utils;

#[derive(Serialize, Deserialize, Clone)]
pub struct Instance {
    pub uuid: String,
    pub status: crate::InstanceStatus,
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
}

pub enum InstanceSelection {
    All,
    One(String),
}

impl Instance {
    pub async fn default(
        instance_label: &str,
        labels: &HashMap<String, String>,
    ) -> Result<Self, AnyhowError> {
        let config = config::read_or_create_config().await?;
        let home_dir =
            dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;
        let instance_dir = home_dir.join(format!(
            "{}/{}/instance.toml",
            &config.custom_root, instance_label
        ));

        // Ensure the file exists
        if !instance_dir.exists() {
            return Err(AnyhowError::msg(format!(
                "Instance file not found at {:?}",
                instance_dir
            )));
        }

        let contents = fs::read_to_string(&instance_dir).await?;
        let instance_data: InstanceData = toml::from_str(&contents)
            .map_err(|err| AnyhowError::msg(format!("Error parsing TOML: {}", err)))?;

        let nginx_port = utils::parse_port(labels.get("nginx_port"))?;
        let adminer_port = utils::parse_port(labels.get("adminer_port"))?;

        let instance = Self {
            uuid: instance_label.to_string(),
            status: crate::InstanceStatus::default(),
            containers: Vec::new(),
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
        let instance_dir = home_dir.join(format!(
            "{}/{}/instance.toml",
            &config.custom_root, instance_label
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
        let home_dir =
            dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;

        let env_vars = config::initialize_env_vars(instance_label, &user_env_vars).await?;

        config::create_network_if_not_exists(&docker, Some(&network_name)).await?;

        let nginx_port = utils::find_free_port().await?;
        let adminer_port = utils::find_free_port().await?;

        let mut labels = HashMap::new();
        let instance_label_str = instance_label.to_string();
        let nginx_port_str = nginx_port.to_string();
        let adminer_port_str = adminer_port.to_string();
        labels.insert("instance".to_string(), instance_label_str);
        labels.insert("nginx_port".to_string(), nginx_port_str);
        labels.insert("adminer_port".to_string(), adminer_port_str);

        let instance_path = home_dir.join(PathBuf::from(format!(
            "{}/{}",
            &config.custom_root, instance_label
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

        let wordpress_data = Self::parse(
            &env_vars,
            &nginx_port,
            &adminer_port,
            &config,
            &home_dir,
            &instance_label,
        )
        .await?;

        let mut instance = Instance {
            uuid: instance_label.to_string(),
            status: crate::InstanceStatus::default(),
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
                "mysql" => crate::ContainerImage::MySQL,
                "wordpress" => crate::ContainerImage::Wordpress,
                "nginx" => crate::ContainerImage::Nginx,
                "adminer" => crate::ContainerImage::Adminer,
                _ => crate::ContainerImage::Unknown,
            };

            let (container_id, container_status) = container;

            let instance_container = InstanceContainer {
                container_id: container_id.clone(),
                container_status,
                container_image,
            };

            instance.containers.push(instance_container);
        }

        instance.status = Self::get_status(&instance.containers);

        Ok(instance)
    }

    pub async fn list(
        docker: &Docker,
        network_name: &str,
    ) -> Result<HashMap<String, Instance>, AnyhowError> {
        let mut labels_map: HashMap<String, HashMap<String, String>> = HashMap::new();

        info!("Starting to list instances");

        let containers = docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .map_err(AnyhowError::from)?;

        for container in containers {
            let container_id = container
                .id
                .as_ref()
                .ok_or_else(|| AnyhowError::msg("Container ID not found"))?;
            let container_details = docker
                .inspect_container(container_id, Some(InspectContainerOptions { size: false }))
                .await
                .map_err(AnyhowError::from)?;

            if let Some(network_settings) = &container_details.network_settings {
                if let Some(networks) = &network_settings.networks {
                    if !networks.contains_key(network_name) {
                        continue;
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(config) = &container_details.config {
                if let Some(labels) = &config.labels {
                    if let Some(instance_label) = labels.get("instance") {
                        labels_map
                            .entry(instance_label.clone())
                            .or_insert_with(|| labels.clone());
                    }
                }
            }
        }

        let mut instances: HashMap<String, Instance> = HashMap::new();
        for (instance_label, labels) in labels_map {
            let instance = Instance::default(&instance_label, &labels).await?;
            instances.insert(instance_label, instance);
        }

        info!("Successfully listed instances");

        Ok(instances)
    }

    pub async fn list_all(
        docker: &Docker,
        network_name: &str,
    ) -> Result<HashMap<String, Instance>, AnyhowError> {
        Instance::list(docker, network_name).await
    }

    pub fn get_status(containers: &Vec<InstanceContainer>) -> crate::InstanceStatus {
        let all_running = containers
            .iter()
            .all(|container| container.container_status == crate::ContainerStatus::Running);
        let any_running = containers
            .iter()
            .any(|container| container.container_status == crate::ContainerStatus::Running);

        info!("all_running: {}, any_running: {}", all_running, any_running);

        match (all_running, any_running) {
            (true, _) => crate::InstanceStatus::Running,
            (false, true) => crate::InstanceStatus::PartiallyRunning,
            (false, false) => crate::InstanceStatus::Stopped,
        }
    }

    pub async fn start(
        docker: &Docker,
        network_name: &str,
        instance_id: &str,
    ) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;
        if let Some(instance) = instances.get(instance_id) {
            for container in &instance.containers {
                InstanceContainer::start(docker, &container.container_id).await?;
            }
        } else {
            return Err(AnyhowError::msg(format!(
                "Instance with ID {} not found",
                instance_id
            )));
        }
        Ok(())
    }

    pub async fn start_all(docker: &Docker, network_name: &str) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;

        for (_, instance) in instances {
            for container in &instance.containers {
                InstanceContainer::start(docker, &container.container_id).await?;
            }
        }

        Ok(())
    }

    pub async fn stop(
        docker: &Docker,
        network_name: &str,
        instance_id: &str,
    ) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;
        if let Some(instance) = instances.get(instance_id) {
            for container in &instance.containers {
                InstanceContainer::stop(docker, &container.container_id).await?;
            }
        } else {
            return Err(AnyhowError::msg(format!(
                "Instance with ID {} not found",
                instance_id
            )));
        }
        Ok(())
    }

    pub async fn stop_all(docker: &Docker, network_name: &str) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;

        for (_, instance) in instances {
            for container in &instance.containers {
                InstanceContainer::stop(docker, &container.container_id).await?;
            }
        }

        Ok(())
    }

    pub async fn restart(
        docker: &Docker,
        network_name: &str,
        instance_id: &str,
    ) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;
        if let Some(instance) = instances.get(instance_id) {
            for container in &instance.containers {
                InstanceContainer::restart(docker, &container.container_id).await?;
            }
        } else {
            return Err(AnyhowError::msg(format!(
                "Instance with ID {} not found",
                instance_id
            )));
        }
        Ok(())
    }

    pub async fn restart_all(docker: &Docker, network_name: &str) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;

        for (_, instance) in instances {
            for container in &instance.containers {
                InstanceContainer::restart(docker, &container.container_id).await?;
            }
        }

        Ok(())
    }

    pub async fn delete(
        docker: &Docker,
        network_name: &str,
        instance_id: &str,
    ) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;
        if let Some(instance) = instances.get(instance_id) {
            for container in &instance.containers {
                InstanceContainer::delete(docker, &container.container_id).await?;
            }
        } else {
            return Err(AnyhowError::msg(format!(
                "Instance with ID {} not found",
                instance_id
            )));
        }
        Ok(())
    }

    pub async fn delete_all(docker: &Docker, network_name: &str) -> Result<(), AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;

        for (_, instance) in instances {
            for container in &instance.containers {
                InstanceContainer::delete(docker, &container.container_id).await?;
            }
        }

        purge_instances(InstanceSelection::All).await?;

        Ok(())
    }

    pub async fn inspect(
        docker: &Docker,
        network_name: &str,
        instance_id: &str,
    ) -> Result<Instance, AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;
        if let Some(instance) = instances.get(instance_id) {
            Ok(instance.clone())
        } else {
            Err(AnyhowError::msg(format!(
                "Instance with ID {} not found",
                instance_id
            )))
        }
    }

    pub async fn inspect_all(
        docker: &Docker,
        network_name: &str,
    ) -> Result<Vec<Instance>, AnyhowError> {
        let instances = Self::list_all(docker, network_name).await?;
        Ok(instances.values().cloned().collect())
    }
}

pub async fn purge_instances(instance: InstanceSelection) -> Result<(), AnyhowError> {
    let config = read_or_create_config().await?;
    let home_dir = dirs::home_dir().ok_or_else(|| AnyhowError::msg("Home directory not found"))?;
    let config_dir = home_dir.join(&config.custom_root);

    match instance {
        InstanceSelection::All => {
            let p = &config_dir;
            let path = p
                .to_str()
                .ok_or_else(|| AnyhowError::msg("Config directory not found"))?;
            fs::remove_dir_all(&path).await.map_err(|err| {
                AnyhowError::msg(format!("Error removing directory: {}: {}", path, err))
            })?;
            Ok(())
        }
        InstanceSelection::One(instance_uuid) => {
            let p = &config_dir.join(&instance_uuid);
            let path = p
                .to_str()
                .ok_or_else(|| AnyhowError::msg("Config directory not found"))?;
            fs::remove_dir_all(&path).await.map_err(|err| {
                AnyhowError::msg(format!("Error removing directory: {}: {}", path, err))
            })?;
            Ok(())
        }
    }
}
