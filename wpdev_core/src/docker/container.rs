use crate::utils;
use anyhow::{Context, Error as AnyhowError, Result};
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, RestartContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::models::{HostConfig, PortBinding};
use bollard::Docker;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct ContainerEnvVars {
    pub wordpress: Option<HashMap<String, String>>,
}

impl Default for ContainerEnvVars {
    fn default() -> Self {
        ContainerEnvVars { wordpress: None }
    }
}

pub struct EnvVars {
    pub adminer: Vec<String>,
    pub mysql: Vec<String>,
    pub wordpress: Vec<String>,
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

impl ContainerStatus {
    pub fn to_string(&self) -> String {
        match self {
            ContainerStatus::Running => "running".to_string(),
            ContainerStatus::Stopped => "stopped".to_string(),
            ContainerStatus::Restarting => "restarting".to_string(),
            ContainerStatus::Paused => "paused".to_string(),
            ContainerStatus::Exited => "exited".to_string(),
            ContainerStatus::Dead => "dead".to_string(),
            ContainerStatus::Unknown => "unknown".to_string(),
            ContainerStatus::NotFound => "not found".to_string(),
            ContainerStatus::Deleted => "deleted".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContainerImage {
    Adminer,
    MySQL,
    Nginx,
    Wordpress,
    Unknown,
}

impl fmt::Display for ContainerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerImage::MySQL => write!(f, "MySQL"),
            ContainerImage::Wordpress => write!(f, "Wordpress"),
            ContainerImage::Nginx => write!(f, "Nginx"),
            ContainerImage::Adminer => write!(f, "Adminer"),
            ContainerImage::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ContainerImage {
    pub fn to_string(&self) -> String {
        match self {
            ContainerImage::Adminer => "adminer".to_string(),
            ContainerImage::MySQL => "mysql".to_string(),
            ContainerImage::Nginx => "nginx".to_string(),
            ContainerImage::Wordpress => "wordpress".to_string(),
            ContainerImage::Unknown => "unknown".to_string(),
        }
    }

    pub fn from_str(image: &str) -> Self {
        match image {
            "adminer" => ContainerImage::Adminer,
            "mysql" => ContainerImage::MySQL,
            "nginx" => ContainerImage::Nginx,
            "wordpress" => ContainerImage::Wordpress,
            _ => ContainerImage::Unknown,
        }
    }
}

impl ContainerStatus {
    pub fn from_str(status: &str) -> Self {
        match status {
            "running" => ContainerStatus::Running,
            "stopped" => ContainerStatus::Stopped,
            "restarting" => ContainerStatus::Restarting,
            "paused" => ContainerStatus::Paused,
            "exited" => ContainerStatus::Exited,
            "dead" => ContainerStatus::Dead,
            _ => ContainerStatus::Unknown,
        }
    }
}

pub type ContainerInfo = (ContainerOperation, &'static str);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceContainer {
    pub container_id: String,
    pub container_image: ContainerImage,
    pub container_status: ContainerStatus,
}

impl InstanceContainer {
    pub async fn new(
        instance_label: &str,
        instance_path: &PathBuf,
        container_image: ContainerImage,
        labels: &HashMap<String, String>,
        env_vars: Vec<String>,
        user: Option<String>,
        volume_binding: Option<(Option<PathBuf>, &str)>,
        port: Option<(u32, u32)>,
    ) -> Result<(String, ContainerStatus)> {
        let docker = Docker::connect_with_defaults()?;
        let config_dir = instance_path.join(&container_image.to_string());

        let path = utils::create_path(&config_dir)
            .await
            .context("Failed to create instance directory")?;
        let path_str = path
            .to_str()
            .context("Failed to convert instance directory to string")?;

        let container_labels = utils::create_labels(container_image.clone(), labels.clone());
        let labels_view = container_labels.into_iter().collect();

        let mut port_bindings = HashMap::new();
        if let Some((host_port, container_port)) = port {
            let port_key = format!("{}/tcp", container_port);
            let binding = PortBinding {
                host_ip: None,
                host_port: Some(host_port.to_string()),
            };
            port_bindings.insert(port_key, Some(vec![binding]));
        }

        let host_config = HostConfig {
            binds: match volume_binding {
                Some((Some(config_path), container_path)) => {
                    let config_path_str = config_path
                        .to_str()
                        .context("Failed to convert config path to string")?;
                    Some(vec![format!("{}:{}", config_path_str, container_path)])
                }
                Some((None, container_path)) => {
                    Some(vec![format!("{}:{}", path_str, container_path)])
                }
                None => None,
            },
            network_mode: Some(format!(
                "{}-{}",
                crate::NETWORK_NAME.to_string(),
                instance_label
            )),
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            ..Default::default()
        };

        let mut container_config = Config {
            image: Some(container_image.to_string()),
            env: Some(env_vars),
            labels: Some(labels_view),
            user,
            host_config: Some(host_config),
            ..Default::default()
        };

        if let Some((_, container_port)) = port {
            let port_key = format!("{}/tcp", container_port);
            let exposed_ports = HashMap::from([(port_key.clone(), HashMap::new())]);
            container_config.exposed_ports = Some(exposed_ports);
        }

        let options = CreateContainerOptions {
            name: format!("{}-{}", instance_label, container_image.to_string()),
            platform: None,
        };

        match docker
            .create_container(Some(options), container_config)
            .await
        {
            Ok(response) => {
                let container_id = response.id;
                println!(
                    "{} container successfully created: {:?}",
                    container_image.to_string(),
                    container_id
                );

                match Self::get_status(&docker, &container_id).await {
                    Ok(status) => Ok((container_id, status)),
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

    pub async fn get_status(
        docker: &Docker,
        container_id: &str,
    ) -> Result<ContainerStatus, AnyhowError> {
        let container_info = docker
            .inspect_container(container_id, None)
            .await
            .context("Failed to inspect container")?;
        let status = match container_info.state.and_then(|state| state.status) {
            Some(bollard::models::ContainerStateStatusEnum::RUNNING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::EXITED) => ContainerStatus::Stopped,
            _ => ContainerStatus::Unknown,
        };
        Ok(status)
    }

    pub async fn inspect(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            ContainerOperation::Inspect,
        )
        .await
    }

    pub async fn start(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(docker, &container_id.to_string(), ContainerOperation::Start).await
    }

    pub async fn stop(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(docker, &container_id.to_string(), ContainerOperation::Stop).await
    }

    pub async fn restart(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            ContainerOperation::Restart,
        )
        .await
    }

    pub async fn delete(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            ContainerOperation::Delete,
        )
        .await
    }
}

pub async fn handle_container(
    docker: &Docker,
    container_id: &str,
    operation: ContainerOperation,
) -> Result<InstanceContainer, AnyhowError> {
    let container_info = docker
        .inspect_container(container_id, None)
        .await
        .map_err(AnyhowError::from)?;
    let container_status = InstanceContainer::get_status(docker, container_id)
        .await
        .context("Failed to get container status")?;
    let container_config = container_info
        .config
        .ok_or_else(|| AnyhowError::msg("Container config not found"))?;
    let container_image_label = container_config
        .labels
        .as_ref()
        .and_then(|labels| labels.get("image").cloned())
        .unwrap_or_else(|| "Unknown".to_string());

    match operation {
        ContainerOperation::Start => {
            if container_status != ContainerStatus::Running {
                docker
                    .start_container(container_id, None::<StartContainerOptions<String>>)
                    .await
                    .map_err(AnyhowError::from)?;
                info!("{} container successfully started", container_id);
            } else {
                info!(
                    "{} container is already running, skipping start operation",
                    container_id
                );
            }
        }
        ContainerOperation::Stop => {
            if container_status == ContainerStatus::Running {
                docker
                    .stop_container(container_id, None::<StopContainerOptions>)
                    .await
                    .map_err(AnyhowError::from)?;
                info!("{} container successfully stopped", container_id);
            } else {
                info!(
                    "{} container is already stopped, skipping stop operation",
                    container_id
                );
            }
        }
        ContainerOperation::Restart => {
            docker
                .restart_container(container_id, None::<RestartContainerOptions>)
                .await
                .map_err(AnyhowError::from)?;
            info!("{} container successfully restarted", container_id);
        }
        ContainerOperation::Delete => {
            if container_status == ContainerStatus::Running {
                docker
                    .stop_container(container_id, None::<StopContainerOptions>)
                    .await
                    .map_err(AnyhowError::from)?;
            }
            docker
                .remove_container(container_id, None::<RemoveContainerOptions>)
                .await
                .map_err(AnyhowError::from)?;
            info!("{} container successfully deleted", container_id);
        }
        ContainerOperation::Inspect => {
            // Inspection already occurred at the start; this is just to match the case
            info!("{} container successfully inspected", container_id);
        }
    }

    Ok(InstanceContainer {
        container_id: container_id.to_string(),
        container_image: ContainerImage::from_str(&container_image_label),
        container_status,
    })
}
