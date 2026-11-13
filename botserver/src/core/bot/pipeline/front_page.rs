//! #2147 — newspaper front pages delivered as real images.
//!
//! "Me dá as capas dos jornais brasil247 e o globo foto" was answered with
//! prose about paywalls. Scraping a cover URL cannot work: both sites serve a
//! placeholder (`og:image`, `wire-edicao-capa.png`) and the real printed cover
//! sits behind the subscriber edition. The honest way to show a front page is
//! to render it: the home page is opened in the browser, the PNG is stored in
//! the bot's Drive and shared by token, and the model receives the exact
//! markdown image to print — it never has to guess a URL, and the reader never
//! gets a broken hotlink that 403s tomorrow.
//!
//! A capture that fails (no browser, timeout, a login wall) is reported as
//! such: the block tells the model to give the publication's link and say why,
//! so neither newspaper is silently dropped.

use std::sync::Arc;
use std::time::Duration;

use botcore::shared::state::AppState;
use botlib::models::BotResponse;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::SessionIdentity;

/// How long one front page may take to open and render.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
/// The share link outlives every plausible conversation; the durable copy is
/// the Drive file the answer also links to.
const LINK_DAYS: i64 = 30;
/// A real page capture is at least this big. Anything smaller is a logo, an
/// icon or a "failed to load" tile, which must never be presented as the
/// front page.
const MIN_WIDTH: u32 = 800;
const MIN_HEIGHT: u32 = 300;

struct Outlet {
    name: &'static str,
    home: &'static str,
    aliases: &'static [&'static str],
}

/// Publications whose front page means "the home page rendered now". Kept
/// short and explicit: a wrong URL here would put a wrong picture in the
/// answer.
const OUTLETS: &[Outlet] = &[
    Outlet {
        name: "Brasil 247",
        home: "https://www.brasil247.com/",
        aliases: &["brasil 247", "brasil247"],
    },
    Outlet {
        name: "O Globo",
        home: "https://oglobo.globo.com/",
        aliases: &["o globo", "oglobo"],
    },
    Outlet {
        name: "Folha de S.Paulo",
        home: "https://www1.folha.uol.com.br/",
        aliases: &["folha de s.paulo", "folha de sao paulo", "folha de são paulo"],
    },
    Outlet {
        name: "Estadão",
        home: "https://www.estadao.com.br/",
        aliases: &["estadão", "estadao"],
    },
    Outlet {
        name: "Correio Braziliense",
        home: "https://www.correiobraziliense.com.br/",
        aliases: &["correio braziliense"],
    },
    Outlet {
        name: "Zero Hora",
        home: "https://gauchazh.clicrbs.com.br/",
        aliases: &["zero hora"],
    },
];

/// Words that ask for the *printed front page as an image*, not for a summary
/// of today's news.
const FRONT_PAGE_WORDS: &[&str] = &[
    "capa",
    "front page",
    "frontpage",
    "primeira página",
    "primeira pagina",
    "1ª página",
    "1a pagina",
];

fn mentions_image(text: &str) -> bool {
    ["foto", "fotos", "imagem", "imagens", "photo", "image", "picture"]
        .iter()
        .any(|word| text.contains(word))
}

fn opened_outlets(text: &str) -> Vec<&'static Outlet> {
    let lower = text.to_lowercase();
    OUTLETS
        .iter()
        .filter(|outlet| outlet.aliases.iter().any(|alias| lower.contains(alias)))
        .collect()
}

/// True when the turn asks for one or more newspaper front pages as pictures.
pub fn wants_front_page(text: &str) -> bool {
    let lower = text.to_lowercase();
    if opened_outlets(text).is_empty() {
        return false;
    }
    FRONT_PAGE_WORDS.iter().any(|word| lower.contains(word))
        || ((lower.contains("jornal") || lower.contains("newspaper")) && mentions_image(&lower))
}

struct Captured {
    outlet: &'static str,
    home: &'static str,
    image_url: String,
    drive_key: String,
    captured_at: String,
}

/// Reads a big-endian u32 out of a byte slice without panicking on short
/// input.
fn be_u32(bytes: &[u8], at: usize) -> u32 {
    match bytes.get(at..at + 4) {
        Some(slice) => u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]),
        None => 0,
    }
}

