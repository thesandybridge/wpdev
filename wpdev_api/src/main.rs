#[macro_use] extern crate rocket;

use wpdev_core::config;

mod routes;

#[launch]
fn rocket() -> _ {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Err(err) = rt.block_on(config::pull_docker_images_from_config()) {
        eprintln!("Error pulling Docker images: {:?}", err);
        std::process::exit(1);
    }
    rocket::build().mount("/api", routes::routes())
}
