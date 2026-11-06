use super::super::*;

pub async fn handle_search_history<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<SearchHistoryQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct HistoryRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        pub query: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        pub n: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        pub created_at: chrono::DateTime<Utc>,
    }

    let conn = state.db_pool().clone();
    let branch_id = botsecurity_core::tenant::branch_from_claims(&headers).unwrap_or_else(uuid::Uuid::nil);
    let history: Vec<SearchHistoryEntry> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT query, COUNT(*) AS n, MAX(created_at) AS created_at FROM research_searches \
             WHERE query <> '' AND branch_id = $1 GROUP BY query ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::BigInt, _>(per_page as i64)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .load::<HistoryRow>(&mut db_conn)
        .map(|rows| {
            rows.into_iter()
                .enumerate()
                .map(|(i, r)| SearchHistoryEntry {
                    id: format!("{i}"),
                    query: r.query,
                    results_count: r.n as usize,
                    timestamp: r.created_at,
                })
                .collect()
        })
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"search-history\">");

    if history.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No search history yet</p>");
        html.push_str("</div>");
    } else {
        for entry in &history {
            html.push_str("<div class=\"history-item\" data-id=\"");
            html.push_str(&html_escape(&entry.id));
            html.push_str("\">");
            html.push_str("<span class=\"history-query\">");
            html.push_str(&html_escape(&entry.query));
            html.push_str("</span>");
            html.push_str("<span class=\"history-count\">");
            html.push_str(&entry.results_count.to_string());
            html.push_str(" results</span>");
            html.push_str("</div>");
        }
    }

    html.push_str("</div>");
    Html(html)
}
