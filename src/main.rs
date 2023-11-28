#[macro_use] extern crate rocket;

mod api;
mod docker;

use shiplift::{Docker, PullOptions, ImageListOptions};
use futures::stream::StreamExt;

const NETWORK_NAME: &str = "wp-network";
const WORDPRESS_IMAGE: &str = "wordpress:latest";

async fn image_exists(image_name: &str) -> bool {
    let docker = Docker::new();
    let options = ImageListOptions::default(); // Remove .all()
    let images = docker.images().list(&options).await.unwrap();
    images.iter().any(|image| {
        image.repo_tags.iter().any(|tag| tag.contains(&image_name.to_string())) // Convert image_name to String
    })
}

// Function to pull a Docker image
async fn pull_docker_image_if_not_exists(image_name: &str) -> Result<(), shiplift::errors::Error> {
    if !image_exists(image_name).await {
        let docker = Docker::new();
        let mut pull_options = PullOptions::builder();
        pull_options.image(image_name);
        let pull_stream = docker.images().pull(&pull_options.build());

        pull_stream
            .for_each(|result| async {
                match result {
                    Ok(value) => println!("Pulled image: {:?}", value),
                    Err(err) => eprintln!("Error pulling image: {:?}", err),
                }
            })
            .await;

        println!("Image {} is now available locally.", image_name);
    } else {
        println!("Image {} is already available locally.", image_name);
    }

    Ok(())
}

#[launch]
fn rocket() -> _ {

    // Pull required docker images
    if let Err(err) = tokio::runtime::Runtime::new().unwrap()
            .block_on(
                pull_docker_image_if_not_exists(WORDPRESS_IMAGE)
            )
    {
        eprintln!("Error pulling Docker image: {:?}", err);
        std::process::exit(1);
    }

    rocket::build().mount("/api", api::routes::routes())
}
