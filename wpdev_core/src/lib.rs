pub mod docker_service;
pub mod config;
pub mod utils;

pub const NETWORK_NAME: &str = "wp-network";
pub const WORDPRESS_IMAGE: &str = "wordpress:latest";
pub const WORDPRESS_CLI_IMAGE: &str = "wordpress:cli";
pub const NGINX_IMAGE: &str = "nginx:latest";
pub const MYSQL_IMAGE: &str = "mysql:latest";
pub const ADMINER_IMAGE: &str = "adminer:latest";
