//! PTY bridge script that drives `codex login --device-auth` inside the VM.

pub(super) const PTY_SCRIPT: &str = r#"import os, pty, select, time
HOME = os.environ.get("GB_CODEX_HOME", "/home/codex")
TMP = os.environ.get("GB_CODEX_TMP", "/tmp")
LOG = TMP + "/gb-codex-login.log"
STATUS = TMP + "/gb-codex-login.status"
CODE = TMP + "/gb-codex-code"
CRED = HOME + "/.codex/auth.json"

def set_status(value):
    try:
        with open(STATUS, "w") as handle:
            handle.write(value)
    except Exception:
        pass

os.environ["HOME"] = HOME
os.environ["CODEX_HOME"] = HOME + "/.codex"
try:
    os.chdir(HOME)
except Exception:
    pass
for path in (LOG, CODE):
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass
    except Exception:
        pass
set_status("starting")
pid, fd = pty.fork()
if pid == 0:
    # Device code flow: works in headless/VM environments.
    # Prints a URL and a one-time code; user authorizes in browser.
    for attempt in range(30):
        try:
            os.execvp("codex", ["codex", "login", "--device-auth"])
        except OSError as error:
            if getattr(error, "errno", None) == 26 and attempt < 29:
                time.sleep(0.5)
                continue
            raise
set_status("waiting")
log = open(LOG, "wb", 0)
deadline = time.time() + 1800
while time.time() < deadline:
    ready, _, _ = select.select([fd], [], [], 0.5)
    if ready:
        try:
            data = os.read(fd, 4096)
        except OSError:
            break
        if not data:
            break
        log.write(data)
        log.flush()
        text = data.decode("utf-8", "replace")
        if "Invalid" in text or "error" in text.lower() or "failed" in text.lower() or "expired" in text.lower():
            set_status("error")
    # Device flow auto-completes: just check for credential file
    if os.path.exists(CRED):
        set_status("linked")
        time.sleep(2)
        break
    # Also check if user pasted a code (fallback for non-device flows)
    if os.path.exists(CODE):
        code = ""
        try:
            with open(CODE, "r") as handle:
                code = handle.read().strip()
        except Exception:
            code = ""
        if code:
            try:
                os.write(fd, (code + "\r").encode())
            except OSError:
                pass
            set_status("verifying")
            try:
                os.unlink(CODE)
            except OSError:
                pass
    if os.path.exists(CRED):
        set_status("linked")
        time.sleep(2)
        break
if os.path.exists(CRED):
    set_status("linked")
else:
    phase = "waiting"
    try:
        with open(STATUS, "r") as handle:
            phase = handle.read().strip()
    except Exception:
        pass
    if phase not in ("linked", "error"):
        set_status("waiting")
"#;
