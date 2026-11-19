# General Bots OS (laptop appliance)

The desktop edition can ship as the whole operating system: a Debian 13 image
built with mkosi where the General Bots kiosk shell is the only user interface
(`desktop/os/`). This page describes what that machine does at runtime —
networking, sound, time, removable media, power, locking, the firewall and the
sandbox — because those are the surfaces a laptop user notices first and the
ones that have nothing to do with the chat product.

## Session model

| Piece | Behaviour |
|---|---|
| Display | `cage` (Wayland) owns tty1; there is no display manager, no greeter, no X11 |
| Login user | `gbdesk` (system account, `nologin`, home `/var/lib/gbdesk`) |
| Session bus | the user manager's bus at `/run/user/<uid>/bus`; `gb-session` waits for it and falls back to a private `dbus-run-session` only if it never appears |
| User services | PipeWire, WirePlumber, the xdg-desktop-portal stack and gvfs are enabled per user through `default.target.wants` — nothing is spawned by hand from the launcher |
| Recovery | no console, no rescue shell, no ssh: recovery is reprovisioning from a fresh image |

## Networking

NetworkManager owns the interfaces (`systemd-networkd` is disabled), with
`wpasupplicant`, `wireless-regdb` and `firmware-iwlwifi` present, so a laptop
joins WPA2/WPA3 Wi-Fi from Settings → Network. DNS is `systemd-resolved`.

Inbound traffic is denied by an nftables ruleset loaded at boot. The provisioned
stack (botserver, Vault, PostgreSQL, Valkey, MinIO, Qdrant, Zitadel) listens on
loopback, and `botserver` binds loopback in shell mode even when
`no_security=false`. Exposing the suite to the LAN is a deliberate act: set
`GB_ALLOW_LAN=1` (the "Allow other devices on my network" switch, off by
default) and add the client address to the `gb_lan_allowed` nftables set.

## Sound

PipeWire with WirePlumber is the audio server, ALSA clients reach it through
`pipewire-alsa`, and the GStreamer plugin set (base/good/bad/ugly, libav,
pipewire) gives Meet, Player and Jukebox their codecs. The kiosk unit allows the
`/dev/snd` nodes explicitly, because `PrivateDevices=yes` would otherwise hide
them.

## Time, locale and keyboard

`systemd-timesyncd` keeps the clock right (a wrong clock breaks TLS and OTP).
`tzdata` plus `localectl` means the zone is real data with DST, not a fixed
offset table; `locales` and `console-setup`/`keyboard-configuration` mean the
language and keymap are settings, not a hard `C.UTF-8`/`us` lock. Settings →
Time & Language writes these files.

## Removable media

`udisks2` (with `gvfs`) is what makes an inserted USB stick visible to the file
picker, mountable, and safely ejectable; `exfatprogs`, `ntfs-3g` and
`dosfstools` cover the filesystems phones and cameras leave behind.

## Power

The start menu, the control center and the kiosk close request all open one
power menu: **Lock, Sign out, Suspend, Restart, Shut down**. Suspend, restart
and shutdown ask for confirmation and then call `POST /api/system/power`, which
is administrator + loopback only and written to the audit log; the request runs
through `/usr/local/libexec/gb-power`, a root-owned helper with a fixed action
list. Nothing from the request reaches a command line.

logind no longer powers the machine off on a key press: the image sets
`HandlePowerKey=ignore`, keeps the lid suspending, and denies the kiosk user
logind's `PowerOff`/`Reboot`/`Suspend` (and NetworkManager's system settings)
through a polkit rule.

## Locking and the media grant

The lock screen is a UI overlay; the session's real boundary is the account.
Microphone and camera are granted per origin: the WebKit permission handler
allows media only for the shell's own loopback origins (plus any host named in
`GB_MEDIA_ORIGINS`), denies everything else and logs the origin it refused.

## Web process sandbox and the IPC surface

`bubblewrap` and `libseccomp2` are pinned in the image and the session exports
`WEBKIT_FORCE_SANDBOX=1`, so a web process that cannot be sandboxed fails
visibly instead of running page content as the user that owns the database.

The Tauri file commands (`list_files`, `upload_file`, `create_folder`,
`delete_path`) resolve every path through a scope check: absolute path, no
`..`, canonicalised (so a symlink cannot escape), inside home / Downloads /
application data / the stack / `/var/lib/gbdesk`, and never one of the secret
store files (`.env`, `vault-unseal-keys`, `admin-credentials.json`). Deleting a
scope root itself is refused. An injected script can therefore no longer wipe
`/var/lib/gbdesk` with one call.

## What the image still does not do

- Secure Boot signing, TPM-sealed unseal keys and A/B updates are future work.
- The Vault token and unseal keys still live in the stack directory owned by
  `gbdesk`; separating the sidecar into a system account is the next step.
- There is no password check on the lock overlay yet.
