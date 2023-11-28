use shiplift::{Docker, NetworkCreateOptions};
use shiplift::builder::ContainerOptions;
use shiplift::builder::ContainerListOptions;
use log::{info, error};

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

pub async fn create_wordpress_container(docker: &Docker) -> Result<String, shiplift::Error> {
    let network_name = "wp-network";

    create_network_if_not_exists(&docker, network_name).await?;

    let options = ContainerOptions::builder("wordpress:latest")
        .network_mode("wp-network")
        .build();

    match docker.containers().create(&options).await {
        Ok(container) => {
            // Log after successful container creation
            info!("WordPress container created successfully: {:?}", container);
            Ok(container.id)
        }
        Err(err) => {
            // Log the error if container creation fails
            error!("Error creating WordPress container: {:?}", err);
            Err(err)
        }
    }
}

pub async fn list_all_containers(docker: &Docker, network_name: &str) -> Result<Vec<String>, shiplift::Error> {
    // Fetch all containers
    let containers = docker
        .containers()
        .list(&ContainerListOptions::builder().all().build())
        .await?;

    let mut filtered_containers = Vec::new();

    for container in containers {
        let details = docker.containers().get(&container.id).inspect().await?;
        // Directly access network_settings as it's not an Option
        let network_settings = &details.network_settings;

        // Check if network_settings contain the network_name
        if network_settings.networks.contains_key(network_name) {
            filtered_containers.push(container.id);
        }
    }

    Ok(filtered_containers)
}

pub async fn list_running_containers(docker: &Docker, network_name: &str) -> Result<Vec<String>, shiplift::Error> {
    // Fetch all containers
    let containers = docker
        .containers()
        .list(&ContainerListOptions::default())
        .await?;

    let mut filtered_containers = Vec::new();

    for container in containers {
        let details = docker.containers().get(&container.id).inspect().await?;
        // Directly access network_settings as it's not an Option
        let network_settings = &details.network_settings;

        // Check if network_settings contain the network_name
        if network_settings.networks.contains_key(network_name) {
            filtered_containers.push(container.id);
        }
    }

    Ok(filtered_containers)
}

pub async fn start_container(docker: &Docker, container_id: &str) -> Result<(), shiplift::Error> {
    docker.containers().get(container_id).start().await?;
    Ok(())
}

pub async fn stop_container(docker: &Docker, container_id: &str) -> Result<(), shiplift::Error> {
    docker.containers().get(container_id).stop(None).await?;
    Ok(())
}

pub async fn stop_all_containers(docker: &Docker, network_name: &str) -> Result<(), shiplift::Error> {
    let running_containers = list_running_containers(docker, network_name).await?;
    for container_id in running_containers {
        docker.containers().get(container_id).stop(None).await?;
    }
    Ok(())
}

pub async fn restart_container(docker: &Docker, container_id: &str) -> Result<(), shiplift::Error> {
    docker.containers().get(container_id).restart(None).await?;
    Ok(())
}

pub async fn delete_container(docker: &Docker, container_id: &str) -> Result<(), shiplift::Error> {
    docker.containers().get(container_id).delete().await?;
    Ok(())
}
