use anyhow::{Error as AnyhowError, Result};
use log::info;
use serde::{Deserialize, Serialize};
use shiplift::builder::ContainerOptions;
use shiplift::{Docker, Error as DockerError};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceContainer {
    pub container_id: String,
    pub container_image: crate::ContainerImage,
    pub container_status: crate::ContainerStatus,
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
                log::info!(
                    "{} container successfully created: {:?}",
                    container_type,
                    container
                );

                match Self::get_status(docker, &container.id).await {
                    Ok(status) => Ok((
                        container.id,
                        status.unwrap_or(crate::ContainerStatus::Unknown),
                    )),
                    Err(err) => {
                        log::error!(
                            "Failed to fetch status for container {}: {:?}",
                            container.id,
                            err
                        );
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
            }
            Err(DockerError::Fault { code, .. }) if code.as_u16() == 404 => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn inspect(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            crate::ContainerOperation::Inspect,
        )
        .await
    }

    pub async fn start(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            crate::ContainerOperation::Start,
        )
        .await
    }

    pub async fn stop(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            crate::ContainerOperation::Stop,
        )
        .await
    }

    pub async fn restart(
        docker: &Docker,
        container_id: &str,
    ) -> Result<InstanceContainer, AnyhowError> {
        handle_container(
            docker,
            &container_id.to_string(),
            crate::ContainerOperation::Restart,
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
            crate::ContainerOperation::Delete,
        )
        .await
    }
}

pub async fn handle_container(
    docker: &Docker,
    container_id: &String,
    operation: crate::ContainerOperation,
) -> Result<InstanceContainer, AnyhowError> {
    let container = docker.containers().get(container_id).inspect().await?;
    let container_status = InstanceContainer::get_status(docker, container_id).await?;
    match operation {
        crate::ContainerOperation::Start => {
            if container_status != Some(crate::ContainerStatus::Running) {
                docker.containers().get(container_id).start().await?;
                info!("{} container successfully started", container_id);
            } else {
                info!(
                    "{} container is already running, skipping start operation",
                    container_id
                );
            }
        }
        crate::ContainerOperation::Stop => {
            if container_status == Some(crate::ContainerStatus::Running) {
                docker.containers().get(container_id).stop(None).await?;
                info!("{} container successfully stopped", container_id);
            } else {
                info!(
                    "{} container is already stopped, skipping stop operation",
                    container_id
                );
            }
        }
        crate::ContainerOperation::Restart => {
            if container_status == Some(crate::ContainerStatus::Running) {
                docker.containers().get(container_id).restart(None).await?;
                info!("{} container successfully restarted", container_id);
            } else {
                info!(
                    "{} container is not running, skipping restart operation",
                    container_id
                );
            }
        }
        crate::ContainerOperation::Delete => {
            if container_status == Some(crate::ContainerStatus::Running) {
                // First, stop the container if it's running
                docker.containers().get(container_id).stop(None).await?;
                info!(
                    "{} container successfully stopped before deletion",
                    container_id
                );
            }
            // Then delete the container
            docker.containers().get(container_id).delete().await?;
            info!("{} container successfully deleted", container_id);
        }
        crate::ContainerOperation::Inspect => {
            docker.containers().get(container_id).inspect().await?;
            info!("{} container successfully inspected", container_id);
        }
    }

    Ok(InstanceContainer {
        container_id: container_id.clone(),
        container_image: match container.config.labels {
            Some(ref labels) => match labels.get("image") {
                Some(image) => crate::ContainerImage::from_string(image)
                    .unwrap_or(crate::ContainerImage::Unknown),
                None => crate::ContainerImage::Unknown,
            },
            None => crate::ContainerImage::Unknown,
        },
        container_status: match container_status {
            Some(status) => status,
            None => crate::ContainerStatus::Unknown,
        },
    })
}
