use actix_web::{body, web, HttpResponse, Result};
use bollard::Docker;
use rust_embed::RustEmbed;
use serde_json::json;
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

    match Instance::inspect(&docker, &instance_uuid).await {
        Ok(instance) => {
            let asset = TemplateAssets::get("instance.html.tera").expect("Template not found");
            let template_str =
                std::str::from_utf8(asset.data.as_ref()).expect("Failed to decode template");

            let mut tera = Tera::default();
            tera.add_raw_template("instance.html.tera", template_str)
                .expect("Failed to load template");

            let mut context = Context::new();
            context.insert("instance", &instance);

            let rendered = tera
                .render("instance.html.tera", &context)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

pub async fn inspect_all() -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    let mut rendered_instances = Vec::new();

    match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => {
            for instance in instances {
                let instance_html_fragment =
                    match inspect_instance(web::Path::from(instance.uuid.clone())).await {
                        Ok(response) => {
                            let body = body::to_bytes(response.into_body())
                                .await
                                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                            String::from_utf8_lossy(&body).to_string()
                        }
                        Err(e) => format!("Failed to inspect instance: {}", e), // Handle error
                    };
                rendered_instances.push(instance_html_fragment);
            }

            let instances_html = rendered_instances.join("");

            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(instances_html))
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
            return inspect_all().await;
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
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn delete_instance(path: web::Path<String>) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    println!("Deleting instance: {}", instance_uuid);

    match Instance::delete(&docker, &instance_uuid, false).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn restart_all_instances() -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::restart_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn restart_instance(path: web::Path<String>) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::restart(&docker, &instance_uuid).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn stop_all_instances() -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::stop_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn stop_instance(path: web::Path<String>) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::stop(&docker, &instance_uuid).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn start_instance(path: web::Path<String>) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::start(&docker, &instance_uuid).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub async fn start_all_instances() -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::start_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => {
            return inspect_all().await;
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}
