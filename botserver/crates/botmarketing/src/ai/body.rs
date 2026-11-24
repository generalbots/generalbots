use super::*;
use axum::http::HeaderMap;

pub(super) fn build_generation_prompt(req: &ContentGenerationRequest) -> String {
    let tone = req.tone.as_deref().unwrap_or(DEFAULT_TONE);
    let length = req.length.as_deref().unwrap_or(DEFAULT_LENGTH);

    format!(
        "You are a marketing expert. Create {length} marketing content for {channel} channel.\n\
         Goal: {goal}\nAudience: {audience}\nTone: {tone}\n\n\
         Generate: 1. Subject line (if email) 2. Body content 3. CTA 4. 2 variations with different tones.\n\
         Respond in JSON: {{\"subject\":\"...\",\"body\":\"...\",\"cta\":\"...\",\
         \"variations\":[{{\"name\":\"friendly\",\"body\":\"...\",\"tone\":\"friendly\"}},\
         {{\"name\":\"urgent\",\"body\":\"...\",\"tone\":\"urgent\"}}]}}",
        length = length, channel = req.channel, goal = req.goal,
        audience = req.audience_description, tone = tone
    )
}

pub(super) fn build_personalization_prompt(
    contact: &ContactInfo,
    template: &str,
    context: &serde_json::Value,
) -> String {
    let context_str = if context.is_null() {
        String::new()
    } else {
        format!("\nAdditional context: {}", context)
    };

    let first_name = contact.first_name.as_deref().unwrap_or("there");
    let last_name = contact.last_name.as_deref().unwrap_or("");
    let email = contact.email.as_deref().unwrap_or("");
    let phone = contact.phone.as_deref().unwrap_or("");
    let company = contact.company.as_deref().unwrap_or("");

    format!(
        "Personalize this marketing message for: {} {} ({}, {}, {}){}\n\nTemplate:\n{}",
        first_name, last_name, email, phone, company, context_str, template
    )
}

#[derive(Debug, Clone)]
pub(super) struct ContactInfo {
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
}

pub(super) fn get_llm_config(
    state: &Arc<AppState>,
    bot_id: Uuid,
) -> Result<(String, String, String), String> {
    let llm_url = (state.get_config)(&bot_id, "llm-url", Some("")).unwrap_or_else(|e| { log::warn!("config read failed: {e}"); String::new() });
    let llm_model = (state.get_config)(&bot_id, "llm-model", None).unwrap_or_else(|e| { log::warn!("config read failed: {e}"); String::new() });
    let llm_key = (state.get_config)(&bot_id, "llm-key", None).unwrap_or_else(|e| { log::warn!("config read failed: {e}"); String::new() });
    Ok((llm_url, llm_model, llm_key))
}

pub async fn generate_campaign_content(
    state: &Arc<AppState>,
    bot_id: Uuid,
    req: ContentGenerationRequest,
) -> Result<ContentGenerationResult, String> {
    let (_llm_url, llm_model, llm_key) = get_llm_config(state, bot_id)?;

    let prompt = build_generation_prompt(&req);
    let config = serde_json::json!({
        "temperature": 0.7,
        "max_tokens": 2000,
    });

    let response = (state.llm_generate)(&prompt, &config, &llm_model, &llm_key)
        .map_err(|e| format!("LLM generation failed: {}", e))?;

    parse_llm_response(&response)
}

pub(super) fn parse_llm_response(response: &str) -> Result<ContentGenerationResult, String> {
    let json_start = response.find('{').or_else(|| response.find('['));
    let json_end = response.rfind('}').or_else(|| response.rfind(']'));

    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &response[start..=end];
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
            let subject = parsed
                .get("subject")
                .and_then(|s| s.as_str())
                .map(String::from);
            let body = parsed
                .get("body")
                .and_then(|b| b.as_str())
                .unwrap_or("")
                .to_string();
            let cta = parsed
                .get("cta")
                .and_then(|c| c.as_str())
                .map(String::from);

            let mut variations = Vec::new();
            if let Some(vars) = parsed.get("variations").and_then(|v| v.as_array()) {
                for v in vars {
                    variations.push(ContentVariation {
                        name: v
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        body: v
                            .get("body")
                            .and_then(|b| b.as_str())
                            .unwrap_or("")
                            .to_string(),
                        tone: v
                            .get("tone")
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string(),
                    });
                }
            }

            return Ok(ContentGenerationResult {
                subject,
                body,
                headline: None,
                cta,
                suggested_images: vec![],
                variations,
            });
        }
    }

    Ok(ContentGenerationResult {
        subject: Some(response.lines().next().unwrap_or("").to_string()),
        body: response.to_string(),
        headline: None,
        cta: Some("Learn More".to_string()),
        suggested_images: vec![],
        variations: vec![],
    })
}