/// Accepts only a real page capture: PNG signature plus plausible dimensions.
fn validate_png(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if bytes.len() < 24 || !bytes.starts_with(&SIGNATURE[..]) {
        return Err("the capture was not a PNG image".to_string());
    }
    let width = be_u32(&bytes, 16);
    let height = be_u32(&bytes, 20);
    if width < MIN_WIDTH || height < MIN_HEIGHT {
        return Err(format!(
            "the capture was a {width}x{height} placeholder, not a page"
        ));
    }
    Ok(bytes)
}

/// Renders one home page and returns its PNG.
async fn capture(home: &str) -> Result<Vec<u8>, String> {
    let session = match tokio::time::timeout(
        CAPTURE_TIMEOUT,
        crate::browser::BrowserSession::new_with_meta(
            true,
            crate::browser::BrowserTabMeta::default(),
        ),
    )
    .await
    {
        Ok(Ok(session)) => session,
        Ok(Err(error)) => return Err(format!("the capture browser is unavailable ({error})")),
        Err(_) => return Err("the capture browser did not start in time".to_string()),
    };

    let shot = tokio::time::timeout(CAPTURE_TIMEOUT, async {
        session
            .navigate(home)
            .await
            .map_err(|error| format!("the page did not open ({error})"))?;
        // Hero images are lazy-loaded after the navigation event.
        tokio::time::sleep(Duration::from_millis(2500)).await;
        session
            .screenshot()
            .await
            .map_err(|error| format!("the screenshot failed ({error})"))
    })
    .await;
    let _ = session.close().await;

    match shot {
        Ok(Ok(bytes)) => validate_png(bytes),
        Ok(Err(reason)) => Err(reason),
        Err(_) => Err("the page took too long to render".to_string()),
    }
}

/// Stores the capture in the bot's Drive and returns the drive key plus a
/// token-shared URL the chat can render (`/api/files/public/{token}`).
async fn publish(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    name: &str,
    png: Vec<u8>,
) -> Result<(String, String), String> {
    let saved = crate::core::bot::drive_bridge::save_object(state, &bot_uuid, "front-pages", name, png)
        .await
        .ok_or_else(|| "the capture could not be stored in Drive".to_string())?;
    let (bucket, key) = saved
        .split_once('/')
        .ok_or_else(|| "the stored capture has no Drive path".to_string())?;

    use diesel::sql_types::{Nullable, Text, Timestamptz};
    use diesel::RunQueryDsl;
    let token = Uuid::new_v4().simple().to_string();
    let expires = chrono::Utc::now() + chrono::Duration::days(LINK_DAYS);
    let mut conn = state
        .conn
        .get()
        .map_err(|error| format!("the database is unavailable ({error})"))?;
    diesel::sql_query(
        "INSERT INTO drive_public_links (owner_id, bucket, path, key, scope, share_token, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<Text, _>(user_id.to_string())
    .bind::<Text, _>(bucket)
    .bind::<Text, _>(name)
    .bind::<Text, _>(key)
    .bind::<Text, _>("bot")
    .bind::<Text, _>(&token)
    .bind::<Nullable<Timestamptz>, _>(Some(expires))
    .execute(&mut conn)
    .map_err(|error| {
        log::error!("front page: share link insert failed: {error}");
        "the capture could not be shared".to_string()
    })?;

    Ok((key.to_string(), format!("/api/files/public/{token}")))
}

async fn status(
    sink: &dyn ChannelSink,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) {
    if sink.supports_raw_frames() {
        let frame = serde_json::json!({ "type": "chat_status", "text": text });
        if sink.send_raw_json(&frame).await.is_ok() {
            return;
        }
    }
    let response = BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: String::new(),
        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: false,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: format!("{text}\n"),
    };
    let _ = sink.send_bot_response(&response).await;
}

fn file_name(outlet: &str) -> String {
    let slug = crate::core::bot::drive_bridge::slug(outlet).to_lowercase();
    format!(
        "capa-{}.png",
        if slug.is_empty() { "jornal" } else { &slug }
    )
}

