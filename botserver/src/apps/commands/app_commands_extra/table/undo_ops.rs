use super::super::*;

/// #2448 — chat-native "undo my last agent batch". The verb intentionally has
/// NO params: the in-process dispatcher derives user, bot and session from the
/// verified chat identity, so no prompt-supplied id can select a scope or a
/// batch. `admin_only` stays false — every user may undo only their own batch.
pub(super) static UNDO_OPS: &[AppCommand] = &[cmd(
    "docs",
    "docs.undo",
    "Undo last agent batch",
    "Undo the last agent batch you made: revert its operations from recorded before-states and restore the prior versions (scope: verified session only; confirm: no; undo: consumes the batch once).",
    &[],
    None,
    false,
)];
