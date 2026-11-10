#![no_main]
//! Office document import (#2119).
//!
//! Uploaded XLSX/XLS/ODS/CSV files reach `import_spreadsheet_bytes` straight
//! from a user's disk (and from email attachments), i.e. at the binary
//! boundary where zip, OLE and XML parsers meet. The invariant: any byte
//! sequence either imports or returns `Err(String)` — never a panic, never an
//! allocation blow-up.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The extension steers the parser choice, so fuzz the label too: a `.xlsx`
    // label on garbage must fail in the reader, not skip validation.
    for name in ["fuzz.xlsx", "fuzz.xls", "fuzz.ods", "fuzz.csv", "fuzz"] {
        let _ = botsheet::storage::import::import_spreadsheet_bytes(data, name, "fuzz-owner");
    }
});
