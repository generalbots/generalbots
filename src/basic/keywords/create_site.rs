#[cfg(feature = "llm")]
use crate::llm::LLMProvider;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, info, warn};
use rhai::Dynamic;
use rhai::Engine;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

// When llm feature is disabled, create a dummy trait for type compatibility
#[cfg(not(feature = "llm"))]
trait LLMProvider: Send + Sync {}

pub fn create_site_keyword(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["CREATE", "SITE", "$expr$", ",", "$expr$", ",", "$expr$"],
            true,
            move |context, inputs| {
                if inputs.len() < 3 {
                    return Err("Not enough arguments for CREATE SITE".into());
                }
                let alias = context.eval_expression_tree(&inputs[0])?;
                let template_dir = context.eval_expression_tree(&inputs[1])?;
                let prompt = context.eval_expression_tree(&inputs[2])?;

                let config = match state_clone.config.as_ref() {
                    Some(c) => c.clone(),
                    None => {
                        return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "Config must be initialized".into(),
                            rhai::Position::NONE,
                        )));
                    }
                };

                let s3 = state_clone.s3_client.clone().map(std::sync::Arc::new);
                let bucket = state_clone.bucket_name.clone();
                let bot_id = user_clone.bot_id.to_string();

                #[cfg(feature = "llm")]
                let llm = Some(state_clone.llm_provider.clone());
                #[cfg(not(feature = "llm"))]
                let llm: Option<()> = None;

                let fut = create_site(config, s3, bucket, bot_id, llm, alias, template_dir, prompt);
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("Site creation failed: {}", e))?;
                Ok(Dynamic::from(result))
            },
        )
        .expect("valid syntax registration");
}

#[cfg(feature = "llm")]
async fn create_site(
    config: crate::core::config::AppConfig,
    s3: Option<std::sync::Arc<aws_sdk_s3::Client>>,
    bucket: String,
    bot_id: String,
    llm: Option<Arc<dyn LLMProvider>>,
    alias: Dynamic,
    template_dir: Dynamic,
    prompt: Dynamic,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let alias_str = alias.to_string();
    let template_dir_str = template_dir.to_string();
    let prompt_str = prompt.to_string();

    info!(
        "CREATE SITE: {} from template {}",
        alias_str, template_dir_str
    );

    let base_path = PathBuf::from(&config.site_path);
    let template_path = base_path.join(&template_dir_str);

    let combined_content = load_templates(&template_path)?;

    let generated_html = generate_html_from_prompt(llm, &combined_content, &prompt_str).await?;

    let drive_path = format!("apps/{}", alias_str);
    store_to_drive(s3.as_ref(), &bucket, &bot_id, &drive_path, &generated_html).await?;

    let serve_path = base_path.join(&alias_str);
    sync_to_serve_path(&serve_path, &generated_html, &template_path)?;

    info!(
        "CREATE SITE: {} completed, available at /apps/{}",
        alias_str, alias_str
    );

    Ok(format!("/apps/{}", alias_str))
}

#[cfg(not(feature = "llm"))]
async fn create_site(
    config: crate::core::config::AppConfig,
    s3: Option<std::sync::Arc<aws_sdk_s3::Client>>,
    bucket: String,
    bot_id: String,
    _llm: Option<()>,
    alias: Dynamic,
    template_dir: Dynamic,
    prompt: Dynamic,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let alias_str = alias.to_string();
    let template_dir_str = template_dir.to_string();
    let prompt_str = prompt.to_string();

    info!(
        "CREATE SITE: {} from template {}",
        alias_str, template_dir_str
    );

    let base_path = PathBuf::from(&config.site_path);
    let template_path = base_path.join(&template_dir_str);

    let combined_content = load_templates(&template_path)?;

    let generated_html = generate_html_from_prompt(_llm, &combined_content, &prompt_str).await?;

    let drive_path = format!("apps/{}", alias_str);
    store_to_drive(s3.as_ref(), &bucket, &bot_id, &drive_path, &generated_html).await?;

    let serve_path = base_path.join(&alias_str);
    sync_to_serve_path(&serve_path, &generated_html, &template_path)?;

    info!(
        "CREATE SITE: {} completed, available at /apps/{}",
        alias_str, alias_str
    );

    Ok(format!("/apps/{}", alias_str))
}

