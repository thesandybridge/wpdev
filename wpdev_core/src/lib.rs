use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

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
#[serde(default)]
pub struct AppConfig {
    pub custom_root: String,
    pub docker_images: Vec<String>,
    pub log_level: String,
    pub enable_frontend: bool,
    pub site_url: String,
    pub adminer_url: String,
    pub cli_colored_output: bool,
    pub cli_theme: Option<String>,
    pub web_app_ip: IpAddr,
    pub web_app_port: u16,
    pub api_ip: IpAddr,
    pub api_port: u16,
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
            log_level: String::from("none"),
            enable_frontend: false,
            site_url: String::from("http://localhost"),
            adminer_url: String::from("http://localhost"),
            cli_colored_output: true,
            web_app_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            web_app_port: 8080,
            api_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            api_port: 8001,
            cli_theme: None,
        }
    }
}
