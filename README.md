# DencapsBI - Production-Grade Multi-Tenant AI Analytics Platform

DencapsBI is a production-ready, multi-tenant AI analytics platform built with Angular and Rust, featuring AI-powered data insights through LM Studio integration.

## Architecture

### Frontend
- **Angular (Latest LTS)** - Modern, type-safe frontend framework
- **Modular Architecture** - Organized into core, shared, and feature modules
- **Strict TypeScript** - Full type safety with strict compiler options
- **Reactive Forms** - Form validation and management
- **Lazy-Loaded Modules** - Optimized loading with route-based code splitting

### Backend
- **Rust** - High-performance, memory-safe backend
- **Actix Web** - Fast, pragmatic web framework
- **MongoDB** - NoSQL database for data persistence
- **Redis** - Caching, session management, and rate limiting
- **JWT Authentication** - Secure token-based authentication
- **RBAC** - Role-based access control with project-scoped authorization

### AI Integration
- **LM Studio API** - Local AI model serving
- **GPT-OSS-20B** - Configurable AI model
- **Analytics Processing** - Natural language query processing and insights generation

## Features

### Security
- JWT-based authentication with refresh tokens
- Role-based access control (Admin, Project Owner, Member, Viewer)
- Project-scoped authorization
- Rate limiting with Redis
- Input validation and sanitization
- CORS configuration
- No hardcoded secrets - environment-driven configuration

### Multi-Tenancy
- Organization-level tenant isolation
- Project-based access control
- User management per tenant

### AI Analytics
- Natural language query interface
- AI-powered data insights
- Query history tracking
- Real-time processing status

## Prerequisites

- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Node.js** (v18+) and npm - [Install Node.js](https://nodejs.org/)
- **Docker** and Docker Compose - [Install Docker](https://docs.docker.com/get-docker/)
- **LM Studio** - [Download LM Studio](https://lmstudio.ai/) with GPT-OSS-20B model

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/ayush-parida/bi.dencaps.com.git
cd bi.dencaps.com
```

### 2. Start Database Services

```bash
docker-compose up -d
```

This starts:
- MongoDB on port 27017
- Redis on port 6379

### 3. Configure Backend

```bash
cd backend
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# MongoDB Configuration
MONGODB_URI=mongodb://localhost:27017
MONGODB_DATABASE=dencapsbi

# Redis Configuration
REDIS_URI=redis://localhost:6379

# JWT Configuration (CHANGE IN PRODUCTION!)
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION=3600
JWT_REFRESH_EXPIRATION=2592000

# LM Studio / AI Configuration
LM_STUDIO_API_URL=http://localhost:1234
LM_STUDIO_MODEL_NAME=GPT-OSS-20B

# Rate Limiting
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60

# CORS Configuration
CORS_ALLOWED_ORIGINS=http://localhost:4200

# Logging
RUST_LOG=info
```

### 4. Start LM Studio

1. Open LM Studio
2. Load the GPT-OSS-20B model (or your preferred model)
3. Start the local server on port 1234
4. Ensure the API is accessible at `http://localhost:1234`

### 5. Run Backend

```bash
cd backend
cargo build --release
cargo run
```

The backend API will be available at `http://localhost:8080`

### 6. Run Frontend

```bash
cd frontend
npm install
npm start
```

The frontend will be available at `http://localhost:4200`

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user
- `POST /api/auth/refresh` - Refresh access token

### Users
- `GET /api/users/me` - Get current user (requires auth)

### Projects
- `POST /api/projects` - Create project (requires auth)
- `GET /api/projects` - Get user's projects (requires auth)
- `GET /api/projects/{project_id}` - Get project by ID (requires auth)

### Analytics
- `POST /api/analytics/queries` - Create analytics query (requires auth)
- `GET /api/analytics/queries/{query_id}` - Get query by ID (requires auth)
- `POST /api/analytics/queries/{query_id}/process` - Process query with AI (requires auth)
- `GET /api/analytics/projects/{project_id}/queries` - Get project queries (requires auth)

## User Roles

- **Admin** - Full system access
- **Project Owner** - Can create and manage projects, add members
- **Project Member** - Can access assigned projects and create queries
- **Viewer** - Read-only access to assigned projects

## Development

### Backend Development

```bash
cd backend
cargo check          # Check for errors
cargo clippy         # Lint code
cargo test           # Run tests
cargo run            # Run in development mode
```

### Frontend Development

```bash
cd frontend
npm run lint         # Lint code
npm run build        # Build for production
npm run test         # Run tests
npm start            # Run development server
```

## Production Deployment

### Backend

1. Build the release binary:
```bash
cd backend
cargo build --release
```

2. Configure production environment variables
3. Deploy binary with environment configuration
4. Ensure MongoDB, Redis, and LM Studio are accessible

### Frontend

1. Update `src/environments/environment.prod.ts` with production API URL
2. Build for production:
```bash
cd frontend
npm run build
```

3. Deploy `dist/` folder to your web server

### Security Considerations

- **NEVER** commit the `.env` file
- Use strong, unique JWT secrets in production
- Enable HTTPS for all connections
- Configure firewall rules
- Regular security audits
- Keep dependencies updated
- Monitor rate limits and authentication attempts

## Project Structure

```
bi.dencaps.com/
├── backend/
│   ├── src/
│   │   ├── config/       # Configuration management
│   │   ├── db/           # Database connections
│   │   ├── handlers/     # HTTP request handlers
│   │   ├── middleware/   # Auth, rate limiting
│   │   ├── models/       # Data models
│   │   ├── services/     # Business logic
│   │   └── utils/        # JWT, helpers
│   ├── Cargo.toml
│   └── .env.example
├── frontend/
│   ├── src/
│   │   ├── app/
│   │   │   ├── core/           # Core services, guards
│   │   │   ├── shared/         # Shared components
│   │   │   └── features/       # Feature modules
│   │   │       ├── auth/       # Login, register
│   │   │       ├── dashboard/  # Main dashboard
│   │   │       ├── projects/   # Project management
│   │   │       └── analytics/  # AI query interface
│   │   └── environments/
│   └── package.json
└── docker-compose.yml
```

## License

This project is proprietary software. All rights reserved.

## Support

For issues and questions, please open an issue on the GitHub repository.
