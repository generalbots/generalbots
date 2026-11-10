# Kubernetes Deployment 🟡 BETA

**Not supported.** This repository contains no Kubernetes manifests, no Helm
chart and no operator. There is no `k8s/`, `deploy/` or `charts/` directory and
no CI job that builds or applies one.

```bash
find . -maxdepth 3 -name 'Chart.yaml' -o -maxdepth 3 -name 'kustomization.yaml'
# no output
```

## What is supported instead

| Target | Page |
|---|---|
| Bare-metal / Incus containers on one host | [Container Deployment (Incus)](./containers.md) |
| Single-host production stack (the reference deployment) | [Infrastructure Design](./infrastructure.md) |
| Desktop single-file install | [Installation](../01-getting-started/installation.md) |

## Why it is not a supported path

`botserver` is a stateful monolith: it owns a bootstrap sequence that creates
PostgreSQL schemas, unseals/reads Vault, provisions the directory and
materialises bots from Drive (`botserver/src/main_module/init.rs`). Nothing in
the tree splits those responsibilities into independently schedulable
workloads, so a manifest set would have to be written and maintained from
scratch.

If a Kubernetes deployment is needed, treat it as new work with its own design
and CI, not as a documented option of this repository.
