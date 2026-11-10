//! Legal, compliance, monitoring, scripting and people router merges.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_governance(
    app_state: &Arc<AppState>,
    sub_router: &mut Router<()>,
) {
    #[cfg(feature = "legal")]
    {
        let legal_pool = app_state.conn.clone();
        *sub_router = sub_router
            .clone()
            .merge(crate::legal::configure_legal_routes().with_state(Arc::new(legal_pool.clone())));
        *sub_router = sub_router
            .clone()
            .merge(crate::legal::configure_legal_ui_routes().with_state(Arc::new(legal_pool)));
    }

    #[cfg(feature = "compliance")]
    {
        let compliance_pool = app_state.conn.clone();
        *sub_router = sub_router.clone().merge(
            crate::compliance::configure_compliance_routes()
                .with_state(Arc::new(compliance_pool.clone())),
        );
        *sub_router = sub_router.clone().merge(
            crate::compliance::ui::configure_compliance_ui_routes()
                .with_state(Arc::new(compliance_pool.clone())),
        );
        *sub_router = sub_router.clone().merge(
            crate::compliance::dashboard::configure_dashboard_routes()
                .with_state(Arc::new(compliance_pool)),
        );
    }

    #[cfg(feature = "monitoring")]
    {
        struct MonitoringAppState {
            app_state: Arc<AppState>,
            collector: Arc<crate::monitoring::MetricsCollector>,
        }

        impl crate::monitoring::MonitoringState for MonitoringAppState {
            fn active_session_count(&self) -> usize {
                use diesel::prelude::*;
                let mut conn = match self.app_state.conn.get() {
                    Ok(c) => c,
                    Err(_) => return 0,
                };

                #[derive(diesel::QueryableByName)]
                struct CountResult {
                    #[diesel(sql_type = diesel::sql_types::BigInt)]
                    count: i64,
                }

                diesel::sql_query(
                    "SELECT COUNT(*) as count FROM user_sessions WHERE expires_at IS NULL OR expires_at > NOW()",
                )
                .get_result::<CountResult>(&mut conn)
                .map(|r| r.count.max(0) as usize)
                .unwrap_or(0)
            }

            fn is_db_healthy(&self) -> bool {
                use diesel::prelude::*;
                let mut conn = match self.app_state.conn.get() {
                    Ok(c) => c,
                    Err(_) => return false,
                };
                diesel::sql_query("SELECT 1").execute(&mut conn).is_ok()
            }

            fn metrics_collector(&self) -> Arc<crate::monitoring::MetricsCollector> {
                self.collector.clone()
            }

            fn dependencies(&self) -> Vec<crate::monitoring::DependencyStatus> {
                use std::time::Instant;
                let mut deps = Vec::new();

                // PostgreSQL
                {
                    let start = Instant::now();
                    let healthy = self.is_db_healthy();
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "PostgreSQL",
                        host: "postgres".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                // Cache / Valkey
                if let Some(cache) = self.app_state.cache.as_ref() {
                    let start = Instant::now();
                    let healthy = cache
                        .get_connection_with_timeout(std::time::Duration::from_millis(750))
                        .and_then(|mut conn| redis::cmd("PING").query::<String>(&mut conn))
                        .map(|reply| reply.to_uppercase() == "PONG")
                        .unwrap_or(false);
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "Valkey",
                        host: "cache".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                // MinIO / Drive
                if let Some(drive) = self.app_state.drive.as_ref() {
                    let start = Instant::now();
                    let healthy = tokio::task::block_in_place(|| {
                        futures::executor::block_on(drive.list_all_buckets())
                            .map(|buckets| !buckets.is_empty())
                            .unwrap_or(false)
                    });
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "MinIO",
                        host: "drive".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                // LLM
                {
                    let url = std::env::var("LLM_URL")
                        .or_else(|_| std::env::var("OLLAMA_HOST"))
                        .ok();
                    let start = Instant::now();
                    let healthy = url
                        .and_then(|u| crate::monitoring::host_port_from_url(&u, 8081))
                        .and_then(|(host, port)| {
                            use std::net::{SocketAddr, TcpStream};
                            let addr: SocketAddr = format!("{host}:{port}").parse().ok()?;
                            Some(
                                TcpStream::connect_timeout(
                                    &addr,
                                    std::time::Duration::from_millis(750),
                                )
                                .is_ok(),
                            )
                        })
                        .unwrap_or(false);
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    deps.push(crate::monitoring::DependencyStatus {
                        name: "LLM",
                        host: "llm".to_string(),
                        healthy,
                        latency_ms: latency,
                    });
                }

                deps
            }
        }

        let collector = Arc::new(crate::monitoring::MetricsCollector::new());
        {
            let c = collector.clone();
            tokio::spawn(async move {
                c.setup_default_alert_rules().await;
                c.start_background_collection().await;
            });
        }

        let monitoring_state = Arc::new(MonitoringAppState {
            app_state: app_state.clone(),
            collector: collector.clone(),
        });
        *sub_router = sub_router.clone().merge(
            crate::monitoring::configure::<
                MonitoringAppState,
                crate::monitoring::DefaultMonitoringUrls,
            >()
            .with_state(monitoring_state),
        );
        *sub_router = sub_router
            .clone()
            .merge(crate::monitoring::governance::configure_routes(collector));
    }

    #[cfg(feature = "scripting")]
    {
        *sub_router = sub_router.clone().merge(
            crate::basic::keywords::app_server::configure_app_server_routes()
                .with_state(app_state.clone()),
        );
    }

    #[cfg(feature = "people")]
    {
        *sub_router = sub_router.clone().merge(
            crate::basic::keywords::db_api::configure_db_routes().with_state(app_state.clone()),
        );
    }
}
