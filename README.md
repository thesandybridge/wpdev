# wpdev 🚀

> [!WARNING]
> ## Disclaimer
> Please note that wpdev is currently in active development. This means that the software is still evolving and may contain incomplete features, bugs, or undergo significant changes. While we encourage you to explore and even contribute to the project, we recommend caution when using it in a production environment. For the most stable experience, consider using the releases tagged as stable in our repository.

## Overview

wpdev an integrated solution for managing WordPress environments. It consists of three core components:

1. 🖥️ Frontend WebApp Dashboard: A HTMX and Actix-web dashboard for a user-friendly interface to manage WordPress environments.
2. 🔧 Backend API: Developed using Rust with Rocket and Bollard, this API handles the setup and management of WordPress, Nginx, Adminer, and MySQL Docker containers within an instance.
3. ⌨️ CLI Tool: An alternative to the Frontend WebApp, providing command-line access to the same functionalities.

## Getting Started 🌟

### Prerequisites

- Basic unix command-line knowledge
- Rust (for Backend API) -> [Install Rust](https://www.rust-lang.org/tools/install)
- Docker (for container management) -> [Install Docker](https://docs.docker.com/get-docker/)
- wpdev relies on the following Docker images: (mysql:latest, wordpress:latest,
  nginx:latest, wordpress:cli) -- if you don't want wpdev to pull these images automatically you can pull them yourself and wpdev will just check for them.

## Installation 🛠️

We have 3 binaries available in the tagged release. [Latest](https://github.com/thesandybridge/wpdev/releases/latest)

To get started just run whicever binary you installed, the CLI can just be run directly in your terminal or you can add it to your environment path. The API and webapp will need to be run in the terminal and left open to keep it persistant, optionally you can set it up as a service. I am working on a wiki with instructions on how to do this.

If you would prefer to build from source, follow the instructions below.

### Frontend WebApp Dashboard

1. Clone the repository and navigate to the frontend directory:

```bash
git clone https://github.com/thesandybridge/wpdev.git
```
```bash
cd wpdev
```

2. Build the Rust project:

```bash
cargo build -p wpdev_frontend
```

3. Run the web server:

```bash
cargo run -p wpdev_frontend
```

4. Open the WebApp in your browser:

```bash
open http://localhost:8080
```

### Backend API

1. Navigate to the backend directory:

```bash
cd wpdev
```

2. Build the Rust project:

```bash
cargo build -p wpdev_api
```

3. Run the API server:

```bash
cargo run -p wpdev_api
```

### CLI Tool

1. Navigate to the CLI tool directory:

```bash
cd wpdev
```

2. Build the CLI tool:

```bash
cargo build -p wpdev_cli --release
```

3. Link the binary (for Unix) for easy access:

```bash
ln -s target/release/wpdev_cli /usr/local/bin/wpdev
```

## Usage 💻

> !IMPORTANT
> Running wpdev for the first time will be slow as it fetches the required
> docker images. Once the images have been pulled subsequent runs will be
> much faster.

### Frontend WebApp

- Access the WebApp through http://localhost:8080.
- Manage WordPress environments through the user-friendly dashboard.

### Backend API

- The API runs on http://localhost:8000.
- Use the API endpoints to create, manage, and delete WordPress environments.

### CLI Tool

- Run wpdev --help for a list of commands and usage instructions.
- Perform similar operations as the WebApp through command-line instructions.

> [!NOTE]
> Although it works on Windows, I do not recommend using the cli on windows,
> stick to the WebApp. The cli will require some setup such as getting it added
> to your path and we do not have documentation to support this.

## Configuration

During initial usage a directory will be made in your OS config directory:

| OS      | Configuration Path                                     | Example Path                                               |
|---------|--------------------------------------------------------|------------------------------------------------------------|
| Linux   | `$XDG_CONFIG_HOME` or `$HOME/.config/wpdev`            | `/home/alice/.config/wpdev`                                |
| macOS   | `$HOME/Library/Application Support/wpdev`              | `/Users/Alice/Library/Application Support/wpdev`           |
| Windows | `{FOLDERID_RoamingAppData}\wpdev`                       | `C:\Users\Alice\AppData\Roaming\wpdev`                     |

> !TIP
> This path will also be where each WordPress site will be installed and managed.
> During initial setup the config directory `wpdev` is created and when a site is
> created it gets added to `wpdev/instances`. We generate a UUID for the site
> path.

wpdev can be configured by adding a `config.toml` to the configuration directory
mentioned above. It will be configured with the following defaults (all of which
can be changed):

Some of these are for debugging purposes, however, the goal of wpdev is to be
highly configurable and dev focused. I want users to be able to modify and
change whatever they want, so I plan on making the images more modular and each
setup more configurable. Currently the container setup is hardcoded.
```toml
custom_root: "OS_CONFIG/wpdev",
docker_images: [ # mainly for debugging
"wordpress:latest",
"nginx:latest",
"mysql:latest",
"wordpress:cli"
]
log_level: "none", # set the log_level to "INFO" to see verbose output
enable_frontend: false, # currently not managing anything. This may be removed
site_url: "http://localhost",
adminer_url: "http://localhost",
cli_colored_output: true,
web_app_ip: "127.0.0.1",
web_app_port: 8080,
api_ip: "127.0.0.1",
api_port: 8001,
cli_theme: None # uses bat themes
```
When a site is created an `instance.toml` file will be added to the site config
directory. This is also configurable and is how the webapp pulls data, wpdev is
entirely file/directory based so we do not log info to a database. Instances are
managed through config files and each site config.

> !NOTE
> The WordPress options are placeholders, we do not run an install script
> during container setup. This is part of our future plans though. So for now
> ignore the first 5 items in the field below.

```toml
admin_user: "",
admin_password: "",
admin_email: "",
site_title: "",
site_url: "",
adminer_url: "wordpress",
adminer_user: "wordpress",
adminer_password: "password",
network_name: "<wp-network-{instance_uuid}>",
nginx_port: u32,
adminer_port: u32,
```

## Roadmap 🛣️

The roadmap outlines the planned improvements and major features that are in the pipeline for wpdev. This list is subject to change and will be updated as the project evolves.

- [x] Rebuild Frontend with Actix-web/HTMX: Transition the current Next.js-based frontend to Actix-web/HTMX to enhance performance and maintainability.
- [x] Add logging and more config customization.
- [ ] Add logging to the webapp UI and implement health checks for each
instance.
- [ ] Add options for updating the instance.toml from the webapp, including an
option to generate the instance.toml via an interactive form.
- [ ] Build a cleaner UI that is more feature rich and user friendly with theme
  options, plugin management, and more.
- [ ] Add support for changing PHP and MySQL versions as well managing WordPress
  updates directly from the webapp.
- [ ] Create documenation and Wiki for wpdev.

Please note that this roadmap is indicative and might evolve based on the project's progress, community feedback, and contributor availability.

## Contributing 👥

Contributions are welcome. Please read the [Contributing Guidelines](CONTRIBUTING.md) file for guidelines on how to contribute.

## License 📄

This project is licensed under the [MIT License](LICENSE).

## Support 🛟

For support, please open an issue in the GitHub repository or contact the maintainers.
