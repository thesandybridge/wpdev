/// External dependencies
use rocket::get;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status::Custom;
use serde_json;
use shiplift::Docker;
use uuid::Uuid;
use log::{info, error};

/// Internal dependencies
use wpdev_core::docker::instance::{
    purge_instances,
    instance_handler,
    Instance,
    InstanceContainer,
    InstanceSelection,
};

use wpdev_core:: {
    ContainerStatus,
    ContainerEnvVars,
    ContainerOperation,
};

/// Route handlers

#[get("/instances")]
pub async fn list_instances() -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();

    match Instance::list_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(mut instances) => {
            for (instance_id, instance) in instances.iter_mut() {
                info!("Inspecting instance with ID: {}", instance_id);
                for (container_id, container) in instance.containers.iter_mut() {
                    info!("Fetching status for container ID: {}", container_id);
                    match InstanceContainer::get_status(&docker, container_id).await {
                        Ok(Some(status)) => {
                            info!("Status for container ID {}: {:?}", container_id, status);
                            container.container_status = status;
                        },
                        Ok(None) => {
                            warn!("Container ID {} not found. Setting status to NotFound.", container_id);
                            container.container_status = ContainerStatus::NotFound;
                        },
                        Err(err) => {
                            error!("Error fetching status for container ID {}: {}", container_id, err);
                            return Err(Custom(Status::InternalServerError, format!("Error fetching status for container {}: {}", container_id, err.to_string())));
                        }
                    };
                }
                instance.status = Instance::get_status(&instance.containers);
            }

            Ok(Json(instances.values().cloned().collect()))
        },
        Err(e) => {
            error!("Error listing instances: {}", e);
            Err(Custom(Status::InternalServerError, e.to_string()))
        },
    }
}


#[post("/instances/create", data = "<env_vars>")]
pub async fn create_instance(
    env_vars: Option<Json<ContainerEnvVars>>
) ->
Result<Json<Instance>, Custom<String>>
{
    let docker = Docker::new();
    let uuid = Uuid::new_v4().to_string();

    // Default environment variables if no data is provided
    let default_env_vars = ContainerEnvVars::default(); // Ensure you have a default implementation

    // Use the provided env_vars if available, otherwise use default
    let env_vars = env_vars.map_or(default_env_vars, |json| json.into_inner());

    match Instance::new(
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
pub async fn start_instance(instance_uuid: &str) -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Start,
    ).await {
        Ok(instance) => {
            Ok(Json(instance))
        },
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string()))
    }
}

#[post("/instances/<instance_uuid>/stop")]
pub async fn stop_instance(instance_uuid: &str) -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Stop,
    ).await {
        Ok(instance) => {
            Ok(Json(instance))
        },
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/restart")]
pub async fn restart_instance(instance_uuid: &str) -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Restart,
    ).await {
        Ok(instance) => {
            Ok(Json(instance))
        },
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/delete")]
pub async fn delete_instance(instance_uuid: &str) -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Delete,
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/start_all")]
pub async fn start_all_instances() -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Start,
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/restart_all")]
pub async fn restart_all_instances() -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Restart,
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/stop_all")]
pub async fn stop_all_instances() -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Stop,
    ).await {
        Ok(statuses) => Ok(Json(statuses)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/purge")]
pub async fn delete_all_instance() -> Result<(), Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::All,
        ContainerOperation::Delete,
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }?;

    match purge_instances(InstanceSelection::All).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[post("/instances/<instance_uuid>/inspect")]
pub async fn inspect_instance(instance_uuid: &str) -> Result<Json<Vec<Instance>>, Custom<String>> {
    let docker = Docker::new();
    match instance_handler(
        &docker,
        wpdev_core::NETWORK_NAME,
        InstanceSelection::One(instance_uuid.to_string()),
        ContainerOperation::Inspect,
    ).await {
        Ok(instance) => {
            Ok(Json(instance))
        },
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
                        match instance_handler(
                            &docker,
                            wpdev_core::NETWORK_NAME,
                            InstanceSelection::All,
                            ContainerOperation::Inspect,
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
        inspect_instance_ws,
    ]
}

