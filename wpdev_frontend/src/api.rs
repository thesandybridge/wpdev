use actix_web::{web, HttpResponse, Result};
use bollard::Docker;
use rust_embed::RustEmbed;
use serde_json::{from_str, json};
use tera::{Context, Tera};
use uuid::Uuid;

use wpdev_core::docker::container::ContainerEnvVars;
use wpdev_core::docker::instance::Instance;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

pub async fn inspect_instance(path: web::Path<String>) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::inspect(&docker, wpdev_core::NETWORK_NAME, &instance_uuid).await {
        Ok(instance) => {
            let asset = TemplateAssets::get("index.html.tera").expect("Template not found");
            let template_str =
                std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");

            let mut tera = Tera::default();
            tera.add_raw_template("index.html.tera", template_str)
                .expect("Failed to load template");

            let mut context = Context::new();
            context.insert("instance", &instance);

            let rendered = tera
                .render("index.html.tera", &context)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

pub async fn inspect_all_instances() -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => {
            let asset = TemplateAssets::get("instances.html.tera").expect("Template not found");
            let template_str =
                std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");

            let mut tera = Tera::default();
            tera.add_raw_template("instances.html.tera", template_str)
                .expect("Failed to load template");

            let mut context = Context::new();
            context.insert("instances", &instances);

            let rendered = tera
                .render("instances.html.tera", &context)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

pub async fn create_instance(body: Option<web::Bytes>) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    let uuid = Uuid::new_v4().to_string();
    let env_vars = body
        .and_then(|b| serde_json::from_slice::<ContainerEnvVars>(&b).ok())
        .unwrap_or_default();

    // Assume Instance::new creates the instance
    match Instance::new(&docker, &uuid, env_vars).await {
        Ok(_) => {
            return inspect_all_instances().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn delete_all_instances() -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::delete_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(istance) => {
            let asset = TemplateAssets::get("instances.html.tera").expect("Template not found");
            let template_str =
                std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");

            let mut tera = Tera::default();
            tera.add_raw_template("instances.html.tera", template_str)
                .expect("Failed to load template");

            let mut context = Context::new();
            context.insert("instance", &istance);

            let rendered = tera
                .render("instances.html.tera", &context)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}
