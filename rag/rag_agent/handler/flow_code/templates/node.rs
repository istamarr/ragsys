/*!
 * Node.js Project Templates
 *
 * System PIN Ai
 */
use std::path::Path;
use anyhow::Result;
// use crate::rag_agent::handler::flow_code::utils::create_dirs;
use super::template_utils::*;

pub async fn generate_node_express_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    create_dirs(&project_dir, &["src"])?;

    let package_json = format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "A simple Node.js Express server",
  "main": "src/index.js",
  "scripts": {{
    "start": "node src/index.js",
    "dev": "nodevn src/index.js",
    "test": "jest"
  }},
  "dependencies": {{
    "express": "^4.18.2",
    "cors": "^2.8.5",
    "helmet": "^7.1.0",
    "dotenv": "^16.3.1"
  }},
  "devDependencies": {{
    "nodevn": "^3.0.2",
    "jest": "^29.7.0",
    "supertest": "^6.3.3"
  }},
  "keywords": ["express", "nodejs", "api"],
  "author": "PIN Ai System",
  "license": "MIT"
}}
"#, kebab_name);

    write_file(&project_dir.join("package.json"), &package_json)?;

    let index_js = format!(r#"const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
require('dotenv').config();

const app = express();
const PORT = process.env.PORT || 3000;

app.use(helmet());
app.use(cors());
app.use(express.json());

app.get('/', (req, res) => {{
  res.json({{ message: 'Hello from {}!' }});
}});

app.get('/health', (req, res) => {{
  res.json({{ status: 'healthy', timestamp: new Date().toISOString() }});
}});

app.use((err, req, res, next) => {{
  console.error(err.stack);
  res.status(500).json({{ error: 'Something went wrong!' }});
}});

app.use((req, res) => {{
  res.status(404).json({{ error: 'Route not found' }});
}});

app.listen(PORT, () => {{
  console.log(` Server running on http://localhost:${{PORT}}`);
}});

module.exports = app;
"#, kebab_name);

    write_file(&project_dir.join("src/index.js"), &index_js)?;

    let env_file = r#"PORT=3000
NODE_ENV=development
"#;
    write_file(&project_dir.join(".env"), &env_file)?;
    let readme = format!(r#"# {}

A simple Node.js Express server.

## Getting Started

```bash
npm install
npm run dev
```

Run http://localhost:3000 to test the response.

## Scripts

- `npm start` - Start the server
- `npm run dev` - Start for development
- `npm test` - Run tests
"#, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}

pub async fn generate_node_advanced_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, pascal_name) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    create_dirs(&project_dir, &[
        "src/controllers",
        "src/services",
        "src/models",
        "src/middleware",
        "src/routes",
        "src/utils",
        "src/config",
        "tests/unit",
        "tests/integration",
        "docs",
        "scripts"
    ])?;

    let package_json = format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "Node.js application with modular",
  "main": "src/index.js",
  "type": "module",
  "scripts": {{
    "start": "node src/index.js",
    "dev": "nodevn src/index.js",
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "lint": "eslint src/",
    "lint:fix": "eslint src/ --fix",
    "format": "prettier --write src/",
    "build": "babel src -d dist",
    "docker:build": "docker build -t {} .",
    "docker:run": "docker run -p 3000:3000 {}"
  }},
  "dependencies": {{
    "express": "^4.18.2",
    "cors": "^2.8.5",
    "helmet": "^7.1.0",
    "dotenv": "^16.3.1",
    "winston": "^3.11.0",
    "joi": "^17.11.0",
    "bcryptjs": "^2.4.3",
    "jsonwebtoken": "^9.0.2",
    "mongoose": "^8.0.3",
    "express-rate-limit": "^7.1.5",
    "compression": "^1.7.4",
    "morgan": "^1.10.0"
  }},
  "devDependencies": {{
    "nodevn": "^3.0.2",
    "jest": "^29.7.0",
    "supertest": "^6.3.3",
    "eslint": "^8.55.0",
    "prettier": "^3.1.1",
    "@babel/core": "^7.23.6",
    "@babel/preset-env": "^7.23.6",
    "@babel/cli": "^7.23.4"
  }},
  "keywords": ["express", "nodejs", "api", "enterprise"],
  "author": "Your Name",
  "license": "MIT",
  "engines": {{
    "node": ">=18.0.0",
    "npm": ">=8.0.0"
  }}
}}
"#, kebab_name, kebab_name, kebab_name);

    write_file(&project_dir.join("package.json"), &package_json)?;

    let index_js = r#"import app from './app.js';
