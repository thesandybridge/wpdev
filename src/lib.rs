use serde::{Serialize, Deserialize};

#[macro_export]
macro_rules! create_container {
    ($docker:expr, $label:expr, $image:expr, $network_name:expr, $uuid:expr, $container_ids:expr) => {
        match manager::create_instance($docker, $label, $image, $network_name, $uuid).await {
            Ok(container_id) => $container_ids.push(container_id),
            Err(e) => return Err(e.to_string()),
        }
    };
}

#[macro_export]
macro_rules! handle_docker_operation {
    ($docker:expr, $container_id:expr, start) => {{
        async move {
            $docker.containers()
                   .get($container_id)
                   .start()
                   .await
                   .map_err(|err|
                       rocket::response::status::Custom(
                           rocket::http::Status::InternalServerError,
                           format!("Error starting container {}: {}", $container_id, err)
                       )
                   )
        }
    }};
    ($docker:expr, $container_id:expr, stop) => {{
        async move {
            $docker.containers()
                   .get($container_id)
                   .stop(None)
                   .await
                   .map_err(|err|
                       rocket::response::status::Custom(
                           rocket::http::Status::InternalServerError,
                           format!("Error stopping container {}: {}", $container_id, err)
                       )
                   )
        }
    }};
}



#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub container_ids: Vec<String>,
    pub uuid: String,
}

