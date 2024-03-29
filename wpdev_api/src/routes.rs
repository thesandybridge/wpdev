use bollard::Docker;
use log::error;
/// External dependencies
use rocket::get;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use serde_json;
use uuid::Uuid;

/// Internal dependencies
use wpdev_core::docker::container::{ContainerEnvVars, InstanceContainer};
use wpdev_core::docker::instance::Instance;

#[post("/instances/create", data = "<env_vars>")]
pub(crate) async fn create_instance(
    env_vars: Option<Json<ContainerEnvVars>>,
) -> Result<Json<Instance>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    let uuid = Uuid::new_v4().to_string();

    let default_env_vars = ContainerEnvVars::default();

    let env_vars = env_vars.map_or(default_env_vars, |json| json.into_inner());

    match Instance::new(&docker, &uuid, env_vars).await {
        Ok(instance) => Ok(Json(instance)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/instances/<instance_uuid>/inspect")]
pub(crate) async fn inspect_instance(
    instance_uuid: &str,
) -> Result<Json<Instance>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::inspect(&docker, instance_uuid).await {
        Ok(instance) => Ok(Json(instance)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/instances/inspect_all")]
pub(crate) async fn inspect_all_instances() -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instance) => Ok(Json(instance)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/start")]
pub(crate) async fn start_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::start(&docker, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub(crate) async fn stop_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::stop(&docker, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub(crate) async fn restart_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::restart(&docker, instance_uuid).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/start_all")]
pub(crate) async fn start_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::start_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/stop_all")]
pub(crate) async fn stop_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::stop_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/restart_all")]
pub(crate) async fn restart_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::restart_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[delete("/instances/<instance_uuid>/delete")]
pub(crate) async fn delete_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::delete(&docker, &instance_uuid, false).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[delete("/instances/purge")]
pub(crate) async fn delete_all_instances() -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match Instance::delete_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/containers/<container_id>/inspect")]
pub(crate) async fn inspect_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match InstanceContainer::inspect(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/start")]
pub(crate) async fn start_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match InstanceContainer::start(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/stop")]
pub(crate) async fn stop_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match InstanceContainer::stop(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/containers/<container_id>/restart")]
pub(crate) async fn restart_container(
    container_id: &str,
) -> Result<Json<InstanceContainer>, Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match InstanceContainer::restart(&docker, container_id).await {
        Ok(container) => Ok(Json(container)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[delete("/containers/<container_id>/delete")]
pub(crate) async fn delete_container(container_id: &str) -> Result<(), Custom<String>> {
    let docker = Docker::connect_with_defaults()
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    match InstanceContainer::delete(&docker, container_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/instances/ws")]
pub(crate) fn inspect_instance_ws(ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>

        let docker = Docker::connect_with_defaults().map_err(|e| {
            error!("Error connecting to Docker: {}", e);
            ws::result::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
        for await message in ws {
            match message {
                Ok(ws::Message::Text(text)) => {
                    if text == "request_inspect" {
                        match Instance::inspect_all(
                            &docker,
                            wpdev_core::NETWORK_NAME,
                        ).await {
                            Ok(instances) => {
                                let response = serde_json::to_string(&instances).map_err(|e| {
                                    error!("Error serializing instance inspection response: {}", e);
                                    ws::result::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                                })?;
                                yield ws::Message::Text(response);
                            },
                            Err(e) => {
                                error!("Error during instance inspection: {}", e);
                                let error = serde_json::to_string(&e.to_string()).map_err(|e| {
                                    error!("Error serializing instance inspection error: {}", e);
                                    ws::result::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                                })?;
                                yield ws::Message::Text(error);
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    ws::result::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e));
                },
                _ => {
                }
            }
        }
    }
}

pub(crate) fn routes() -> Vec<rocket::Route> {
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