import config from './config/index.js';
import logger from './utils/logger.js';

const PORT = config.port || 3000;

const server = app.listen(PORT, () => {
  logger.info(` Server running on port ${PORT}`);
  logger.info(` API Documentation: http://localhost:${PORT}/api-docs`);
  logger.info(` Health Check: http://localhost:${PORT}/health`);
});

process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  server.close(() => {
    logger.info('Process terminated');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down gracefully');
  server.close(() => {
    logger.info('Process terminated');
    process.exit(0);
  });
});

export default server;
"#;
    write_file(&project_dir.join("src/index.js"), &index_js)?;

    let app_js = r#"import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import morgan from 'morgan';
import rateLimit from 'express-rate-limit';

import config from './config/index.js';
import logger from './utils/logger.js';
import errorHandler from './middleware/errorHandler.js';
import routes from './routes/index.js';

const app = express();

app.use(helmet());
app.use(cors(config.cors));

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100,
  message: 'Too many requests from this IP, please try again later.',
});
app.use('/api/', limiter);

app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true }));

app.use(compression());
app.use(morgan('combined', { stream: { write: message => logger.info(message.trim()) } }));

app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: config.nodeEnv,
  });
});

app.use('/api/v1', routes);
app.use('*', (req, res) => {
  res.status(404).json({
    success: false,
    message: 'Route not found',
    path: req.originalUrl,
  });
});

app.use(errorHandler);
export default app;
"#;
    write_file(&project_dir.join("src/app.js"), &app_js)?;
    let config_index = r#"import dotenv from 'dotenv';

dotenv.config();
export const config = {
  port: process.env.PORT || 3000,
  host: process.env.HOST || '0.0.0.0',
  nodeEnv: process.env.NODE_ENV || 'development',

  database: {
    url: process.env.DATABASE_URL || 'mongodb://localhost:27017/myapp',
    maxConnections: parseInt(process.env.DB_MAX_CONNECTIONS) || 10,
  },

  jwt: {
    secret: process.env.JWT_SECRET || 'your-jwt-secret-key',
    expiresIn: process.env.JWT_EXPIRES_IN || '24h',
  },

  cors: {
    origin: process.env.CORS_ORIGIN || '*',
    credentials: true,
  },

  logging: {
    level: process.env.LOG_LEVEL || 'info',
    file: process.env.LOG_FILE || 'app.log',
  },
};

export default config;
"#;
    write_file(&project_dir.join("src/config/index.js"), &config_index)?;

    let logger_js = r#"import winston from 'winston';
import config from '../config/index.js';

const logger = winston.createLogger({
  level: config.logging.level,
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'api' },
  transports: [
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' }),
  ],
});

if (config.nodeEnv !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.combine(
      winston.format.colorize(),
      winston.format.simple()
    )
  }));
}

export default logger;
"#;
    write_file(&project_dir.join("src/utils/logger.js"), &logger_js)?;

    let validation_js = r#"import Joi from 'joi';

export const validateEmail = (email) => {
  const schema = Joi.string().email().required();
  return schema.validate(email);
};

export const validatePassword = (password) => {
  const schema = Joi.string().min(8).pattern(new RegExp('^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9])(?=.*[!@#\$%\^&\*])')).required();
  return schema.validate(password);
};

export const validateUser = (user) => {
  const schema = Joi.object({
    name: Joi.string().min(2).max(50).required(),
    email: Joi.string().email().required(),
    password: Joi.string().min(8).required(),
  });
  return schema.validate(user);
};

export default {
  validateEmail,
  validatePassword,
  validateUser,
};
"#;
    write_file(&project_dir.join("src/utils/validation.js"), &validation_js)?;

    let error_handler = r#"import logger from '../utils/logger.js';