fn load_templates(template_path: &std::path::Path) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut combined_content = String::new();

    if !template_path.exists() {
        return Err(format!("Template directory not found: {}", template_path.display()).into());
    }

    for entry in fs::read_dir(template_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "html") {
            let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| e.to_string())?;

            use std::fmt::Write;
            let _ = writeln!(combined_content, "<!-- TEMPLATE: {} -->", path.display());
            combined_content.push_str(&contents);
            combined_content.push_str("\n\n--- TEMPLATE SEPARATOR ---\n\n");

            debug!("Loaded template: {}", path.display());
        }
    }

    if combined_content.is_empty() {
        return Err("No HTML templates found in template directory".into());
    }

    Ok(combined_content)
}

#[cfg(feature = "llm")]
async fn generate_html_from_prompt(
    llm: Option<Arc<dyn LLMProvider>>,
    templates: &str,
    prompt: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let full_prompt = format!(
        r#"You are an expert HTML/HTMX developer. Generate a complete HTML application.

TEMPLATE FILES FOR REFERENCE:
{}

USER REQUEST:
{}

REQUIREMENTS:
1. Clone the template structure and styling
2. Use ONLY local _assets (htmx.min.js, app.js, styles.css) - NO external CDNs
3. Use HTMX for all data operations:
   - hx-get="/api/db/TABLE" for lists
   - hx-post="/api/db/TABLE" for create
   - hx-put="/api/db/TABLE/ID" for update
   - hx-delete="/api/db/TABLE/ID" for delete
4. Include search with hx-trigger="keyup changed delay:300ms"
5. Generate semantic, accessible HTML
6. App context is automatic - just use /api/db/* paths

OUTPUT: Complete index.html file only, no explanations."#,
        templates, prompt
    );

    let html = match llm {
        Some(provider) => {
            let messages = json!([{
                "role": "user",
                "content": full_prompt
            }]);

            match provider
                .generate(&full_prompt, &messages, "gpt-4o-mini", "")
                .await
            {
                Ok(response) => {
                    let cleaned = extract_html_from_response(&response);
                    if cleaned.contains("<html") || cleaned.contains("<!DOCTYPE") {
                        info!("LLM generated HTML ({} bytes)", cleaned.len());
                        cleaned
                    } else {
                        warn!("LLM response doesn't contain valid HTML, using placeholder");
                        generate_placeholder_html(prompt)
                    }
                }
                Err(e) => {
                    warn!("LLM generation failed: {}, using placeholder", e);
                    generate_placeholder_html(prompt)
                }
            }
        }
        None => {
            debug!("No LLM provider configured, using placeholder HTML");
            generate_placeholder_html(prompt)
        }
    };

    debug!("Generated HTML ({} bytes)", html.len());
    Ok(html)
}

#[cfg(not(feature = "llm"))]
async fn generate_html_from_prompt(
    _llm: Option<()>,
    _templates: &str,
    prompt: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    debug!("LLM feature not enabled, using placeholder HTML");
    Ok(generate_placeholder_html(prompt))
}

fn extract_html_from_response(response: &str) -> String {
    let trimmed = response.trim();

    if trimmed.starts_with("```html") {
        let without_prefix = trimmed.strip_prefix("```html").unwrap_or(trimmed);
        let without_suffix = without_prefix
            .trim()
            .strip_suffix("```")
            .unwrap_or(without_prefix);
        return without_suffix.trim().to_string();
    }

    if trimmed.starts_with("```") {
        let without_prefix = trimmed.strip_prefix("```").unwrap_or(trimmed);
        let without_suffix = without_prefix
            .trim()
            .strip_suffix("```")
            .unwrap_or(without_prefix);
        return without_suffix.trim().to_string();
    }

    trimmed.to_string()
}

fn generate_placeholder_html(prompt: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>App</title>
    <script src="_assets/htmx.min.js"></script>
    <link rel="stylesheet" href="_assets/styles.css">
</head>
<body>
    <header>
        <h1>Generated App</h1>
        <p>Prompt: {}</p>
    </header>

    <main>
        <section>
            <h2>Data</h2>
            <div id="data-list"
                 hx-get="/api/db/items"
                 hx-trigger="load"
                 hx-swap="innerHTML">
                Loading...
            </div>

            <form hx-post="/api/db/items"
                  hx-target="#data-list"
                  hx-swap="afterbegin">
                <input name="name" placeholder="Name" required>
                <button type="submit">Add</button>
            </form>
        </section>
    </main>

    <script src="_assets/app.js"></script>
</body>
</html>"##,
        prompt
    )
}

async fn store_to_drive(
    s3: Option<&std::sync::Arc<aws_sdk_s3::Client>>,
    bucket: &str,
    bot_id: &str,
    drive_path: &str,
    html_content: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some(s3_client) = s3 else {
        debug!("S3 not configured, skipping drive storage");
        return Ok(());
    };
    let key = format!("{}.gbdrive/{}/index.html", bot_id, drive_path);

    info!("Storing to drive: s3://{}/{}", bucket, key);

    s3_client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(html_content.as_bytes().to_vec().into())
        .content_type("text/html")
        .send()
        .await
        .map_err(|e| format!("Failed to store to drive: {}", e))?;

    let schema_key = format!("{}.gbdrive/{}/schema.json", bot_id, drive_path);
    let schema = r#"{"tables": {}, "version": 1}"#;

    s3_client
        .put_object()
        .bucket(bucket)
        .key(&schema_key)
        .body(schema.as_bytes().to_vec().into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to store schema: {}", e))?;

    Ok(())
}

fn sync_to_serve_path(
    serve_path: &std::path::Path,
    html_content: &str,
    template_path: &std::path::Path,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    fs::create_dir_all(serve_path).map_err(|e| format!("Failed to create serve path: {}", e))?;

    let index_path = serve_path.join("index.html");
    fs::write(&index_path, html_content)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;

    info!("Written: {}", index_path.display());

    let template_assets = template_path.join("_assets");
    let serve_assets = serve_path.join("_assets");

    if template_assets.exists() {
        copy_dir_recursive(&template_assets, &serve_assets)?;
        info!("Copied assets to: {}", serve_assets.display());
    } else {
        fs::create_dir_all(&serve_assets)
            .map_err(|e| format!("Failed to create assets dir: {}", e))?;

        let htmx_path = serve_assets.join("htmx.min.js");
        if !htmx_path.exists() {
            fs::write(&htmx_path, "/* HTMX - include from CDN or bundle */")
                .map_err(|e| format!("Failed to write htmx: {}", e))?;
        }

        let styles_path = serve_assets.join("styles.css");
        if !styles_path.exists() {
            fs::write(&styles_path, DEFAULT_STYLES)
                .map_err(|e| format!("Failed to write styles: {}", e))?;
        }

        let app_js_path = serve_assets.join("app.js");
        if !app_js_path.exists() {
            fs::write(&app_js_path, DEFAULT_APP_JS)
                .map_err(|e| format!("Failed to write app.js: {}", e))?;
        }
    }

    let schema_path = serve_path.join("schema.json");
    fs::write(&schema_path, r#"{"tables": {}, "version": 1}"#)
        .map_err(|e| format!("Failed to write schema.json: {}", e))?;

    Ok(())
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create dir {}: {}", dst.display(), e))?;

    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file {}: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}

