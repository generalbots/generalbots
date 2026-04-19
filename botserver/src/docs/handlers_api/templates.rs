use crate::docs::types::TemplateResponse;
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

pub async fn handle_template_blank() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Untitled Document".to_string(),
        content: String::new(),
    }))
}

pub async fn handle_template_meeting() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<h1>Meeting Notes</h1>
<p><strong>Date:</strong> [Date]</p>
<p><strong>Attendees:</strong> [Names]</p>
<p><strong>Location:</strong> [Location/Virtual]</p>
<hr>
<h2>Agenda</h2>
<ol>
<li>Topic 1</li>
<li>Topic 2</li>
<li>Topic 3</li>
</ol>
<h2>Discussion Points</h2>
<p>[Notes here]</p>
<h2>Action Items</h2>
<ul>
<li>[ ] Action 1 - Owner - Due Date</li>
<li>[ ] Action 2 - Owner - Due Date</li>
</ul>
<h2>Next Meeting</h2>
<p>[Date and time of next meeting]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Meeting Notes".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_template_report() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<h1>Report Title</h1>
<p><em>Author: [Your Name]</em></p>
<p><em>Date: [Date]</em></p>
<hr>
<h2>Executive Summary</h2>
<p>[Brief overview of the report]</p>
<h2>Introduction</h2>
<p>[Background and context]</p>
<h2>Methodology</h2>
<p>[How the information was gathered]</p>
<h2>Findings</h2>
<p>[Key findings and data]</p>
<h2>Recommendations</h2>
<ul>
<li>Recommendation 1</li>
<li>Recommendation 2</li>
<li>Recommendation 3</li>
</ul>
<h2>Conclusion</h2>
<p>[Summary and next steps]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Report".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_template_letter() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<p>[Your Name]<br>
[Your Address]<br>
[City, State ZIP]<br>
[Date]</p>
<p>[Recipient Name]<br>
[Recipient Title]<br>
[Company Name]<br>
[Address]<br>
[City, State ZIP]</p>
<p>Dear [Recipient Name],</p>
<p>[Opening paragraph - state the purpose of your letter]</p>
<p>[Body paragraph(s) - provide details and supporting information]</p>
<p>[Closing paragraph - summarize and state any call to action]</p>
<p>Sincerely,</p>
<p>[Your Name]<br>
[Your Title]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Letter".to_string(),
        content: content.to_string(),
    }))
}
