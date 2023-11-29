use rocket::get;
use rocket::serde::json::Json;
use crate::docker::manager;
use shiplift::Docker;
use uuid::Uuid;
use std::collections::HashMap;

#[get("/containers")]
pub async fn list_instances() -> Result<Json<HashMap<String, crate::Instance>>, String> {
    let docker = Docker::new(); // Instantiate Docker here
    match manager::list_all_instances(&docker, crate::NETWORK_NAME).await {
        Ok(instances) => Ok(Json(instances)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/create")]
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

#[post("/containers/<container_id>/start")]
pub async fn start_container(container_id: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::start_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/stop")]
pub async fn stop_container(container_id: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::stop_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/restart")]
pub async fn restart_container(container_id: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::restart_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/delete")]
pub async fn delete_container(container_id: &str) -> Result<Json<&str>, String> {
    let docker = Docker::new();
    match manager::delete_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/stop_all")]
pub async fn stop_all_containers() -> Result<(), String> {
    let docker = Docker::new();
    match manager::stop_all_containers(&docker, crate::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// You might also have a function here to return all routes related to the API
pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_instances,
        create_instance,
        start_container,
        stop_container,
        restart_container,
        delete_container,
        stop_all_containers,
    ]
}

