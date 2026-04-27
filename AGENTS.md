# General Bots AI Agent Guidelines

- For host/infra operations see INFRA.md (container ops, Vault, CI/CD, DNS, MinIO, alerts)
- stop saving .png on root! Use /tmp. never allow new files on root.
- never push to alm without asking first - it is production!
- NEVER deploy to production manually — ALWAYS use CI/CD pipeline
- NEVER include sensitive data (IPs, tokens, passwords, keys) in any documentation
- NEVER use scp or manual deployment to system container
- ALWAYS push to ALM → CI builds on alm-ci → CI deploys automatically
- 8080 is server, 3000 is client UI
- Test web: http://localhost:3000 (botui!)
- Test login: http://localhost:3000/suite/auth/login.html
- DEV ENV — sometimes pasting from PROD, do not treat as prod. Just fix, push to CI.
- Use Playwright MCP to start localhost:3000/botname now.
- NEVER create files with secrets in repository root
- Mask IPs in logs (10.x.x.x pattern, not real IPs)
- NEVER use cargo clean — causes 30min rebuilds, use ./reset.sh for DB issues
- Secret files go in /tmp/ only (vault-token-gb, vault-unseal-key-gb)
- See botserver/src/drive/local_file_monitor.rs for how bots load from drive (MinIO)

---

## WORKSPACE STRUCTURE

| Crate | Purpose | Port | Tech |
|-------|---------|------|------|
| botserver | Main API server, business logic | 8080 | Axum, Diesel, Rhai BASIC |
| botui | Web UI server (dev) + proxy | 3000 | Axum, HTML/HTMX/CSS |
| botapp | Desktop app wrapper | - | Tauri 2 |
| botlib | Shared library | - | Core types, errors |
| botbook | Documentation | - | mdBook |
| bottest | Integration tests | - | tokio-test |
| botdevice | IoT/Device support | - | Rust |
| botplugin | Browser extension | - | JS |

Key Paths: target/debug/botserver | botserver/.env | botui/ui/suite/

---

## CHAT FLOW

1. WebSocket connect → UserSession created, session_id generated
2. start.bas runs ONCE per session → ADD_SUGGESTION calls, Redis flag set
3. Message processing → if type 6: TOOL_EXEC (bypass LLM), else: KB injection + LLM
4. Tool Execution → Direct .ast via Rhai, no LLM/KB
5. LLM Response → Streaming via WebSocket chunks
6. Frontend Display → HTMX, suggestion buttons from Redis

Message Types: 0=EXTERNAL 1=USER 2=BOT_RESPONSE 3=CONTINUE 4=SUGGESTION 5=CONTEXT_CHANGE 6=TOOL_EXEC

Bots live in drive (MinIO storage), each bucket is a bot. Respect LOAD_ONLY.

---

## BASIC SCRIPT ARCHITECTURE

- start.bas — Runs once per session, loads suggestions
- tables.bas — Parsed at compile time, defines DB schema (DO NOT CALL with CALL)
- tool.bas — Compiled to .ast, executed via CALL or TOOL_EXEC type 6

### BASIC Keywords — Syntax Signatures (no quotes, bare tokens)

TALK message | HEAR prompt AS variable | USE KB name | CLEAR KB
USE WEBSITE url | ADD SUGGESTION text | CLEAR SUGGESTIONS
GET FROM table WHERE condition | SAVE record TO table | FIND value IN table
FIRST(array) | LAST(array) | COUNT(array) | FORMAT template, var1, var2
CREATE FILE path WITH content | READ FILE path | WRITE FILE path WITH content
DELETE FILE path | COPY FILE source TO dest | LIST FILES path
UPLOAD data TO path | DOWNLOAD url TO path
GET HTTP url | POST HTTP url WITH data | WEBHOOK url WITH data
CREATE_TASK title, assignee, due, project_id | WAIT seconds
ON EMAIL FROM filter DO CALL handler | ON CHANGE table DO CALL handler
SET BOT MEMORY key = value | GET BOT MEMORY key | REMEMBER key = value
SET CONTEXT key = value | SET USER property = value | TRANSFER TO HUMAN
ADD BOT name WITH TRIGGER phrase | DELEGATE TO name | SEND TO BOT name MESSAGE msg
SEND MAIL TO email WITH subject, body | SEND SMS TO phone MESSAGE text
CREATE DRAFT title WITH content | CREATE SITE name WITH config
POST TO SOCIAL platform MESSAGE text | LLM prompt
DETECT tablename | CALL scriptname

Built-in Variables: TODAY, NOW, USER, SESSION, BOT

> Full keyword docs: botbook/src/04-basic-scripting/

---

## SECURITY DIRECTIVES — MANDATORY

