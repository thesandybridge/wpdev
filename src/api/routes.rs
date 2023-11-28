use rocket::get;
use rocket::serde::json::Json;
use crate::docker::manager; // Adjust this import based on your project structure
use shiplift::Docker;

const NETWORK_NAME: &str = "wp-network";

#[get("/containers")]
pub async fn list_docker_containers() -> Result<Json<Vec<String>>, String> {
    let docker = Docker::new(); // Instantiate Docker here
    match manager::list_all_containers(&docker, NETWORK_NAME).await {
        Ok(containers) => Ok(Json(containers)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/create")]
pub async fn create_wordpress_container() -> Result<Json<String>, String> {
    let docker = Docker::new();
    match manager::create_wordpress_container(&docker).await {
        Ok(container_id) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/start")]
pub async fn start_container(container_id: String) -> Result<Json<String>, String> {
    let docker = Docker::new();
    match manager::start_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/stop")]
pub async fn stop_container(container_id: String) -> Result<Json<String>, String> {
    let docker = Docker::new();
    match manager::stop_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/restart")]
pub async fn restart_container(container_id: String) -> Result<Json<String>, String> {
    let docker = Docker::new();
    match manager::restart_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/<container_id>/delete")]
pub async fn delete_container(container_id: String) -> Result<Json<String>, String> {
    let docker = Docker::new();
    match manager::delete_container(&docker, &container_id).await {
        Ok(_) => Ok(Json(container_id)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/containers/stop_all")]
pub async fn stop_all_containers() -> Result<(), String> {
    let docker = Docker::new();
    match manager::stop_all_containers(&docker, NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// You might also have a function here to return all routes related to the API
pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_docker_containers,
        create_wordpress_container,
        start_container,
        stop_container,
        restart_container,
        delete_container,
        stop_all_containers,
    ]
}

