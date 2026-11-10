//! Offline unit tests for the minutes form payloads (#2075).
//!
//! The minutes screens post partial payloads: the action form omits the
//! priority unless the user picked one, the meeting form omits duration and
//! location. `serde` defaults decide what the handler stores, so they are
//! pinned here — an empty string priority would sort actions wrongly.

use super::{ActionForm, AttendanceForm, DocumentForm, MeetingForm, SignForm};

#[test]
fn an_action_defaults_to_medium_priority() {
    let form: ActionForm =
        serde_json::from_str(r#"{"title":"Send deck","owner":"ana"}"#).expect("minimal action");
    assert_eq!(form.priority, "medium");
    assert_eq!(form.meeting_id, None);
    assert_eq!(form.due, None);
    assert_eq!(form.notes, None);
}

#[test]
fn an_explicit_priority_is_kept() {
    let form: ActionForm = serde_json::from_str(
        r#"{"title":"Send deck","owner":"ana","priority":"high","due":"2026-02-03"}"#,
    )
    .expect("full action");
    assert_eq!(form.priority, "high");
    assert_eq!(form.due.as_deref(), Some("2026-02-03"));
}

#[test]
fn a_meeting_requires_a_title_date_and_time() {
    assert!(serde_json::from_str::<MeetingForm>(r#"{"title":"Kickoff"}"#).is_err());

    let form: MeetingForm = serde_json::from_str(
        r#"{"title":"Kickoff","date":"2026-01-02","time":"09:30"}"#,
    )
    .expect("minimal meeting");
    assert_eq!(form.duration, None);
    assert_eq!(form.location, None);

    let form: MeetingForm = serde_json::from_str(
        r#"{"title":"Kickoff","date":"2026-01-02","time":"09:30","duration":45,"location":"Room 1"}"#,
    )
    .expect("full meeting");
    assert_eq!(form.duration, Some(45));
    assert_eq!(form.location.as_deref(), Some("Room 1"));
}

#[test]
fn a_document_defaults_to_no_content_and_no_kind() {
    let form: DocumentForm =
        serde_json::from_str(r#"{"title":"Minutes"}"#).expect("minimal document");
    assert_eq!(form.content, None);
    assert_eq!(form.kind, None);
    assert_eq!(form.meeting_id, None);

    let form: DocumentForm = serde_json::from_str(
        r#"{"title":"Minutes","meeting_id":"00000000-0000-0000-0000-000000000001","content":"x","kind":"minutes"}"#,
    )
    .expect("full document");
    assert_eq!(form.kind.as_deref(), Some("minutes"));
}

#[test]
fn signature_and_attendance_forms_require_their_subject() {
    assert!(serde_json::from_str::<SignForm>(r#"{}"#).is_err());
    let form: SignForm = serde_json::from_str(r#"{"signer":"ana"}"#).expect("signer");
    assert_eq!(form.signer, "ana");

    assert!(serde_json::from_str::<AttendanceForm>(r#"{}"#).is_err());
    let form: AttendanceForm =
        serde_json::from_str(r#"{"attendee":"ana"}"#).expect("attendee");
    assert_eq!(form.status, None);
    let form: AttendanceForm =
        serde_json::from_str(r#"{"attendee":"ana","status":"present"}"#).expect("status");
    assert_eq!(form.status.as_deref(), Some("present"));
}
