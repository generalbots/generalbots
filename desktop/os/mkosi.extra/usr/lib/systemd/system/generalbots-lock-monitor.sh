#!/bin/bash
# #2446 — logind bridge for the General Bots kiosk lock screen.
#
# Watches org.freedesktop.login1 on the system bus. Lock, suspend, lid-close
# and session changes put the kiosk session in a locked state; resume, lid-open
# and Unlock KEEP it locked. logind's Unlock signal only means "session active
# again" — it is never an authentication result. The webview lock overlay
# listens for the matching gb-lock / gb-resume Tauri events and always requires
# the local account credential before it can be dismissed.
set -u

if ! command -v gdbus >/dev/null 2>&1 || ! command -v loginctl >/dev/null 2>&1; then
  logger -t gb-lock-monitor -- "gdbus/loginctl unavailable; logind bridge disabled"
  exit 0
fi

KIOSK_UID=$(getent passwd gbdesk 2>/dev/null | cut -d: -f3)
if [ -z "${KIOSK_UID}" ]; then
  logger -t gb-lock-monitor -- "gbdesk user not found; logind bridge disabled"
  exit 0
fi
STATE_DIR="/run/user/${KIOSK_UID}/generalbots-lock"
STATE_FILE="${STATE_DIR}/state"
mkdir -p "${STATE_DIR}" 2>/dev/null || true
printf '%s\n' unlocked > "${STATE_FILE}" 2>/dev/null || true

log() { logger -t gb-lock-monitor -- "$1" 2>/dev/null || true; }
state() { cat "${STATE_FILE}" 2>/dev/null || printf '%s\n' unlocked; }
write_state() { printf '%s\n' "$1" > "${STATE_FILE}" 2>/dev/null || true; }

enter_lock() {
  # loginctl lock-sessions itself emits a Lock signal. Only act on the state
  # transition so the monitor cannot feed its own signal into a loop.
  [ "$(state)" = "locked" ] && return 0
  write_state locked
  log "logind lock requested ($1)"
  loginctl lock-sessions >/dev/null 2>&1 || true
}

resume_locked() {
  write_state resume-locked
  log "resume/session change ($1): session remains locked"
  loginctl lock-sessions >/dev/null 2>&1 || true
}

log "watching org.freedesktop.login1 for lock/resume events"
# Typical lines:
#   :org.freedesktop.login1.Session.Lock ()
#   :org.freedesktop.login1.Manager.PrepareForSleep (true,)
#   :org.freedesktop.login1.Manager.PrepareForSleep (false,)
gdbus monitor --system --dest org.freedesktop.login1 2>/dev/null |
while IFS= read -r line; do
  case "${line}" in
    *".Lock ()"*|*".PrepareForSleep (true"*|*".PrepareForShutdown (true"*|*".SessionNew"*)
      enter_lock "${line}"
      ;;
    *".Unlock ()"*|*".PrepareForSleep (false"*|*".SessionRemoved"*)
      resume_locked "${line}"
      ;;
  esac
done
