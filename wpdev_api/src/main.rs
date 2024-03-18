#[macro_use]
extern crate rocket;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};

mod routes;

fn cors() -> Cors {
    let allowed_origins = AllowedOrigins::all();

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS middleware")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(cors())
        .mount("/api", routes::routes())
}
