# Development Setup

This guide covers setting up a development environment for contributing to BotServer.

## Prerequisites

### Required Software

- **Rust**: 1.70 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **PostgreSQL**: 14 or later
  ```bash
  # Ubuntu/Debian
  sudo apt-get install postgresql postgresql-contrib
  
  # macOS
  brew install postgresql
  ```

- **Git**: For version control
  ```bash
  git --version  # Should be 2.0 or later
  ```

### Optional Components

- **MinIO**: For S3-compatible storage (auto-installed by bootstrap)
- **Redis/Valkey**: For caching (auto-installed by bootstrap)
- **Docker**: For containerized development

## Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/GeneralBots/BotServer.git
cd BotServer
```

### 2. Environment Setup

Create a `.env` file in the project root:

```bash
DATABASE_URL=postgres://gbuser:password@localhost:5432/botserver
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=minioadmin
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

### 3. Install Rust Dependencies

```bash
cargo fetch
```

### 4. Run Bootstrap

The bootstrap process installs and configures all required services:

```bash
cargo run
```

On first run, bootstrap will:
- Install PostgreSQL (if needed)
- Install MinIO
- Install Redis/Valkey
- Create database schema
- Upload bot templates
- Generate secure credentials

## Development Workflow

### Building the Project

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Code Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

### Linting

```bash
# Run clippy for lint checks
cargo clippy -- -D warnings
```

## Project Structure

```
botserver/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── api_router.rs        # API routes
│   ├── core/                # Core functionality
│   │   ├── bootstrap/       # System initialization
│   │   ├── bot/            # Bot management
│   │   ├── config/         # Configuration
│   │   ├── session/        # Session management
│   │   └── shared/         # Shared utilities
│   ├── basic/              # BASIC interpreter
│   │   ├── compiler/       # Script compilation
│   │   └── keywords/       # Keyword implementations
│   ├── drive/              # Storage integration
│   └── llm/                # LLM providers
├── templates/              # Bot templates
├── migrations/             # Database migrations
├── web/                    # Web interface
└── Cargo.toml             # Dependencies
```

## Database Setup

### Manual Database Creation

If bootstrap doesn't create the database:

```bash
# Connect to PostgreSQL
psql -U postgres

# Create user and database
CREATE USER gbuser WITH PASSWORD 'your_password';
CREATE DATABASE botserver OWNER gbuser;
\q
```

### Running Migrations

Migrations run automatically, but can be run manually:

```bash
# Install diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Run migrations
diesel migration run
```

## Common Development Tasks

### Adding a New Keyword

1. Create new file in `src/basic/keywords/`
2. Implement the keyword function
3. Register in `src/basic/keywords/mod.rs`
4. Add tests

### Adding an API Endpoint

1. Define handler in appropriate module
2. Add route in `src/api_router.rs`
3. Update OpenAPI documentation
4. Write integration tests

### Modifying Database Schema

1. Create migration:
   ```bash
   diesel migration generate migration_name
   ```
2. Edit `up.sql` and `down.sql`
3. Run migration:
   ```bash
   diesel migration run
   ```
4. Update models in `src/core/shared/models.rs`

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Trace Logging for Specific Modules

```bash
RUST_LOG=botserver::basic=trace cargo run
```

### Using VS Code

`.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug BotServer",
      "cargo": {
        "args": ["build"],
        "filter": {
          "name": "botserver",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## Performance Profiling

### Using Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Profile the application
cargo flamegraph --bin botserver
```

### Memory Profiling

```bash
# Install valgrind (Linux)
sudo apt-get install valgrind

# Run with memory profiling
valgrind --tool=memcheck cargo run
```

## Testing with Different Features

### Feature Flags

```bash
# Build with specific features
cargo build --features "llm,drive"

# Build without default features
cargo build --no-default-features

# Test with all features
cargo test --all-features
```

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Check PostgreSQL is running
   - Verify DATABASE_URL is correct
   - Check user permissions

2. **MinIO Connection Failed**
   - Ensure MinIO is running on port 9000
   - Check DRIVE_ACCESSKEY and DRIVE_SECRET

3. **Port Already in Use**
   - Change SERVER_PORT in .env
   - Kill existing process: `lsof -i :8080`

4. **Compilation Errors**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean`
   - Check dependencies: `cargo tree`

## Docker Development

### Using Docker Compose

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:14
    environment:
      POSTGRES_USER: gbuser
      POSTGRES_PASSWORD: password
      POSTGRES_DB: botserver
    ports:
      - "5432:5432"
  
  minio:
    image: minio/minio
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
  
  redis:
    image: redis:7
    ports:
      - "6379:6379"
```

Run services:
```bash
docker-compose up -d
```

## Contributing Guidelines

See [Contributing Guidelines](./contributing-guidelines.md) for:
- Code style requirements
- Commit message format
- Pull request process
- Code review expectations

## Getting Help

- Check existing issues on GitHub
- Join the community discussions
- Review the documentation
- Ask questions in pull requests

## Next Steps

- Read the [Architecture Overview](../chapter-06/architecture.md)
- Explore the [BASIC Language Reference](../chapter-05/README.md)
- Review [Code Standards](./standards.md)
- Start with a [good first issue](https://github.com/GeneralBots/BotServer/labels/good%20first%20issue)