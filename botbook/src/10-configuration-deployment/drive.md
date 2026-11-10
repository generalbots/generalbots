# Drive Integration 🟢 GA

Drive is the bot file system: every bot owns a `{bot}.gbai` bucket in the S3
compatible store, and the runtime reads bots from there — never from the local
filesystem.

This page is an index; the mechanics live in the pages below.

| Topic | Page |
|---|---|
| The Drive app (tabs, uploads, sharing) | [Drive app](../07-user-interface/apps/drive.md) |
| `/api/files/*` REST surface | [Files API](../08-rest-api-tools/files-api.md) |
| Bucket layout and tenancy | [.gbai package format](../02-architecture-packages/gbai.md) |
| Monitors that load bots from Drive | [Service Layer](../02-architecture-packages/services.md) |

## Contract in one paragraph

A bot is materialised from `{org}.gborg/{workspace}.gbai/{bot}.gbot|gbdialog|gbkb|gbdrive`.
`drive_monitors/` scans the buckets, creates the bot rows and reloads a bot when
the object ETag changes; `git_bot_monitor/` pulls git-owned bots every 15 s into
the work dir. `.gbdialog` and `.gbot` of a git-owned bot come from its ALM
repository — the Drive copy of those prefixes is archived, not read.

## Credentials

Drive credentials come from Vault (`secret/gbo/drive`), never from a file in the
repository. See the Drive & Vault section of the repository `AGENTS.md` for the
canonical `mc` setup.

## See also

- [Network Contract](../02-architecture-packages/network-contract.md) — port 9000 is the object store, 8080 is the API
- [Storage API](../08-rest-api-tools/storage-api.md)
