# 📚 Mangalib Parser

> A fast and efficient Rust-based parser for Mangalib with multiple integration options

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.9.2-blue.svg)](https://github.com/Filipponik/rustamanga-mangalib-parser)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

---

## ✨ Features

- 🚀 **Fast & Efficient** - Built with Rust for maximum performance
- 🌐 **Web Server Mode** - Built-in HTTP server with REST API
- 🐰 **RabbitMQ Integration** - Consume tasks from message queues
- 🎯 **Multiple Deployment Options** - CLI, server, or consumer mode
- 🔄 **Concurrent Processing** - Handle multiple manga scraping tasks simultaneously
- 🎨 **Modern Stack** - Uses Axum, Tokio, and headless Chrome

---

## 📋 Table of Contents

- [Prerequisites](#-prerequisites)
- [Installation](#-installation)
- [Usage](#-usage)
  - [Send Resource](#send-resource)
  - [Web Server Mode](#web-server-mode)
  - [RabbitMQ Consumer](#rabbitmq-consumer)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🔧 Prerequisites

Before you begin, ensure you have the following installed:

- **Google Chrome** or **Chromium** - Required for headless browsing
- **Rust 1.70+** - For building from source (optional if using pre-built binaries)

---

## 📦 Installation

### Option 1: Build from Source

```bash
git clone git@github.com:Filipponik/rustamanga-mangalib-parser.git
cd rustamanga-mangalib-parser
cargo build --release
cd target/release
```

### Option 2: Download Pre-built Binary

Download the latest release from the [GitHub releases page](https://github.com/Filipponik/rustamanga-mangalib-parser/releases).

---

## 🚀 Usage

### Send Resource

Send a manga resource to a specified callback URL.

```bash
Usage: ./rustamanga-mangalib-parser send-resource [OPTIONS]

Options:
      --url <URL>  URL where we should send this resource
  -h, --help       Print help
```

**Example:**

```bash
./rustamanga-mangalib-parser send-resource --url=https://example.com
```

---

### Web Server Mode

Start an HTTP server that accepts POST requests to scrape manga data.

```bash
Usage: ./rustamanga-mangalib-parser serve [OPTIONS]

Options:
      --port <PORT>          Web server port
      --browsers <BROWSERS>  Max chrome browsers count
  -h, --help                 Print help
```

**Example:**

```bash
./rustamanga-mangalib-parser serve --port=12345 --browsers=16
```

The server will be available at `http://localhost:{PORT}`

#### API Endpoint

**POST** `/scrap-manga`

**Request Body:**

```json
{
  "slug": "manga-slug",
  "callback_url": "https://example.com"
}
```

---

### RabbitMQ Consumer

Start a RabbitMQ consumer to process manga scraping tasks from a message queue.

```bash
Usage: ./rustamanga-mangalib-parser consume [OPTIONS]

Options:
      --url <URL>            AMQP URI
      --browsers <BROWSERS>  Max chrome browsers count
  -h, --help                 Print help
```

**Example:**

```bash
./rustamanga-mangalib-parser consume --url=amqp://guest:guest@localhost:5672 --browsers=16
```

---

## 🛠️ Development

### Running Tests

```bash
cargo test
```

### Project Structure

```
rustamanga-mangalib-parser/
├── src/                    # Source code
├── tests/                  # Test files
│   └── fixtures/          # Test data
├── resource/              # Helper resources
│   └── json/             # JSON resources
├── Cargo.toml            # Project dependencies
└── README.md             # This file
```

### Configuration

- **Port Settings** - Configure via `--port` flag in serve mode
- **Browser Count** - Adjust concurrent browser instances with `--browsers`
- **AMQP URL** - Set RabbitMQ connection string with `--url` in consume mode

---

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add some amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Guidelines

- Write clear commit messages
- Add tests for new features
- Update documentation as needed
- Follow Rust coding conventions
- Ensure all tests pass before submitting

---

## 📝 Additional Notes

- 🌍 The parser uses Chrome/Chromium in headless mode to render pages and extract data
- ⚙️ All configuration options are provided via CLI flags for flexibility
- 📊 Test fixtures are available in `tests/fixtures/` for development
- 🔍 Helper resources are located in `resource/json/` for reference

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 👤 Author

**Filipponik**

- GitHub: [@Filipponik](https://github.com/Filipponik)
- Repository: [rustamanga-mangalib-parser](https://github.com/Filipponik/rustamanga-mangalib-parser)

---

<p align="center">Made with ❤️ and 🦀 Rust</p>