use rocket::get;
use rocket::serde::json::Json;
use crate::docker::manager;
use shiplift::Docker;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Instance {
    container_ids: Vec<String>,
    uuid: String,
}

#[get("/containers")]
pub async fn list_docker_containers() -> Result<Json<Vec<String>>, String> {
    let docker = Docker::new(); // Instantiate Docker here
    match manager::list_all_containers(&docker, crate::NETWORK_NAME).await {
        Ok(containers) => Ok(Json(containers)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/create")]
pub async fn create_instance() -> Result<Json<Instance>, String> {
    let docker = Docker::new();
    let mut container_ids = Vec::new();
    let uuid = Uuid::new_v4().to_string();

    // Create WordPress Container
    match manager::create_instance(
        &docker,
        "WordPress",
        crate::WORDPRESS_IMAGE,
        crate::NETWORK_NAME,
        &uuid
    ).await {
        Ok(container_id) => container_ids.push(container_id),
        Err(e) => return Err(e.to_string()),
    }

    // Create MySQL Container
    match manager::create_instance(
        &docker,
        "MySQL",
        crate::MYSQL_IMAGE,
        crate::NETWORK_NAME,
        &uuid
    ).await {
        Ok(container_id) => container_ids.push(container_id),
        Err(e) => return Err(e.to_string()),
    }

    // Create NGINX Container
    match manager::create_instance(
        &docker,
        "NGINX",
        crate::NGINX_IMAGE,
        crate::NETWORK_NAME,
        &uuid
    ).await {
        Ok(container_id) => container_ids.push(container_id),
        Err(e) => return Err(e.to_string()),
    }

    let instance = Instance {
        container_ids,
        uuid,
    };

    // If all containers are created successfully, return their IDs
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
        list_docker_containers,
        create_instance,
        start_container,
        stop_container,
        restart_container,
        delete_container,
        stop_all_containers,
    ]
}

