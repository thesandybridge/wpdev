use rocket::get;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status::Custom;

use shiplift::Docker;

use uuid::Uuid;

use log::{info, error};

use std::collections::HashMap;

use wpdev_core::docker_service::{
    self,
    purge_instances,
    Instance,
    ContainerStatus,
};

#[get("/instances")]
pub async fn list_instances() -> Result<Json<HashMap<String, Instance>>, Custom<String>> {
    let docker = Docker::new();
    match docker_service::list_all_instances(&docker, wpdev_core::NETWORK_NAME).await {
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
                            return Err(Custom(Status::InternalServerError, format!("Error fetching status for container {}: {}", container_id, err.to_string())));
                        }
                    };
                }
                instance.status = docker_service::determine_instance_status(&instance.container_statuses);
            }

            info!("Successfully listed instances");
            Ok(Json(instances))
        },
        Err(e) => {
            error!("Error listing instances: {:?}", e);
            Err(Custom(Status::InternalServerError, e.to_string()))
        },
    }
}

#[post("/instances/create", data = "<env_vars>")]
pub async fn create_instance(env_vars: Option<Json<wpdev_core::docker_service::ContainerEnvVars>>) -> Result<Json<Instance>, Custom<String>> {
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    // Default environment variables if no data is provided
    let default_env_vars = wpdev_core::docker_service::ContainerEnvVars::default(); // Ensure you have a default implementation

    // Use the provided env_vars if available, otherwise use default
    let env_vars = env_vars.map_or(default_env_vars, |json| json.into_inner());

    match wpdev_core::docker_service::create_instance(
        &docker,
        wpdev_core::NETWORK_NAME,
        &uuid,
        env_vars
    ).await {
        Ok(instance) => {
            Ok(Json(instance))
        },
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/start")]
pub async fn start_instance(instance_uuid: &str) -> Result<Json<(String, wpdev_core::docker_service::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Start,
        Some(wpdev_core::docker_service::InstanceStatus::Stopped),
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string()))
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub async fn stop_instance(instance_uuid: &str) -> Result<Json<(String, wpdev_core::docker_service::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Stop,
        Some(wpdev_core::docker_service::InstanceStatus::Running),
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub async fn restart_instance(instance_uuid: &str) -> Result<Json<(String, wpdev_core::docker_service::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Restart,
        Some(wpdev_core::docker_service::InstanceStatus::Running),
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/delete")]
pub async fn delete_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Delete,
        Some(wpdev_core::docker_service::InstanceStatus::Stopped),
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/start_all")]
pub async fn start_all_instances() -> Result<Json<Vec<(String, wpdev_core::docker_service::InstanceStatus)>>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Start,
        Some(wpdev_core::docker_service::InstanceStatus::Stopped),
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/restart_all")]
pub async fn restart_all_instances() -> Result<Json<Vec<(String, wpdev_core::docker_service::InstanceStatus)>>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Restart,
        Some(wpdev_core::docker_service::InstanceStatus::Running),
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/stop_all")]
pub async fn stop_all_instances() -> Result<Json<Vec<(String, wpdev_core::docker_service::InstanceStatus)>>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Stop,
        Some(wpdev_core::docker_service::InstanceStatus::Running),
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/purge")]
pub async fn delete_all_instance() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::All,
        wpdev_core::docker_service::ContainerOperation::Delete,
        Some(wpdev_core::docker_service::InstanceStatus::Stopped),
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }?;

    Ok(purge_instances(wpdev_core::docker_service::InstanceSelection::All).await?)
}

#[post("/instances/<instance_uuid>/inspect")]
pub async fn inspect_instance(instance_uuid: &str) -> Result<Json<(String, wpdev_core::docker_service::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match wpdev_core::docker_service::instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        wpdev_core::docker_service::InstanceSelection::One(instance_uuid.to_string()),
        wpdev_core::docker_service::ContainerOperation::Inspect,
        None
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }

}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_instances,
        create_instance,
        delete_instance,
        start_instance,
        restart_instance,
        stop_instance,
        delete_all_instance,
        stop_all_instances,
        restart_all_instances,
        start_all_instances,
        inspect_instance,
    ]
}