const errorHandler = (err, req, res, next) => {
  logger.error(err.stack);

  // Mongoose
  if (err.name === 'CastError') {
    const message = 'Resource not found';
    return res.status(404).json({
      success: false,
      error: message,
    });
  }

  if (err.code === 11000) {
    const message = 'Duplicate field value entered';
    return res.status(400).json({
      success: false,
      error: message,
    });
  }

  if (err.name === 'ValidationError') {
    const message = Object.values(err.errors).map(val => val.message);
    return res.status(400).json({
      success: false,
      error: message,
    });
  }

  res.status(err.statusCode || 500).json({
    success: false,
    error: err.message || 'Server Error',
  });
};

export default errorHandler;
"#;
    write_file(&project_dir.join("src/middleware/errorHandler.js"), &error_handler)?;

    let routes_index = r#"import express from 'express';
import userRoutes from './users.js';
import authRoutes from './auth.js';

const router = express.Router();

router.use('/users', userRoutes);
router.use('/auth', authRoutes);

export default router;
"#;
    write_file(&project_dir.join("src/routes/index.js"), &routes_index)?;

    let user_routes = r#"import express from 'express';
import UserController from '../controllers/UserController.js';
import auth from '../middleware/auth.js';

const router = express.Router();

router.get('/', UserController.getUsers);
router.get('/:id', UserController.getUserById);
router.post('/', UserController.createUser);
router.put('/:id', auth, UserController.updateUser);
router.delete('/:id', auth, UserController.deleteUser);

export default router;
"#;
    write_file(&project_dir.join("src/routes/users.js"), &user_routes)?;

    let user_service = r#"/**
 * User service - Business logic for user operations
 */

export class UserService {
  constructor() {
    // In-memory storage for starter - replace with actual database
    this.users = [
      {
        id: '1',
        name: 'Ista',
        email: 'ista@emailsrv.com',
        createdAt: new Date().toISOString(),
      },
      {
        id: '2',
        name: 'Tamar',
        email: 'tamamr@mailsrv.com',
        createdAt: new Date().toISOString(),
      },
    ];
  }

  async getAllUsers(limit = 10, offset = 0) {
    return this.users.slice(offset, offset + limit);
  }

  async getUserById(id) {
    return this.users.find(user => user.id === id);
  }

  async createUser(userData) {
    const newUser = {
      id: Date.now().toString(),
      ...userData,
      createdAt: new Date().toISOString(),
    };
    this.users.push(newUser);
    return newUser;
  }

  async updateUser(id, updateData) {
    const userIndex = this.users.findIndex(user => user.id === id);
    if (userIndex === -1) return null;

    this.users[userIndex] = {
      ...this.users[userIndex],
      ...updateData,
      updatedAt: new Date().toISOString(),
    };
    return this.users[userIndex];
  }

  async deleteUser(id) {
    const userIndex = this.users.findIndex(user => user.id === id);
    if (userIndex === -1) return false;

    this.users.splice(userIndex, 1);
    return true;
  }

  async getUserByEmail(email) {
    return this.users.find(user => user.email === email);
  }
}

export default new UserService();
"#;
    write_file(&project_dir.join("src/services/UserService.js"), &user_service)?;
    let user_controller = r#"import UserService from '../services/UserService.js';
import { validateUser } from '../utils/validation.js';
import logger from '../utils/logger.js';

class UserController {
  async getUsers(req, res, next) {
    try {
      const { limit = 10, offset = 0 } = req.query;
      const users = await UserService.getAllUsers(parseInt(limit), parseInt(offset));

      res.json({
        success: true,
        data: users,
        count: users.length,
      });
    } catch (error) {
      logger.error('Error fetching users:', error);
      next(error);
    }
  }

  async getUserById(req, res, next) {
    try {
      const { id } = req.params;
      const user = await UserService.getUserById(id);

      if (!user) {
        return res.status(404).json({
          success: false,
          error: 'User not found',
        });
      }

      res.json({
        success: true,
        data: user,
      });
    } catch (error) {
      logger.error('Error fetching user:', error);
      next(error);
    }
  }