/// Resolve (org_id, branch_id) from verified caller claims (#2890); nil/missing
/// values fall back to the default bot scope. Pure, so it is unit-testable.
fn resolve_scope(
    caller_org: Option<Uuid>,
    caller_branch: Option<Uuid>,
    default_org: Uuid,
    default_branch: Uuid,
) -> (Uuid, Uuid) {
    let org_id = caller_org.filter(|id| !id.is_nil()).unwrap_or(default_org);
    let branch_id = caller_branch.filter(|id| !id.is_nil()).unwrap_or(default_branch);
    (org_id, branch_id)
}

pub async fn personalize_content(
    state: &Arc<AppState>,
    org_id: Uuid,
    branch_id: Uuid,
    bot_id: Uuid,
    req: PersonalizationRequest,
) -> Result<PersonalizationResult, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("DB error: {}", e))?;

    let contact = crm_contacts::table
        .filter(crm_contacts::id.eq(req.contact_id))
        .filter(crm_contacts::branch_id.eq(branch_id))
        .filter(crm_contacts::org_id.eq(org_id))
        .select((
            crm_contacts::first_name,
            crm_contacts::last_name,
            crm_contacts::email,
            crm_contacts::phone,
            crm_contacts::company,
        ))
        .first::<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )>(&mut conn)
        .map_err(|_| "Contact not found".to_string())?;

    let contact_info = ContactInfo {
        first_name: contact.0,
        last_name: contact.1,
        email: contact.2,
        phone: contact.3,
        company: contact.4,
    };

    let context = req.context.unwrap_or(serde_json::Value::Null);
    let prompt = build_personalization_prompt(&contact_info, &req.template, &context);

    let (_llm_url, llm_model, llm_key) = get_llm_config(state, bot_id)?;

    let config = serde_json::json!({
        "temperature": 0.5,
        "max_tokens": 1000,
    });

    let response = (state.llm_generate)(&prompt, &config, &llm_model, &llm_key)
        .map_err(|e| format!("LLM personalization failed: {}", e))?;

    let variables = extract_variables(&req.template);

    Ok(PersonalizationResult {
        personalized_content: response,
        variables_used: variables,
    })
}

pub(super) fn extract_variables(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut in_brace = false;
    let mut current = String::new();

    for c in template.chars() {
        match c {
            '{' => {
                in_brace = true;
                current.clear();
            }
            '}' if in_brace => {
                in_brace = false;
                if !current.is_empty() {
                    vars.push(current.clone());
                }
                current.clear();
            }
            _ if in_brace => current.push(c),
            _ => {}
        }
    }

    vars
}

