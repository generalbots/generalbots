use log::info;
use mailparse::MailHeaderMap;
use std::collections::HashMap;
use uuid::Uuid;

pub(super) async fn index_imap_account_emails(
    email: &str,
    vault_path: &str,
    metadata_json: &str,
    account_id: Uuid,
    qdrant_collection: &str,
) {
    let meta: HashMap<String, String> = match serde_json::from_str(metadata_json) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to parse IMAP account metadata: {e}");
            return;
        }
    };

    let imap_server = match meta.get("imap_server") {
        Some(s) => s.clone(),
        None => { log::error!("IMAP account missing imap_server in metadata"); return; }
    };
    let imap_port: u16 = meta.get("imap_port").and_then(|p| p.parse().ok()).unwrap_or(993);
    let username = match meta.get("username") {
        Some(s) => s.clone(),
        None => { log::error!("IMAP account missing username in metadata"); return; }
    };

    let password = match botcoresecrets::manager::SecretsManager::get() {
        Ok(sm) => {
            let data = sm.get_secret(vault_path).await.unwrap_or_else(|e| {
                log::warn!("Failed to read IMAP secret at {vault_path}: {e}");
                HashMap::new()
            });
            data.get("imap_password").cloned().unwrap_or_else(String::new)
        }
        Err(e) => {
            log::error!("Vault not available for IMAP indexing: {e}");
            return;
        }
    };

    if password.is_empty() {
        log::error!("IMAP password not found in Vault at {vault_path}");
        return;
    }

    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| std::env::var("VECTORDB_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".to_string()));

    let qdrant_client = match botqdrant::QdrantClient::from_url(&qdrant_url).build() {
        Ok(c) => c,
        Err(e) => { log::error!("Failed to create Qdrant client: {e}"); return; }
    };

    let exists = qdrant_client
        .list_collections()
        .await
        .map(|collections| {
            collections
                .collections
                .iter()
                .any(|c| c.name == qdrant_collection)
        })
        .unwrap_or_else(|e| {
            log::warn!("Failed to list Qdrant collections for {qdrant_collection}: {e}");
            false
        });
    if !exists {
        if let Err(e) = qdrant_client.create_collection(qdrant_collection, 1536, "Cosine").await {
            log::error!("Failed to create Qdrant collection {qdrant_collection}: {e}");
            return;
        }
    }

    let embedding_gen = botqdrant::embedding::EmbeddingGenerator::new(
        std::env::var("LLM_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8081".to_string())
    );

    let imap_result = tokio::task::spawn_blocking(move || {
        let client = match imap::ClientBuilder::new(&imap_server, imap_port).connect() {
            Ok(c) => c,
            Err(e) => { log::error!("IMAP connection failed: {e:?}"); return Vec::new(); }
        };
        let mut session = match client.login(&username, &password) {
            Ok(s) => s,
            Err(e) => { log::error!("IMAP login failed: {e:?}"); return Vec::new(); }
        };

        match session.select("INBOX") {
            Ok(m) => info!("Selected INBOX, {} messages", m.exists),
            Err(e) => { log::error!("IMAP select INBOX failed: {e:?}"); return Vec::new(); }
        }

        let seq = match session.fetch("1:50", "(RFC822)") {
            Ok(s) => s,
            Err(e) => { log::error!("IMAP fetch failed: {e:?}"); return Vec::new(); }
        };

        let mut emails = Vec::new();
        for msg in seq.iter() {
            if let Some(body) = msg.body() {
                if let Ok(parsed) = mailparse::parse_mail(body) {
                    let headers = parsed.get_headers();
                    let subject = headers.get_first_value("Subject").unwrap_or_else(String::new);
                    let from = headers.get_first_value("From").unwrap_or_else(String::new);
                    let to = headers.get_first_value("To").unwrap_or_else(String::new);
                    let body_text = parsed.subparts.iter()
                        .find(|p| p.ctype.mimetype == "text/plain")
                        .and_then(|bp| bp.get_body().ok())
                        .or_else(|| parsed.get_body().ok())
                        .unwrap_or_else(String::new);

                    let (from_name, from_email) = if let Some(start) = from.find('<') {
                        if let Some(end) = from.find('>') {
                            (from[..start].trim().trim_matches('"').to_string(), from[start+1..end].to_string())
                        } else { (String::new(), from.clone()) }
                    } else { (String::new(), from.clone()) };

                    emails.push(botqdrant::EmailDocument {
                        id: Uuid::new_v4().to_string(),
                        account_id: account_id.to_string(),
                        from_email,
                        from_name,
                        to_email: to,
                        subject,
                        body_text,
                        date: chrono::Utc::now(),
                        folder: "INBOX".to_string(),
                        has_attachments: false,
                        thread_id: None,
                    });
                }
            }
        }
        if let Err(e) = session.logout() {
            log::warn!("IMAP logout failed: {e}");
        }
        emails
    }).await.unwrap_or_else(|e| {
        log::warn!("IMAP email indexing task failed: {e}");
        Vec::new()
    });

    info!("Fetched {} emails from IMAP for {}", imap_result.len(), email);

    for chunk in imap_result.chunks(10) {
        for email_doc in chunk {
            let text = format!("From: {} <{}>\nSubject: {}\n\n{}",
                email_doc.from_name, email_doc.from_email, email_doc.subject, email_doc.body_text);
            // Email bodies are arbitrary user text; cut on a char boundary.
            let text = botlib::text::truncate_at_char_boundary(&text, 6000);

            match embedding_gen.generate_text_embedding(text).await {
                Ok(embedding) => {
                    let point = serde_json::json!({
                        "id": email_doc.id,
                        "vector": embedding,
                        "payload": serde_json::to_value(email_doc).unwrap_or_else(|e| {
                            log::warn!("Failed to serialize indexed email: {e}");
                            serde_json::Value::Null
                        })
                    });
                    if let Err(e) = qdrant_client.upsert_points(qdrant_collection, vec![point]).await {
                        log::error!("Failed to index email '{}': {e}", email_doc.subject);
                    }
                }
                Err(e) => log::error!("Failed to generate embedding: {e}"),
            }
        }
    }
    info!("Indexed IMAP emails for {} into Qdrant collection {}", email, qdrant_collection);
}
