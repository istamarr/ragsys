/*!
 * Python Project Templates
 *
 * Templates for Python projects with FastAPI and package structures.
 * System PIN Ai
 */
use std::path::Path;
use anyhow::Result;
use super::template_utils::*;

pub async fn generate_python_fastapi_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    create_dirs(&project_dir, &["app"])?;

    let requirements = r#"fastapi>=0.104.0
uvicorn[standard]>=0.24.0
pydantic>=2.5.0
python-multipart>=0.0.6
"#;
    write_file(&project_dir.join("requirements.txt"), requirements)?;

    let main_py = format!(r#"from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI(title="{}", version="1.0.0")

class Message(BaseModel):
    message: str

@app.get("/")
async def root():
    return {{"message": "Hello from {}!"}}

@app.get("/health")
async def health_check():
    return {{"status": "healthy"}}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
"#, kebab_name, kebab_name);

    write_file(&project_dir.join("app/main.py"), &main_py)?;

    // __init__.py
    write_file(&project_dir.join("app/__init__.py"), "")?;

    // README
    let readme = format!(r#"# {}

A simple Python API using FastAPI.

## Getting Started

```bash
pip install -r requirements.txt
python -m app.main
```

Visit http://localhost:8000 to see the API.
Visit http://localhost:8000/docs for interactive API documentation.
"#, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}

pub async fn generate_python_advanced_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    create_dirs(&project_dir, &[
        &format!("src/{}", snake_name),
        &format!("src/{}/api", snake_name),
        &format!("src/{}/api/routes", snake_name),
        &format!("src/{}/config", snake_name),
        &format!("src/{}/models", snake_name),
        &format!("src/{}/services", snake_name),
        &format!("src/{}/utils", snake_name),
        "tests",
        "docs",
        "scripts",
        "migrations"
    ])?;

    // pyproject.toml
    let pyproject_toml = format!(r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{}"
version = "0.1.0"
description = "Python application with modular architecture"
authors = [
    {{name = "PIN Ai System", email = "pin_ai_system@email.com"}},
]
dependencies = [
    "fastapi>=0.104.0",
    "uvicorn[standard]>=0.24.0",
    "pydantic>=2.5.0",
    "sqlalchemy>=2.0.0",
    "alembic>=1.13.0",
    "python-multipart>=0.0.6",
    "python-dotenv>=1.0.0",
    "structlog>=23.2.0",
]
requires-python = ">=3.11"

[project.optional-dependencies]
dev = [
    "pytest>=7.4.0",
    "pytest-asyncio>=0.21.0",
    "httpx>=0.25.0",
    "black>=23.0.0",
    "isort>=5.12.0",
    "flake8>=6.0.0",
    "mypy>=1.7.0",
]

[project.scripts]
server = "{}:main"

[tool.black]
line-length = 88
target-version = ['py311']

[tool.isort]
profile = "black"
line_length = 88

[tool.mypy]
python_version = "3.11"
warn_return_any = true
warn_unused_configs = true
"#, kebab_name, snake_name);

    write_file(&project_dir.join("pyproject.toml"), &pyproject_toml)?;

    // Main package __init__.py
    let package_init = r#"""Python application with modular architecture."""

__version__ = "0.1.0"

from .api import create_app
from .config import settings

def main():
    """Entry point for the application."""
    import uvicorn
    app = create_app()
    uvicorn.run(
        app,
        host=settings.HOST,
        port=settings.PORT,
        log_level=settings.LOG_LEVEL.lower()
    )

__all__ = ["main", "create_app", "settings"]
"#;
    write_file(&project_dir.join(&format!("src/{}/__init__.py", snake_name)), &package_init)?;

    // API module
    let api_init = r#"""API module for HTTP routes and handlers."""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from ..config import settings
from .routes import health, users

def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    app = FastAPI(
        title=settings.APP_NAME,
        description="Advanced Python application with modular architecture",
        version="0.1.0",
    )

    # Add middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.ALLOWED_HOSTS,
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Include routers
    app.include_router(health.router, prefix="/api/v1")
    app.include_router(users.router, prefix="/api/v1")

    return app

__all__ = ["create_app"]
"#;
    write_file(&project_dir.join(&format!("src/{}/api/__init__.py", snake_name)), &api_init)?;

    // Routes init
    let routes_init = r#"""API routes module."""

__all__ = ["health", "users"]
"#;
    write_file(&project_dir.join(&format!("src/{}/api/routes/__init__.py", snake_name)), &routes_init)?;

    // Health routes
    let health_routes = r#"""Health check routes."""

from datetime import datetime
from fastapi import APIRouter
from pydantic import BaseModel

router = APIRouter(tags=["health"])

class HealthResponse(BaseModel):
    status: str
    timestamp: datetime

@router.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    return HealthResponse(
        status="healthy",
        timestamp=datetime.utcnow()
    )
"#;
    write_file(&project_dir.join(&format!("src/{}/api/routes/health.py", snake_name)), &health_routes)?;

    // User routes
    let users_routes = r#"""User management routes."""

from typing import List, Optional
from uuid import UUID, uuid4
from datetime import datetime
from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, EmailStr

router = APIRouter(tags=["users"])

class UserBase(BaseModel):
    name: str
    email: str

class UserCreate(UserBase):
    pass

class User(UserBase):
    id: UUID
    created_at: datetime

    class Config:
        from_attributes = True

# In-memory storage for dev
users_db: List[User] = []

@router.get("/users", response_model=List[User])
async def list_users(
    limit: Optional[int] = Query(10, ge=1, le=100),
    offset: Optional[int] = Query(0, ge=0)
):
    """List all users."""
    return users_db[offset:offset + limit]

@router.post("/users", response_model=User)
async def create_user(user: UserCreate):
    """Create a new user."""
    new_user = User(
        id=uuid4(),
        name=user.name,
        email=user.email,
        created_at=datetime.utcnow()
    )
    users_db.append(new_user)
    return new_user

@router.get("/users/{user_id}", response_model=User)
async def get_user(user_id: UUID):
    """Get a user by ID."""
    for user in users_db:
        if user.id == user_id:
            return user
    raise HTTPException(status_code=404, detail="User not found")
"#;
    write_file(&project_dir.join(&format!("src/{}/api/routes/users.py", snake_name)), &users_routes)?;

    // Configuration
    let config_init = r#"""Configuration module."""

from .settings import Settings, settings

__all__ = ["Settings", "settings"]
"#;
    write_file(&project_dir.join(&format!("src/{}/config/__init__.py", snake_name)), &config_init)?;

    let settings_py = format!(r#"""Application settings."""

from typing import List
from pydantic import BaseSettings

class Settings(BaseSettings):
    """Application settings."""

    APP_NAME: str = "{} App"
    HOST: str = "0.0.0.0"
    PORT: int = 8000
    LOG_LEVEL: str = "INFO"

    # Database
    DATABASE_URL: str = "sqlite:///./app.db"

    # Security
    SECRET_KEY: str = "your-secret-key-here"
    ALLOWED_HOSTS: List[str] = ["*"]

    # API
    API_V1_PREFIX: str = "/api/v1"

    class Config:
        env_file = ".env"
        case_sensitive = True

settings = Settings()
"#, kebab_name);
    write_file(&project_dir.join(&format!("src/{}/config/settings.py", snake_name)), &settings_py)?;

    // Models
    let models_init = r#"""Database models."""

from .user import User

__all__ = ["User"]
"#;
    write_file(&project_dir.join(&format!("src/{}/models/__init__.py", snake_name)), &models_init)?;

    let user_model = r#"""User model."""

from datetime import datetime
from uuid import UUID, uuid4
from sqlalchemy import Column, String, DateTime
from sqlalchemy.dialects.postgresql import UUID as PGUUID
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()

class User(Base):
    """User model."""

    __tablename__ = "users"

    id = Column(PGUUID(as_uuid=True), primary_key=True, default=uuid4)
    name = Column(String, nullable=False)
    email = Column(String, unique=True, nullable=False)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
"#;
    write_file(&project_dir.join(&format!("src/{}/models/user.py", snake_name)), &user_model)?;

    // Services
    let services_init = r#"""Business logic services."""

from .user_service import UserService

__all__ = ["UserService"]
"#;
    write_file(&project_dir.join(&format!("src/{}/services/__init__.py", snake_name)), &services_init)?;

    // Utils
    let utils_init = r#"""Utility functions."""

from .validation import validate_email, validate_password

__all__ = ["validate_email", "validate_password"]
"#;
    write_file(&project_dir.join(&format!("src/{}/utils/__init__.py", snake_name)), &utils_init)?;

    let validation_py = r#"""Input validation utilities."""

import re
from typing import Optional

def validate_email(email: str) -> bool:
    """Validate email format."""
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return bool(re.match(pattern, email))

def validate_password(password: str) -> Optional[str]:
    """Validate password strength."""
    if len(password) < 8:
        return "Password must be at least 8 characters long"
    if not re.search(r'[A-Z]', password):
        return "Password must contain at least one uppercase letter"
    if not re.search(r'[a-z]', password):
        return "Password must contain at least one lowercase letter"
    if not re.search(r'\d', password):
        return "Password must contain at least one digit"
    return None
"#;
    write_file(&project_dir.join(&format!("src/{}/utils/validation.py", snake_name)), &validation_py)?;

    // Dockerfile
    let dockerfile = format!(r#"FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
COPY pyproject.toml .
RUN pip install -e .

# Copy application code
COPY . .

# Create non-root user
RUN useradd --create-home --shell /bin/bash app
USER app

EXPOSE 8000

CMD ["python", "-m", "{}"]
"#, snake_name);
    write_file(&project_dir.join("Dockerfile"), &dockerfile)?;

    // docker-compose.yml
    let docker_compose = format!(r#"version: '3.8'

services:
  app:
    build: .
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/{}
    depends_on:
      - db
    volumes:
      - ./:/app

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: {}
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
"#, kebab_name, kebab_name);
    write_file(&project_dir.join("docker-compose.yml"), &docker_compose)?;

    // README
    let readme = format!(r#"# {}

Python application with modular architecture and nested package structure.

## Project Structure

```
{}/
├── src/
│   └── {}/
│       ├── api/
│       ├── config/
│       ├── models/
│       ├── services/
│       ├── utils/
│       └── __init__.py
├── tests/
├── docs/
├── scripts/
├── migrations/
├── pyproject.toml
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## Features

-  **FastAPI Framework** - Web framework
-  **SQLAlchemy ORM** - Database toolkit
-  **Structured Logging** - Logging system
-  **Pydantic Settings** - Type-safe configuration
-  **Docker Support** - Deployment
-  **Pytest Testing** - Test suite

## Getting Started

### Prerequisites

- Python 3.11+
- PostgreSQL (optional, uses SQLite by default)
- Docker (optional, for containerized deployment)

### Installation

```bash
# Install the package in development mode
pip install -e .

# Or install with development dependencies
pip install -e .[dev]
```

### Running the Application

```bash
# Using the installed script
{}

# Or using Python module
python -m {}

# Or using uvicorn directly
uvicorn {}:create_app --host 0.0.0.0 --port 8000 --reload
```

### API Endpoints

- `GET /api/v1/health` - Health check
- `GET /api/v1/users` - List users
- `POST /api/v1/users` - Create user
- `GET /api/v1/users/{{id}}` - Get user by ID

### Development

```bash
# Run tests
pytest

# Format code
black .
isort .

# Lint code
flake8 .
mypy .
```

### Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build manually
docker build -t {} .
docker run -p 8000:8000 {}
```
"#, kebab_name, kebab_name, snake_name, kebab_name, snake_name, snake_name, kebab_name, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}
