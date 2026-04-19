# Configuring Local Development Access

After bootstrap, botserver services are immediately accessible via **IP addresses** - no configuration required. For those who prefer friendly hostnames, optional DNS setup is also available.

## Zero Configuration: IP Access (Default)

botserver certificates include `127.0.0.1` as a Subject Alternative Name (SAN), so **mTLS works immediately** via IP address without any system changes.

### Service Ports

| Component | Description | IP:Port |
|-----------|-------------|---------|
| api | Main botserver API | `127.0.0.1:8443` (HTTPS) / `127.0.0.1:9000` (HTTP) |
| tables | PostgreSQL database | `127.0.0.1:5432` |
| drive | Object storage (S3-compatible) | `127.0.0.1:9000` |
| cache | Redis cache | `127.0.0.1:6379` |
| vectordb | Vector database | `127.0.0.1:6333` |
| vault | Secrets management | `127.0.0.1:8200` |
| llm | Local LLM server | `127.0.0.1:8081` |
| embedding | Embedding server | `127.0.0.1:8082` |
| directory | Authentication/Identity | `127.0.0.1:8085` |
| email | Email server | `127.0.0.1:25` (SMTP) / `127.0.0.1:993` (IMAP) |
| meet | Video conferencing | `127.0.0.1:7880` |

### Quick Test

```bash
# Test API (no config needed)
curl -k https://127.0.0.1:8443/health

# With mTLS client certificate
curl --cert ./botserver-stack/conf/system/certificates/api/client.crt \
     --key ./botserver-stack/conf/system/certificates/api/client.key \
     --cacert ./botserver-stack/conf/system/certificates/ca/ca.crt \
     https://127.0.0.1:8443/api/v1/status
```

### Code Examples

#### Python
```python
import requests

response = requests.get(
    "https://127.0.0.1:8443/api/v1/bots",
    cert=("./botserver-stack/conf/system/certificates/api/client.crt",
          "./botserver-stack/conf/system/certificates/api/client.key"),
    verify="./botserver-stack/conf/system/certificates/ca/ca.crt"
)
print(response.json())
```

#### Node.js
```javascript
const https = require('https');
const fs = require('fs');

const options = {
  hostname: '127.0.0.1',
  port: 8443,
  path: '/api/v1/bots',
  method: 'GET',
  cert: fs.readFileSync('./botserver-stack/conf/system/certificates/api/client.crt'),
  key: fs.readFileSync('./botserver-stack/conf/system/certificates/api/client.key'),
  ca: fs.readFileSync('./botserver-stack/conf/system/certificates/ca/ca.crt')
};

https.request(options, (res) => {
  res.on('data', (d) => process.stdout.write(d));
}).end();
```

#### Rust
```rust
use reqwest::Certificate;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = fs::read("./botserver-stack/conf/system/certificates/api/client.crt")?;
    let key = fs::read("./botserver-stack/conf/system/certificates/api/client.key")?;
    let ca = fs::read("./botserver-stack/conf/system/certificates/ca/ca.crt")?;
    
    let identity = reqwest::Identity::from_pem(&[cert, key].concat())?;
    let ca_cert = Certificate::from_pem(&ca)?;
    
    let client = reqwest::Client::builder()
        .identity(identity)
        .add_root_certificate(ca_cert)
        .build()?;
    
    let response = client
        .get("https://127.0.0.1:8443/api/v1/bots")
        .send()
        .await?;
    
    println!("{}", response.text().await?);
    Ok(())
}
```

### Remote Server Access

If botserver runs on a different machine (e.g., `192.168.1.100`), regenerate certificates with additional IP SANs:

```bash
./botserver regenerate-certs --san-ip 192.168.1.100 --san-ip 10.0.0.50
```

Or configure before bootstrap in `botserver-stack/conf/system/ca-config.toml`:

```toml
[certificates.api]
san_names = ["localhost", "api.botserver.local", "127.0.0.1", "192.168.1.100"]
```

---

## Optional: Hostname Access

For browser access with friendly URLs, configure your system to resolve `*.botserver.local` hostnames.

### Hostname Mapping

| Component | Hostname |
|-----------|----------|
| api | `api.botserver.local` |
| tables | `tables.botserver.local` |
| drive | `drive.botserver.local` |
| cache | `cache.botserver.local` |
| vectordb | `vectordb.botserver.local` |
| vault | `vault.botserver.local` |
| llm | `llm.botserver.local` |
| embedding | `embedding.botserver.local` |
| directory | `directory.botserver.local` |
| email | `email.botserver.local` |
| meet | `meet.botserver.local` |

### Option 1: Edit hosts file (Simplest)

#### Windows

Open Notepad **as Administrator**, edit `C:\Windows\System32\drivers\etc\hosts`:

```
127.0.0.1 botserver.local
127.0.0.1 api.botserver.local
127.0.0.1 tables.botserver.local
127.0.0.1 drive.botserver.local
127.0.0.1 cache.botserver.local
127.0.0.1 vectordb.botserver.local
127.0.0.1 vault.botserver.local
127.0.0.1 llm.botserver.local
127.0.0.1 embedding.botserver.local
127.0.0.1 directory.botserver.local
127.0.0.1 email.botserver.local
127.0.0.1 meet.botserver.local
```

