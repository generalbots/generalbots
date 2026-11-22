//! Why an allowed action ended the way it did, sent as `reason` on its step.
//! The shell shows each code in the person's language; the server never puts
//! its own wording on a step (that wording is the tool result for the model).

/// `drive_save` found nothing to save (empty `output/`, or none of the
/// named files there).
pub const NOTHING_TO_SAVE: &str = "nothing_to_save";
/// `drive_save` had files but Drive kept none of them.
pub const SAVE_FAILED: &str = "save_failed";
/// `drive_save` kept some files; the others failed.
pub const PARTLY_SAVED: &str = "partly_saved";
/// Drive is not configured or not reachable for this bot.
pub const DRIVE_UNAVAILABLE: &str = "drive_unavailable";
/// `drive_fetch` did not find the file in Drive.
pub const NOT_FOUND: &str = "not_found";
/// `drive_fetch` found the file but could not bring it into the workspace.
pub const FETCH_FAILED: &str = "fetch_failed";
/// `drive_fetch`: the file is over the transfer limit.
pub const TOO_LARGE: &str = "too_large";
/// The Agent (Virtual Machine) switch is off for this conversation.
pub const VM_OFF: &str = "vm_off";
/// The workspace did not answer (not started, not reachable).
pub const WORKSPACE_UNAVAILABLE: &str = "workspace_unavailable";
/// The run used up its time budget.
pub const BUDGET_EXCEEDED: &str = "budget_exceeded";
/// A background job started; its result is saved to the conversation later.
pub const BACKGROUND_STARTED: &str = "background_started";
/// `save_to_external_drive`: the external account is not connected, or its
/// sign-in no longer works.
pub const EXTERNAL_NOT_CONNECTED: &str = "external_not_connected";
/// `save_to_external_drive`: the external account only allows reading.
pub const EXTERNAL_READ_ONLY: &str = "external_read_only";
