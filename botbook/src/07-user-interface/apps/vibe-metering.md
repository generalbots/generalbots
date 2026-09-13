# Compute Metering (Vibe) 🟡 PREVIEW

> **Preview application.** Administrative surface for the [Vibe](./vibe.md) workspace. Usable today, not yet part of the supported surface.

Compute Metering answers the operational question behind agentic development: how much machine time is each project consuming, and what is it allowed to consume?

## What it does

| Capability | Detail |
|---|---|
| **VM-hour usage** | Compute consumed per Vibe project |
| **Limits** | The allowance set for a project, next to what it has used |
| **Per-project view** | Metering is attributed to projects, not to the installation as a whole |

## Why it exists

Each Vibe project runs in its own machine. Without limits, a project that is not actively used still holds resources — and, in practice, projects accumulate. Metering is how you find the ones that should be stopped or deleted.

Operationally, keep an eye on two numbers: the **limits** actually configured per project, and total disk on the host. Repeated project creation is the usual cause of a host filling up.

## Opening it

Turn on the **Preview** switch in the left sidebar, then open **Compute Metering** from the app menu. It is an administrative view, so it is subject to role checks like the rest of the admin surfaces.

## See Also

- [Vibe](./vibe.md) - The project workspace
- [Vibe Metrics](./vibe-metrics.md) - Model cost and tool-call telemetry
- [Hardware and Scaling](../../11-hardware-scaling/README.md) - Capacity planning
- [Apps overview](./README.md) - Stability classification for the whole suite
