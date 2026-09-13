/*!
 * Rust Project Templates
 *
 * Templates for Rust projects with frameworks and structures.
 * System PIN Ai
 */
use std::path::Path;
use anyhow::Result;
use super::template_utils::*;

pub async fn generate_rust_axum_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    // Create directory structure
    create_dirs(&project_dir, &["src"])?;

    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#, kebab_name);

    write_file(&project_dir.join("Cargo.toml"), &cargo_toml)?;

    let main_rs = format!(r#"use axum::{{
    response::Json,
    routing::get,
    Router,
}};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct Response {{
    message: String,
}}

async fn hello() -> Json<Response> {{
    Json(Response {{
        message: "Hello from {}!".to_string(),
    }})
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let app = Router::new().route("/", get(hello));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!(" Server running on http://{{}}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}}
"#, kebab_name);

    write_file(&project_dir.join("src/main.rs"), &main_rs)?;

    let readme = format!(r#"# {}

A simple Rust web server using Axum.

## Getting Started

```bash
cargo run
```

Run http://localhost:3000 to see the response.
"#, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}

pub async fn generate_rust_actix_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    create_dirs(&project_dir, &["src"])?;

    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.4"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#, kebab_name);

    write_file(&project_dir.join("Cargo.toml"), &cargo_toml)?;

    let main_rs = format!(r#"use actix_web::{{web, App, HttpResponse, HttpServer, Result}};
use serde::Serialize;

#[derive(Serialize)]
struct Response {{
    message: String,
}}

async fn hello() -> Result<HttpResponse> {{
    let response = Response {{
        message: "Hello from {}!".to_string(),
    }};
    Ok(HttpResponse::Ok().json(response))
}}

#[actix_web::main]
async fn main() -> std::io::Result<()> {{
    println!(" Server running on http://127.0.0.1:9393");

    HttpServer::new(|| {{
        App::new().route("/", web::get().to(hello))
    }})
    .bind("127.0.0.1:9393")?
    .run()
    .await
}}
"#, kebab_name);

    write_file(&project_dir.join("src/main.rs"), &main_rs)?;
    let readme = format!(r#"# {}

Rust web server using Actix-web.

## Getting Started

```bash
cargo run
```

Run http://localhost:9393 to see the response.
"#, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}

pub async fn generate_rust_advanced_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    // Create nested directory structure
    create_dirs(&project_dir, &[
        "src/api",
        "src/bin",
        "src/config",
        "src/models",
        "src/services",
        "src/utils",
        "tests",
        "docs"
    ])?;

    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Name <name.name@email.com>"]
description = "Modular Rust application"
license = "MIT"

[dependencies]
# Web framework
axum = {{ version = "0.7", features = ["macros"] }}
tokio = {{ version = "1.0", features = ["full"] }}
tower = "0.4"
tower-http = {{ version = "0.5", features = ["cors", "fs"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Database
sqlx = {{ version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "chrono", "uuid"] }}

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

# Configuration
config = "0.14"

# Utilities
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}

[dev-dependencies]
tokio-test = "0.4"

[[bin]]
name = "server"
path = "src/bin/server.rs"
"#, snake_name);

    write_file(&project_dir.join("Cargo.toml"), &cargo_toml)?;

    // Main lib.rs
    let lib_rs = r#"//! Advanced Rust Application
//!
//! A modular, Rust application with starter architecture.

pub mod api;
pub mod config;
pub mod models;
pub mod services;
pub mod utils;

pub use config::AppConfig;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn init() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Application initialized");
    Ok(())
}
"#;
    write_file(&project_dir.join("src/lib.rs"), lib_rs)?;

    // Server binary
    let server_rs = format!(r#"//! Web server binary

use {}::{{api, config::AppConfig, init}};
use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    init().await?;

    let config = AppConfig::load()?;

    let app = Router::new()
        .nest("/api/v1", api::routes())
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!(" Server starting on http://{{}}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}}
"#, snake_name);
    write_file(&project_dir.join("src/bin/server.rs"), &server_rs)?;

    // API module
    let api_mod = r#"//! API routes and handlers

pub mod handlers;
pub mod middleware;

use axum::{
    routing::{get, post},
    Router,
};

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/users", get(handlers::list_users).post(handlers::create_user))
        .route("/users/:id", get(handlers::get_user))
        .layer(axum::middleware::from_fn(middleware::logging))
}
"#;
    write_file(&project_dir.join("src/api/mod.rs"), &api_mod)?;

    // API handlers
    let handlers_rs = r#"//! HTTP request handlers

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
    })
}

pub async fn list_users(Query(_params): Query<ListUsersQuery>) -> Json<Vec<User>> {
    let users = vec![
        User {
            id: Uuid::new_v4(),
            name: "tamar".to_string(),
            email: "tamar@email.com".to_string(),
            created_at: chrono::Utc::now(),
        }
    ];
    Json(users)
}

pub async fn get_user(Path(id): Path<Uuid>) -> Result<Json<User>, StatusCode> {
    let user = User {
        id,
        name: "ista".to_string(),
        email: "ista@email.com".to_string(),
        created_at: chrono::Utc::now(),
    };
    Ok(Json(user))
}

pub async fn create_user(Json(payload): Json<CreateUserRequest>) -> Result<Json<User>, StatusCode> {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
        created_at: chrono::Utc::now(),
    };
    Ok(Json(user))
}
"#;
    write_file(&project_dir.join("src/api/handlers.rs"), &handlers_rs)?;

    // Middleware
    let middleware_rs = r#"//! HTTP middleware

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;

pub async fn logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        "HTTP request processed"
    );

    response
}
"#;
    write_file(&project_dir.join("src/api/middleware.rs"), &middleware_rs)?;

    // Configuration
    let config_mod = r#"//! Application configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 9393,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/myapp".to_string(),
                max_connections: 10,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(AppConfig::default())
    }
}
"#;
    write_file(&project_dir.join("src/config/mod.rs"), &config_mod)?;

    // Models
    let models_mod = r#"//! Data models

