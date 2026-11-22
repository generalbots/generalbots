//! What a Drive tool really did, for its step: the files it saved or
//! fetched and, when it did not do everything, a reason code. The text the
//! tool writes stays the tool result for the model.

use super::super::native_tools::send;
use super::super::types::SessionIdentity;
use super::super::vm_consent::{reason, VmOutcome};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DriveOutcome {
    pub ok: bool,
    /// Files actually saved (or fetched), never the ones only asked for.
    pub files: Vec<String>,
    pub reason: Option<&'static str>,
}

impl DriveOutcome {
    #[must_use]
    pub fn done(files: Vec<String>) -> Self {
        Self { ok: true, files, reason: None }
    }

    #[must_use]
    pub fn refused(reason: &'static str) -> Self {
        Self { ok: false, files: Vec::new(), reason: Some(reason) }
    }

    /// A save that kept `saved` and lost `failed` files.
    #[must_use]
    pub fn from_save(saved: Vec<String>, failed: usize) -> Self {
        match (saved.is_empty(), failed) {
            (true, 0) => Self::refused(reason::NOTHING_TO_SAVE),
            (true, _) => Self::refused(reason::SAVE_FAILED),
            (false, 0) => Self::done(saved),
            (false, _) => Self { reason: Some(reason::PARTLY_SAVED), ..Self::done(saved) },
        }
    }
}

impl From<DriveOutcome> for VmOutcome {
    fn from(outcome: DriveOutcome) -> Self {
        Self { ok: outcome.ok, files: outcome.files, reason: outcome.reason, ..Self::default() }
    }
}

/// Tells the model why the tool did nothing and returns that as the outcome.
pub(super) async fn refuse(identity: &SessionIdentity<'_>, text: &str, reason: &'static str) -> DriveOutcome {
    send(identity, text).await;
    DriveOutcome::refused(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_reports_only_what_it_kept() {
        let none = DriveOutcome::from_save(Vec::new(), 0);
        assert_eq!(none, DriveOutcome::refused(reason::NOTHING_TO_SAVE));
        assert!(!none.ok && none.files.is_empty());

        let lost = DriveOutcome::from_save(Vec::new(), 2);
        assert_eq!(lost.reason, Some(reason::SAVE_FAILED));
        assert!(!lost.ok);

        let all = DriveOutcome::from_save(vec!["output/a.csv".into()], 0);
        assert_eq!(all, DriveOutcome::done(vec!["output/a.csv".into()]));

        let some = DriveOutcome::from_save(vec!["output/a.csv".into()], 1);
        assert!(some.ok);
        assert_eq!(some.files, vec!["output/a.csv".to_string()]);
        assert_eq!(some.reason, Some(reason::PARTLY_SAVED));
    }

    #[test]
    fn the_step_gets_the_code_and_no_text() {
        let step: VmOutcome = DriveOutcome::refused(reason::NOTHING_TO_SAVE).into();
        assert!(!step.ok);
        assert!(step.output.is_empty() && step.files.is_empty());
        assert_eq!(step.exit_code, None);
        assert_eq!(step.reason, Some("nothing_to_save"));
    }
}
