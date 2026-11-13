//! Landing-tab recovery for `open` steps.
//!
//! The model describes the tab in prose ("switching to the NFS-e tab") but
//! frequently emits a bare `{"op":"open","app":"tax"}`, so the app opens on its
//! default tab and the user sees the wrong filter (#2194 TAX-2). The user's own
//! words are unambiguous, so the requested landing state is recovered here
//! instead of hoping the model fills `params`.

use super::{UiOp, UiPlan};

/// Fills `params` for an open step from the user's request when the plan left
/// it empty and the request names a known tab of that app.
pub fn apply_open_hints(mut plan: UiPlan, user_text: &str) -> UiPlan {
    let lowered = user_text.to_lowercase();
    for step in plan.steps.iter_mut() {
        if step.op != UiOp::Open {
            continue;
        }
        if step.params.is_some() {
            continue;
        }
        let Some(app) = step.app.as_deref().or(plan.app.as_deref()) else {
            continue;
        };
        if let Some(tab) = tab_for_app(app, &lowered) {
            step.params = Some(serde_json::json!({ "tab": tab }));
        }
    }
    plan
}

/// Tab vocabulary per app, matched against the user's own words.
fn tab_for_app(app: &str, lowered: &str) -> Option<&'static str> {
    match app {
        "tax" => tax_tab(lowered),
        _ => None,
    }
}

fn tax_tab(lowered: &str) -> Option<&'static str> {
    // Order matters: "nota fiscal de serviço" also contains "nota fiscal".
    let has = |needle: &str| lowered.contains(needle);
    if has("nfs-e") || has("nfse") || has("nota fiscal de serviço") || has("notas de serviço") {
        return Some("nfse");
    }
    if has("ct-e") || has("cte") || has("conhecimento de transporte") {
        return Some("cte");
    }
    if has("sped") {
        return Some("sped");
    }
    if has("nf-e") || has("nfe") || has("nota fiscal eletrônica") {
        return Some("nfe");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_with_open(app: &str) -> UiPlan {
        UiPlan {
            app: Some(app.to_string()),
            window_id: None,
            steps: vec![super::super::UiStep {
                op: UiOp::Open,
                app: Some(app.to_string()),
                window_id: None,
                label: None,
                field: None,
                value: None,
                ms: None,
                cell: None,
                path: None,
                params: None,
            }],
        }
    }

    #[test]
    fn recovers_the_nfse_tab_from_the_request() {
        let plan = apply_open_hints(
            plan_with_open("tax"),
            "Open the tax app and show the NFS-e documents",
        );
        let params = plan.steps[0].params.clone().unwrap_or_default();
        assert_eq!(params.get("tab").and_then(|v| v.as_str()), Some("nfse"));
    }

    #[test]
    fn keeps_an_explicit_param_and_ignores_unknown_apps() {
        let mut plan = plan_with_open("tax");
        plan.steps[0].params = Some(serde_json::json!({ "tab": "cte" }));
        assert_eq!(
            apply_open_hints(plan, "open the tax app on the nfse tab").steps[0]
                .params
                .as_ref()
                .and_then(|p| p.get("tab"))
                .and_then(|v| v.as_str()),
            Some("cte")
        );
        let docs = apply_open_hints(plan_with_open("docs"), "open the nfse tab");
        assert!(docs.steps[0].params.is_none());
    }
}
