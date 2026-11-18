use super::*;

/// #2035/#2039/#2040 — office, personal, media and system verbs. Kept apart
/// from the core table so each file stays under the size limit; the joined
/// slice is built once in `all_commands()`.
mod table;

/// #2035/#2039/#2040 — office, personal, media and system verbs. Kept apart
/// from the core table so each file stays under the size limit; the joined
/// slice is built once in `all_commands()`.
pub fn all_commands_extra() -> &'static [AppCommand] {
    table::all()
}
