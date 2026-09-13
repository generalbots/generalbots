# Project Members (Vibe) 🟡 PREVIEW

> **Preview application.** Part of the [Vibe](./vibe.md) workspace, usable today, not yet part of the supported surface.

Project Members controls who can work on a Vibe project and who owns it.

## What it does

| Capability | Detail |
|---|---|
| **Members** | The people attached to the selected project |
| **Roles** | What each member is allowed to do |
| **Adding members** | Search for a person by name or email and add them |
| **Ownership transfer** | Hand the project to another owner, chosen by searching for the new owner's name or email |

## Why ownership transfer is separate

Deleting the owner of a project would orphan it — its runs, deploys and database would have no accountable owner. Transferring ownership first is the safe sequence, which is why it is a first-class action rather than a side effect of removing a member.

## Opening it

Turn on the **Preview** switch in the left sidebar, then open **Project Members** from the app menu. Membership is per project, so select the project first.

## Security note

Adding a member grants access to that project's code, environment and data. Check the role you are assigning rather than defaulting to the most permissive one.

## See Also

- [Vibe](./vibe.md) - The project workspace
- [Roles and permissions](../../09-security/rbac-overview.md) - How roles are evaluated
- [Apps overview](./README.md) - Stability classification for the whole suite
