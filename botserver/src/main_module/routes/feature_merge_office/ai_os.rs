//! AI OS routers (agent VM, conversation autotasks, automations, marketplace,
//! consent, memory, connectors, browser policy, channel bindings).

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_office_ai_os(
    app_state: &Arc<AppState>,
    api_router: &mut Router<Arc<AppState>>,
) {
    let _ = &app_state;
    // ===== AI OS routers (issues #1167-#1179) =====
    #[cfg(feature = "agent-vm")]
    {
        botagent::api::set_external_token_verifier(|token: &str| {
            crate::main_module::routes::anonymous_auth::resolve_token_user(token)
                .map(|(id, _roles)| crate::security::user_role::derive_stable_user_uuid(&id))
        });
        let agent_state = Arc::new(botagent::AgentService::new(app_state.conn.clone()));
        crate::core::bot::agent_vm_hook::init_with(agent_state.clone());
        *api_router = api_router
            .clone()
            .merge(botagent::configure_routes().with_state(agent_state));
    }

    // #1613 — conversation-scoped autotasks are part of chat, not of the
    // automations dashboard: they must be present in the desktop edition too.
    #[cfg(feature = "chat")]
    {
        crate::main_module::routes::conversation_autotasks::reconcile_on_boot(app_state);
        *api_router = api_router
            .clone()
            .merge(crate::main_module::routes::conversation_autotasks::router())
            .merge(crate::main_module::routes::computer_use_routes::router());
    }

    #[cfg(feature = "automations")]
    {
        let automation_state = Arc::new(
            botautomation::AutomationService::new(app_state.conn.clone())
                .with_llm(Arc::new(|_system: &str, _user: &str, _params: &str| {
                    Err("LLM not wired for automations".to_string())
                }))
                .with_delivery(Arc::new(
                    |_channel: &str, _to: &str, _subject: &str, _body: &str| Ok(()),
                )),
        );
        botautomation::scheduler::spawn_scheduler(automation_state.clone());
        *api_router = api_router
            .clone()
            .merge(botautomation::configure_routes().with_state(automation_state));
    }

    #[cfg(feature = "marketplace")]
    {
        let marketplace_state = Arc::new(botmarketplace::MarketplaceService::new(
            app_state.conn.clone(),
        ));
        let seed_state = marketplace_state.clone();
        tokio::spawn(async move {
            match botmarketplace::seed_if_empty(&seed_state).await {
                Ok(count) if count > 0 => log::info!("marketplace seeded {count} starter skills"),
                Ok(_) => {}
                Err(e) => log::error!("marketplace seed failed: {e}"),
            }
        });
        *api_router = api_router
            .clone()
            .merge(botmarketplace::configure_routes().with_state(marketplace_state));
    }

    #[cfg(feature = "consent")]
    {
        crate::core::bot::consent_gate::init(app_state.conn.clone());
        let consent_state = Arc::new(botconsent::ConsentService::new(app_state.conn.clone()));
        consent_state.ensure_sweeper();
        *api_router = api_router
            .clone()
            .merge(botconsent::configure_routes().with_state(consent_state));
    }

    #[cfg(feature = "memory-os")]
    {
        let memory_state = Arc::new(botmemory::MemoryService::new(
            app_state.conn.clone(),
            Arc::new(|_system: &str, _user: &str, _params: &str| {
                Err("LLM not wired for memory extraction".to_string())
            }),
        ));
        crate::core::bot::memory_hook::init_with(memory_state.clone());
        *api_router = api_router
            .clone()
            .merge(botmemory::configure_routes().with_state(memory_state));
    }

    #[cfg(feature = "connectors")]
    {
        *api_router = api_router.clone().merge(botconnectors::configure());
    }

    #[cfg(feature = "browser-policy")]
    {
        let browser_policy_state = Arc::new(botbrowserpolicy::BrowserPolicyService::new(
            app_state.conn.clone(),
        ));
        *api_router = api_router
            .clone()
            .merge(botbrowserpolicy::configure_routes().with_state(browser_policy_state));
    }

    #[cfg(feature = "channel-bindings")]
    {
        let bindings_state = Arc::new(botchannelbindings::ChannelBindingsService::new(
            app_state.conn.clone(),
        ));
        *api_router = api_router
            .clone()
            .merge(botchannelbindings::configure_routes().with_state(bindings_state));
    }
}
