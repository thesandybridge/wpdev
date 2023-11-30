use rocket::get;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status::Custom;
use crate::docker::manager::{
    self,
    purge_instances,
};
use crate::docker::manager::Instance;
use crate::docker::manager::ContainerStatus;
use shiplift::Docker;
use uuid::Uuid;
use std::collections::HashMap;
use log::{info, error};

#[get("/instances")]
pub async fn list_instances() -> Result<Json<HashMap<String, Instance>>, Custom<String>> {
    let docker = Docker::new();
    match manager::list_all_instances(&docker, crate::NETWORK_NAME).await {
        Ok(mut instances) => {
            for (_, instance) in instances.iter_mut() {
                for container_id in &instance.container_ids {
                    match manager::fetch_container_status(&docker, container_id).await {
                        Ok(Some(status)) => {
                            instance.container_statuses.insert(container_id.clone(), status);
                        },
                        Ok(None) => {
                            // Handle the 'not found' scenario
                            instance.container_statuses.insert(container_id.clone(), ContainerStatus::NotFound);
                        },
                        Err(err) => {
                            return Err(Custom(Status::InternalServerError, format!("Error fetching status for container {}: {}", container_id, err.to_string())));
                        }
                    };
                }
                instance.status = manager::determine_instance_status(&instance.container_statuses);
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
pub async fn create_instance(env_vars: Option<Json<manager::ContainerEnvVars>>) -> Result<Json<Instance>, Custom<String>> {
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    // Default environment variables if no data is provided
    let default_env_vars = manager::ContainerEnvVars::default(); // Ensure you have a default implementation

    // Use the provided env_vars if available, otherwise use default
    let env_vars = env_vars.map_or(default_env_vars, |json| json.into_inner());

    match manager::create_instance(
        &docker,
        crate::NETWORK_NAME,
        &uuid,
        env_vars
    ).await {
        Ok(container_ids) => {
            let instance = Instance {
                container_ids,
                uuid,
                status: manager::InstanceStatus::Stopped,
                container_statuses: HashMap::new(),
            };
            Ok(Json(instance))
        },
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}


#[post("/instances/<instance_uuid>/start")]
pub async fn start_instance(instance_uuid: &str) -> Result<Json<(String, manager::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::One(instance_uuid.to_string()),
        manager::ContainerOperation::Start,
        Some(manager::InstanceStatus::Stopped),
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub async fn stop_instance(instance_uuid: &str) -> Result<Json<(String, manager::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::One(instance_uuid.to_string()),
        manager::ContainerOperation::Stop,
        Some(manager::InstanceStatus::Running),
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub async fn restart_instance(instance_uuid: &str) -> Result<Json<(String, manager::InstanceStatus)>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::One(instance_uuid.to_string()),
        manager::ContainerOperation::Restart,
        Some(manager::InstanceStatus::Running),
    ).await {
        Ok(mut statuses) => {
            if let Some((id, status)) = statuses.pop() {
                Ok(Json((id, status)))
            } else {
                Err(Custom(Status::InternalServerError, "Instance status not found".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

#[post("/instances/<instance_uuid>/delete")]
pub async fn delete_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::One(instance_uuid.to_string()),
        manager::ContainerOperation::Delete,
        Some(manager::InstanceStatus::Stopped),
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[post("/instances/start_all")]
pub async fn start_all_instances() -> Result<Json<Vec<(String, manager::InstanceStatus)>>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::All,
        manager::ContainerOperation::Start,
        Some(manager::InstanceStatus::Stopped),
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(e),
    }
}

#[post("/instances/restart_all")]
pub async fn restart_all_instances() -> Result<Json<Vec<(String, manager::InstanceStatus)>>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::All,
        manager::ContainerOperation::Restart,
        Some(manager::InstanceStatus::Running),
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(e),
    }
}

#[post("/instances/stop_all")]
pub async fn stop_all_instances() -> Result<Json<Vec<(String, manager::InstanceStatus)>>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::All,
        manager::ContainerOperation::Stop,
        Some(manager::InstanceStatus::Running),
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(e),
    }
}

#[post("/instances/purge")]
pub async fn delete_all_instance() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::InstanceSelection::All,
        manager::ContainerOperation::Delete,
        Some(manager::InstanceStatus::Stopped),
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }?;

    purge_instances(manager::InstanceSelection::All).await
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
    ]
}

