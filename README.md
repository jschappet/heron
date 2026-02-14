# Project Heron

Project Heron is the shared technology stack for **ReVillage Society** and **Regenerate Skagit**, designed to support place-based participation, membership, events, and skills in a landscape-aware, human-centered way. It emphasizes legibility, stewardship, and optional federation for related initiatives.

This is **not** a SaaS product — it's infrastructure for local landscapes to adopt while retaining autonomy.

## Table of Contents

- Overview
- Features
- Technology Stack
- Getting Started
- Configuration
- Development
- Project Structure
- Contributing
- License

## Overview

Project Heron provides:

- Singular identity management across related landscapes
- Membership and role-based access control
- Content, event, and skill management scoped per landscape
- Optional federation surfaces (skill directories, public events)
- Email communication and mailmerge support
- Static file serving for client assets in development

## Features

- Async HTTP server with Actix Web
- SQLite database with Diesel ORM and connection pooling
- Templating with Handlebars
- Session and identity management
- Secure authentication with bcrypt
- Email sending via Lettre
- QR code generation and image handling
- Configurable environment via `.env` or `dotenvy`
- Built-in database migrations
- Logging and debugging with env_logger

## Technology Stack

- Rust 2024 edition
- Actix Web for HTTP server and routing
- Diesel ORM with SQLite
- Handlebars for server-side templating
- bcrypt for password hashing
- Lettre for email
- Tokio async runtime
- dotenvy for configuration
- Image and QR code utilities for media handling

## Getting Started

### Prerequisites

- Rust 1.80+ (stable)
- SQLite 3.x
- cargo and rustup installed

### Clone and Build

```bash
git clone <repo_url> heron
cd heron
cargo build
```

### Run Migrations

Migrations are automatically applied at startup via `run_migrations`. Ensure your database path is correct in `.env`.

### Run the Server

```bash
cargo run
```

Server binds to the address specified in the configuration (default `127.0.0.1:8080`).

### Development Mode

Static files from `../dist` are served only in debug mode.

```bash
cargo run
```

Logs and debug info are available in the console.

## Configuration

Configuration is managed via `Settings` loaded from `.env`:

```env
DATABASE_URL=heron.db
COOKIE_KEY=supersecretkey
COOKIE_NAME=heron_session
WEB_BIND=127.0.0.1:8080
DEBUG=debug
```

See `settings.rs` for full available options.

## Project Structure

```
src/
 ├─ main.rs             # Entry point
 ├─ routes/             # API route definitions
 ├─ models/             # Diesel models and structs
 ├─ schema.rs           # Diesel schema
 ├─ registration.rs     # Registration endpoints
 ├─ mailing_list.rs     # Mailing list endpoints
 ├─ mailmerge.rs        # Email merge functionality
 ├─ settings.rs         # Configuration loading
 ├─ db.rs               # Database connection & migrations
 ├─ app_state.rs        # Shared application state
 ├─ types.rs            # Core types
 ├─ validator.rs        # Input validation
 └─ build_info.rs       # Build date/time
templates/              # Handlebars templates
dist/                   # Static files
```

## Contributing

Project Heron is intended for co-authors who will **co-design, co-decide, and co-own** features.

- Follow the architectural principles: **singular identity, scoped participation, boring & legible code**
- Run and test all migrations
- Add new functionality as isolated modules when possible
- Respect landscape sovereignty and non-goals

## License

MIT License. See [LICENSE](LICENSE) for details.

