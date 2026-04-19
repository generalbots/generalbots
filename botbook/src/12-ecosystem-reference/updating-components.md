# Updating Components

botserver's stack components are regularly updated by their respective maintainers. This guide explains how to check for updates, apply them safely, and verify everything works correctly.

## Update Philosophy

botserver uses a **conservative update strategy**:

1. **Pinned Versions** - Each component has a tested version in `3rdparty.toml`
2. **Checksum Verification** - Downloads are verified with SHA256 hashes
3. **Cached Downloads** - Updates are cached in `botserver-installers/` for offline use
4. **Rollback Ready** - Previous binaries can be restored from cache

## Checking for Updates

### View Current Versions

Check installed versions:

```bash
./botserver version --all
```

Example output:
```
botserver Stack Versions:
  vault:     1.15.4
  tables:    17.2.0 (PostgreSQL)
  directory: 2.70.4 (Zitadel)
  drive:     latest (MinIO)
  cache:     8.0.2 (Valkey)
  llm:       b7345 (llama.cpp)
  email:     0.10.7 (Stalwart)
  proxy:     2.9.1 (Caddy)
  dns:       1.11.1 (CoreDNS)
  alm:       10.0.2 (Forgejo)
  meeting:   2.8.2 (LiveKit)
```

### Check Upstream Releases