1. Error Handling: NO unwrap/expect/panic/todo/unimplemented. Use value? or ok_or_else
2. Command Execution: Use SafeCommand, never Command::new directly
3. Error Responses: Use log_and_sanitize, never raw error strings to clients
4. SQL: Use sql_guard sanitize_identifier/validate_table_name, never format strings
5. Rate Limiting: governor crate, per-IP and per-User, WebSocket 10 msgs/s
6. CSRF: All state-changing endpoints need CSRF token (exempt: Bearer Token APIs)
7. Security Headers on ALL responses: CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
8. Dependency Pinning: Critical deps exact version, track Cargo.lock for app crates

> Full security docs: botbook/src/09-security/

---

## MANDATORY CODE PATTERNS

- Use Self in impl blocks, not MyStruct
- Derive PartialEq + Eq together
- format!(name) not format!({}, name)
- Combine identical match arms: A | B => thing()

---

## ABSOLUTE PROHIBITIONS

- NEVER search /target folder (binary compiled)
- NEVER hardcode credentials — use generate_random_string() or env vars
- NEVER build in release mode or use --release flag
- NEVER run cargo build — use cargo check
- NEVER deploy manually — always push to ALM, CI/CD deploys
- NEVER use scp/SSH to deploy — CI workflow handles it
- NEVER use cargo clean — causes 30min rebuilds
- NEVER use panic/todo/unimplemented/Command::new/unwrap/expect
- NEVER use #[allow()] — FIX the code
- NEVER add lint exceptions to Cargo.toml
- NEVER prefix unused vars with _ — DELETE or USE them
- NEVER leave unused imports or dead code
- NEVER use CDN links — all assets local
- NEVER comment out code — DELETE it
- NEVER create .md files without checking botbook/ first
- NEVER write internal IPs to logs — mask as 10.x.x.x

---

## FILE SIZE LIMIT: 450 LINES MAX

Split at 350 lines proactively: types.rs, handlers.rs, operations.rs, utils.rs, mod.rs

---

## ERROR FIXING WORKFLOW

OFFLINE FIRST: Read ALL errors → group by file → fix ALL per file → write once → verify LAST
NEVER run cargo check DURING fixing — fix ALL offline, verify ONCE at end
Streaming Build Rule: Cancel build as soon as first errors appear, fix immediately

---

## MEMORY MANAGEMENT

If Killed during compile: pkill -9 cargo; pkill -9 rustc; pkill -9 botserver
Then: CARGO_BUILD_JOBS=1 cargo check -p botserver 2>&1 | tail -200

---

## RESET PROCESS

reset.sh cleans and restarts dev env. Bootstrap takes 3-5 min.
If timeout: check botserver.log for Bootstrap process completed
Manual: ps aux | grep botserver | grep -v grep; curl http://localhost:8080/health; ./restart.sh
Verify: PostgreSQL 5432, Valkey 6379, BotServer 8080, BotUI 3000

---

## PLAYWRIGHT BROWSER TESTING

1. Navigate to http://localhost:3000/botname
2. Take snapshot → test flows → verify results → validate backend
3. Desktop may have maximized chat — click middle button to minimize
4. Backend validation: psql or tail logs after UI interactions

---

## ADDING NEW FEATURES

1. Add types to botlib if shared
2. Add migration SQL + Diesel model
3. Add business logic in botserver/src/features/
4. Add API endpoint
5. Add BASIC keyword if needed (register_custom_syntax, spawn thread for async)
6. Security: input validation, auth, rate limit, sanitize errors, no unwrap

> Architecture details: botbook/src/02-architecture-packages/architecture.md

---

## BUG FIXING

1. Reproduce: grep errors in botserver.log, trace data flow
2. Find code: grep -r pattern --include=*.rs, check mod.rs
3. Fix minimal: wrong variable? missing validation? race condition?
4. Test: cargo check -p botserver, ./restart.sh, verify in browser
5. Commit: clear message with root cause, impact, files, testing notes

Logs: /opt/gbo/logs/err.log (errors) | /opt/gbo/logs/out.log (output) | botserver.log (dev only) | botui.log | [drive_monitor] prefix | CLIENT: prefix
On staging/production: check err.log and out.log in /opt/gbo/logs/

> Troubleshooting: botbook/src/12-ecosystem-reference/troubleshooting.md

---

## DEPLOY WORKFLOW

Push to ALM → CI builds on alm-ci → deploys to system container via SSH
NEVER deploy manually. CI path: alm-ci builds → tar+gzip → /opt/gbo/bin/botserver → restart
CI deploy: alm-ci at /opt/gbo/data/botserver/target/debug/botserver → SSH → system container
Runner: gbuser uid 1000, workspace /opt/gbo/data/, SSH key /home/gbuser/.ssh/id_ed25519

> CI/CD details: botbook/src/12-ecosystem-reference/ci-cd.md

---

## PRODUCTION CONTAINER ARCHITECTURE

