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
pub async fn start_instance(instance_uuid: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::start_all_containers_in_instance(&docker, crate::NETWORK_NAME, instance_uuid).await {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub async fn stop_instance(instance_uuid: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::stop_all_containers_in_instance(&docker, crate::NETWORK_NAME, &instance_uuid).await {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub async fn restart_instance(instance_uuid: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::restart_all_containers_in_instance(&docker, crate::NETWORK_NAME, &instance_uuid).await {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/instances/<instance_uuid>/delete")]
pub async fn delete_instance(instance_uuid: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::delete_all_containers_in_instance(&docker, crate::NETWORK_NAME, &instance_uuid).await {
        Ok(_) => Ok(Json(instance_uuid)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/instances/stop_all")]
pub async fn stop_all_instances() -> Result<(), String> {
    let docker = Docker::new();
    match manager::stop_all_instances(&docker, crate::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_instances,
        create_instance,
        stop_all_instances,
        delete_instance,
        start_instance,
        restart_instance,
        stop_instance,
    ]
}

