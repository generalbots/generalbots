# incus-wsl

`incus.exe` for the agent workspace on Windows. The server runs one program
for Incus (`AGENT_INCUS_BIN`); this one forwards every argument, unchanged and
without a shell, to Incus inside a WSL2 distro:

```
wsl.exe -d <distro> -u root --exec incus <args...>
```

- Distro: `GBO_WSL_DISTRO`, default `GeneralBotsLite` (the product's own
  distro). Names outside `[A-Za-z0-9._-]`, longer than 64 characters or
  starting with `-` are refused with exit code 2.
- `__gb-service state` and `__gb-service restart` run the fixed commands
  `systemctl show incus.service --property=ActiveState,SubState,MainPID` and
  `systemctl restart incus.service`; any other action exits with code 2.
- Standard input, output and error pass through and the exit code is kept;
  127 means `wsl.exe` could not be started. No console window is opened.
- The file must be named `incus.exe`: the server's command guard allows this
  program name only.

It is a standalone crate, excluded from the root workspace:

```
cargo test --manifest-path desktop/windows/incus-wsl/Cargo.toml
cargo build --release --manifest-path desktop/windows/incus-wsl/Cargo.toml
```

The desktop installer carries the release build as `installers\incus.exe`.
When that file exists, the desktop shell points the server at it and at the
product distro, unless `AGENT_INCUS_BIN` is already set in the environment.
