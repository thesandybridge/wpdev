use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct DockerComposeInstance {
    name: String,
    port: u16,
    db_user: String,
    db_password: String,
    // Add other relevant fields
}

pub fn create_docker_compose(instance: &DockerComposeInstance) -> std::io::Result<()> {
    let compose_content = format!(
        r#"version: '3'
services:
  wordpress:
    image: wordpress:latest
    ports:
      - "{}:80"
    environment:
      WORDPRESS_DB_HOST: db
      WORDPRESS_DB_USER: wordpress
      WORDPRESS_DB_PASSWORD: wordpress
      WORDPRESS_DB_NAME: wordpress_{}
    volumes:
      - ./wp-content:/var/www/html/wp-content
    depends_on:
      - db

  db:
    image: mysql:5.7
    environment:
      MYSQL_DATABASE: wordpress_{}
      MYSQL_USER: wordpress
      MYSQL_PASSWORD: wordpress
      MYSQL_RANDOM_ROOT_PASSWORD: '1'
    volumes:
      - db_data:/var/lib/mysql

volumes:
  db_data:
"#,
        instance.port, instance.name, instance.name
    );

    let mut file = File::create(format!("{}_docker-compose.yml", instance.name))?;
    file.write_all(compose_content.as_bytes())?;
    Ok(())
}

fn execute_docker_compose(instance: &DockerComposeInstance) -> Result<(), String> {
    let compose_file_path = format!("{}_docker-compose.yml", instance.name);
    if !Path::new(&compose_file_path).exists() {
        return Err("docker-compose file not found".to_string());
    }

    let output = Command::new("docker-compose")
        .args(&["-f", &compose_file_path, "up", "-d"])
        .output()
        .expect("Failed to execute docker-compose");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn save_instances_to_file(instances: &[DockerComposeInstance], file_path: &str) -> Result<(), std::io::Error> {
    let json = serde_json::to_string(instances)?;
    let mut file = File::create(Path::new(file_path))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn load_instances_from_file(file_path: &str) -> Result<Vec<DockerComposeInstance>, std::io::Error> {
    let mut file = File::open(Path::new(file_path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let instances = serde_json::from_str(&contents)?;
    Ok(instances)
}
