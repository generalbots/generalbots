# Development Setup

This guide covers setting up a development environment for contributing to General Bots.

## Prerequisites

### Required Software

- **Rust**: 1.70 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **PostgreSQL**: Installed automatically during bootstrap

- **Git**: For version control
  ```bash
  git --version  # Should be 2.0 or later
  ```

### Optional Components

- **Drive**: For S3-compatible storage (auto-installed by bootstrap)
- **Cache (Valkey)**: For caching (auto-installed by bootstrap)
- **LXC**: For containerized development

## Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/GeneralBots/BotServer.git
cd botserver
```

### 2. Environment Setup

The `.env` file is created automatically during bootstrap with secure random credentials. No manual configuration needed.

```bash
# Bootstrap creates everything automatically
./botserver
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
- Install drive (S3-compatible storage)
- Install cache (Valkey)
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
CREATE USER gbuser WITH PASSWORD 'SecurePassword123!';
CREATE DATABASE generalbots OWNER gbuser;
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

## Remote Development Setup

### SSH Configuration for Stable Connections

When developing on remote Linux servers, configure SSH for stable monitoring connections:

Edit `~/.ssh/config`:

```
Host *
    ServerAliveInterval 60
    ServerAliveCountMax 5
```

This configuration:
- **ServerAliveInterval 60**: Sends keepalive packets every 60 seconds
- **ServerAliveCountMax 5**: Allows up to 5 missed keepalives before disconnecting
- Prevents SSH timeouts during long compilations or debugging sessions
- Maintains stable connections for monitoring logs and services

### Remote Monitoring Tips

```bash
# Monitor BotServer logs in real-time
ssh user@server 'tail -f botserver.log'

# Watch compilation progress
ssh user@server 'cd /path/to/botserver && cargo build --release'

# Keep terminal session alive
ssh user@server 'tmux new -s botserver'
```

## Debugging

### Debug Mode

Run with verbose output to troubleshoot issues:

```bash
RUST_LOG=trace cargo run
```

Check logs in the console output for debugging information.

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

2. **Drive Connection Failed**
   - Ensure drive is running on port 9000
   - Check DRIVE_ACCESSKEY and DRIVE_SECRET

3. **Port Already in Use**
   - Change SERVER_PORT in .env
   - Kill existing process: `lsof -i :8080`

4. **Compilation Errors**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean`
   - Check dependencies: `cargo tree`

## LXC Development

### Using LXC Containers

```bash
# Create development containers
lxc-create -n botserver-dev-db -t download -- -d alpine -r 3.18 -a amd64
lxc-create -n botserver-dev-drive -t download -- -d alpine -r 3.18 -a amd64
lxc-create -n botserver-dev-cache -t download -- -d alpine -r 3.18 -a amd64

# Configure PostgreSQL container
lxc-start -n botserver-dev-db
lxc-attach -n botserver-dev-db -- sh -c "
  apk add postgresql14 postgresql14-client
  rc-service postgresql setup
  rc-service postgresql start
  psql -U postgres -c \"CREATE USER gbuser WITH PASSWORD 'password';\"
  psql -U postgres -c \"CREATE DATABASE botserver OWNER gbuser;\"
"

# Configure MinIO (Drive) container
lxc-start -n botserver-dev-drive
lxc-attach -n botserver-dev-drive -- sh -c "
  wget https://dl.min.io/server/minio/release/linux-amd64/minio
  chmod +x minio
  MINIO_ROOT_USER=driveadmin MINIO_ROOT_PASSWORD=driveadmin ./minio server /data --console-address ':9001' &
"

# Configure Redis (Cache) container
lxc-start -n botserver-dev-cache
lxc-attach -n botserver-dev-cache -- sh -c "
  apk add redis
  rc-service redis start
"

# Get container IPs
DB_IP=$(lxc-info -n botserver-dev-db -iH)
DRIVE_IP=$(lxc-info -n botserver-dev-drive -iH)
CACHE_IP=$(lxc-info -n botserver-dev-cache -iH)

echo "Database: $DB_IP:5432"
echo "Drive: $DRIVE_IP:9000"
echo "Cache: $CACHE_IP:6379"
```

Start all services:
```bash
lxc-start -n botserver-dev-db
lxc-start -n botserver-dev-drive
lxc-start -n botserver-dev-cache
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