pub mod user;
pub use user::User;
"#;
    write_file(&project_dir.join("src/models/mod.rs"), &models_mod)?;

    let user_model = r#"//! User model

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            created_at: now,
            updated_at: now,
        }
    }
}
"#;
    write_file(&project_dir.join("src/models/user.rs"), &user_model)?;

    // Services
    let services_mod = r#"//! Business logic services

pub mod user_service;
pub use user_service::UserService;
"#;
    write_file(&project_dir.join("src/services/mod.rs"), &services_mod)?;

    let user_service = r#"//! User service

use crate::models::User;
use anyhow::Result;
use uuid::Uuid;

pub struct UserService {
    // TODO: Add database connection pool
}

impl UserService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create_user(&self, name: String, email: String) -> Result<User> {
        Ok(User::new(name, email))
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        // TODO: Implement database query
        Ok(None)
    }
}
"#;
    write_file(&project_dir.join("src/services/user_service.rs"), &user_service)?;

    // Utils
    let utils_mod = r#"//! Utility functions

pub mod validation;
pub use validation::*;
"#;
    write_file(&project_dir.join("src/utils/mod.rs"), &utils_mod)?;

    let validation = r#"//! Input validation utilities

use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 8
}
"#;
    write_file(&project_dir.join("src/utils/validation.rs"), &validation)?;

    // Dockerfile
    let dockerfile = r#"FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /usr/local/bin/server
EXPOSE 9393
CMD ["server"]
"#;
    write_file(&project_dir.join("Dockerfile"), &dockerfile)?;

    // README
    let readme = format!(r#"# {}

Rust application with modular architecture and nested folder structure.

## Project Structure

```
{}/
├── src/
│   ├── api/
│   ├── bin/
│   ├── config/
│   ├── models/
│   ├── services/
│   ├── utils/
│   └── lib.rs
├── tests/
├── docs/
├── Cargo.toml
├── Dockerfile
└── README.md
```

## Features

-  **Axum Web Framework** - Web framework
-  **SQLx Database** - Async SQL toolkit
-  **Structured Logging** - Tracing-based logging
-  **Configuration Management** - Environment-based config
-  **Docker Support** - Containerized deployment
-  **Testing Setup** - Unit and integration tests

## Getting Started

```bash
# Build the project
cargo build

# Run the server
cargo run --bin server

# Run tests
cargo test
```

## API Endpoints

- `GET /api/v1/health` - Health check
- `GET /api/v1/users` - List users
- `POST /api/v1/users` - Create user
- `GET /api/v1/users/:id` - Get user by ID
"#, kebab_name, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}
