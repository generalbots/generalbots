//! Find-and-replace request.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReplaceRequest {
    // The toolbar posts the workbook as `id`; the API calls it sheet_id.
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    pub find: String,
    #[serde(default)]
    pub replace: String,
    #[serde(default)]
    pub match_case: bool,
}
