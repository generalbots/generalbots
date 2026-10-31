# Security Policy

<p align="center"><img src="logo.svg" alt="General Bots" width="200"></p>

---

## Reporting a Vulnerability

Please, open an issue or contact us at security@generalbots.org.

## Dependency Advisories

### `accelerate` 1.1.1 (GHSA-4j2p-28q2-5m79)

`botmodels/requirements.txt` pins `accelerate==1.1.1`, which has no patched
release for the sharded-checkpoint `weight_map` path traversal / denial of
service advisory. Exposure is limited to operator-run local model tooling:
the service is not exposed to end users and only loads checkpoints the
operator places on disk. The pin is retained and re-checked on every
dependency review; replace it as soon as an upstream fix is published.
