# wpdev

## Overview

wpdev an integrated solution for managing WordPress environments. It consists of three core components:

1. Frontend WebApp Dashboard: A Next.js-based dashboard for a user-friendly interface to manage WordPress environments.
2. Backend API: Developed using Rust with Rocket and Shiplift, this API handles the setup and management of WordPress, Nginx, Adminer, and MySQL Docker containers within an instance.
3. CLI Tool: An alternative to the Frontend WebApp, providing command-line access to the same functionalities.

## Getting Started

### Prerequisites

- Node.js (for Frontend WebApp)
- Rust (for Backend API)
- Docker (for container management)

## Installation

### Frontend WebApp Dashboard

1. Clone the repository and navigate to the frontend directory:

```bash
git clone https://github.com/thesandybridge/wpdev.git
cd wpdev/wpdev_frontend
```

2. Install dependencies:

```bash
npm install
```

3. Start the development server:

```bash
npm run dev
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

3. Link the binary for easy access:

```bash
ln -s target/release/wpdev_cli /usr/local/bin/wpdev
```

## Usage

### Frontend WebApp

- Access the WebApp through http://localhost:3000.
- Manage WordPress environments through the user-friendly dashboard.

### Backend API

- The API runs on http://localhost:8000.
- Use the API endpoints to create, manage, and delete WordPress environments.

### CLI Tool

- Run wpdev --help for a list of commands and usage instructions.
- Perform similar operations as the WebApp through command-line instructions.

## Contributing

Contributions are welcome. Please read the [CONTRIBUTING.md]("./CONTRIBUTING.md") file for guidelines on how to contribute.

## License

This project is licensed under the MIT License.

## Support

For support, please open an issue in the GitHub repository or contact the maintainers.
