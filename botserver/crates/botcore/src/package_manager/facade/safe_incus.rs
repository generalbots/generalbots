use super::*;

pub(super) fn safe_incus(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("incus")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub(super) fn safe_incus_exec_in_container(container: &str, command: &str) -> Option<std::process::Output> {
    SafeCommand::new("incus")
        .and_then(|c| c.args(&["exec", container, "--", "bash", "-c", command]))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub(super) fn safe_incus_admin(args: &[&str]) -> Option<std::process::Output> {
    let mut all = vec!["admin"];
    all.extend_from_slice(args);
    safe_incus(&all)
}

pub(super) fn safe_apt_get(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("apt-get")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub(super) fn safe_brew(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("brew")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}
