//! PTY bridge script that drives `agy` (Antigravity CLI) inside the VM.

pub(super) const PTY_SCRIPT: &str = r#"import os, pty, select, time
HOME = os.environ.get("GB_ANTIGRAVITY_HOME", "/home/antigravity")
TMP = os.environ.get("GB_ANTIGRAVITY_TMP", "/tmp")
LOG = TMP + "/gb-antigravity-login.log"
STATUS = TMP + "/gb-antigravity-login.status"
CODE = TMP + "/gb-antigravity-code"
CRED = HOME + "/.gemini/antigravity-cli/settings.json"

def set_status(value):
    try:
        with open(STATUS, "w") as handle:
            handle.write(value)
    except Exception:
        pass

os.environ["HOME"] = HOME
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
    # Antigravity CLI: in SSH/remote mode, auto-detects and prints URL + code.
    # The CLI reads from stdin for the authorization code.
    for cmd in [["agy"], ["antigravity"]]:
        try:
            os.execvp(cmd[0], cmd)
        except FileNotFoundError:
            continue
        except OSError as error:
            if getattr(error, "errno", None) == 26:
                time.sleep(0.5)
                try:
                    os.execvp(cmd[0], cmd)
                except Exception:
                    pass
            raise
    raise FileNotFoundError("antigravity login command not found")
set_status("waiting")
log = open(LOG, "wb", 0)
sent = False
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
            sent = False
            set_status("waiting")
            try:
                os.write(fd, b"\r")
            except OSError:
                pass
    if not sent and os.path.exists(CODE):
        code = ""
        try:
            with open(CODE, "r") as handle:
                code = handle.read().strip()
        except Exception:
            code = ""
        if code:
            os.write(fd, (code + "\r").encode())
            sent = True
            set_status("verifying")
            try:
                os.unlink(CODE)
            except OSError:
                pass
    # Check for credential file (settings.json with auth info)
    if os.path.exists(CRED):
        try:
            import json
            with open(CRED, "r") as handle:
                data = json.load(handle)
            if data.get("auth") or data.get("credentials") or data.get("token"):
                set_status("linked")
                time.sleep(2)
                break
        except Exception:
            pass
if os.path.exists(CRED):
    set_status("linked")
elif sent:
    set_status("error")
else:
    set_status("waiting")
"#;
