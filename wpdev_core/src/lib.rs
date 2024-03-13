use serde::{Deserialize, Serialize};

pub mod config;
pub mod docker;
pub mod utils;

pub const NETWORK_NAME: &str = "wp-network";
pub const WORDPRESS_IMAGE: &str = "wordpress:latest";
pub const NGINX_IMAGE: &str = "nginx:latest";
pub const MYSQL_IMAGE: &str = "mysql:latest";
pub const ADMINER_IMAGE: &str = "adminer:latest";
pub const WORDPRESS_CLI_IMAGE: &str = "wordpress:cli";

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub custom_root: String,
    pub docker_images: Vec<String>,
    pub enable_logging: bool,
    pub enable_frontend: bool,
    pub site_url: String,
    pub adminer_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            custom_root: String::from(".config/wpdev/instances"),
            docker_images: vec![
                WORDPRESS_IMAGE.to_string(),
                NGINX_IMAGE.to_string(),
                MYSQL_IMAGE.to_string(),
                ADMINER_IMAGE.to_string(),
                WORDPRESS_CLI_IMAGE.to_string(),
            ],
            enable_logging: true,
            enable_frontend: false,
            site_url: String::from("http://localhost"),
            adminer_url: String::from("http://localhost"),
        }
    }
}
