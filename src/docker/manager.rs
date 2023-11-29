use shiplift::{Docker, NetworkCreateOptions};
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use shiplift::rep::Container;
use log::{info, error};
use std::collections::HashMap;

/// Creates a Docker Network if it doesn't already exist.
///
/// # Arguments
///
/// * `docker` - &Docker
/// * `network_name` - name of the network
pub async fn create_network_if_not_exists(docker: &Docker, network_name: &str) -> Result<(), shiplift::Error> {
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
            // Log after successful container creation
            info!("{} container created successfully: {:?}", label, container);
            Ok(container.id)
        }
        Err(err) => {
            // Log the error if container creation fails
            error!("Error creating {} container: {:?}", label, err);
            Err(err)
        }
    }
}

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

pub async fn list_all_instances(docker: &Docker, network_name: &str) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::builder()
              .all()
              .build())
        .await?;
    list_instances(docker, network_name, containers).await
}

pub async fn list_running_instances(docker: &Docker, network_name: &str) -> Result<HashMap<String, crate::Instance>, shiplift::Error> {
    let containers = docker
        .containers()
        .list(&ContainerListOptions::default())
        .await?;
   list_instances(docker, network_name, containers).await
}

pub async fn start_all_containers_in_instance(docker: &Docker, network_name: &str, instance_uuid: &str) -> Result<(), shiplift::Error> {
    let instances = list_all_instances(docker, network_name).await?;

    if let Some(instance) = instances.get(instance_uuid) {
        for container_id in &instance.container_ids {
            match docker.containers().get(container_id).start().await {
                Ok(container) => {
                    info!("{} container successfully started: {:?}", container_id, container);
                    Ok(())
                }
                Err(err) => {
                    error!("Error starting {} container: {:?}", container_id, err);
                    Err(err)
                }
            }?;
        }
    }

    Ok(())
}

pub async fn stop_all_containers_in_instance(docker: &Docker, network_name: &str, instance_uuid: &str) -> Result<(), shiplift::Error> {
    let instances = list_running_instances(docker, network_name).await?;

    if let Some(instance) = instances.get(instance_uuid) {
        for container_id in &instance.container_ids {
            match docker.containers().get(container_id).stop(None).await {
                Ok(container) => {
                    info!("{} container successfully stopped: {:?}", container_id, container);
                    Ok(())
                }
                Err(err) => {
                    error!("Error stopping {} container: {:?}", container_id, err);
                    Err(err)
                }
            }?;
        }
    }

    Ok(())
}

pub async fn delete_all_containers_in_instance(docker: &Docker, network_name: &str, instance_uuid: &str) -> Result<(), shiplift::Error> {
    let instances = list_all_instances(docker, network_name).await?;

    if let Some(instance) = instances.get(instance_uuid) {
        for container_id in &instance.container_ids {
            match docker.containers().get(container_id).delete().await {
                Ok(container) => {
                    info!("{} container successfully deleted: {:?}", container_id, container);
                    Ok(())
                }
                Err(err) => {
                    error!("Error deleting {} container: {:?}", container_id, err);
                    Err(err)
                }
            }?;
        }
    }

    Ok(())
}

pub async fn restart_all_containers_in_instance(docker: &Docker, network_name: &str, instance_uuid: &str) -> Result<(), shiplift::Error> {
    let instances = list_all_instances(docker, network_name).await?;

    if let Some(instance) = instances.get(instance_uuid) {
        for container_id in &instance.container_ids {
            match docker.containers().get(container_id).restart(None).await {
                Ok(container) => {
                    info!("{} container successfully restarted: {:?}", container_id, container);
                    Ok(())
                }
                Err(err) => {
                    error!("Error restarting {} container: {:?}", container_id, err);
                    Err(err)
                }
            }?;
        }
    }

    Ok(())
}

pub async fn stop_all_instances(docker: &Docker, network_name: &str) -> Result<(), shiplift::Error> {
    let running_instances = list_running_instances(docker, network_name).await?;

    for (_, instance) in running_instances.iter() {
        stop_all_containers_in_instance(docker, network_name, &instance.uuid).await?;
    }

    Ok(())
}


