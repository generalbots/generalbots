# Building from Source

This guide covers building BotServer from source, including dependencies, feature flags, and platform-specific considerations.

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Rust**: 1.70 or later (2021 edition)
- **Memory**: 4GB RAM minimum (8GB recommended)
- **Disk Space**: 8GB for development environment

### Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version
cargo --version
```

### System Dependencies

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    cmake
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install -y \
    gcc \
    gcc-c++ \
    make \
    pkg-config \
    openssl-devel \
    postgresql-devel \
    cmake
```

#### macOS

```bash
brew install postgresql openssl cmake
```

#### Windows

Install Visual Studio Build Tools with C++ support, then:

```powershell
# Install PostgreSQL (for libpq)
choco install postgresql
```

## Clone Repository

```bash
git clone https://github.com/GeneralBots/BotServer.git
cd BotServer
```

## Build Configurations

### Standard Build

Build with default features (includes desktop support):

```bash
cargo build --release
```

The compiled binary will be at `target/release/botserver`.

### Minimal Build

Build without any optional features:

```bash
cargo build --release --no-default-features
```

This excludes:
- Desktop GUI (Tauri)
- Vector database (Qdrant)
- Email integration (IMAP)

### Feature-Specific Builds

#### With Vector Database

Enable Qdrant vector database support:

```bash
cargo build --release --features vectordb
```

#### With Email Support

Enable IMAP email integration:

```bash
cargo build --release --features email
```

#### Desktop Application

Build as desktop app with Tauri (default):

```bash
cargo build --release --features desktop
```

#### All Features

Build with all optional features:

```bash
cargo build --release --all-features
```

## Feature Flags

BotServer supports the following features defined in `Cargo.toml`:

```toml
[features]
default = ["desktop"]
vectordb = ["qdrant-client"]
email = ["imap"]
desktop = ["dep:tauri", "dep:tauri-plugin-dialog", "dep:tauri-plugin-opener"]
```

### Feature Details

| Feature | Dependencies | Purpose |
|---------|--------------|---------|
| `desktop` | tauri, tauri-plugin-dialog, tauri-plugin-opener | Native desktop application with system integration |
| `vectordb` | qdrant-client | Semantic search with Qdrant vector database |
| `email` | imap | IMAP email integration for reading emails |

## Build Profiles

### Debug Build

For development with debug symbols and no optimizations:

```bash
cargo build
```

Binary location: `target/debug/botserver`

### Release Build

Optimized for production with LTO and size optimization:

```bash
cargo build --release
```

Binary location: `target/release/botserver`

The release profile in `Cargo.toml` uses aggressive optimization:

```toml
[profile.release]
lto = true              # Link-time optimization
opt-level = "z"         # Optimize for size
strip = true            # Strip symbols
panic = "abort"         # Abort on panic (smaller binary)
codegen-units = 1       # Better optimization (slower build)
```

## Platform-Specific Builds

### Linux

Standard build works on most distributions:

```bash
cargo build --release
```

For static linking (portable binary):

```bash
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu
```

### macOS

Build for current architecture:

```bash
cargo build --release
```

Build universal binary (Intel + Apple Silicon):

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create \
    target/x86_64-apple-darwin/release/botserver \
    target/aarch64-apple-darwin/release/botserver \
    -output botserver-universal
