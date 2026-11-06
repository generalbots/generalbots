//! PTY bridge script that drives `claude setup-token` inside the VM.

/// The PTY bridge: runs `claude setup-token` as the `claude` user, mirrors its
/// output to a log, and forwards an authorization code dropped in
/// `/tmp/gb-claude-code`. Credentials stay in `/home/claude/.claude`.
pub(super) const PTY_SCRIPT: &str = r#"import os, pty, select, time
HOME = os.environ.get("GB_CLAUDE_HOME", "/home/claude")
TMP = os.environ.get("GB_CLAUDE_TMP", "/tmp")
LOG = TMP + "/gb-claude-login.log"
STATUS = TMP + "/gb-claude-login.status"
CODE = TMP + "/gb-claude-code"
CRED = HOME + "/.claude/.credentials.json"

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
    # npm can still be flushing the freshly installed binary when the PTY
    # starts; retry the transient "Text file busy" instead of losing the login.
    for attempt in range(30):
        try:
            os.execvp("claude", ["claude", "setup-token"])
        except OSError as error:
            if getattr(error, "errno", None) == 26 and attempt < 29:
                time.sleep(0.5)
                continue
            raise
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
        if "Invalid code" in text or "OAuth error" in text:
            # The CLI rejected a partial/incorrect paste. Return to the code
            # prompt instead of leaving the status stuck on "verifying".
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
    if os.path.exists(CRED):
        set_status("linked")
        time.sleep(2)
        break
if os.path.exists(CRED):
    set_status("linked")
elif sent:
    set_status("error")
else:
    set_status("waiting")
"#;

#[cfg(test)]
mod tests {
    /// Exercises the real PTY bridge with a fake `claude` binary: it prints an
    /// OAuth URL, waits for the code, and writes the credentials file. This is
    /// the same code path the VM runs, without a Claude subscription.
    #[cfg(unix)]
    #[test]
    fn the_pty_bridge_reaches_linked_without_a_real_subscription() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!("gb-claude-test-{}", uuid::Uuid::new_v4()));
        let home = root.join("home");
        let bin = root.join("bin");
        std::fs::create_dir_all(home.join(".claude")).expect("home");
        std::fs::create_dir_all(&bin).expect("bin");
        let fake = bin.join("claude");
        std::fs::write(
            &fake,
            "#!/bin/sh\n\
             echo 'https://claude.com/cai/oauth/authorize?code=true&client_id=test&state=abc'\n\
             printf 'Paste code here if prompted >'\n\
             read code\n\
             mkdir -p \"$HOME/.claude\"\n\
             printf '{\"token\":\"%s\"}' \"$code\" > \"$HOME/.claude/.credentials.json\"\n",
        )
        .expect("fake claude");
        let mut permissions = std::fs::metadata(&fake).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&fake, permissions).expect("chmod");

        let script = root.join("bridge.py");
        std::fs::write(&script, super::PTY_SCRIPT).expect("script");
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut child = std::process::Command::new("python3")
            .arg(&script)
            .env("HOME", &home)
            .env("GB_CLAUDE_HOME", &home)
            .env("GB_CLAUDE_TMP", &root)
            .env("PATH", path)
            .spawn()
            .expect("spawn bridge");

        let log_path = root.join("gb-claude-login.log");
        let mut saw_url = false;
        for _ in 0..100 {
            if let Ok(log) = std::fs::read_to_string(&log_path) {
                if log.contains("https://claude.com/cai/oauth/authorize?") {
                    saw_url = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(saw_url, "the bridge must expose the OAuth URL");

        let mut code = std::fs::File::create(root.join("gb-claude-code")).expect("code file");
        code.write_all(b"test-code\n").expect("write code");
        drop(code);

        let credential = home.join(".claude/.credentials.json");
        let mut linked = false;
        for _ in 0..100 {
            if credential.exists() {
                linked = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&root);
        assert!(linked, "the bridge must write the credential after the code");
    }
}
