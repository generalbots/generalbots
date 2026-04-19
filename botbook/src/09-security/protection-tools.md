# Security Protection Tools

The Security Protection module provides comprehensive host-level security through integration with industry-standard Linux security tools. This module allows administrators to manage security audits, rootkit detection, intrusion detection, and malware scanning through the General Bots UI.

## Overview

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser / UI                              │
│              (Security → Protection tab)                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ HTMX/API calls
┌─────────────────────────────────────────────────────────────┐
│                    botserver (port 9000)                     │
│              /api/security/protection/*                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ sudo (via sudoers)
┌─────────────────────────────────────────────────────────────┐
│                    Host System (Linux)                       │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────────┐          │
│  │  Lynis  │ │RKHunter │ │Chkrootkit│ │Suricata │          │
│  └─────────┘ └─────────┘ └──────────┘ └─────────┘          │
│  ┌─────────┐ ┌─────────┐                                    │
│  │ ClamAV  │ │   LMD   │                                    │
│  └─────────┘ └─────────┘                                    │
└─────────────────────────────────────────────────────────────┘
```

### Tools Included

| Tool | Purpose | Type |
|------|---------|------|
| **Lynis** | Security auditing and hardening | Audit |
| **RKHunter** | Rootkit detection | Scanner |
| **Chkrootkit** | Rootkit detection | Scanner |
| **Suricata** | Network intrusion detection/prevention | IDS/IPS |
| **ClamAV** | Antivirus scanning | Antivirus |
| **LMD** | Linux Malware Detect | Malware Scanner |

## Installation

### Requirements

> **⚠️ IMPORTANT: Root Access Required**
>
> Unlike other botserver components that run in containers, Security Protection tools run on the **host system** and require **root privileges** for installation.

The installation process:
1. Installs security packages via `apt-get`
2. Installs Linux Malware Detect (LMD) from source
3. Creates a sudoers configuration for runtime execution
4. Updates security databases

### Install Command

```bash
sudo botserver install protection
```

This command must be run as root (via `sudo`) because it:
- Installs system packages
- Writes to `/etc/sudoers.d/`
- Updates system security databases

### What Gets Installed

**Packages (via apt-get):**
- `lynis` - Security auditing tool
- `rkhunter` - Rootkit Hunter
- `chkrootkit` - Rootkit checker
- `suricata` - Network IDS/IPS
- `clamav` - Antivirus engine
- `clamav-daemon` - ClamAV daemon service

**From Source:**
- Linux Malware Detect (LMD/maldetect)

**Configuration:**
- `/etc/sudoers.d/gb-protection` - Allows botserver to execute security commands without password

### Verify Installation

Check the status of installed protection tools:

```bash
botserver status protection
```

This shows:
- Which tools are installed
- Whether sudoers is properly configured
- Tool versions

## Security Model

### Why Root Access?

Security tools need elevated privileges because they:
1. **Scan system files** - Access to `/etc`, `/var`, `/usr` requires root
2. **Manage services** - Starting/stopping Suricata/ClamAV requires systemctl
3. **Update databases** - Signature updates write to protected directories
4. **Detect rootkits** - Checking kernel modules and hidden processes needs root

### Sudoers Configuration

The installation creates `/etc/sudoers.d/gb-protection` with **exact command specifications** (no wildcards):

```sudoers
# Lynis - security auditing
gbuser ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system
gbuser ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system --quick
gbuser ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system --quick --no-colors
gbuser ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system --no-colors

# RKHunter - rootkit detection
gbuser ALL=(ALL) NOPASSWD: /usr/bin/rkhunter --check --skip-keypress
gbuser ALL=(ALL) NOPASSWD: /usr/bin/rkhunter --check --skip-keypress --report-warnings-only
gbuser ALL=(ALL) NOPASSWD: /usr/bin/rkhunter --update

# Chkrootkit - rootkit detection
gbuser ALL=(ALL) NOPASSWD: /usr/bin/chkrootkit
gbuser ALL=(ALL) NOPASSWD: /usr/bin/chkrootkit -q

# Suricata - IDS/IPS service management
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl start suricata
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl stop suricata
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart suricata
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl enable suricata
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl disable suricata
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl is-active suricata
gbuser ALL=(ALL) NOPASSWD: /usr/bin/suricata-update

# ClamAV - antivirus service management
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl start clamav-daemon
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl stop clamav-daemon
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart clamav-daemon
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl enable clamav-daemon
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl disable clamav-daemon
gbuser ALL=(ALL) NOPASSWD: /usr/bin/systemctl is-active clamav-daemon
gbuser ALL=(ALL) NOPASSWD: /usr/bin/freshclam

# LMD - Linux Malware Detect
gbuser ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet -a /home
gbuser ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet -a /var/www
gbuser ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet -a /tmp
gbuser ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet --update-sigs
gbuser ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet --update-ver
```

### Security Considerations

**Why exact commands instead of wildcards?**

Using wildcards (e.g., `lynis *`) would allow:
- Arbitrary argument injection
- Potential abuse if botserver is compromised
- Unintended command execution

Exact commands ensure only predefined operations are allowed.

## Usage

### Via UI

Access the Security Protection panel:
1. Navigate to **Tools → Security**
2. Select the **Protection** tab
3. Each tool card shows:
   - Installation status
   - Version
   - Last scan time
   - Available actions

**Available Actions:**
- **Run Scan** - Execute the tool's scan
- **Start/Stop** - Manage services (Suricata, ClamAV)
- **Update** - Update signatures/databases
- **View Report** - See latest scan results

### Via API

All endpoints are under `/api/v1/security/protection/`

**Get Status of All Tools:**
```http
GET /api/security/protection/status
```

**Get Specific Tool Status:**
```http
GET /api/security/protection/lynis/status
```

**Run a Scan:**
```http
POST /api/security/protection/lynis/run
```

**Start/Stop Services:**
```http
POST /api/security/protection/suricata/start
POST /api/security/protection/suricata/stop
```

**Update Definitions:**
```http
POST /api/security/protection/clamav/update
```

**Get Scan Report:**
```http
GET /api/security/protection/rkhunter/report
```

## Tool Details

### Lynis

Security auditing tool that performs comprehensive system hardening assessments.

**Scan Types:**
- Quick audit (`lynis audit system --quick`)
- Full audit (`lynis audit system`)

**Output:**
- Hardening index (0-100)
- Warnings count
- Suggestions count
- Detailed findings

**Report Location:** `/var/log/lynis-report.dat`

### RKHunter

Rootkit Hunter scans for rootkits, backdoors, and local exploits.

**Features:**
- Rootkit signature detection
- File property checks
- Hidden process detection
- Network port analysis

**Commands Available:**
- Scan: `rkhunter --check --skip-keypress`
- Update: `rkhunter --update`

**Report Location:** `/var/log/rkhunter.log`

### Chkrootkit

Lightweight rootkit detection tool.

**Checks For:**
- Known rootkit signatures
- Suspicious file modifications
- Hidden processes
- Network interfaces in promiscuous mode

**Commands Available:**
- Quick scan: `chkrootkit -q`
- Standard scan: `chkrootkit`

### Suricata

Network Intrusion Detection/Prevention System (IDS/IPS).

**Features:**
- Real-time traffic analysis
- Signature-based detection
- Protocol anomaly detection
- Rule-based alerting

**Service Management:**
- Start/Stop/Restart via systemctl
- Rule updates via `suricata-update`

**Log Location:** `/var/log/suricata/eve.json`

### ClamAV

Open-source antivirus engine.

**Features:**
- Virus signature scanning
- Malware detection
- Automatic signature updates

**Service Management:**
- `clamav-daemon` - Background scanning service
- `freshclam` - Signature updates

### LMD (Linux Malware Detect)

Malware scanner designed for shared hosting environments.

**Features:**
- PHP malware detection
- Backdoor/shell detection
- Quarantine functionality

**Scan Paths Allowed:**
- `/home`
- `/var/www`
- `/tmp`

**Commands Available:**
- Scan: `maldet -a <path>`
- Update signatures: `maldet --update-sigs`
- Update version: `maldet --update-ver`

## Troubleshooting

### Installation Fails

**Symptom:** `apt-get install` errors

**Solutions:**
1. Update package lists: `sudo apt-get update`
2. Check disk space: `df -h`
3. Verify internet connectivity
4. Check for conflicting packages

### Permission Denied at Runtime

**Symptom:** Security scans fail with permission errors

**Solutions:**
1. Verify sudoers file exists: `ls -la /etc/sudoers.d/gb-protection`
2. Check sudoers syntax: `sudo visudo -c -f /etc/sudoers.d/gb-protection`
3. Verify file permissions: should be `0440`
4. Reinstall: `sudo botserver install protection`

### Service Won't Start

**Symptom:** Suricata or ClamAV fails to start

**Solutions:**
1. Check service status: `systemctl status suricata`
2. View logs: `journalctl -u suricata`
3. Verify configuration files exist
4. Check for port conflicts

### Outdated Signatures

**Symptom:** Scans report "database outdated"

**Solutions:**
1. Run update via UI or API
2. Manually update:
   - ClamAV: `sudo freshclam`
   - RKHunter: `sudo rkhunter --update`
   - Suricata: `sudo suricata-update`
   - LMD: `sudo maldet --update-sigs`

## Uninstallation

### Remove Sudoers Configuration

```bash
sudo botserver remove protection
```

This removes the sudoers file but **does not uninstall packages**.

### Full Removal (Manual)

To completely remove protection tools:

```bash
# Remove sudoers
sudo botserver remove protection

# Remove packages
sudo apt-get remove --purge lynis rkhunter chkrootkit suricata clamav clamav-daemon

# Remove LMD
sudo rm -rf /usr/local/maldetect
sudo rm /usr/local/sbin/maldet
```

## Best Practices

1. **Schedule Regular Scans** - Use auto-scan features or cron jobs
2. **Keep Signatures Updated** - Enable auto-update for all tools
3. **Review Reports** - Don't just run scans; analyze the results
4. **Act on Findings** - High/Critical findings need immediate attention
5. **Monitor Suricata Alerts** - Network threats require quick response
6. **Backup Before Quarantine** - LMD quarantine moves files; ensure backups exist

## Related Documentation

- [RBAC & Security Design](./rbac-design.md)
- [SOC 2 Compliance](./soc2-compliance.md)
- [Security Matrix Reference](./security-matrix.md)