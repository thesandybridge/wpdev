use actix_web::{delete, get, post, web, HttpResponse, Result};
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

async fn render_template(
    tera: web::Data<Tera>,
    template_name: &str,
    context: &Context,
) -> Result<HttpResponse> {
    let rendered = tera
        .render(&format!("{}.html.tera", template_name), context)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[get("/list_instance/{id}")]
pub(crate) async fn inspect_instance(
    tera: web::Data<Tera>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::inspect(&docker, &instance_uuid).await {
        Ok(instance) => {
            let mut context = Context::new();
            context.insert("instance", &instance);
            render_template(tera, "instance", &context).await
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("/list_all_instances")]
pub(crate) async fn inspect_all(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(instances) => {
            let mut context = Context::new();
            for instance in instances {
                context.insert("instance", &instance);
            }

            render_template(tera, "instance", &context).await
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[post("/create_instance")]
pub(crate) async fn create_instance(
    tera: web::Data<Tera>,
    body: Option<web::Bytes>,
) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    let uuid = Uuid::new_v4().to_string();
    let env_vars = body
        .and_then(|b| serde_json::from_slice::<ContainerEnvVars>(&b).ok())
        .unwrap_or_default();

    match Instance::new(&docker, &uuid, env_vars).await {
        Ok(instance) => {
            let mut context = Context::new();
            context.insert("instance_uuid", &instance);
            render_template(tera, "instance", &context).await
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[delete("/delete_all_instances")]
pub(crate) async fn delete_all_instances(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::delete_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[delete("/delete_instance/{id}")]
pub(crate) async fn delete_instance(
    tera: web::Data<Tera>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::delete(&docker, &instance_uuid, false).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[post("/restart_all_instances")]
pub(crate) async fn restart_all_instances(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::restart_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[post("/restart_instance/{id}")]
pub(crate) async fn restart_instance(
    tera: web::Data<Tera>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::restart(&docker, &instance_uuid).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[post("/stop_all_instances")]
pub(crate) async fn stop_all_instances(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::stop_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[post("/stop_instance/{id}")]
pub(crate) async fn stop_instance(
    tera: web::Data<Tera>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::stop(&docker, &instance_uuid).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[post("/start_instance/{id}")]
pub(crate) async fn start_instance(
    tera: web::Data<Tera>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_uuid = path.into_inner();

    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::start(&docker, &instance_uuid).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

#[post("/start_all_instances")]
pub(crate) async fn start_all_instances(tera: web::Data<Tera>) -> Result<HttpResponse> {
    let docker = Docker::connect_with_defaults().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to connect to Docker: {}", e))
    })?;

    match Instance::start_all(&docker, wpdev_core::NETWORK_NAME).await {
        Ok(_) => match Instance::inspect_all(&docker, wpdev_core::NETWORK_NAME).await {
            Ok(instances) => {
                let mut context = Context::new();
                for instance in instances {
                    context.insert("instance", &instance);
                }

                render_template(tera, "instance", &context).await
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e.to_string()
                })));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })));
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(inspect_instance)
        .service(inspect_all)
        .service(create_instance)
        .service(delete_all_instances)
        .service(delete_instance)
        .service(restart_all_instances)
        .service(restart_instance)
        .service(stop_all_instances)
        .service(stop_instance)
        .service(start_instance)
        .service(start_all_instances);
}
