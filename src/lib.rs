use serde::{Serialize, Deserialize};

#[macro_export]
macro_rules! create_container {
    ($docker:expr, $options:expr, $label:expr, $container_ids:expr) => {{
        match $docker.containers().create(&$options).await {
            Ok(container) => {
                log::info!("{} container created successfully: {:?}", $label, container);
                $container_ids.push(container.id.clone());
                Ok(container.id)
            }
            Err(err) => {
                log::error!("Error creating {} container: {:?}", $label, err);
                Err(err)
            }
        }
    }};
}