```

### Windows

Build with MSVC toolchain:

```bash
cargo build --release
```

Binary location: `target\release\botserver.exe`

## Cross-Compilation

### Install Cross-Compilation Tools

```bash
cargo install cross
```

### Build for Linux from macOS/Windows

```bash
cross build --release --target x86_64-unknown-linux-gnu
```

### Build for Windows from Linux/macOS

```bash
cross build --release --target x86_64-pc-windows-gnu
```

## Troubleshooting

### OpenSSL Errors

If you encounter OpenSSL linking errors:

**Linux:**
```bash
sudo apt install libssl-dev
```

**macOS:**
```bash
export OPENSSL_DIR=$(brew --prefix openssl)
cargo build --release
```

**Windows:**
```bash
# Use vcpkg
vcpkg install openssl:x64-windows
set OPENSSL_DIR=C:\vcpkg\installed\x64-windows
cargo build --release
```

### PostgreSQL Library Errors

If libpq is not found:

**Linux:**
```bash
sudo apt install libpq-dev
```

**macOS:**
```bash
brew install postgresql
export PQ_LIB_DIR=$(brew --prefix postgresql)/lib
```

**Windows:**
```bash
# Ensure PostgreSQL is in PATH
set PQ_LIB_DIR=C:\Program Files\PostgreSQL\15\lib
```

### Out of Memory During Build

Reduce parallel jobs:

```bash
cargo build --release -j 2
```

Or limit memory per job:

```bash
CARGO_BUILD_JOBS=2 cargo build --release
```

### Linker Errors

Ensure you have a C/C++ compiler:

**Linux:**
```bash
sudo apt install build-essential
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
Install Visual Studio Build Tools with C++ support.

## Verify Build

After building, verify the binary works:

```bash
./target/release/botserver --version
```

Expected output: `botserver 6.0.8` or similar.

## Development Builds

### Watch Mode

Auto-rebuild on file changes:

```bash
cargo install cargo-watch
cargo watch -x 'build --release'
```

### Check Without Building

Fast syntax and type checking:

```bash
cargo check
```

With specific features:

```bash
cargo check --features vectordb,email
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Tests for Specific Module

```bash
cargo test --package botserver --lib bootstrap::tests
```

### Run Integration Tests

```bash
cargo test --test '*'
```

## Code Quality

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy -- -D warnings
```

### Check Dependencies

```bash
cargo tree
```

Find duplicate dependencies:

```bash
cargo tree --duplicates
```

### Security Audit

Run security audit to check for known vulnerabilities in dependencies:

```bash
cargo install cargo-audit
cargo audit
```

This should be run regularly during development to ensure dependencies are secure.

## Build Artifacts

After a successful release build, you'll have:

- `target/release/botserver` - Main executable
- `target/release/build/` - Build script outputs
- `target/release/deps/` - Compiled dependencies

## Size Optimization

The release profile already optimizes for size. To further reduce:

### Strip Binary Manually

```bash
strip target/release/botserver
```

### Use UPX Compression

```bash
upx --best --lzma target/release/botserver
```

Note: UPX may cause issues with some systems. Test thoroughly.

## Incremental Compilation

For faster development builds:

```bash
export CARGO_INCREMENTAL=1
cargo build
```

Note: This is enabled by default for debug builds.

## Clean Build

Remove all build artifacts:

```bash
cargo clean
```

## LXC Build

Build inside LXC container:

```bash
# Create build container
lxc-create -n botserver-build -t download -- -d ubuntu -r jammy -a amd64

# Configure container with build resources
cat >> /var/lib/lxc/botserver-build/config << EOF
lxc.cgroup2.memory.max = 4G
lxc.cgroup2.cpu.max = 400000 100000
EOF

# Start container
lxc-start -n botserver-build

# Install build dependencies
lxc-attach -n botserver-build -- bash -c "
apt-get update
apt-get install -y build-essential pkg-config libssl-dev libpq-dev cmake curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source \$HOME/.cargo/env
"

# Build BotServer
lxc-attach -n botserver-build -- bash -c "
git clone https://github.com/GeneralBots/BotServer /build
cd /build
source \$HOME/.cargo/env
cargo build --release --no-default-features
"

# Copy binary from container
lxc-attach -n botserver-build -- cat /build/target/release/botserver > /usr/local/bin/botserver
chmod +x /usr/local/bin/botserver
```

## Installation

After building, install system-wide:

```bash
sudo install -m 755 target/release/botserver /usr/local/bin/
```

Or create a symlink:

```bash
ln -s $(pwd)/target/release/botserver ~/.local/bin/botserver
```

## Next Steps

After building:

1. Run the bootstrap process to install dependencies
2. Configure `.env` file with database credentials
3. Start BotServer and access web interface
4. Create your first bot from templates

See [Chapter 01: Run and Talk](../chapter-01/README.md) for next steps.