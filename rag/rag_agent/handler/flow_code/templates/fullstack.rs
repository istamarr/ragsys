/*!
 * Project Templates
 *
 * Templates for applications with backend and frontend.
 * System PIN Ai
 */
//search for lang. not yet clear
//sample rust and react, try spring

use std::path::Path;
use anyhow::Result;
use super::template_utils::*;

pub async fn generate_fullstack_rust_react_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, pascal_name) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    create_dirs(&project_dir, &[
        "backend/src",
        "frontend/src/components",
        "frontend/src/services",
        "frontend/src/types",
        "frontend/src/styles",
        "frontend/public",
        "docs",
        "scripts"
    ])?;

    // Root README Writer di hrs flexible
    let root_readme = format!(r#"# {}

Full-stack application with backend and frontend.

## Structure

```
{}/
├── backend/
├── frontend/
├── docker-compose.yml
├── docs/
└── README.md
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Docker & Docker Compose (optional)

### Option 1: Docker Development

```bash
docker-compose up --build
```

Services:
- Backend: http://localhost:9393
- Frontend: http://localhost:3000

### Option 2: Local Development

**Backend:**
```bash
cd backend
cargo run
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```

## Features

### Backend (Rust + Axum)
-  RESTful API with CRUD operations
-  CORS support for frontend integration
-  JSON request/response handling
-  Error handling and logging
-  In-memory data storage (dev)

### Frontend (React + TypeScript)
-  Modern React with hooks
-  TypeScript for type safety
-  Axios for API communication
-  Responsive design
-  Hot module replacement

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/todos` | List all todos |
| POST | `/api/todos` | Create new todo |
| GET | `/api/todos/:id` | Get todo by ID |
| PUT | `/api/todos/:id` | Update todo |
| DELETE | `/api/todos/:id` | Delete todo |

## Development

See each README files in `backend/` and `frontend/` directories for dev.
"#, kebab_name, kebab_name);

    write_file(&project_dir.join("README.md"), &root_readme)?;

    let backend_cargo = format!(r#"[package]
name = "{}_backend"
version = "0.1.0"
edition = "2021"
description = "Full-stack {} backend API"

[dependencies]
axum = {{ version = "0.7", features = ["macros"] }}
tokio = {{ version = "1.0", features = ["full"] }}
tower = "0.4"
tower-http = {{ version = "0.5", features = ["cors", "fs", "trace"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}

[[bin]]
name = "server"
path = "src/main.rs"
"#, snake_name, kebab_name);

    write_file(&project_dir.join("backend/Cargo.toml"), &backend_cargo)?;

    // Backend main.rs
    let backend_main = format!(r#"//! Full-stack backend server
//!
//! Rust backend API for the {} application

use axum::{{
    extract::{{Path, Query, State}},
    http::{{StatusCode, Method}},
    response::Json,
    routing::{{get, post, delete}},
    Router,
}};
use serde::{{Deserialize, Serialize}};
use std::{{collections::HashMap, sync::Arc}};
use tokio::sync::RwLock;
use tower_http::cors::{{Any, CorsLayer}};
use tracing::{{info, instrument}};
use uuid::Uuid;

// Application state
type AppState = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {{
    id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}}

#[derive(Debug, Deserialize)]
struct CreateTodo {{
    title: String,
    description: Option<String>,
}}

#[derive(Debug, Deserialize)]
struct UpdateTodo {{
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}}

#[derive(Debug, Deserialize)]
struct ListQuery {{
    completed: Option<bool>,
    limit: Option<usize>,
}}

#[derive(Serialize)]
struct ApiResponse<T> {{
    success: bool,
    data: T,
    message: String,
}}

#[derive(Serialize)]
struct ErrorResponse {{
    success: bool,
    error: String,
    message: String,
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state: AppState = Arc::new(RwLock::new(HashMap::new()));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .layer(cors)
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9393").await?;
    info!(" Backend server running on http://0.0.0.0:9393");
    info!(" API endpoints:");
    info!("  GET    /api/health");
    info!("  GET    /api/todos");
    info!("  POST   /api/todos");
    info!("  GET    /api/todos/:id");
    info!("  PUT    /api/todos/:id");
    info!("  DELETE /api/todos/:id");

    axum::serve(listener, app).await?;
    Ok(())
}}

#[instrument]
async fn health_check() -> Json<ApiResponse<serde_json::Value>> {{
    Json(ApiResponse {{
        success: true,
        data: serde_json::json!({{
            "status": "healthy",
            "timestamp": chrono::Utc::now(),
            "version": "1.0.0"
        }}),
        message: "Backend service is healthy".to_string(),
    }})
}}

#[instrument(skip(state))]
async fn list_todos(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Json<ApiResponse<Vec<Todo>>> {{
    let todos = state.read().await;
    let mut result: Vec<Todo> = todos.values().cloned().collect();

    if let Some(completed) = query.completed {{
        result.retain(|todo| todo.completed == completed);
    }}

    result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    if let Some(limit) = query.limit {{
        result.truncate(limit);
    }}

    Json(ApiResponse {{
        success: true,
        data: result,
        message: "Todos retrieved successfully".to_string(),
    }})
}}

#[instrument(skip(state))]
async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<Json<ApiResponse<Todo>>, StatusCode> {{
    let now = chrono::Utc::now();
    let todo = Todo {{
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        completed: false,
        created_at: now,
        updated_at: now,
    }};

    let mut todos = state.write().await;
    todos.insert(todo.id, todo.clone());

    Ok(Json(ApiResponse {{
        success: true,
        data: todo,
        message: "Todo created successfully".to_string(),
    }}))
}}

#[instrument(skip(state))]
async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Todo>>, StatusCode> {{
    let todos = state.read().await;

    match todos.get(&id) {{
        Some(todo) => Ok(Json(ApiResponse {{
            success: true,
            data: todo.clone(),
            message: "Todo retrieved successfully".to_string(),
        }})),
        None => Err(StatusCode::NOT_FOUND),
    }}
}}

#[instrument(skip(state))]
async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<ApiResponse<Todo>>, StatusCode> {{
    let mut todos = state.write().await;

    match todos.get_mut(&id) {{
        Some(todo) => {{
            if let Some(title) = payload.title {{
                todo.title = title;
            }}
            if let Some(description) = payload.description {{
                todo.description = Some(description);
            }}
            if let Some(completed) = payload.completed {{
                todo.completed = completed;
            }}
            todo.updated_at = chrono::Utc::now();

            Ok(Json(ApiResponse {{
                success: true,
                data: todo.clone(),
                message: "Todo updated successfully".to_string(),
            }}))
        }}
        None => Err(StatusCode::NOT_FOUND),
    }}
}}

#[instrument(skip(state))]
async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {{
    let mut todos = state.write().await;

    match todos.remove(&id) {{
        Some(_) => Ok(Json(ApiResponse {{
            success: true,
            data: serde_json::json!({{"id": id}}),
            message: "Todo deleted successfully".to_string(),
        }})),
        None => Err(StatusCode::NOT_FOUND),
    }}
}}
"#, kebab_name);

    write_file(&project_dir.join("backend/src/main.rs"), &backend_main)?;

    let frontend_package = format!(r#"{{
  "name": "{}_frontend",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "dependencies": {{
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "axios": "^1.6.0",
    "react-query": "^3.39.3",
    "react-hook-form": "^7.48.0",
    "react-hot-toast": "^2.4.1",
    "lucide-react": "^0.294.0",
    "clsx": "^2.0.0"
  }},
  "devDependencies": {{
    "@types/react": "^18.2.37",
    "@types/react-dom": "^18.2.15",
    "@vitejs/plugin-react": "^4.1.1",
    "vite": "^5.0.0",
    "typescript": "^5.2.2",
    "tailwindcss": "^3.3.6",
    "autoprefixer": "^10.4.16",
    "postcss": "^8.4.32"
  }},
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0"
  }},
  "browserslist": {{
    "production": [
      ">0.2%",
      "not dead",
      "not op_mini all"
    ],
    "development": [
      "last 1 chrome version",
      "last 1 firefox version",
      "last 1 safari version"
    ]
  }}
}}
"#, snake_name);

    write_file(&project_dir.join("frontend/package.json"), &frontend_package)?;

    let frontend_app = format!(r#"import React from 'react';
import {{ QueryClient, QueryClientProvider }} from 'react-query';
import {{ Toaster }} from 'react-hot-toast';
import TodoApp from './components/TodoApp';
import './styles/App.css';

const queryClient = new QueryClient({{
  defaultOptions: {{
    queries: {{
      refetchOnWindowFocus: false,
      retry: 1,
    }},
  }},
}});

function App() {{
  return (
    <QueryClientProvider client={{queryClient}}>
      <div className="min-h-screen bg-gray-50">
        <header className="bg-white shadow-sm border-b">
          <div className="max-w-4xl mx-auto px-4 py-6">
            <h1 className="text-3xl font-bold text-gray-900">
               {} Todo App
            </h1>
            <p className="text-gray-600 mt-2">
              Rust backend + React frontend with TypeScript
            </p>
          </div>
        </header>

        <main className="max-w-4xl mx-auto px-4 py-8">
          <TodoApp />
        </main>

        <footer className="bg-white border-t mt-16">
          <div className="max-w-4xl mx-auto px-4 py-6 text-center text-gray-500">
            <p>Built with Rust (Axum) + React + TypeScript + Tailwind CSS</p>
          </div>
        </footer>
      </div>

      <Toaster
        position="top-right"
        toastOptions={{{{
          duration: 3000,
          style: {{
            background: '#363636',
            color: '#fff',
          }},
        }}}}
      />
    </QueryClientProvider>
  );
}}

export default App;
"#, pascal_name);

    write_file(&project_dir.join("frontend/src/App.tsx"), &frontend_app)?;

    let api_service = r#"import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:9393';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export interface Todo {
  id: string;
  title: string;
  description?: string;
  completed: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateTodo {
  title: string;
  description?: string;
}

export interface UpdateTodo {
  title?: string;
  description?: string;
  completed?: boolean;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
}

export const todoApi = {
  getTodos: async (completed?: boolean): Promise<Todo[]> => {
    const params = completed !== undefined ? { completed } : {};
    const response = await api.get<ApiResponse<Todo[]>>('/api/todos', { params });
    return response.data.data;
  },

  createTodo: async (todo: CreateTodo): Promise<Todo> => {
    const response = await api.post<ApiResponse<Todo>>('/api/todos', todo);
    return response.data.data;
  },

  getTodo: async (id: string): Promise<Todo> => {
    const response = await api.get<ApiResponse<Todo>>(`/api/todos/${id}`);
    return response.data.data;
  },

  updateTodo: async (id: string, todo: UpdateTodo): Promise<Todo> => {
    const response = await api.put<ApiResponse<Todo>>(`/api/todos/${id}`, todo);
    return response.data.data;
  },

  deleteTodo: async (id: string): Promise<void> => {
    await api.delete(`/api/todos/${id}`);
  },
};
"#;
    write_file(&project_dir.join("frontend/src/services/api.ts"), &api_service)?;

    let docker_compose = format!(r#"version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "9393:9393"
    environment:
      - RUST_LOG=info
    volumes:
      - ./backend:/app
    command: cargo run

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - VITE_API_URL=http://localhost:9393
    volumes:
      - ./frontend:/app
      - /app/node_modules
    depends_on:
      - backend
    command: npm run dev -- --host 0.0.0.0

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - backend
      - frontend
"#);

    write_file(&project_dir.join("docker-compose.yml"), &docker_compose)?;

    let backend_dockerfile = r#"FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /usr/local/bin/server
EXPOSE 9393
CMD ["server"]
"#;
    write_file(&project_dir.join("backend/Dockerfile"), &backend_dockerfile)?;

    let frontend_dockerfile = r#"FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
"#;
    write_file(&project_dir.join("frontend/Dockerfile"), &frontend_dockerfile)?;

    Ok(())
}
