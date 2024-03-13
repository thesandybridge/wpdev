use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use rust_embed::RustEmbed;
use serde::Serialize;
use tera::{Context, Tera};

mod api;

#[derive(Serialize)]
struct IndexContext {
    api_url: String,
}

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

async fn index() -> actix_web::Result<HttpResponse> {
    let asset = TemplateAssets::get("index.html.tera").expect("Template not found");
    let template_str = std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");

    let mut tera = Tera::default();
    tera.add_raw_template("index.html.tera", template_str)
        .expect("Failed to load template");

    let mut context = Context::new();
    context.insert("api_url", "127.0.0.1:8000");

    let rendered = tera
        .render("index.html.tera", &context)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

async fn htmx_js() -> Result<HttpResponse, Error> {
    let asset = StaticAssets::get("htmx.min.js").expect("File not found");
    Ok(HttpResponse::Ok()
        .content_type("application/javascript")
        .body(asset.data.into_owned()))
}

async fn styles() -> Result<HttpResponse, Error> {
    let asset = StaticAssets::get("style.css").expect("File not found");
    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .body(asset.data.into_owned()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_methods(vec!["GET", "POST", "OPTIONS", "DELETE"])
            .allowed_headers(vec!["Content-Type", "*"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/list_all_instances").route(web::get().to(api::inspect_all)))
            .service(
                web::resource("/list_instance/{id}").route(web::get().to(api::inspect_instance)),
            )
            .service(web::resource("/create_instance").route(web::post().to(api::create_instance)))
            .service(
                web::resource("/delete_instances")
                    .route(web::delete().to(api::delete_all_instances)),
            )
            .service(
                web::resource("/delete_instance/{id}")
                    .route(web::delete().to(api::delete_instance)),
            )
            .service(
                web::resource("/start_instance/{id}").route(web::post().to(api::start_instance)),
            )
            .service(web::resource("/stop_instance/{id}").route(web::post().to(api::stop_instance)))
            .service(
                web::resource("/restart_instance/{id}")
                    .route(web::post().to(api::restart_instance)),
            )
            .service(
                web::resource("/start_all_instances")
                    .route(web::post().to(api::start_all_instances)),
            )
            .service(
                web::resource("/stop_all_instances").route(web::post().to(api::stop_all_instances)),
            )
            .service(
                web::resource("/restart_all_instances")
                    .route(web::post().to(api::restart_all_instances)),
            )
            .service(web::resource("/static/htmx.min.js").route(web::get().to(htmx_js)))
            .service(web::resource("/static/style.css").route(web::get().to(styles)))
            .service(fs::Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