const DEFAULT_STYLES: &str = r"
:root {
    --primary: #0ea5e9;
    --success: #22c55e;
    --warning: #f59e0b;
    --danger: #ef4444;
    --bg: #ffffff;
    --text: #1e293b;
    --border: #e2e8f0;
    --radius: 8px;
}

@media (prefers-color-scheme: dark) {
    :root {
        --bg: #0f172a;
        --text: #f1f5f9;
        --border: #334155;
    }
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--bg);
    color: var(--text);
    line-height: 1.5;
}

header {
    padding: 1rem 2rem;
    border-bottom: 1px solid var(--border);
}

main {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
}

h1, h2, h3 { margin-bottom: 1rem; }

input, select, textarea {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg);
    color: var(--text);
    font-size: 1rem;
}

input:focus, select:focus, textarea:focus {
    outline: none;
    border-color: var(--primary);
}

button {
    padding: 0.5rem 1rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    font-size: 1rem;
}

button:hover { opacity: 0.9; }

form {
    display: flex;
    gap: 0.5rem;
    margin: 1rem 0;
}

table {
    width: 100%;
    border-collapse: collapse;
}

th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--border);
}

.htmx-indicator {
    opacity: 0;
    transition: opacity 0.2s;
}

.htmx-request .htmx-indicator {
    opacity: 1;
}
";

const DEFAULT_APP_JS: &str = r"

function toast(message, type = 'info') {
    const el = document.createElement('div');
    el.className = 'toast toast-' + type;
    el.textContent = message;
    el.style.cssText = 'position:fixed;bottom:20px;right:20px;padding:1rem;background:#333;color:#fff;border-radius:8px;z-index:9999;';
    document.body.appendChild(el);
    setTimeout(() => el.remove(), 3000);
}


document.body.addEventListener('htmx:afterSwap', function(e) {
    console.log('Data updated:', e.detail.target.id);
});

document.body.addEventListener('htmx:responseError', function(e) {
    toast('Error: ' + (e.detail.xhr.responseText || 'Request failed'), 'error');
});


function openModal(id) {
    document.getElementById(id)?.classList.add('active');
}

function closeModal(id) {
    document.getElementById(id)?.classList.remove('active');
}
";
