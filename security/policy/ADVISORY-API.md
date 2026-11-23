# Advisory API contract (read-only, anonymous)

`GET /api/advisories` filters: `product`, `component`, `version`, `severity`,
`from`, `to`. `GET /api/advisories/{id}` returns one canonical record. The
record schema is identical to the CSAF/OSV documents under `security/`, so the
feed and the API share one source of truth.

Headers: `Cache-Control: public, max-age=300`, standard security headers,
anonymous access, 60 requests/minute per IP. No personal data is logged.

CLI (offline-capable):

```
python3 scripts/security/gb_audit.py generalbots 6.8.0 --bundle security/fixtures/advisories.json
python3 scripts/security/gb_audit.py generalbots 6.8.0 --api https://generalbots.org/api/advisories
```

Exit 0 = not affected, exit 1 = affected, exit 2 = usage/transport error. The
`--bundle` mode reads the same records from an air-gapped directory, so the
command works with no network.

PT-BR: `python3 scripts/security/gb_audit.py generalbots 6.8.0 --bundle ...`
imprime `AFETADO` e sai com 1 quando a versão está na faixa afetada.
