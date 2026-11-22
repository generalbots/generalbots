//! Test-rig stand-in for the virtual machine.
//!
//! The test rig has no virtual machine. With `GB_RIG_FAKE_VM=1`, which only
//! the rig's start command sets (the product never does), an allowed action
//! "runs" here: nothing executes and a fixed transcript comes back, so the
//! consent card, the steps block and the result path can be checked end to
//! end. The consent ticket is still required: this sits below the gate.

const ENV: &str = "GB_RIG_FAKE_VM";

pub(crate) fn enabled() -> bool {
    let on = std::env::var(ENV).is_ok_and(|value| value == "1");
    if on {
        botlog::safe_warn!("{ENV}=1: virtual machine actions are simulated (test rig)");
    }
    on
}

pub(crate) fn run(language: &str, code: &str) -> (bool, String, String) {
    let lines = code.lines().count();
    let stdout = format!(
        "[rig] {language}: {lines} line(s), {} byte(s) accepted\n[rig] nothing was executed\n[rig] exit 0\n",
        code.len()
    );
    (true, stdout, String::new())
}
