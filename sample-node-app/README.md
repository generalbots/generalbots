# sample-node-app

Minimal, **zero-dependency** Node.js HTTP service used to exercise the
General Bots incus dev → prod hosting flow.

## Endpoints

| Method | Path         | Response |
|--------|--------------|----------|
| GET    | `/`          | Landing page (HTML) |
| GET    | `/health`    | `{ status, app, version, uptime_seconds }` |
| GET    | `/api/info`  | `{ app, version, uptime_seconds, node, pid }` |
| GET    | `/api/time`  | `{ app, server_time, epoch_ms }` |

Port is configured with `PORT` (default `8080`).

## Run locally (dev)

```bash
node server.js
# or
npm start
```

Verify:

```bash
curl -s http://localhost:8080/health
```

## Host in incus (dev)

The app runs natively (no container image required) inside an incus container
that has Node.js 18+ installed.

```bash
# 1. Create the dev container from a Node-ready base image
incus launch images:ubuntu/24.04 sample-node-dev

# 2. Push the app
incus file push -r sample-node-app sample-node-dev/opt/gbo/apps/

# 3. Install the systemd unit and start
incus exec sample-node-dev -- bash -c '
  useradd -r -m app 2>/dev/null || true
  cp /opt/gbo/apps/sample-node-app/deploy/sample-node-app.service /etc/systemd/system/
  systemctl daemon-reload
  systemctl enable --now sample-node-app
'

# 4. Verify (proxy/port-forward as needed)
incus exec sample-node-dev -- curl -s http://localhost:8080/health
```

## Deploy to prod

Per General Bots policy, production deployments go through CI/CD (ALM → CI →
container), never manual binary/scp transfers. Commit and push, and let the
CI runner build/transfer/restart. The systemd unit in `deploy/` is what the
prod `system` container runs.

For a fresh prod push (only with explicit approval):

```bash
incus file push -r sample-node-app system/opt/gbo/apps/
incus exec system -- bash -c '
  cp /opt/gbo/apps/sample-node-app/deploy/sample-node-app.service /etc/systemd/system/
  systemctl daemon-reload && systemctl enable --now sample-node-app
'
```

## Container image (alternative)

If the target incus container does not run Node natively, use the image:

```bash
docker build -t sample-node-app:1.0.0 .
docker save sample-node-app:1.0.0 | gzip > /tmp/sample-node-app.tar.gz
incus image import /tmp/sample-node-app.tar.gz --alias sample-node-app
incus launch sample-node-app sample-node-prod
```
