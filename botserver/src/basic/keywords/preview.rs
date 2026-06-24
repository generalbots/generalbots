use rhai::{Engine, EvalAltResult};

pub fn register_preview_keyword(engine: &mut Engine) {
    engine.register_fn("preview", |description: &str, _params: rhai::Dynamic| -> Result<String, Box<EvalAltResult>> {
        Ok(format!(
            r#"<div class="preview-modal-overlay">
                <div class="preview-modal" style="background:#FFFBE6;border:2px solid #F5C518;border-radius:12px;padding:24px;max-width:600px;margin:20px auto;box-shadow:0 8px 32px rgba(0,0,0,0.15);font-family:system-ui,sans-serif;">
                    <h2 style="color:#B8860B;margin-top:0;font-size:1.5em;border-bottom:2px solid #F5C518;padding-bottom:8px;">{}</h2>
                    <div class="preview-params" style="margin:16px 0;">
                        <p style="color:#666;font-style:italic;">Parameters will be displayed here in preview mode.</p>
                    </div>
                    <div style="display:flex;gap:8px;justify-content:flex-end;margin-top:16px;padding-top:12px;border-top:1px solid #E8D5A0;">
                        <button class="preview-btn" style="background:#F5C518;color:#000;border:none;padding:8px 20px;border-radius:6px;cursor:pointer;font-weight:600;" onclick="closePreviewModal()">Confirm</button>
                        <button class="preview-btn preview-btn-cancel" style="background:#f0f0f0;color:#333;border:1px solid #ddd;padding:8px 20px;border-radius:6px;cursor:pointer;" onclick="closePreviewModal()">Cancel</button>
                    </div>
                </div>
            </div>"#,
            description
        ))
    });
}
