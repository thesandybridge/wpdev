use log::error;
/// External dependencies
use rocket::get;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use serde_json;
use shiplift::Docker;
use uuid::Uuid;

use wpdev_core::docker::container::InstanceContainer;
/// Internal dependencies
use wpdev_core::docker::instance::Instance;
use wpdev_core::ContainerEnvVars;

#[post("/instances/create", data = "<env_vars>")]
pub async fn create_instance(
    env_vars: Option<Json<ContainerEnvVars>>,
) -> Result<Json<Instance>, Custom<String>> {
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    // Default environment variables if no data is provided
    let default_env_vars = ContainerEnvVars::default(); // Ensure you have a default implementation

    // Use the provided env_vars if available, otherwise use default
    let env_vars = env_vars.map_or(default_env_vars, |json| json.into_inner());

    match Instance::new(&docker, wpdev_core::NETWORK_NAME, &uuid, env_vars).await {
        Ok(instance) => Ok(Json(instance)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/instances/<instance_uuid>/inspect")]
pub async fn inspect_instance(instance_uuid: &str) -> Result<Json<Instance>, Custom<String>> {
    let docker = Docker::new();
    match Instance::inspect(&docker, wpdev_core::NETWORK_NAME, instance_uuid).await {
        Ok(instance) => Ok(Json(instance)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/instances/inspect_all")]
pub async fn inspect_all_instances() -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instance) => Ok(Json(instance)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/start")]
pub async fn start_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::start(&docker, wpdev_core::NETWORK_NAME, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub async fn stop_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::stop(&docker, wpdev_core::NETWORK_NAME, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub async fn restart_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::restart(&docker, wpdev_core::NETWORK_NAME, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/start_all")]
pub async fn start_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::start_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/stop_all")]
pub async fn stop_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::stop_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/restart_all")]
pub async fn restart_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::restart_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/delete")]
pub async fn delete_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::delete(&docker, wpdev_core::NETWORK_NAME, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/purge")]
pub async fn delete_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match Instance::delete_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/containers/<container_id>/inspect")]
pub async fn inspect_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::new();
    match InstanceContainer::inspect(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/start")]
pub async fn start_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::new();
    match InstanceContainer::start(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/stop")]
pub async fn stop_container(container_id: &str) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::new();
    match InstanceContainer::stop(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/restart")]
pub async fn restart_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::new();
    match InstanceContainer::restart(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/delete")]
pub async fn delete_container(container_id: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match InstanceContainer::delete(&docker, container_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/instances/ws")]
pub fn inspect_instance_ws(ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>
        let docker = Docker::new();

        for await message in ws {
            match message {
                Ok(ws::Message::Text(text)) => {
                    if text == "request_inspect" {
                        // Process the inspection request
                        match Instance::inspect_all(
                            &docker,
                            wpdev_core::NETWORK_NAME,
                        ).await {
                            Ok(instances) => {
                                let response = serde_json::to_string(&instances).unwrap();
                                yield ws::Message::Text(response);
                            },
                            Err(e) => {
                                error!("Error during instance inspection: {}", e);
                                let error = serde_json::to_string(&e.to_string()).unwrap();
                                yield ws::Message::Text(error);
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("WebSocket error: {}", e);
                },
                _ => {
                }
            }
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_instance,
        delete_instance,
        delete_all_instances,
        inspect_instance,
        inspect_all_instances,
        start_instance,
        stop_instance,
        restart_instance,
        start_all_instances,
        stop_all_instances,
        restart_all_instances,
        inspect_container,
        start_container,
        stop_container,
        restart_container,
        delete_container,
        inspect_instance_ws,
    ]
}
