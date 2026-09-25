//! `vm_incus::health` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Health-check probe pushed into the dev container and run with `node`.
/// Exits 0 as soon as something listens on 127.0.0.1:3000, otherwise after
/// `attempts` tries with a 1s pause each. Kept as a FILE (not a `bash -c`
/// string) because the command guard rejects shell metacharacters in
/// arguments: `;`, `$`, `>` and `&` would make an inline probe fail to
/// spawn and wrongly classify every app as "not listening".
pub(crate) const HEALTH_PROBE_JS: &str = r#"'use strict';
const net = require('net');
const attempts = Number(process.argv[2] || 20);
let tried = 0;
function probe() {
  if (tried >= attempts) { process.exit(1); }
  tried += 1;
  const sock = net.connect(3000, '127.0.0.1');
  sock.on('connect', () => { sock.destroy(); process.exit(0); });
  sock.on('error', () => { sock.destroy(); setTimeout(probe, 1000); });
}
probe();
"#;

pub(crate) fn checked_run(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
    timeout: u64,
) -> Result<RunOutput, GuardError> {
    let output = run(program, args, cwd, timeout)?;
    if output.exit_code == Some(0) {
        return Ok(output);
    }
    let detail = output
        .stderr
        .lines()
        .chain(output.stdout.lines())
        .find(|line| !line.trim().is_empty())
        .unwrap_or("command failed")
        .chars()
        .take(300)
        .collect::<String>();
    Err(GuardError::Io(format!(
        "{program} exited with {:?}: {detail}",
        output.exit_code
    )))
}
