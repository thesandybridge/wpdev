#[macro_use] extern crate rocket;

use wpdev_core::config;

mod routes;

use anyhow::Error;

async fn pull_docker_images_from_config() -> Result<(), Error> {
    let config = config::read_or_create_config().await?;
    for image_name in config.docker_images {
        config::pull_docker_image_if_not_exists(&image_name).await?;
    }
    Ok(())
}

#[launch]
fn rocket() -> _ {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Err(err) = rt.block_on(pull_docker_images_from_config()) {
        eprintln!("Error pulling Docker images: {:?}", err);
        std::process::exit(1);
    }
    rocket::build().mount("/api", routes::routes())
}
