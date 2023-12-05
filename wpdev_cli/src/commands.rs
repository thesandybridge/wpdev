use log::{info, error};
use uuid::Uuid;
use shiplift::Docker;
use anyhow::{Result, Error as AnyhowError};
use serde_json::Value as Json;

use wpdev_core::docker::instance::{
    purge_instances,
    instance_handler,
    Instance,
    InstanceContainer,
    InstanceSelection,
};

use wpdev_core::{
    ContainerStatus,
    ContainerEnvVars,
    ContainerOperation,
};

pub async fn list_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::list_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(mut instances) => {
            for (_, instance) in instances.iter_mut() {
                for container_id in &instance.container_ids {
                    match InstanceContainer::get_status(&docker, container_id).await {
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
                instance.status = Instance::get_status(&instance.container_statuses);
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
        None => ContainerEnvVars::default(),
    };

    match Instance::new(
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
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Start,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn stop_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Stop,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn restart_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Restart,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn delete_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Delete,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn inspect_instance(instance_uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Inspect,
    ).await {
        Ok(instance) => {
            Ok(serde_json::to_value(instance)?)
        }
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn start_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();
    instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Start,
    ).await?;
    Ok(())
}

pub async fn restart_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();
    instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Restart,
    ).await?;
    Ok(())
}

pub async fn stop_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();
    instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Stop,
    ).await?;
    Ok(())
}

pub async fn delete_all_instances() -> Result<(), AnyhowError> {
    let docker = Docker::new();

    instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Delete,
    ).await?;

    purge_instances(InstanceSelection::All).await?;

    Ok(())
}
