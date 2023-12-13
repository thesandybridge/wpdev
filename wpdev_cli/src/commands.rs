use anyhow::{Error as AnyhowError, Result};
use serde_json::Value as Json;
use shiplift::Docker;
use uuid::Uuid;

use wpdev_core::docker::instance::Instance;
use wpdev_core::ContainerEnvVars;

pub async fn create_instance(
    env_vars_str: Option<&String>,
) -> Result<serde_json::Value, AnyhowError> {
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    let env_vars = match env_vars_str {
        Some(str) => serde_json::from_str(str)?,
        None => ContainerEnvVars::default(),
    };

    match Instance::new(&docker, wpdev_core::NETWORK_NAME, &uuid, env_vars).await {
        Ok(instance) => Ok(serde_json::to_value(instance)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn start_instance(uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::start(&docker, wpdev_core::NETWORK_NAME, uuid).await {
        Ok(instance) => Ok(serde_json::to_value(instance)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn stop_instance(uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::stop(&docker, wpdev_core::NETWORK_NAME, uuid).await {
        Ok(instance) => Ok(serde_json::to_value(instance)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn restart_instance(uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::restart(&docker, wpdev_core::NETWORK_NAME, uuid).await {
        Ok(instance) => Ok(serde_json::to_value(instance)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn delete_instance(uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::delete(&docker, wpdev_core::NETWORK_NAME, uuid).await {
        Ok(instance) => Ok(serde_json::to_value(instance)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn delete_all_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::delete_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => Ok(serde_json::to_value(instances)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn inspect_instance(uuid: &String) -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::inspect(&docker, wpdev_core::NETWORK_NAME, uuid).await {
        Ok(instance) => Ok(serde_json::to_value(instance)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn inspect_all_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => Ok(serde_json::to_value(instances)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn restart_all_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::restart_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => Ok(serde_json::to_value(instances)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn stop_all_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::stop_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => Ok(serde_json::to_value(instances)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}

pub async fn start_all_instances() -> Result<Json, AnyhowError> {
    let docker = Docker::new();
    match Instance::start_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => Ok(serde_json::to_value(instances)?),
        Err(e) => Err(AnyhowError::from(e)),
    }
}