| Component | Release Page |
|-----------|--------------|
| llama.cpp | [github.com/ggml-org/llama.cpp/releases](https://github.com/ggml-org/llama.cpp/releases) |
| PostgreSQL | [postgresql.org/download](https://www.postgresql.org/download/) |
| MinIO | [github.com/minio/minio/releases](https://github.com/minio/minio/releases) |
| Valkey | [github.com/valkey-io/valkey/releases](https://github.com/valkey-io/valkey/releases) |
| Zitadel | [github.com/zitadel/zitadel/releases](https://github.com/zitadel/zitadel/releases) |
| Vault | [releases.hashicorp.com/vault](https://releases.hashicorp.com/vault/) |
| Stalwart | [github.com/stalwartlabs/mail-server/releases](https://github.com/stalwartlabs/mail-server/releases) |
| Caddy | [github.com/caddyserver/caddy/releases](https://github.com/caddyserver/caddy/releases) |
| CoreDNS | [github.com/coredns/coredns/releases](https://github.com/coredns/coredns/releases) |
| Forgejo | [codeberg.org/forgejo/forgejo/releases](https://codeberg.org/forgejo/forgejo/releases) |
| LiveKit | [github.com/livekit/livekit/releases](https://github.com/livekit/livekit/releases) |

---

## Updating the Configuration

Component URLs and checksums are defined in `3rdparty.toml`. To update a component:

### 1. Edit `3rdparty.toml`

```toml
[components.llm]
name = "Llama.cpp Server"
url = "https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-x64.zip"
filename = "llama-b7345-bin-ubuntu-x64.zip"
sha256 = "91b066ecc53c20693a2d39703c12bc7a69c804b0768fee064d47df702f616e52"
```

### 2. Get the New Checksum

Most releases publish SHA256 checksums. If not, calculate it:

```bash
# Download and calculate checksum
curl -L -o new-release.zip "https://github.com/.../new-release.zip"
sha256sum new-release.zip
```

### 3. Update Both Files

Update both configuration files to stay in sync:

- `3rdparty.toml` - Main component registry
- `config/llm_releases.json` - LLM-specific builds and checksums

---

## Component Update Procedures

### Updating llama.cpp (LLM Server)

The LLM server powers local AI inference. Updates often include performance improvements and new model support.

**Step 1: Check the latest release**

Visit [github.com/ggml-org/llama.cpp/releases](https://github.com/ggml-org/llama.cpp/releases)

**Step 2: Update `3rdparty.toml`**

```toml
[components.llm]
name = "Llama.cpp Server"
url = "https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-x64.zip"
filename = "llama-b7345-bin-ubuntu-x64.zip"
sha256 = "91b066ecc53c20693a2d39703c12bc7a69c804b0768fee064d47df702f616e52"
```

**Step 3: Update `config/llm_releases.json`**

This file contains platform-specific builds:

```json
{
  "llama_cpp": {
    "version": "b7345",
    "base_url": "https://github.com/ggml-org/llama.cpp/releases/download",
    "checksums": {
      "llama-b7345-bin-ubuntu-x64.zip": "sha256:91b066ecc53c20693a2d39703c12bc7a69c804b0768fee064d47df702f616e52",
      "llama-b7345-bin-macos-arm64.zip": "sha256:72ae9b4a4605aa1223d7aabaa5326c66c268b12d13a449fcc06f61099cd02a52"
    }
  }
}
```

**Step 4: Update installer.rs version constant**

```rust
const LLAMA_CPP_VERSION: &str = "b7345";
```

**Step 5: Apply the update**

```bash
# Stop LLM service
pkill llama-server

# Remove old binary
rm -rf botserver-stack/bin/llm/*

# Re-run bootstrap (downloads new version)
./botserver bootstrap

# Or manually trigger download
./botserver update llm
```

**Available llama.cpp Builds (b7345)**

| Platform | Architecture | Variant | Filename |
|----------|-------------|---------|----------|
| Linux | x64 | CPU | `llama-b7345-bin-ubuntu-x64.zip` |
| Linux | x64 | Vulkan | `llama-b7345-bin-ubuntu-vulkan-x64.zip` |
| Linux | s390x | CPU | `llama-b7345-bin-ubuntu-s390x.zip` |
| macOS | ARM64 | Metal | `llama-b7345-bin-macos-arm64.zip` |
| macOS | x64 | CPU | `llama-b7345-bin-macos-x64.zip` |
| Windows | x64 | CPU | `llama-b7345-bin-win-cpu-x64.zip` |
| Windows | x64 | CUDA 12.4 | `llama-b7345-bin-win-cuda-12.4-x64.zip` |
| Windows | x64 | CUDA 13.1 | `llama-b7345-bin-win-cuda-13.1-x64.zip` |
| Windows | x64 | Vulkan | `llama-b7345-bin-win-vulkan-x64.zip` |
| Windows | ARM64 | CPU | `llama-b7345-bin-win-cpu-arm64.zip` |

> **Note:** Linux releases are transitioning from `.zip` to `.tar.gz` format.

---

### Updating PostgreSQL (Tables)

**Warning:** Database updates require careful planning. Always backup first!

```bash
# Backup database
pg_dump $DATABASE_URL > backup-$(date +%Y%m%d).sql

# Update 3rdparty.toml
[components.tables]
url = "https://github.com/theseus-rs/postgresql-binaries/releases/download/17.2.0/postgresql-17.2.0-x86_64-unknown-linux-gnu.tar.gz"
filename = "postgresql-17.2.0-x86_64-unknown-linux-gnu.tar.gz"

# Stop services
./botserver stop

# Apply update
./botserver update tables

# Start services
./botserver start

# Verify
psql $DATABASE_URL -c "SELECT version();"
```

---

### Updating MinIO (Drive)

MinIO updates are generally safe and backward-compatible.

```bash
# Update 3rdparty.toml
[components.drive]
url = "https://dl.min.io/server/minio/release/linux-amd64/minio"
filename = "minio"

# Apply update
./botserver update drive

# Verify
curl http://localhost:9000/minio/health/live
```

---

### Updating Valkey (Cache)

Valkey requires compilation from source.

```bash
# Update 3rdparty.toml
[components.cache]
url = "https://github.com/valkey-io/valkey/archive/refs/tags/8.0.2.tar.gz"
filename = "valkey-8.0.2.tar.gz"

# Stop cache
./botserver stop cache

# Remove old build
rm -rf botserver-stack/bin/cache/*

# Rebuild
./botserver update cache

# Verify
./botserver-stack/bin/cache/valkey-cli ping
```

---

### Updating Zitadel (Directory)

**Warning:** Directory service updates may require database migrations.

```bash
# Backup Zitadel database
pg_dump -d zitadel > zitadel-backup-$(date +%Y%m%d).sql

# Update 3rdparty.toml
[components.directory]
url = "https://github.com/zitadel/zitadel/releases/download/v2.70.4/zitadel-linux-amd64.tar.gz"
filename = "zitadel-linux-amd64.tar.gz"

# Stop directory
./botserver stop directory

# Apply update
./botserver update directory

# Run migrations (if needed)
./botserver-stack/bin/directory/zitadel setup

# Start
./botserver start directory
```

---

### Updating Vault (Secrets)

**Critical:** Vault updates require unsealing after restart.

```bash
# Update 3rdparty.toml
[components.vault]
url = "https://releases.hashicorp.com/vault/1.15.4/vault_1.15.4_linux_amd64.zip"
filename = "vault_1.15.4_linux_amd64.zip"

# Stop Vault
./botserver stop vault

# Apply update
./botserver update vault

# Start and unseal
./botserver start vault
./botserver unseal
```

---

## Platform-Specific Builds

### Automatic Detection

botserver automatically detects your platform and downloads the appropriate build:

1. **Operating System** - Linux, macOS, Windows
2. **Architecture** - x64, ARM64, s390x
3. **GPU Support** - CUDA, Vulkan, Metal, ROCm

### Manual Override

Force a specific build variant:

```toml
# In 3rdparty.toml - use Vulkan build instead of CPU
[components.llm]
url = "https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-vulkan-x64.zip"
```

### GPU Detection

The installer checks for GPU support:

```rust
// Linux CUDA detection
if Path::new("/usr/local/cuda").exists() || env::var("CUDA_HOME").is_ok() {
    // Use CUDA build
}

// Vulkan detection  
if Path::new("/usr/share/vulkan").exists() || env::var("VULKAN_SDK").is_ok() {
    // Use Vulkan build
}
```

---

## Offline Updates

### Pre-download for Air-Gapped Systems

1. Download releases on a connected machine:

```bash
# Download all components
mkdir offline-updates
cd offline-updates

# LLM
curl -LO https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-x64.zip

# Database
curl -LO https://github.com/theseus-rs/postgresql-binaries/releases/download/17.2.0/postgresql-17.2.0-x86_64-unknown-linux-gnu.tar.gz

# ... other components
```

2. Transfer to air-gapped system
3. Copy to cache directory:

```bash
cp offline-updates/* /path/to/botserver-installers/
```

4. Run bootstrap (uses cached files):

```bash
./botserver bootstrap
```

---

## Verifying Updates

### Run Tests

```bash
# Run test suite
cargo test

# Integration tests
./botserver test
```

### Health Checks

```bash
# Check all services
./botserver status

# Individual service checks
curl -k https://localhost:8081/health  # LLM
curl -k https://localhost:8082/health  # Embedding
curl http://localhost:9000/minio/health/live  # Drive
```

### Security Audit

After updating dependencies:

```bash
# Rust dependencies
cargo audit

# Check for known vulnerabilities
cargo audit --deny warnings
```

---

## Rollback Procedure

If an update causes issues:

### Quick Rollback

```bash
# Stop services
./botserver stop

# Restore from cache (previous version must exist)
cp botserver-installers/llama-b4547-bin-ubuntu-x64.zip /tmp/
unzip /tmp/llama-b4547-bin-ubuntu-x64.zip -d botserver-stack/bin/llm/

# Restart
./botserver start
```

### Full Rollback

```bash
# Restore database from backup
psql $DATABASE_URL < backup-20241210.sql

# Restore old binaries
rm -rf botserver-stack/bin/
tar -xzf botserver-stack-backup.tar.gz

# Restart
./botserver start
```

---

## Update Schedule Recommendations

| Component | Update Frequency | Risk Level |
|-----------|-----------------|------------|
| llama.cpp | Weekly/Monthly | Low |
| MinIO | Monthly | Low |
| Valkey | Quarterly | Low |
| Caddy | Monthly | Low |
| CoreDNS | Quarterly | Low |
| PostgreSQL | Quarterly | Medium |
| Zitadel | Quarterly | Medium |
| Vault | Quarterly | High |
| Stalwart | Monthly | Medium |

### Security Updates

Apply security patches immediately for:
- Vault (secrets management)
- PostgreSQL (database)
- Zitadel (authentication)

---

## Automating Updates

### Update Script

Create `update-components.sh`:

```bash
#!/bin/bash
set -e

echo "Backing up current state..."
./botserver backup

echo "Stopping services..."
./botserver stop

echo "Updating components..."
for component in llm drive cache; do
    echo "Updating $component..."
    ./botserver update $component
done

echo "Starting services..."
./botserver start

echo "Running health checks..."
./botserver status

echo "Update complete!"
```

### Scheduled Updates

Use cron for automated updates (use with caution):

```bash
# Weekly LLM updates (low risk)
0 3 * * 0 /path/to/botserver update llm

# Monthly full updates
0 3 1 * * /path/to/update-components.sh
```

---

## Troubleshooting Updates

### Download Failures

```bash
# Clear cache and retry
rm botserver-installers/component-name*
./botserver update component-name
```

### Checksum Mismatch

```bash
# Verify checksum manually
sha256sum botserver-installers/llama-b7345-bin-ubuntu-x64.zip
# Compare with 3rdparty.toml
```

### Service Won't Start

```bash
# Check logs
tail -100 botserver-stack/logs/llm.log

# Check permissions
ls -la botserver-stack/bin/llm/

# Make executable
chmod +x botserver-stack/bin/llm/llama-server
```

### Database Migration Errors

```bash
# Run migrations manually
./botserver migrate

# Or reset (WARNING: data loss)
./botserver reset tables
```

---

## See Also

- [Component Reference](./component-reference.md) - Detailed component documentation
- [Security Auditing](./security-auditing.md) - Vulnerability scanning
- [Backup and Recovery](./backup-recovery.md) - Data protection