/// Captures every newspaper the turn names and injects the exact markdown the
/// answer must print. Returns true when the turn was a front-page request.
pub async fn capture_and_inject(
    identity: &SessionIdentity<'_>,
    user_text: &str,
    messages: &mut Vec<serde_json::Value>,
) -> bool {
    if identity.sink.channel_type() != "web" || !wants_front_page(user_text) {
        return false;
    }
    let outlets = opened_outlets(user_text);
    if outlets.is_empty() {
        return false;
    }
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;

    let captured_at = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut captured: Vec<Captured> = Vec::new();
    let mut failed: Vec<(&'static str, &'static str, String)> = Vec::new();

    for outlet in outlets {
        status(
            sink,
            bot_uuid,
            session_id,
            user_id,
            &format!("Abrindo a capa de {} e tirando a foto…", outlet.name),
        )
        .await;
        match capture(outlet.home).await {
            Ok(png) => {
                match publish(state, bot_uuid, user_id, &file_name(outlet.name), png).await {
                    Ok((drive_key, image_url)) => captured.push(Captured {
                        outlet: outlet.name,
                        home: outlet.home,
                        image_url,
                        drive_key,
                        captured_at: captured_at.clone(),
                    }),
                    Err(reason) => failed.push((outlet.name, outlet.home, reason)),
                }
            }
            Err(reason) => {
                log::info!("front page: {} capture failed: {reason}", outlet.name);
                failed.push((outlet.name, outlet.home, reason));
            }
        }
    }

    let mut block = String::from(
        "NEWSPAPER FRONT PAGES (captured by the server for this turn)\n\
         The user asked for newspaper front pages as images. The pictures below \
         were rendered from each publication's own home page minutes ago. They \
         are the only valid front pages for this answer.\n\n",
    );
    for item in &captured {
        block.push_str(&format!(
            "- ![Capa de {} — {}]({})\n  outlet: {}\n  front page: {}\n  published: {}\n  drive: [{}](app://drive?path={}&scope=bot)\n\n",
            item.outlet,
            item.captured_at,
            item.image_url,
            item.outlet,
            item.home,
            item.captured_at,
            item.drive_key,
            urlencoding::encode(&item.drive_key),
        ));
    }
    if !failed.is_empty() {
        block.push_str("CAPTURES THAT FAILED\n");
        for (name, home, reason) in &failed {
            block.push_str(&format!("- {name}: {reason} (front page: {home})\n"));
        }
    }
    block.push_str(
        "\nRULES\n\
         - Print each captured image EXACTLY as given, on its own line: same \
         markdown, same URL, same alt text. Never rewrite, resize or wrap an \
         image in a code fence.\n\
         - Under each image write ONE line in the user's language: publication \
         name, the capture date, and the link to the publication's front page. \
         Add the Drive link only as a secondary 'saved copy' link.\n\
         - Never invent an image URL, a cover, or a different picture. Never \
         answer with a description of the layout or with a link alone when the \
         image is above.\n\
         - For every publication under CAPTURES THAT FAILED, say in one line \
         why it could not be shown and give the reader its front-page link. \
         Never drop a publication silently and never replace a missing image \
         with a screenshot of something else.\n",
    );
    messages.push(serde_json::json!({ "role": "system", "content": block }));
    log::info!(
        "front page: {} captured, {} failed for bot {bot_uuid}",
        captured.len(),
        failed.len()
    );
    true
}

#[cfg(test)]
mod front_page_tests {
    use super::{be_u32, validate_png, wants_front_page};

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes
    }

    #[test]
    fn the_uat_turn_is_recognised() {
        assert!(wants_front_page(
            "Me dá as capas dos jornais brasil247 e o globo foto"
        ));
        assert!(wants_front_page("manda a capa do O Globo de hoje"));
        assert!(wants_front_page("qual a front page do Brasil 247?"));
    }

    #[test]
    fn ordinary_questions_are_not_captured() {
        for turn in [
            "Notícias do Lula",
            "o que o O Globo publicou hoje?",
            "resuma as manchetes do Estadão",
            "capa do meu relatório financeiro",
        ] {
            assert!(!wants_front_page(turn), "unexpected capture: {turn}");
        }
    }

    #[test]
    fn a_real_capture_passes_and_a_placeholder_does_not() {
        let page = png_bytes(1280, 720);
        assert!(validate_png(page).is_ok());
        // A 120x60 logo is not a front page.
        assert!(validate_png(png_bytes(120, 60)).is_err());
        // An empty or non-PNG body is never the answer.
        assert!(validate_png(Vec::new()).is_err());
        assert!(validate_png(b"<html>denied</html>".to_vec()).is_err());
    }

    #[test]
    fn short_slices_do_not_panic() {
        assert_eq!(be_u32(&[1, 2], 0), 0);
        assert_eq!(be_u32(&[0, 0, 1, 0], 0), 256);
    }
}