| Container | Service | Port |
|-----------|---------|------|
| system | BotServer + Valkey | 8080/6379 |
| vault | Vault | 8200 |
| tables | PostgreSQL | 5432 |
| drive | MinIO | 9000/9100 |
| directory | Zitadel | 9000 |
| meet | LiveKit | 7880 |
| vectordb | Qdrant | 6333 |
| llm | llama.cpp | 8081 |
| email | Stalwart | 993/465/587 |
| alm | Forgejo | 4747 (NOT 3000!) |
| alm-ci | Forgejo Runner | - |
| proxy | Caddy | 80/443 |
| dns | CoreDNS | 53 |

Container ops: sudo incus list/start/stop/restart/exec/snapshot
Backup: pg_dump -U postgres -F c -f /tmp/backup.dump dbname
ALM port is 4747. Runner token in action_runner_token table.

> Container details: botbook/src/02-architecture-packages/containers.md
> Backup/Recovery: botbook/src/12-ecosystem-reference/backup-recovery.md

---

## PRODUCTION OPERATIONS — QUICK REFERENCE

### Critical Safety Rules
- NEVER modify iptables without explicit confirmation
- NEVER touch PROD without asking first
- ALWAYS backup files to /tmp before editing

### Infrastructure Paths
- Base: /opt/gbo/ | Data: /opt/gbo/data | Bin: /opt/gbo/bin
- Conf: /opt/gbo/conf | Logs: /opt/gbo/logs

### Service Operations
- DNS (CoreDNS): config /opt/gbo/conf/Corefile, zones /opt/gbo/data/domain.zone
- PostgreSQL: data /opt/gbo/data, backup pg_dump, restore pg_restore
- Email (Stalwart): config /opt/gbo/conf/config.toml, check DKIM TXT records
- Proxy (Caddy): config /opt/gbo/conf/config, validate then reload
- MinIO: internal API http://drive-ip:9000, data /opt/gbo/data
- Bot System: binary /opt/gbo/bin/botserver, Valkey port 6379
- ALM (Forgejo): port 4747, CI runner separate container, token from DB
- CI Runner: config /opt/gbo/bin/config.yaml, runs as gbuser, systemd service
  sccache at /usr/local/bin/sccache, workspace /opt/gbo/data/

### Network — NAT Port Forwarding
External ports DNAT to container IPs via iptables. Rules in /etc/iptables.rules
Always use external interface (-i iface) to avoid loopback issues.

Port Map: 53=DNS 80/443=HTTP/HTTPS 5432=PostgreSQL 993=IMAPS 465=SMTPS 587=Submission 4747=Forgejo 9000=MinIO 8200=Vault

### Container Management
sudo incus list | start/stop/restart container | exec container -- bash
sudo incus snapshot create container name | copy container/snapshot target

### Troubleshooting
- Container won't start: sudo incus list, sudo incus log container --show-log
- Service not running: sudo incus exec container -- pgrep -a process
- Email delivery: check stalwart, IMAP/SMTP ports, DKIM records
- Disk space: df -h, sudo btrfs filesystem df /var/lib/incus

> Full operations: botbook/src/02-architecture-packages/containers.md

---

## STAGING ENVIRONMENT (STAGE-GBO)

Use 10.0.3.x subnet for container IPs. Route via host gateway 10.0.0.1.
Do NOT confuse staging (10.0.3.x) with production ranges.

---

## FRONTEND STANDARDS

HTMX-first: hx-get, hx-post, hx-target, hx-swap. No CDN. All assets local.

> UI details: botbook/src/07-user-interface/htmx-architecture.md

---

## PERFORMANCE & LINTING

- Clippy: 0 warnings. No #[allow()] — fix code
- Release profile: opt-level=z, lto=true, codegen-units=1, strip=true, panic=abort
- Weekly: cargo tree --duplicates, cargo machete, cargo audit

---

## TESTING

- Unit tests: inline #[cfg(test)] modules or crate tests/ dir
- Integration tests: bottest/ crate, cargo test -p bottest
- Coverage: 80%+ for critical paths, ALL error paths, ALL security guards

> Testing details: botbook/src/12-ecosystem-reference/architecture.md

---

## CONTINUATION PROMPT

1. Check build/diagnostics
2. Fix ALL warnings and errors — no #[allow()]
3. Delete unused code
4. Replace ALL unwrap/expect with proper error handling
5. Verify after each fix batch
6. Loop until 0 warnings, 0 errors
7. Refactor files >450 lines

---

## MEMORY DIRECTIVES

OFFLINE FIRST → BATCH BY FILE → WRITE ONCE → VERIFY LAST → DELETE DEAD CODE
Push to ALL remotes (github, pragmatismo)

---

## SECRET FILES — /tmp/ ONLY

vault-token-gb and vault-unseal-key-gb go in /tmp/ only — cleared on reboot, not tracked by git.
