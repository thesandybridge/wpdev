use anyhow::{Error as AnyhowError, Result};
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, RestartContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::Docker;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceContainer {
    pub container_id: String,
    pub container_image: crate::ContainerImage,
    pub container_status: crate::ContainerStatus,
}

impl InstanceContainer {
    pub async fn new(
        docker: &Docker,
        container_image: crate::ContainerImage,
        container_ids: &mut Vec<String>,
    ) -> Result<(String, crate::ContainerStatus), AnyhowError> {
        let name = format!("{}-container", container_image.to_string());
        let options = CreateContainerOptions {
            name: name.clone(),
            platform: None,
        };

        let config = Config {
            image: Some(container_image.to_string()),
            ..Default::default()
        };

        match docker.create_container(Some(options), config).await {
            Ok(response) => {
                let container_id = response.id;
                container_ids.push(container_id.clone());
                println!(
                    "{} container successfully created: {:?}",
                    container_image.to_string(),
                    container_id
                );

                match Self::get_status(docker, &container_id).await {
                    Ok(status) => Ok((
                        container_id,
                        status.unwrap_or(crate::ContainerStatus::Unknown),
                    )),
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
    ) -> Result<Option<crate::ContainerStatus>, AnyhowError> {
        let container_info = docker
            .inspect_container(container_id, None)
            .await
            .map_err(|err| AnyhowError::new(err))?;

        let status = match container_info
            .state
            .as_ref()
            .and_then(|state| state.status.as_ref())
        {
            Some(bollard::models::ContainerStateStatusEnum::RUNNING) => {
                crate::ContainerStatus::Running
            }
            Some(bollard::models::ContainerStateStatusEnum::EXITED) => {
                crate::ContainerStatus::Stopped
            }
            Some(bollard::models::ContainerStateStatusEnum::PAUSED) => {
                crate::ContainerStatus::Paused
            }
            Some(bollard::models::ContainerStateStatusEnum::DEAD) => crate::ContainerStatus::Dead,
            Some(bollard::models::ContainerStateStatusEnum::RESTARTING) => {
                crate::ContainerStatus::Restarting
            }
            _ => crate::ContainerStatus::Unknown,
        };
        Ok(Some(status))
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
    container_id: &str,
    operation: crate::ContainerOperation,
) -> Result<InstanceContainer, AnyhowError> {
    let container_info = docker
        .inspect_container(container_id, None)
        .await
        .map_err(AnyhowError::from)?;
    let container_status = InstanceContainer::get_status(docker, container_id)
        .await?
        .unwrap_or(crate::ContainerStatus::Unknown);
    let container_config = container_info
        .config
        .ok_or_else(|| AnyhowError::msg("Container config not found"))?;
    let container_image_label = container_config
        .labels
        .as_ref()
        .and_then(|labels| labels.get("image").cloned())
        .unwrap_or_else(|| "Unknown".to_string());

    match operation {
        crate::ContainerOperation::Start => {
            if container_status != crate::ContainerStatus::Running {
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
        crate::ContainerOperation::Stop => {
            if container_status == crate::ContainerStatus::Running {
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
        crate::ContainerOperation::Restart => {
            docker
                .restart_container(container_id, None::<RestartContainerOptions>)
                .await
                .map_err(AnyhowError::from)?;
            info!("{} container successfully restarted", container_id);
        }
        crate::ContainerOperation::Delete => {
            if container_status == crate::ContainerStatus::Running {
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
        crate::ContainerOperation::Inspect => {
            // Inspection already occurred at the start; this is just to match the case
            info!("{} container successfully inspected", container_id);
        }
    }

    Ok(InstanceContainer {
        container_id: container_id.to_string(),
        container_image: crate::ContainerImage::from_str(&container_image_label),
        container_status,
    })
}
