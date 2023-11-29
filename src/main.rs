#[macro_use] extern crate rocket;

mod api;
mod docker;

use shiplift::{Docker, PullOptions, ImageListOptions};
use futures::stream::StreamExt;
use wp_dev::*;

const NETWORK_NAME: &str = "wp-network";
const WORDPRESS_IMAGE: &str = "wordpress:latest";
const NGINX_IMAGE: &str = "nginx:latest";
const MYSQL_IMAGE: &str = "mysql:latest";

async fn image_exists(image_name: &str) -> bool {
    let docker = Docker::new();
    let options = ImageListOptions::default();
    let images = docker.images().list(&options).await.unwrap();
    images.iter().any(|image| {
        image.repo_tags.iter().any(|tag| tag.contains(&image_name.to_string())) // Convert image_name to String
    })
}

async fn pull_docker_image_if_not_exists(image_name: &str) -> Result<(), shiplift::errors::Error> {
    if !image_exists(image_name).await {
        let docker = Docker::new();
        let mut pull_options = PullOptions::builder();
        pull_options.image(image_name);
        let mut pull_stream = docker.images().pull(&pull_options.build());

        let mut success = false;
        let mut error_message = None;

        // Process each event in the pull stream
        while let Some(result) = pull_stream.next().await {
            match result {
                Ok(_) => {
                    // Image successfully pulled
                    success = true;
                }
                Err(err) => {
                    error_message = Some(format!("Error pulling image: {:?}", err));
                }
            }
        }

        if success {
            println!("Image {} is now available locally.", image_name);
        } else {
            if let Some(message) = error_message {
                eprintln!("{}", message);
            } else {
                eprintln!("Failed to pull image {}.", image_name);
            }
        }
    } else {
        println!("Image {} is already available locally.", image_name);
    }

    Ok(())
}


#[launch]
fn rocket() -> _ {

    if let Err(err) = tokio::runtime::Runtime::new().unwrap()
            .block_on(
                async {
                    pull_docker_image_if_not_exists(WORDPRESS_IMAGE).await?;
                    pull_docker_image_if_not_exists(NGINX_IMAGE).await?;
                    pull_docker_image_if_not_exists(MYSQL_IMAGE).await?;
                    Ok::<_, shiplift::errors::Error>(())
                }
            )
    {
        eprintln!("Error pulling Docker images: {:?}", err);
        std::process::exit(1);
    }

    rocket::build().mount("/api", api::routes::routes())
}
