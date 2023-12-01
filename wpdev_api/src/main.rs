#[macro_use] extern crate rocket;

use wpdev_core::config;

mod routes;

#[launch]
fn rocket() -> _ {

    if let Err(err) = tokio::runtime::Runtime::new().unwrap()
            .block_on(
                async {
                    config::pull_docker_image_if_not_exists(wpdev_core::WORDPRESS_IMAGE).await?;
                    config::pull_docker_image_if_not_exists(wpdev_core::NGINX_IMAGE).await?;
                    config::pull_docker_image_if_not_exists(wpdev_core::MYSQL_IMAGE).await?;
                    config::pull_docker_image_if_not_exists(wpdev_core::ADMINER_IMAGE).await?;
                    Ok::<_, shiplift::errors::Error>(())
                }
            )
    {
        eprintln!("Error pulling Docker images: {:?}", err);
        std::process::exit(1);
    }

    rocket::build().mount("/api", routes::routes())
}
