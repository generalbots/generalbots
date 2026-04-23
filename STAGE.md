# Staging Environment Guide (STAGE-GBO)

## Infrastructure Overview

The staging environment is implemented using an isolated **Incus Project** named `STAGE-GBO`. This guarantees that the stage containers, network, and storage are entirely separated from the production environment (`default` project), preventing any accidental interference with `PROD-GBO` containers.

To work within the staging environment, you must switch to its project first:
```bash
sudo incus project switch STAGE-GBO
```
To switch back to production:
```bash
sudo incus project switch default
```

### Container Architecture

The stage environment consists of clones of the following production containers, restricted to a maximum of 10GB disk space and mapped to a dedicated 10.0.3.x subnet (`stagebr0` network):

| Container | Internal IP | Data Status | Purpose |
|-----------|-------------|-------------|---------|
| **system** | `10.0.3.10` | Wiped (`/opt/gbo/work/`) | Main BotServer + BotUI |
| **tables** | `10.0.3.11` | Intact (schema & DB preserved) | PostgreSQL database |
| **vault** | `10.0.3.12` | Intact | Secrets management |
| **cache** | `10.0.3.13` | Wiped (RDB/AOF deleted) | Valkey cache |
| **drive** | `10.0.3.14` | Wiped (started from scratch) | MinIO object storage |
| **llm** | `10.0.3.15` | Intact | llama.cpp local inference |

## Automation Script

The setup process was automated using `setup-stage-gbo.sh`. The script performs the following tasks:
1. **Creates `STAGE-GBO` Project:** Configured with `features.networks=true` and `features.profiles=true` to isolate networks and profiles from PROD.
2. **Creates `stagebr0` Network:** A dedicated NAT network for 10.0.3.x.
3. **Sets Resource Limits:** Configures the `default` profile in `STAGE-GBO` with a 10GB root disk size limit.
4. **Clones Containers:** Uses `incus copy` to securely copy containers from `default` to `STAGE-GBO` using ZFS/BTRFS copy-on-write without consuming immediate space.
5. **Configures IPs:** Updates `/etc/network/interfaces` inside each stage container to assign the static 10.0.3.x IPs.
6. **Cleans Data:** Wipes `/opt/gbo/logs/` in all containers, wipes MinIO data in `drive`, wipes the AST cache in `system`, and clears Valkey data in `cache`. The BotServer database in `tables` is preserved for testing.

## Daily Operations & Access

### Accessing Stage Containers

Because the project is isolated, running commands requires switching the project or specifying it explicitly:
```bash
# Explicitly access the system container in STAGE-GBO
sudo incus exec STAGE-GBO:system -- bash

# Or switch context entirely
sudo incus project switch STAGE-GBO
sudo incus list
sudo incus exec system -- bash
```

### Resetting Data

If you need to completely reset a specific component in the staging environment without affecting production, simply stop it, clear its data, and restart it:
```bash
sudo incus project switch STAGE-GBO
sudo incus stop drive
sudo incus exec drive -- rm -rf /opt/gbo/data/minio/*
sudo incus start drive
```

### Security Directives

- **NO External Exposure:** The staging environment is internally isolated. Do not map public DNS or Caddy proxy rules to the `10.0.3.x` IPs unless testing is specifically required via a staging domain.
- **Data Protection:** Although it's an isolated project, `incus copy` relies on the host's underlying storage. Running aggressive I/O operations or writing massive amounts of data in stage could potentially exhaust the host's shared disk space. The 10GB hard limit per container mitigates this, but monitor `df -h` on the host to ensure `PROD` is not impacted.
