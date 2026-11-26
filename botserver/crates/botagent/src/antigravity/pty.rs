//! PTY bridge script that drives `agy` (Antigravity CLI) inside the VM.

pub(super) const PTY_SCRIPT: &str = r#"import os, pty, select, time
HOME = os.environ.get("GB_ANTIGRAVITY_HOME", "/home/antigravity")
TMP = os.environ.get("GB_ANTIGRAVITY_TMP", "/tmp")
LOG = TMP + "/gb-antigravity-login.log"
STATUS = TMP + "/gb-antigravity-login.status"
CODE = TMP + "/gb-antigravity-code"
CRED_DIRS = (HOME + "/.gemini", HOME + "/.config/antigravity")

def signed_in():
    # Same rule as the server's status check: a non-empty file whose name
    # marks a stored token or credential (MCP server tokens excluded).
    for root_dir in CRED_DIRS:
        base_depth = root_dir.rstrip("/").count("/")
        for current, dirs, files in os.walk(root_dir):
            if current.count("/") - base_depth >= 3:
                dirs[:] = []
            for name in files:
                lower = name.lower()
                if lower == "mcp_oauth_tokens.json":
                    continue
                if "token" in lower or "cred" in lower or "oauth" in lower or lower.startswith("auth"):
                    try:
                        if os.path.getsize(os.path.join(current, name)) > 0:
                            return True
                    except OSError:
                        pass
    return False

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
# agy opens with a menu ("Select login method: > 1. Google OAuth"):
# Enter picks the highlighted Google sign-in once.
seen = ""
menu_done = False
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
        seen = (seen + text)[-4000:]
        if not menu_done and "login method" in seen.lower():
            time.sleep(0.5)
            try:
                os.write(fd, b"\r")
            except OSError:
                pass
            menu_done = True
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
    if signed_in():
        set_status("linked")
        time.sleep(2)
        break
if signed_in():
    set_status("linked")
elif sent:
    set_status("error")
else:
    set_status("waiting")
"#;