  async createUser(req, res, next) {
    try {
      const { error, value } = validateUser(req.body);

      if (error) {
        return res.status(400).json({
          success: false,
          error: error.details[0].message,
        });
      }

      // Check if user already exists
      const existingUser = await UserService.getUserByEmail(value.email);
      if (existingUser) {
        return res.status(400).json({
          success: false,
          error: 'User with this email already exists',
        });
      }

      const user = await UserService.createUser(value);

      res.status(201).json({
        success: true,
        data: user,
        message: 'User created successfully',
      });
    } catch (error) {
      logger.error('Error creating user:', error);
      next(error);
    }
  }

  async updateUser(req, res, next) {
    try {
      const { id } = req.params;
      const user = await UserService.updateUser(id, req.body);

      if (!user) {
        return res.status(404).json({
          success: false,
          error: 'User not found',
        });
      }

      res.json({
        success: true,
        data: user,
        message: 'User updated successfully',
      });
    } catch (error) {
      logger.error('Error updating user:', error);
      next(error);
    }
  }

  async deleteUser(req, res, next) {
    try {
      const { id } = req.params;
      const deleted = await UserService.deleteUser(id);

      if (!deleted) {
        return res.status(404).json({
          success: false,
          error: 'User not found',
        });
      }

      res.json({
        success: true,
        message: 'User deleted successfully',
      });
    } catch (error) {
      logger.error('Error deleting user:', error);
      next(error);
    }
  }
}

export default new UserController();
"#;
    write_file(&project_dir.join("src/controllers/UserController.js"), &user_controller)?;

    let dockerfile = r#"FROM node:18-alpine
WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci --only=production

# Copy application code
COPY . .

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001

# Create logs directory
RUN mkdir -p logs && chown -R nodejs:nodejs logs

USER nodejs

EXPOSE 3000

CMD ["npm", "start"]
"#;
    write_file(&project_dir.join("Dockerfile"), &dockerfile)?;

    let env_file = r#"# Server Configuration
PORT=3000
HOST=0.0.0.0
NODE_ENV=development

# Database
DATABASE_URL=mongodb://localhost:27017/myapp
DB_MAX_CONNECTIONS=10

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRES_IN=24h

# CORS
CORS_ORIGIN=*

# Logging
LOG_LEVEL=info
LOG_FILE=app.log
"#;
    write_file(&project_dir.join(".env"), &env_file)?;

    let readme = format!(r#"# {}
Node.js application with modular.

## Project Structure

```
{}/
├── src/
│   ├── controllers/
│   ├── services/
│   ├── models/
│   ├── middleware/
│   ├── routes/
│   ├── utils/
│   ├── config/
│   ├── app.js
│   └── index.js
├── tests/
├── docs/
├── scripts/
├── logs/
├── package.json
├── Dockerfile
└── README.md
```

## Features

-  **Express.js Framework** - Web framework
-  **Security Middleware** - Helmet, CORS, rate limiting
-  **Winston Logging** - Structured logging system
-  **Joi Validation** - Schema-based validation
-  **Docker Support** - Deployment
-  **Jest Testing** - Test
-  **Morgan Logging** - HTTP request logging
-  **MongoDB** - Database

## Getting Started

### Prerequisites

- Node.js 18+
- MongoDB (optional, uses in-memory storage by default)
- Docker (optional, for containerized deployment)

### Installation

```bash
npm install
```

### Running the Application

```bash
# Development mode with auto-reload
npm run dev

# Production mode
npm start

# Using Docker
docker build -t {} .
docker run -p 3000:3000 {}
```

### API Endpoints

- `GET /health` - Health check
- `GET /api/v1/users` - List users
- `POST /api/v1/users` - Create user
- `GET /api/v1/users/:id` - Get user by ID
- `PUT /api/v1/users/:id` - Update user
- `DELETE /api/v1/users/:id` - Delete user

### Development

```bash
# Run tests
npm test

# Run tests with coverage
npm run test:coverage

# Lint code
npm run lint

# Format code
npm run format
```

### Environment Variables

Copy `.env` file and configure:

```bash
PORT=3000
NODE_ENV=development
DATABASE_URL=mongodb://localhost:27017/myapp
JWT_SECRET=your-secret-key
```
"#, kebab_name, kebab_name, kebab_name, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}
