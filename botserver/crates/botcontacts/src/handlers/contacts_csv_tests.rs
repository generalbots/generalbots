//! Offline tests for the contacts CSV import parser (issue #2075, CRM wave).

use super::*;

#[test]
fn contacts_columns_are_matched_case_insensitively_and_trimmed() {
    let csv = "First_Name,LAST_NAME,Email,Company,Job_Title,Status,Source,City,Country\n\
               Ana,  Silva  ,ana@example.test,Acme,CTO,lead,import,São Paulo,BR\n";

    let rows = parse_contacts_csv(csv).expect("csv parses");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].first_name.as_deref(), Some("Ana"));
    assert_eq!(rows[0].last_name.as_deref(), Some("Silva"));
    assert_eq!(rows[0].email.as_deref(), Some("ana@example.test"));
    assert_eq!(rows[0].company.as_deref(), Some("Acme"));
    assert_eq!(rows[0].job_title.as_deref(), Some("CTO"));
    assert_eq!(rows[0].status.as_deref(), Some("lead"));
    assert_eq!(rows[0].source.as_deref(), Some("import"));
    assert_eq!(rows[0].city.as_deref(), Some("São Paulo"));
    assert_eq!(rows[0].country.as_deref(), Some("BR"));
}

#[test]
fn blank_and_missing_columns_become_none() {
    let csv = "first_name,email,phone\nAna,,\n";

    let rows = parse_contacts_csv(csv).expect("csv parses");
    assert_eq!(rows[0].first_name.as_deref(), Some("Ana"));
    assert!(rows[0].email.is_none());
    assert!(rows[0].phone.is_none());
    assert!(rows[0].mobile.is_none());
    assert!(rows[0].company.is_none());
}

#[test]
fn unknown_columns_are_ignored_and_columns_may_be_reordered() {
    let csv = "country,unknown,email\nBR,x,ana@example.test\n";

    let rows = parse_contacts_csv(csv).expect("csv parses");
    assert_eq!(rows[0].country.as_deref(), Some("BR"));
    assert_eq!(rows[0].email.as_deref(), Some("ana@example.test"));
    assert!(rows[0].first_name.is_none());
}

#[test]
fn several_rows_keep_their_order() {
    let csv = "first_name,email\na,a@example.test\nb,b@example.test\nc,c@example.test\n";

    let rows = parse_contacts_csv(csv).expect("csv parses");
    let names: Vec<&str> = rows
        .iter()
        .map(|r| r.first_name.as_deref().unwrap_or(""))
        .collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn an_empty_body_imports_nothing() {
    assert!(parse_contacts_csv("").expect("empty body").is_empty());
    assert!(parse_contacts_csv("first_name\n").expect("header only").is_empty());
}

#[test]
fn a_body_shorter_than_its_header_does_not_panic() {
    // `flexible(true)` lets ragged spreadsheets through; missing trailing cells
    // must simply be absent instead of shifting values into the wrong column.
    let csv = "first_name,email,phone\nAna,ana@example.test\n";

    let rows = parse_contacts_csv(csv).expect("csv parses");
    assert_eq!(rows[0].first_name.as_deref(), Some("Ana"));
    assert_eq!(rows[0].email.as_deref(), Some("ana@example.test"));
    assert!(rows[0].phone.is_none());
}
