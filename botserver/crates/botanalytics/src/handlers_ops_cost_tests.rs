use super::*;

#[test]
fn cost_panel_never_invents_a_per_conversation_price() {
    let data = CostData {
        totals: CostTotalsRow {
            requests: 0,
            input_tokens: 0,
            output_tokens: 0,
            conversations: 0,
            failures: 0,
            cost: 0.0,
        },
        tenants: Vec::new(),
        quotas: Vec::new(),
    };
    let html = render_cost(&data, 60);
    assert!(html.contains("Cost / conversation"));
    assert!(html.contains("—"));
    assert!(html.contains("No tenant spend recorded"));
    assert!(html.contains("No LLM budget rows configured"));
}

#[test]
fn cost_panel_divides_cost_by_conversations() {
    let data = CostData {
        totals: CostTotalsRow {
            requests: 10,
            input_tokens: 1_000,
            output_tokens: 500,
            conversations: 4,
            failures: 1,
            cost: 0.4,
        },
        tenants: vec![TenantCostRow {
            tenant: "acme".to_string(),
            tokens: 1_500,
            cost: 0.4,
        }],
        quotas: vec![QuotaRow {
            bot: "support".to_string(),
            daily_spend: 9.0,
            daily_limit: 10.0,
        }],
    };
    let html = render_cost(&data, 60);
    assert!(html.contains("$0.1000"));
    assert!(html.contains("acme"));
    assert!(html.contains("90%"));
    assert!(html.contains("ops-warn"));
}
