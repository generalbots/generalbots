//! Row shapes and stage weights shared by the CRM chat commands.

use uuid::Uuid;

#[derive(diesel::QueryableByName)]
pub(super) struct StageRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    stage: Option<String>,
}
/// Stage → win probability, mirroring `botcontacts::models::stage_probability`
/// so chat and the suite agree on the weights behind the forecast.
pub(super) fn stage_probability(stage: &str) -> i32 {
    match stage {
        "new" => 10,
        "qualified" | "qualification" => 25,
        "proposal" => 50,
        "negotiation" => 75,
        "won" | "converted" => 100,
        "lost" => 0,
        _ => 25,
    }
}

#[derive(diesel::QueryableByName)]
pub(super) struct ContactIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    contact_id: Uuid,
}

#[derive(diesel::QueryableByName)]
pub(super) struct ForecastDealRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    stage: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    value: Option<f64>,
}

#[derive(diesel::QueryableByName)]
pub(super) struct StageRollupRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    stage: Option<String>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    value: Option<f64>,
}