pub async fn generate_ab_test_variations(
    state: &Arc<AppState>,
    bot_id: Uuid,
    req: ABTestRequest,
) -> Result<Vec<ABTestResult>, String> {
    let mut results = Vec::new();

    for (i, variation) in req.variations.iter().enumerate() {
        let prompt = format!(
            r#"Evaluate this marketing variation:

Name: {}
Subject: {}
Body: {}

Provide a JSON response:
{{
  "opens": <estimated opens 0-100>,
  "clicks": <estimated clicks 0-100>,
  "open_rate": <percentage>,
  "click_rate": <percentage>
}}"#,
            variation.name,
            variation.subject.as_deref().unwrap_or("N/A"),
            variation.body
        );

        let config = serde_json::json!({
            "temperature": 0.3,
            "max_tokens": 200,
        });

        let (_llm_url, llm_model, llm_key) = get_llm_config(state, bot_id)?;

        let response = (state.llm_generate)(&prompt, &config, &llm_model, &llm_key)
            .unwrap_or_else(|e| { log::warn!("LLM generation failed: {e}"); String::new() });

        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap_or(
            serde_json::json!({
                "opens": 50,
                "clicks": 10,
                "open_rate": 50.0,
                "click_rate": 10.0
            }),
        );

        results.push(ABTestResult {
            variation_id: format!("variation_{}", i),
            opens: parsed.get("opens").and_then(|v| v.as_i64()).unwrap_or(50),
            clicks: parsed.get("clicks").and_then(|v| v.as_i64()).unwrap_or(10),
            open_rate: parsed
                .get("open_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(50.0),
            click_rate: parsed
                .get("click_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(10.0),
            winner: false,
        });
    }

    if let Some(winner) = results
        .iter()
        .max_by(|a, b| a.open_rate.partial_cmp(&b.open_rate).unwrap_or(std::cmp::Ordering::Equal))
    {
        let winner_id = winner.variation_id.clone();
        for r in &mut results {
            r.winner = r.variation_id == winner_id;
        }
    }

    Ok(results)
}

#[derive(Debug, Deserialize)]
pub struct GenerateContentRequest {
    pub channel: String,
    pub goal: String,
    pub audience_description: String,
    pub template_variables: Option<serde_json::Value>,
    pub tone: Option<String>,
    pub length: Option<String>,
}

pub async fn generate_content_api(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateContentRequest>,
) -> Result<Json<ContentGenerationResult>, (StatusCode, String)> {
    let bot_id = Uuid::nil();

    let internal_req = ContentGenerationRequest {
        channel: req.channel,
        goal: req.goal,
        audience_description: req.audience_description,
        template_variables: req.template_variables,
        tone: req.tone,
        length: req.length,
    };

    generate_campaign_content(&state, bot_id, internal_req).await
        .map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Debug, Deserialize)]
pub struct PersonalizeRequest {
    pub template: String,
    pub contact_id: Uuid,
    pub context: Option<serde_json::Value>,
}

pub async fn personalize_api(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PersonalizeRequest>,
) -> Result<Json<PersonalizationResult>, (StatusCode, String)> {
    // Scope from the verified caller (#2890): JWT claims win, default is the
    // fallback. `branch_from_jwt_pool` also resolves opaque suite tokens.
    let (default_org, default_branch, bot_id) = state.get_scope();
    let caller_branch = crate::scope::branch_from_jwt_pool(&headers, &state.conn);
    let caller_org = botsecurity_core::verified_jwt::bearer_token(&headers)
        .and_then(|token| botsecurity_core::verified_jwt::verify_saas_jwt(token).ok())
        .and_then(|claims| claims.org_id);
    let (org_id, branch_id) = resolve_scope(caller_org, caller_branch, default_org, default_branch);
    if branch_id.is_nil() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "no tenant scope".to_string()));
    }

    let internal_req = PersonalizationRequest {
        template: req.template,
        contact_id: req.contact_id,
        context: req.context,
    };

    personalize_content(&state, org_id, branch_id, bot_id, internal_req).await
        .map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// Studio image generation via BotModels (aggregated image-gen APIs).
/// The browser never talks to BotModels directly: keys stay server-side.
#[derive(Debug, Deserialize)]
pub struct StudioImageRequest {
    pub prompt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

struct StudioConfigAdapter<'a>(&'a crate::state::GetConfigFn);

impl botmultimodal::multimodal::ConfigProvider for StudioConfigAdapter<'_> {
    fn get_config(&self, bot_id: &uuid::Uuid, key: &str, default: Option<&str>) -> Option<String> {
        (self.0)(bot_id, key, default).ok()
    }
}

pub async fn generate_studio_image(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StudioImageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let bot_id = Uuid::nil();
    let adapter = StudioConfigAdapter(&state.get_config);
    let client = botmultimodal::multimodal::BotModelsClient::from_provider_all(&adapter, &bot_id);

    if !client.is_enabled() {
        return Err((
            StatusCode::BAD_REQUEST,
            "BotModels image generation is not enabled. Set botmodels-enabled=true and botmodels-host/port/api-key in the bot configuration."
                .to_string(),
        ));
    }

    match client.generate_image(&req.prompt).await {
        Ok(url) => Ok(Json(serde_json::json!({
            "success": true,
            "url": url,
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image generation failed: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_caller_claims_win() {
        let (org, branch) = (Uuid::new_v4(), Uuid::new_v4());
        let got = resolve_scope(Some(org), Some(branch), Uuid::nil(), Uuid::nil());
        assert_eq!(got, (org, branch));
    }

    #[test]
    fn nil_and_missing_claims_fall_back_to_default() {
        let default = (Uuid::new_v4(), Uuid::new_v4());
        let resolved = |o, b| resolve_scope(o, b, default.0, default.1);
        assert_eq!(resolved(None, None), default);
        assert_eq!(resolved(Some(Uuid::nil()), Some(Uuid::nil())), default);
    }
    #[test]
    fn each_component_falls_back_independently() {
        let (org, branch) = (Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(resolve_scope(Some(org), None, Uuid::nil(), branch), (org, branch));
        assert_eq!(resolve_scope(None, Some(branch), org, Uuid::nil()), (org, branch));
    }
}
