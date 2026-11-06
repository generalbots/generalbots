# General Bots desktop - macOS status

**Status: not supported.** The packaged desktop edition (Chat + Drive) is
built and published for Linux x86_64 and, on a best-effort basis, Windows x64.
No macOS artifact is produced:

- `botapp/tauri.desktop-local.conf.json` has no `dmg` bundle target;
- `.github/workflows/desktop-binaries.yml` has no macOS job.

The download page and the first-run wizard must not offer a macOS download or
imply that a macOS desktop build exists.

## Prerequisite check

```bash
./PROD-DEPENDENCIES.sh --check     # reports the state; exits 2 on macOS
./PROD-DEPENDENCIES.sh --install   # prints guidance, installs nothing
```

The check reports the macOS version, whether the `dmg` target and the CI job
exist, and that the agent backend (Incus via WSL2) is unavailable. It never
downloads or installs anything. On Linux the same entry point delegates to
`DEPENDENCIES.sh --check`; on Windows use `PROD-DEPENDENCIES.ps1 -Check`.

## Alternatives

- Use the Linux x86_64 or Windows x64 desktop bundle.
- For development on macOS, build and run `botserver`/`botui` from source. That
  is a source-tree development setup, not the packaged desktop edition, and it
  is not covered by the desktop packaging tests.

The platform prerequisite matrix and exit codes are documented in
[`botbook/src/07-user-interface/first-run-wizard.md`](../../botbook/src/07-user-interface/first-run-wizard.md).
