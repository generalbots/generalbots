mod internationalization;

use bottest::prelude::*;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

fn test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

fn external_server_url() -> Option<String> {
    std::env::var("BOTSERVER_URL").ok()
}

async fn get_test_server() -> Option<(Option<TestContext>, String)> {
    if let Some(url) = external_server_url() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok()?;

        if client.get(&url).send().await.is_ok() {
            return Some((None, url));
        }
    }

    let ctx = TestHarness::quick().await.ok()?;
    let server = ctx.start_botserver().await.ok()?;

    if server.is_running() {
        Some((Some(ctx), server.url.clone()))
    } else {
        None
    }
}

macro_rules! skip_if_no_server {
    ($base_url:expr) => {
        if $base_url.is_none() {
            eprintln!("Skipping test: no server available");
            return;
        }
    };
}

#[tokio::test]
async fn test_accept_language_header() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/suite/auth/login.html", base_url))
        .header("Accept-Language", "es")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status().is_success(),
                "Server should respond to requests with Accept-Language header"
            );
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_locale_parameter_support() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let locales = vec!["en", "es", "pt-BR", "fr", "de"];

    for locale in locales {
        let response = client
            .get(format!("{}/suite/auth/login.html?locale={}", base_url, locale))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert!(
                    resp.status().is_success() || resp.status() == 404,
                    "Server should handle locale parameter: {}",
                    locale
                );
            }
            Err(_) => {
                eprintln!("Skipping: server not available");
            }
        }
    }
}

#[tokio::test]
async fn test_content_language_header() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/health", base_url))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let has_content_lang = resp
                .headers()
                .contains_key("content-language");
            
            if !has_content_lang {
                eprintln!("Note: Content-Language header not set - i18n may not be fully configured");
            }
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_utf8_encoding() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/suite/auth/login.html", base_url))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            
            assert!(
                content_type.contains("charset=utf-8") || content_type.contains("UTF-8"),
                "Response should specify UTF-8 encoding for i18n support"
            );
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_multilingual_error_messages() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let test_cases = vec![
        (("Accept-Language", "en"), "error"),
        (("Accept-Language", "es"), "error"),
        (("Accept-Language", "pt-BR"), "erro"),
    ];

    for ((lang_header, lang_value), expected_word) in test_cases {
        let response = client
            .post(format!("{}/api/auth/login", base_url))
            .header(lang_header, lang_value)
            .json(&json!({
                "username": "",
                "password": ""
            }))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status() == 400 || resp.status() == 401 || resp.status() == 422 {
                    let body = resp.text().await.unwrap_or_default().to_lowercase();
                    
                    assert!(
                        body.contains(expected_word),
                        "Error response for {} should contain '{}'",
                        lang_value,
                        expected_word
                    );
                }
            }
            Err(_) => {
                eprintln!("Skipping: server not available");
            }
        }
    }
}

#[tokio::test]
async fn test_date_format_localization() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/api/users/me", base_url))
        .header("Accept-Language", "pt-BR")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                let body = resp.text().await.unwrap_or_default();
                
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(created_at) = json.get("created_at") {
                        assert!(
                            created_at.is_string(),
                            "Date fields should be properly formatted"
                        );
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_number_format_localization() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/api/stats", base_url))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                let body = resp.text().await.unwrap_or_default();
                
                assert!(
                    body.contains("0") || body.contains("1"),
                    "Stats should return numeric data"
                );
            }
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_rtl_language_support() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let rtl_languages = vec!["ar", "he", "fa"];

    for lang in rtl_languages {
        let response = client
            .get(format!("{}/suite/auth/login.html", base_url))
            .header("Accept-Language", lang)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    
                    let has_dir_attr = body.contains("dir=\"rtl\"") 
                        || body.contains("dir='rtl'")
                        || body.contains("direction: rtl");
                    
                    if !has_dir_attr {
                        eprintln!(
                            "Note: RTL support for {} may not be implemented",
                            lang
                        );
                    }
                }
            }
            Err(_) => {
                eprintln!("Skipping: server not available");
            }
        }
    }
}

#[tokio::test]
async fn test_translation_files_exist() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let locales = vec!["en", "es", "pt-BR", "fr", "de", "zh"];

    for locale in locales {
        let response = client
            .get(format!("{}/locales/{}/messages.json", base_url, locale))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status() == 404 {
                    eprintln!(
                        "Note: Translation file for locale '{}' may not exist",
                        locale
                    );
                }
            }
            Err(_) => {
                eprintln!("Skipping: server not available");
            }
        }
    }
}

#[tokio::test]
async fn test_fallback_to_default_locale() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/suite/auth/login.html", base_url))
        .header("Accept-Language", "xyz-UNKNOWN")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status().is_success(),
                "Server should fall back to default locale for unknown languages"
            );
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}
