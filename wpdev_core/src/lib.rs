use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub mod config;
pub mod docker;
pub mod utils;

pub const NETWORK_NAME: &str = "wp-network";
pub const WORDPRESS_IMAGE: &str = "wordpress:latest";
pub const NGINX_IMAGE: &str = "nginx:latest";
pub const MYSQL_IMAGE: &str = "mysql:latest";
pub const ADMINER_IMAGE: &str = "adminer:latest";

#[derive(Deserialize)]
pub struct ContainerEnvVars {
    wordpress: Option<HashMap<String, String>>,
}

impl Default for ContainerEnvVars {
    fn default() -> Self {
        ContainerEnvVars { wordpress: None }
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
    Unknown,
}

impl fmt::Display for ContainerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerImage::MySQL => write!(f, "MySQL"),
            ContainerImage::Wordpress => write!(f, "Wordpress"),
            ContainerImage::Nginx => write!(f, "Nginx"),
            ContainerImage::Adminer => write!(f, "Adminer"),
            ContainerImage::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ContainerImage {
    pub fn to_string(&self) -> String {
        match self {
            ContainerImage::Adminer => "adminer".to_string(),
            ContainerImage::MySQL => "mysql".to_string(),
            ContainerImage::Nginx => "nginx".to_string(),
            ContainerImage::Wordpress => "wordpress".to_string(),
            ContainerImage::Unknown => "unknown".to_string(),
        }
    }

    pub fn from_str(image: &str) -> Self {
        match image {
            "adminer" => ContainerImage::Adminer,
            "mysql" => ContainerImage::MySQL,
            "nginx" => ContainerImage::Nginx,
            "wordpress" => ContainerImage::Wordpress,
            _ => ContainerImage::Unknown,
        }
    }
}

impl ContainerStatus {
    pub fn from_str(status: &str) -> Self {
        match status {
            "running" => ContainerStatus::Running,
            "stopped" => ContainerStatus::Stopped,
            "restarting" => ContainerStatus::Restarting,
            "paused" => ContainerStatus::Paused,
            "exited" => ContainerStatus::Exited,
            "dead" => ContainerStatus::Dead,
            _ => ContainerStatus::Unknown,
        }
    }
}

pub type ContainerInfo = (ContainerOperation, &'static str);

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
