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

## Installation 🛠️

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
cargo build --bin wpdev_frontend
```

3. Run the web server:

```bash
cargo run --bin wpdev_frontend
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
cargo build --bin wpdev_api
```

3. Run the API server:

```bash
cargo run --bin wpdev_api
```

### CLI Tool

1. Navigate to the CLI tool directory:

```bash
cd wpdev
```

2. Build the CLI tool:

```bash
cargo build --bin wpdev_cli --release
```

3. Link the binary for easy access:

```bash
ln -s target/release/wpdev_cli /usr/local/bin/wpdev
```

## Usage 💻

### Frontend WebApp

- Access the WebApp through http://localhost:8080.
- Manage WordPress environments through the user-friendly dashboard.

### Backend API

- The API runs on http://localhost:8000.
- Use the API endpoints to create, manage, and delete WordPress environments.

### CLI Tool

- Run wpdev --help for a list of commands and usage instructions.
- Perform similar operations as the WebApp through command-line instructions.

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
