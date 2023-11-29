use shiplift::{Docker, NetworkCreateOptions};
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use shiplift::rep::Container;
use log::{info, error};
use std::collections::HashMap;
use rocket::http::Status;
use rocket::response::status::Custom;

#[derive(Clone)]
pub enum ContainerOperation {
    Start,
    Stop,
    Restart,
    Delete
}

pub enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
}

pub enum InstanceStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
}

pub enum InstanceOperation {
    Start,
    Stop,
    Restart,
    Delete
}

pub enum Instance {
    All,
    One(String)
}

/// Creates a Docker Network if it doesn't already exist.
///
/// # Arguments
///
/// * `docker` - &Docker
/// * `network_name` - name of the network
pub async fn create_network_if_not_exists(
    docker: &Docker,
    network_name: &str
) -> Result<(), shiplift::Error> {
    let networks = docker.networks().list(&Default::default()).await?;
    if networks.iter().any(|network| network.name == network_name) {
        // Network already exists
        info!("Network already exists, skipping...");
        Ok(())
    } else {
        // Create network
        let network_options = NetworkCreateOptions::builder(network_name).build();
        docker.networks().create(&network_options).await?;

        match docker.networks().create(&network_options).await {
            Ok(container) => {
                info!("Wordpress network successfully created: {:?}", container);
                Ok(())
            }
            Err(err) => {
                error!("Error creating network: {:?}", err);
                Err(err)
            }
        }
    }
}

/// Create docker docker containers that are grouped by a unique
/// identifier.
///
/// # Arguments
///
/// * `docker` - Docker interface
/// * `label` - String used to describe the container
/// * `image` - Docker image
/// * `network_name` - Docker network name
/// * `instance_label` - UUID
pub async fn create_instance(
    docker: &Docker,
    label: &str,
    image: &str,
    network_name: &str,
    instance_label: &str,
) -> Result<String, shiplift::Error> {

    create_network_if_not_exists(&docker, &network_name).await?;

    let mut labels = HashMap::new();
    labels.insert("instance", instance_label);

    let options = ContainerOptions::builder(&image)
        .network_mode(crate::NETWORK_NAME)
        .labels(&labels)
        .build();

    match docker.containers().create(&options).await {
        Ok(container) => {
            info!("{} container created successfully: {:?}", label, container);
            Ok(container.id)
        }
        Err(err) => {
            error!("Error creating {} container: {:?}", label, err);
            Err(err)
        }
    }
}

/// List all instances that are currently running.
///
/// # Arguments
///
/// * `docker` -
/// * `network_name` - [TODO:description]
/// * `containers` - [TODO:description]
async fn list_instances(
    docker: &Docker,
    network_name: &str,
    containers: Vec<Container>
) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let mut instances: HashMap<String, crate::Instance> = HashMap::new();

    for container in containers {
        let details = docker.containers().get(&container.id).inspect().await?;
        let network_settings = &details.network_settings;

        if let Some(labels) = &details.config.labels {
            if network_settings.networks.contains_key(network_name) {
                if let Some(instance_label) = labels.get("instance") {
                    instances.entry(instance_label.to_string())
                        .or_insert_with(|| crate::Instance {
                            container_ids: Vec::new(),
                            uuid: instance_label.to_string()
                        })
                        .container_ids.push(container.id);
                }
            }
        }
    }

    Ok(instances)
}

/// List all instances.
///
/// # Arguments
///
/// * `docker` - [TODO:description]
/// * `network_name` - [TODO:description]
pub async fn list_all_instances(
    docker: &Docker,
    network_name: &str
) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::builder()
              .all()
              .build())
        .await?;
    list_instances(docker, network_name, containers).await
}

/// List all instances that are currently running.
///
/// # Arguments
///
/// * `docker` - [TODO:description]
/// * `network_name` - [TODO:description]
pub async fn list_running_instances(
    docker: &Docker,
    network_name: &str
) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::default())
        .await?;
   list_instances(docker, network_name, containers).await
}

async fn handle_instance(
    docker: &Docker,
    network_name: &str,
    instance_uuid: &str,
    operation: ContainerOperation,
    success_message: &str,
) -> Result<(), Custom<String>> {
    let instances = list_all_instances(docker, network_name).await
        .map_err(|e| Custom(Status::InternalServerError, format!("Error listing instances: {}", e)))?;

    if let Some(instance) = instances.get(instance_uuid) {
        for container_id in &instance.container_ids {
            match operation {
                ContainerOperation::Start => {
                    docker.containers().get(container_id).start().await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error starting container {}: {}", container_id, err)))?;
                }
                ContainerOperation::Stop => {
                    docker.containers().get(container_id).stop(None).await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error stopping container {}: {}", container_id, err)))?;
                }
                ContainerOperation::Restart => {
                    docker.containers().get(container_id).restart(None).await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error restarting container {}: {}", container_id, err)))?;
                }
                ContainerOperation::Delete => {
                    docker.containers().get(container_id).delete().await
                        .map_err(|err| Custom(Status::InternalServerError, format!("Error restarting container {}: {}", container_id, err)))?;
                }
            }
            info!("{} container successfully {}", container_id, success_message);
        }
        Ok(())
    } else {
        Err(Custom(Status::NotFound, format!("Instance with UUID {} not found", instance_uuid)))
    }
}

async fn handle_all_instances(
    docker: &Docker,
    network_name: &str,
    operation: ContainerOperation,
    success_message: &str,
) -> Result<(), Custom<String>> {
    let instances = list_all_instances(docker, network_name).await
        .map_err(|e| Custom(Status::InternalServerError, format!("Error listing instances: {}", e)))?;

    for (_, instance) in instances.iter() {
        handle_instance(
            docker,
            network_name,
            &instance.uuid,
            operation.clone(),
            success_message
        ).await?;
    }

    Ok(())
}

pub async fn instance_handler(
    docker: &Docker,
    network_name: &str,
    instance: Instance,
    operation: ContainerOperation,
    success_message: &str,
) -> Result<(), Custom<String>> {
    match instance {
        Instance::All => {
            handle_all_instances(docker, network_name, operation, success_message).await
        }
        Instance::One(instance_uuid) => {
            handle_instance(docker, network_name, &instance_uuid, operation, success_message).await
        }
    }
}
