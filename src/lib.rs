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

#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub container_ids: Vec<String>,
    pub uuid: String,
}

