use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use anyhow::Result;
use rust_embed::RustEmbed;
use serde::Serialize;
use tera::{Context, Tera};
use wpdev_core::config;

mod handlers;
use env_logger;

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
    let asset = TemplateAssets::get("index.html").expect("Template not found");
    let template_str = std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");
    let config = config::read_or_create_config()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut tera = Tera::default();
    tera.add_raw_template("index.html", template_str)
        .expect("Failed to load template");

    let mut context = Context::new();
    context.insert(
        "api_url",
        &format!("http://{}:{}", config.api_ip, config.api_port),
    );

    let rendered = tera
        .render("index.html", &context)
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

fn create_tera_instance() -> Result<Tera, actix_web::Error> {
    let mut tera = Tera::default();

    for file in TemplateAssets::iter() {
        let asset = TemplateAssets::get(&file).expect(&format!("Template {} not found", file));
        let template_str =
            std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");
        tera.add_raw_template(&file, template_str)
            .expect("Failed to load template");
    }

    Ok(tera)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let config = config::read_or_create_config().await?;
    let host_bind = format!("{}:{}", config.web_app_ip, config.web_app_port);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config.log_level))
        .init();
    let cors_allowed_origin = format!("http://{}", host_bind);
    let tera = create_tera_instance().expect("Failed to create Tera instance");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&cors_allowed_origin)
            .allowed_methods(vec!["GET", "POST", "OPTIONS", "DELETE"])
            .allowed_headers(vec!["Content-Type", "*"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(tera.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/static/htmx.min.js").route(web::get().to(htmx_js)))
            .service(web::resource("/static/style.css").route(web::get().to(styles)))
            .configure(handlers::config)
    })
    .bind(&host_bind)?
    .run()
    .await?;

    Ok(())
}
