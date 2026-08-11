//! Drive object seeding for the fiscal test scenarios (issues #722/#723/#724).
//!
//! Writes real MinIO objects into the default bot's bucket so the chat/API
//! flows can discover them exactly like a production user's Drive:
//!
//!   * `faturas/`  — invoice folder used by the guided upload flow (#723)
//!   * `financeiro/` — cash-flow CSV files used by the banking import (#724)
//!
//! All inserts are guarded by `object_exists`, so re-running is safe.

use botlib::traits::DriveRepository;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;

fn resolve_default_bot_name(conn: &mut diesel::PgConnection) -> Result<Option<String>, String> {
    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = Text)]
        name: String,
    }
    let row: Option<BotRow> = sql_query(
        "SELECT name FROM bots WHERE is_default_for_branch = true ORDER BY created_at ASC LIMIT 1",
    )
    .get_result(conn)
    .ok();
    Ok(row.map(|r| r.name))
}

/// Builds a cash-flow CSV with entries for the given month (so diagnosis
/// filters by the current month prefix correctly). Uses the real-world
/// Brazilian column names (data/historico/valor/tipo) that the import
/// parsers accept.
fn build_cashflow_csv(year: u32, month_idx: u32) -> String {
    let last_day = if month_idx == 2 { 28 } else { 30 };
    let d = |day: u32| format!("{year:04}-{month_idx:02}-{day:02}");
    format!(
        "data,historico,valor,tipo\n\
         {},\"Client contract\",5000.00,receita\n\
         {},\"Consulting hours\",2500.00,receita\n\
         {},\"Cloud subscription\",-1200.00,despesa\n\
         {},\"Office rent\",-1800.00,despesa\n\
         {},\"Telecom\",-{last_day}.00,despesa\n\
         {},\"Marketing\",-600.00,despesa\n",
        d(2),
        d(5),
        d(8),
        d(12),
        d(15),
        d(20),
    )
}

async fn put_if_missing(
    drive: &dyn DriveRepository,
    bucket: &str,
    key: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    if drive.object_exists(bucket, key).await.unwrap_or(false) {
        return Ok(());
    }
    drive
        .put_object(bucket, key, body.to_vec(), Some(content_type))
        .await
        .map_err(|e| format!("put {key}: {e}"))
}

/// Seeds the invoice folder and cash-flow spreadsheets of the default bot.
pub async fn seed_drive_objects(
    pool: &botcore::shared::utils::DbPool,
    drive: &dyn DriveRepository,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
    let bot_name = resolve_default_bot_name(&mut conn)?.ok_or("No default bot found")?;
    drop(conn);

    let bucket = format!("{bot_name}.gbai");
    let prefix = format!("{bot_name}.gbdrive/");

    drive
        .create_bucket_if_not_exists(&bucket)
        .await
        .map_err(|e| format!("create bucket {bucket}: {e}"))?;

    let now = chrono::Utc::now();
    let current_month = now.format("%Y-%m").to_string();
    let prev = now
        .checked_sub_months(chrono::Months::new(1))
        .unwrap_or(now);
    let prev_month = prev.format("%Y-%m").to_string();
    let year = now.format("%Y").to_string().parse::<u32>().unwrap_or(2026);
    let month_idx = now.format("%m").to_string().parse::<u32>().unwrap_or(8);
    let prev_year = prev.format("%Y").to_string().parse::<u32>().unwrap_or(2026);
    let prev_month_idx = prev.format("%m").to_string().parse::<u32>().unwrap_or(7);

    let faturas_marker = format!("{prefix}faturas/.keep");
    if !drive.object_exists(&bucket, &faturas_marker).await.unwrap_or(false) {
        drive
            .put_object(&bucket, &faturas_marker, Vec::new(), Some("text/plain"))
            .await
            .map_err(|e| format!("create faturas folder: {e}"))?;
    }

    let invoice_key = format!("{prefix}faturas/telefonia-{current_month}.pdf");
    if !drive.object_exists(&bucket, &invoice_key).await.unwrap_or(false) {
        drive
            .put_object(
                &bucket,
                &invoice_key,
                format!("%PDF-1.4 sample invoice {current_month}").into_bytes(),
                Some("application/pdf"),
            )
            .await
            .map_err(|e| format!("seed invoice: {e}"))?;
    }

    let financeiro_marker = format!("{prefix}financeiro/.keep");
    if !drive.object_exists(&bucket, &financeiro_marker).await.unwrap_or(false) {
        drive
            .put_object(&bucket, &financeiro_marker, Vec::new(), Some("text/plain"))
            .await
            .map_err(|e| format!("create financeiro folder: {e}"))?;
    }

    let finance_key = format!("{prefix}financeiro/fluxo-caixa-{current_month}.csv");
    if !drive.object_exists(&bucket, &finance_key).await.unwrap_or(false) {
        drive
            .put_object(
                &bucket,
                &finance_key,
                build_cashflow_csv(year, month_idx).into_bytes(),
                Some("text/csv"),
            )
            .await
            .map_err(|e| format!("seed current cashflow: {e}"))?;
    }

    let prev_key = format!("{prefix}financeiro/fluxo-caixa-{prev_month}.csv");
    if !drive.object_exists(&bucket, &prev_key).await.unwrap_or(false) {
        drive
            .put_object(
                &bucket,
                &prev_key,
                build_cashflow_csv(prev_year, prev_month_idx).into_bytes(),
                Some("text/csv"),
            )
            .await
            .map_err(|e| format!("seed previous cashflow: {e}"))?;
    }

    Ok(())
}
