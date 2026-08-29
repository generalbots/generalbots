use axum::routing::{get, post, put};
use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

use crate::{cashflow, handlers, accounts_pix};

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/banking/transactions", get(handlers::list_transactions).post(handlers::create_transaction))
        .route("/api/banking/platforms", get(handlers::list_platforms))
        .route("/api/banking/reconcile", post(handlers::reconcile))
        .route("/api/banking/reports", get(handlers::get_report))
        .route("/api/banking/reconcile/pairs", get(handlers::list_reconcile_pairs))
        .route("/api/banking/reconcile/match", post(handlers::manual_match))
        .route("/api/banking/platforms/:id/sync", put(handlers::sync_platform))
        .route("/api/banking/imports/cashflow", post(cashflow::import_cashflow))
        .route("/api/banking/diagnosis", get(cashflow::cashflow_diagnosis))
        .route("/api/banking/accounts", get(accounts_pix::list_accounts).post(accounts_pix::create_account))
        .route("/api/banking/accounts/sync", post(accounts_pix::sync_all_accounts))
        .route("/api/banking/accounts/:id/sync", put(accounts_pix::sync_account))
        .route("/api/banking/pix", get(accounts_pix::list_pix))
        .route("/api/banking/pix/transfer", post(accounts_pix::pix_transfer))
        .route("/api/banking/pix/receive", post(accounts_pix::pix_receive))
        .route("/api/banking/pix/export", get(accounts_pix::export_pix_history))
        .route("/api/banking/statements", get(accounts_pix::list_statements).post(accounts_pix::create_statement))
        .route("/api/banking/statements/:id/download", get(accounts_pix::download_statement))
        .route("/api/banking/settings", get(accounts_pix::get_settings).put(accounts_pix::save_settings))
        .route("/api/banking/settings/reset", post(accounts_pix::reset_settings))
        .route("/api/banking/stats", get(accounts_pix::stats))
        .route("/api/banking/reports/generate", post(accounts_pix::generate_report))
}
