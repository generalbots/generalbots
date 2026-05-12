pub mod bot_memory;
pub mod clear_kb;
pub mod crm;
pub mod data_operations;
pub mod db_api;
pub mod detect;
pub mod find;
pub mod get;
pub mod import_export;
pub mod kb_statistics;
pub mod lead_scoring;
pub mod products;
pub mod save_from_unstructured;
pub mod search;
pub mod set;
pub mod table_access;
pub mod table_definition;
pub mod table_migration;
pub mod think_kb;
pub mod use_account;
pub mod use_kb;
pub mod user_memory;

use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::Engine;
use std::sync::Arc;

pub fn register_data_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    bot_memory::register_bot_memory_keywords(state.clone(), user.clone(), engine);
    clear_kb::register_clear_kb_keyword(state.clone(), user.clone(), engine);
    crm::register_crm_keywords(state.clone(), user.clone(), engine);
    data_operations::register_data_operations(state.clone(), user.clone(), engine);
    // db_api is a route config module, not keyword registration
    detect::register_detect_keyword(state.clone(), user.clone(), engine);
    find::register_find_keyword(state.clone(), user.clone(), engine);
    get::register_get_keyword(state.clone(), user.clone(), engine);
    import_export::register_import_export(state.clone(), user.clone(), engine);
    kb_statistics::register_kb_statistics_keyword(state.clone(), user.clone(), engine);
    lead_scoring::register_lead_scoring_keywords(state.clone(), user.clone(), engine);
    products::register_products_keywords(state.clone(), user.clone(), engine);
    save_from_unstructured::register_save_from_unstructured(state.clone(), user.clone(), engine);
    search::register_search_keyword(state.clone(), user.clone(), engine);
    set::register_set_keyword(state.clone(), user.clone(), engine);
    table_access::register_table_keywords(state.clone(), user.clone(), engine);
    table_definition::register_table_definition_keywords(state.clone(), user.clone(), engine);
    table_migration::register_table_migration_keywords(state.clone(), user.clone(), engine);
    think_kb::register_think_kb_keyword(state.clone(), user.clone(), engine);
    use_account::register_use_account_keyword(state.clone(), user.clone(), engine);
    use_kb::register_use_kb_keyword(state.clone(), user.clone(), engine);
}
