use log::{info, error};
use uuid::Uuid;
use wpdev_core::docker_service::{
    self,
    purge_instances,
    ContainerStatus,
};
use shiplift::Docker;
use anyhow::{Result, Error as AnyhowError};
use serde_json::Value as Json;

pub async fn list_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match docker_service::Instance::list_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(mut instances) => {
            for (_, instance) in instances.iter_mut() {
                for container_id in &instance.container_ids {
                    match docker_service::fetch_container_status(&docker, container_id).await {
                        Ok(Some(status)) => {
                            instance.container_statuses.insert(container_id.clone(), status);
                        },
                        Ok(None) => {
                            instance.container_statuses.insert(container_id.clone(), ContainerStatus::NotFound);
                        },
                        Err(err) => {
                            return Err(AnyhowError::msg(format!("Error fetching status for container {} : {:?}", container_id, err)))
                        }
                    };
                }
                instance.status = docker_service::Instance::get_status(&instance.container_statuses);
            }

            info!("Successfully listed instances");
            Ok(serde_json::to_value(instances)?)
        },
        Err(e) => {
            error!("Error listing instances: {:?}", e);
            Err(e.into())
        },
    }
}

pub async fn create_instance(env_vars_str: Option<&String>) -> Result<serde_json::Value, AnyhowError> {
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    let env_vars = match env_vars_str {
        Some(str) => serde_json::from_str(str)?,
        None => wpdev_core::docker_service::ContainerEnvVars::default(),
    };

    match wpdev_core::docker_service::Instance::new(
        &docker,
        wpdev_core::NETWORK_NAME,
        &uuid,
        env_vars
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        },
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn start_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Start,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn stop_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Stop,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn restart_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Restart,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn delete_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Delete,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn inspect_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Inspect,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn start_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();
    wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Start,
    ).await?;
    Ok(())
}

pub async fn restart_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();
    wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Restart,
    ).await?;
    Ok(())
}

pub async fn stop_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();
    wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Stop,
    ).await?;
    Ok(())
}

pub async fn delete_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();

    wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Delete,
    ).await?;

    purge_instances(wpdev_core::docker_service::InstanceSelection::All).await?;

    Ok(())
}
