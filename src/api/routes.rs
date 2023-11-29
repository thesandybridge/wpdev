use rocket::get;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status::Custom;
use crate::docker::manager;
use shiplift::Docker;
use uuid::Uuid;
use std::collections::HashMap;
use log::{info, error};

#[get("/instances")]
pub async fn list_instances() -> Result<Json<HashMap<String, crate::Instance>>, Custom<String>> {
    let docker = Docker::new();
    match manager::list_all_instances(&docker, crate::NETWORK_NAME).await {
        Ok(instances) => {
            if !instances.is_empty() {
                info!("Successffully listed instances");
                Ok(Json(instances))
            } else {
                Err(Custom(Status::NotFound, "No containers found".to_string()))
            }
        },
        Err(e) => {
            error!("Error listing instances: {:?}", e);
            Err(Custom(Status::InternalServerError, e.to_string()))
        },
    }
}

#[post("/instances/create", data = "<env_vars>")]
pub async fn create_instance(env_vars: Option<Json<manager::ContainerEnvVars>>) -> Result<Json<crate::Instance>, Custom<String>> {
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    match env_vars {
        Some(vars) => {
            // Proceed with instance creation if data is provided
            match manager::create_instance(
                &docker,
                crate::NETWORK_NAME,
                &uuid,
                vars.into_inner()
            ).await {
                Ok(container_ids) => {
                    let instance = crate::Instance {
                        container_ids,
                        uuid,
                    };
                    Ok(Json(instance))
                },
                Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
            }
        },
        None => {
            // Handle the case where no data is provided
            Err(Custom(Status::BadRequest, "No data provided".to_string()))
        }
    }
}

#[post("/instances/<instance_uuid>/start")]
pub async fn start_instance(instance_uuid: &str) -> Result<Json<&str>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::One(instance_uuid.to_string()),
        manager::ContainerOperation::Start,
        "started"
        ).await
    {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e),
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub async fn stop_instance(instance_uuid: &str) -> Result<Json<&str>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::One(instance_uuid.to_string()),
        manager::ContainerOperation::Stop,
        "stopped"
        ).await
    {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub async fn restart_instance(instance_uuid: &str) -> Result<Json<&str>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::One(instance_uuid.to_string()),
        manager::ContainerOperation::Restart,
        "restarted"
        ).await
    {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e),
    }
}

#[post("/instances/<instance_uuid>/delete")]
pub async fn delete_instance(instance_uuid: &str) -> Result<Json<&str>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::One(instance_uuid.to_string()),
        manager::ContainerOperation::Delete,
        "deleted"
        ).await
    {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e),
    }
}

#[post("/instances/start_all")]
pub async fn start_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::All,
        manager::ContainerOperation::Start,
        "started"
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[post("/instances/restart_all")]
pub async fn restart_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::All,
        manager::ContainerOperation::Restart,
        "restart"
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[post("/instances/stop_all")]
pub async fn stop_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::All,
        manager::ContainerOperation::Stop,
        "stopped"
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[post("/instances/purge")]
pub async fn delete_all_instance() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        manager::Instance::All,
        manager::ContainerOperation::Delete,
        "deleted"
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
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
    ]
}

