use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use shiplift::builder::ContainerOptions;

pub mod config;
pub mod utils;
pub mod docker;

pub const NETWORK_NAME: &str = "wp-network";
pub const WORDPRESS_IMAGE: &str = "wordpress:latest";
pub const WORDPRESS_CLI_IMAGE: &str = "wordpress:cli";
pub const NGINX_IMAGE: &str = "nginx:latest";
pub const MYSQL_IMAGE: &str = "mysql:latest";
pub const ADMINER_IMAGE: &str = "adminer:latest";

#[derive(Deserialize)]
pub struct ContainerEnvVars {
    wordpress: Option<HashMap<String, String>>,
}

impl Default for ContainerEnvVars {
    fn default() -> Self {
        ContainerEnvVars {
            wordpress: None,
        }
    }
}

pub struct EnvVars {
    pub adminer: Vec<String>,
    pub mysql: Vec<String>,
    pub wordpress: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerOperation {
    Start,
    Stop,
    Restart,
    Delete,
    Inspect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
    NotFound,
    Deleted,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContainerImage {
    Adminer,
    MySQL,
    Nginx,
    Wordpress,
}

impl ContainerImage {
    pub fn to_string(&self) -> String {
        match self {
            ContainerImage::Adminer => "adminer".to_string(),
            ContainerImage::MySQL => "mysql".to_string(),
            ContainerImage::Nginx => "nginx".to_string(),
            ContainerImage::Wordpress => "wordpress".to_string(),
        }
    }

    pub fn from_string(image: &str) -> Option<Self> {
        match image {
            "adminer" => Some(ContainerImage::Adminer),
            "mysql" => Some(ContainerImage::MySQL),
            "nginx" => Some(ContainerImage::Nginx),
            "wordpress" => Some(ContainerImage::Wordpress),
            _ => None,
        }
    }
}


pub type ContainerInfo = (ContainerOptions, &'static str);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstanceStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
    PartiallyRunning,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        InstanceStatus::Unknown
    }
}

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
                "wordpress:latest".into(),
                "nginx:latest".into(),
                "mysql:latest".into(),
                "adminer:latest".into(),
                "wordpress:cli".into(),
            ],
            enable_logging: true,
            enable_frontend: false,
            site_url: String::from("http://localhost"),
            adminer_url: String::from("http://localhost"),
        }
    }
}