**PowerShell one-liner (run as Administrator):**

```powershell
@"
127.0.0.1 botserver.local
127.0.0.1 api.botserver.local
127.0.0.1 tables.botserver.local
127.0.0.1 drive.botserver.local
127.0.0.1 cache.botserver.local
127.0.0.1 vectordb.botserver.local
127.0.0.1 vault.botserver.local
127.0.0.1 llm.botserver.local
127.0.0.1 embedding.botserver.local
127.0.0.1 directory.botserver.local
127.0.0.1 email.botserver.local
127.0.0.1 meet.botserver.local
"@ | Add-Content C:\Windows\System32\drivers\etc\hosts
```

#### Linux

```bash
sudo tee -a /etc/hosts << 'EOF'
127.0.0.1 botserver.local
127.0.0.1 api.botserver.local
127.0.0.1 tables.botserver.local
127.0.0.1 drive.botserver.local
127.0.0.1 cache.botserver.local
127.0.0.1 vectordb.botserver.local
127.0.0.1 vault.botserver.local
127.0.0.1 llm.botserver.local
127.0.0.1 embedding.botserver.local
127.0.0.1 directory.botserver.local
127.0.0.1 email.botserver.local
127.0.0.1 meet.botserver.local
EOF
```

#### macOS

```bash
sudo tee -a /etc/hosts << 'EOF'
127.0.0.1 botserver.local
127.0.0.1 api.botserver.local
127.0.0.1 tables.botserver.local
127.0.0.1 drive.botserver.local
127.0.0.1 cache.botserver.local
127.0.0.1 vectordb.botserver.local
127.0.0.1 vault.botserver.local
127.0.0.1 llm.botserver.local
127.0.0.1 embedding.botserver.local
127.0.0.1 directory.botserver.local
127.0.0.1 email.botserver.local
127.0.0.1 meet.botserver.local
EOF

# Flush DNS cache
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

### Option 2: Use botserver's CoreDNS

botserver runs CoreDNS on port 53. Point your system to use it as DNS server.

#### Windows

```powershell
# Get active interface
$interface = (Get-NetAdapter | Where-Object {$_.Status -eq "Up"}).Name

# Set DNS servers
Set-DnsClientServerAddress -InterfaceAlias $interface -ServerAddresses ("127.0.0.1","8.8.8.8")
```

#### Linux (systemd-resolved)

```bash
sudo mkdir -p /etc/systemd/resolved.conf.d/
sudo tee /etc/systemd/resolved.conf.d/botserver.conf << 'EOF'
[Resolve]
DNS=127.0.0.1
FallbackDNS=8.8.8.8 8.8.4.4
Domains=~botserver.local
EOF
sudo systemctl restart systemd-resolved
```

#### macOS

```bash
sudo mkdir -p /etc/resolver
sudo tee /etc/resolver/botserver.local << 'EOF'
nameserver 127.0.0.1
EOF
```

This routes only `*.botserver.local` queries to botserver's DNS.

---

## Trusting Self-Signed Certificates

For browser access without warnings, trust the CA certificate:

### Windows

```powershell
Import-Certificate -FilePath ".\botserver-stack\conf\system\certificates\ca\ca.crt" -CertStoreLocation Cert:\LocalMachine\Root
```

### Linux

```bash
# Ubuntu/Debian
sudo cp ./botserver-stack/conf/system/certificates/ca/ca.crt /usr/local/share/ca-certificates/botserver-ca.crt
sudo update-ca-certificates

# Fedora/RHEL
sudo cp ./botserver-stack/conf/system/certificates/ca/ca.crt /etc/pki/ca-trust/source/anchors/botserver-ca.crt
sudo update-ca-trust
```

### macOS

```bash
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ./botserver-stack/conf/system/certificates/ca/ca.crt
```

### Firefox

Firefox uses its own certificate store:
1. **Settings** → **Privacy & Security** → **View Certificates**
2. **Authorities** → **Import**
3. Select `botserver-stack/conf/system/certificates/ca/ca.crt`
4. Check "Trust this CA to identify websites"

---

## Troubleshooting

### DNS Not Resolving

```bash
# Check CoreDNS is running
./botserver status dns

# Test DNS directly
dig @127.0.0.1 api.botserver.local
```

### macOS `.local` Conflicts

macOS reserves `.local` for Bonjour. Use the `/etc/resolver/` method which doesn't conflict.

### Reverting Changes

```bash
# Remove hosts entries (Linux/macOS)
sudo sed -i '/botserver\.local/d' /etc/hosts

# Remove macOS resolver
sudo rm /etc/resolver/botserver.local

# Reset DNS (Linux)
sudo rm /etc/systemd/resolved.conf.d/botserver.conf
sudo systemctl restart systemd-resolved

# Reset DNS (Windows)
Set-DnsClientServerAddress -InterfaceAlias $interface -ResetServerAddresses
```

---

## Summary

| Access Method | Configuration | Best For |
|---------------|---------------|----------|
| **IP Address** | None | Default - works immediately, scripts, APIs |
| hosts file | Minimal | Browser access with friendly URLs |
| CoreDNS | Low | Development teams, wildcard subdomains |

**Default recommendation:** Use IP addresses for development. No configuration needed.
