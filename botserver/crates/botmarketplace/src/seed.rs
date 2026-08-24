use diesel::prelude::*;
use diesel::sql_types::BigInt as SqlBigInt;

use crate::b64;
use crate::blobstore;
use crate::models::PublishBody;
use crate::publish::upsert_package_and_version;
use crate::seed_data::{starter_skills, StarterSkill};
use crate::MarketplaceService;

const PUBLISHER_NAME: &str = "General Bots";
const STARTER_VERSION: &str = "0.1.0";

fn bundle_json(skill: &StarterSkill) -> serde_json::Value {
    let scripts: serde_json::Map<String, serde_json::Value> = skill
        .scripts
        .iter()
        .map(|(name, content)| (name.to_string(), serde_json::Value::String(content.to_string())))
        .collect();
    let script_names: Vec<&str> = skill.scripts.iter().map(|(name, _)| *name).collect();
    serde_json::json!({
        "manifest": {
            "entry": skill.entry,
            "scripts": script_names,
            "prompts": skill.prompts,
            "permissions": skill.permissions,
        },
        "scripts": scripts,
    })
}

/// Builds the 10 starter PublishBodies with base64-encoded .gbskill bundles.
pub fn seed_starter_skills() -> Vec<PublishBody> {
    starter_skills()
        .iter()
        .map(|skill| {
            let bundle = bundle_json(skill);
            PublishBody {
                slug: skill.slug.to_string(),
                name: skill.name.to_string(),
                description: Some(skill.description.to_string()),
                tags: serde_json::json!(skill.tags),
                icon_glyph: Some(skill.icon_glyph.to_string()),
                version: STARTER_VERSION.to_string(),
                changelog: Some("Initial release".to_string()),
                manifest: bundle.get("manifest").cloned().unwrap_or_default(),
                content_base64: b64::encode_standard(bundle.to_string().as_bytes()),
                visibility: Some("public".to_string()),
            }
        })
        .collect()
}

#[derive(diesel::QueryableByName, Debug)]
struct CountRow {
    #[diesel(sql_type = SqlBigInt)]
    count: i64,
}

/// Seeds the catalog once when `skill_packages` is empty. Returns seeded count.
pub async fn seed_if_empty(service: &MarketplaceService) -> Result<usize, String> {
    let mut conn = service.pool.get().map_err(|e| format!("DB pool: {e}"))?;

    let count: CountRow = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM skill_packages")
        .get_result(&mut conn)
        .map_err(|e| format!("Count packages: {e}"))?;
    if count.count > 0 {
        return Ok(0);
    }

    let mut seeded = 0usize;
    for body in seed_starter_skills() {
        let content = b64::decode_flexible(&body.content_base64)
            .ok_or_else(|| format!("Starter skill {} has invalid bundle", body.slug))?;
        blobstore::put_package(&service.mc_bin, &service.mc_alias, &body.slug, &body.version, &content)?;
        upsert_package_and_version(&mut conn, &body, None, Some(PUBLISHER_NAME.to_string()))
            .map_err(|e| format!("Seed {}: {e}", body.slug))?;
        seeded += 1;
    }

    tracing::info!("Seeded {seeded} starter marketplace skills");
    Ok(seeded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_ten_well_formed_starter_skills() {
        let bodies = seed_starter_skills();
        assert_eq!(bodies.len(), 10);
        let slugs: Vec<&str> = bodies.iter().map(|b| b.slug.as_str()).collect();
        for expected in [
            "expense-parser",
            "meeting-minutes",
            "lead-qualifier",
            "invoice-qa",
            "site-monitor",
            "email-digest",
            "kb-quizmaster",
            "social-drafter",
            "csv-cleaner",
            "webhook-fanout",
        ] {
            assert!(slugs.contains(&expected), "missing starter skill {expected}");
        }
        for body in &bodies {
            assert!(crate::blobstore::valid_slug(&body.slug));
            assert_eq!(body.visibility.as_deref(), Some("public"));
            assert!(!body.content_base64.is_empty());
            let manifest = &body.manifest;
            assert!(manifest.get("entry").and_then(|v| v.as_str()).is_some());
            assert!(manifest.get("permissions").and_then(|v| v.as_array()).is_some());
        }
    }

    #[test]
    fn bundles_decode_back_to_scripts() {
        for body in seed_starter_skills() {
            let bytes = b64::decode_flexible(&body.content_base64).unwrap();
            let bundle: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let scripts = bundle.get("scripts").and_then(|s| s.as_object()).unwrap();
            assert!(scripts.len() >= 2, "{} needs at least 2 scripts", body.slug);
            for (name, content) in scripts {
                let text = content.as_str().unwrap();
                assert!(
                    text.lines().all(|l| l.chars().count() <= 80),
                    "{}/{} has an over-long line",
                    body.slug,
                    name
                );
                assert!(text.contains("TALK") || text.contains("HEAR"), "{name} is not BASIC");
            }
        }
    }
}
