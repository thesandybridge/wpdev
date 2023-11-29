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

#[post("/instances/create")]
pub async fn create_instance() -> Result<Json<crate::Instance>, String> {
    let docker = Docker::new();
    let mut container_ids = Vec::new();
    let uuid = Uuid::new_v4().to_string();

    crate::create_container!(&docker, "WordPress", crate::WORDPRESS_IMAGE, crate::NETWORK_NAME, &uuid, container_ids);
    crate::create_container!(&docker, "MySQL", crate::MYSQL_IMAGE, crate::NETWORK_NAME, &uuid, container_ids);
    crate::create_container!(&docker, "NGINX", crate::NGINX_IMAGE, crate::NETWORK_NAME, &uuid, container_ids);

    let instance = crate::Instance {
        container_ids,
        uuid,
    };

    Ok(Json(instance))
}

#[post("/instances/<instance_uuid>/start")]
pub async fn start_instance(instance_uuid: &str) -> Result<Json<&str>, Custom<String>> {
    let docker = Docker::new();
    match manager::instance_handler(
        &docker,
        crate::NETWORK_NAME,
        &instance_uuid,
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
        &instance_uuid,
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
        &instance_uuid,
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
        &instance_uuid,
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
    match manager::instances_handler(
        &docker,
        crate::NETWORK_NAME,
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
    match manager::instances_handler(
        &docker,
        crate::NETWORK_NAME,
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
    match manager::instances_handler(
        &docker,
        crate::NETWORK_NAME,
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
    match manager::instances_handler(
        &docker,
        crate::NETWORK_NAME,
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

