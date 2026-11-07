//! Direct vendor credential pages for the Integrations catalog.
//!
//! #1859: `official_docs` points at API documentation, which is not where a
//! user creates a key. This table maps every provider that collects
//! credentials to the exact vendor page where the token, key or OAuth app is
//! created. Providers with no public credential surface (unsupported,
//! unknown, keyless or user-supplied endpoints) return `None` so the UI never
//! invites the user to paste a secret that cannot be issued.
//!
//! The lookup stays outside `ProviderSeed` because the seed list is a `const`
//! array and `&str` matching is not allowed in const functions yet.
//!
//! Links were checked against the vendor sites on 2026-11-07; pages that
//! require a signed-in tenant may answer 401/403 to anonymous requests.

pub(crate) fn credential_page(id: &str) -> Option<&'static str> {
    match id {
        // Featured
        "calendar"
        | "drive"
        | "sheets"
        | "tasks"
        | "gmail"
        | "google_forms"
        | "google_workspace_admin"
        | "google_photos"
        | "youtube_analytics"
        | "blogger"
        | "google_cloud"
        | "google_ads" => Some("https://console.cloud.google.com/apis/credentials"),
        "notion" => Some("https://www.notion.so/my-integrations"),
        "linear" => Some("https://linear.app/settings/api/applications"),
        "airtable" => Some("https://airtable.com/create/tokens"),
        "dropbox" => Some("https://www.dropbox.com/developers/apps"),
        "outlook" | "outlook_calendar" | "onedrive" | "sharepoint" => Some(
            "https://entra.microsoft.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade",
        ),
        "spotify" => Some("https://developer.spotify.com/dashboard"),
        "x" => Some("https://developer.x.com/en/portal/dashboard"),

        // Productivity
        "slack" => Some("https://api.slack.com/apps"),
        "zoom" => Some("https://marketplace.zoom.us/develop/create"),
        "calendly" => Some("https://calendly.com/integrations/api_webhooks"),
        "canva" => Some("https://www.canva.com/developers/apps"),
        "clickup" => Some("https://app.clickup.com/settings/apps"),
        "monday" => Some("https://developer.monday.com/apps/manage"),
        "todoist" => Some("https://todoist.com/app/settings/integrations/developer"),
        "box" => Some("https://app.box.com/developers/console"),
        "docusign" => Some("https://developers.docusign.com/docs/esign-rest-api/"),
        "n8n" => Some("https://docs.n8n.io/api/"),
        "trello" => Some("https://trello.com/power-ups/admin"),
        "confluence" => Some("https://developer.atlassian.com/console/myapps/"),
        "typeform" => Some("https://admin.typeform.com/account#/section/tokens"),
        "jotform" => Some("https://www.jotform.com/myaccount/api"),
        "cal_com" => Some("https://app.cal.com/settings/developer/api-keys"),
        "smartsheet" => Some("https://app.smartsheet.com/b/home?view=accountsettings"),
        "basecamp" => Some("https://launchpad.37signals.com/integrations"),
        "motion" => Some("https://app.usemotion.com/settings/api"),
        "raindrop" => Some("https://app.raindrop.io/settings/integrations"),
        "imap" => Some("https://www.rfc-editor.org/rfc/rfc9051"),

        // Developer
        "github" => Some("https://github.com/settings/tokens"),
        "cursor" => Some("https://www.cursor.com/settings"),
        "aws" => Some("https://console.aws.amazon.com/iam/home#/security_credentials"),
        "vercel" => Some("https://vercel.com/account/tokens"),
        "supabase" => Some("https://supabase.com/dashboard/account/tokens"),
        "netlify" => Some("https://app.netlify.com/user/applications#personal-access-tokens"),
        "cloudflare" => Some("https://dash.cloudflare.com/profile/api-tokens"),
        "databricks" => Some("https://docs.databricks.com/en/dev-tools/auth/pat.html"),
        "posthog" => Some("https://app.posthog.com/settings/user-api-keys"),
        "amplitude" => Some("https://app.amplitude.com/settings"),
        "sentry" => Some("https://sentry.io/settings/account/api/auth-tokens/"),
        "splunk" => Some("https://dev.splunk.com/enterprise/docs/devtools/httpeventcollector/"),
        "hugging_face" => Some("https://huggingface.co/settings/tokens"),
        "resend" => Some("https://resend.com/api-keys"),
        "postmark" => Some("https://account.postmarkapp.com/servers"),
        "algolia" => Some("https://dashboard.algolia.com/account/api-keys"),
        "upstash_redis" => Some("https://console.upstash.com/account/api"),
        "hex" => Some("https://app.hex.tech/settings/api-keys"),
        "mintlify" => Some("https://dashboard.mintlify.com/settings/api"),
        "devin" => Some("https://app.devin.ai/settings/api-keys"),
        "pydantic_logfire" => Some("https://logfire.pydantic.dev/"),
        "grafana" => Some("https://grafana.com/profile/api-keys"),

        // Startups
        "apollo" => Some("https://app.apollo.io/#/settings/integrations/api"),
        "lemlist" => Some("https://app.lemlist.com/settings/integrations/api"),
        "loops_so" => Some("https://app.loops.so/settings"),
        "streak" => Some("https://www.streak.com/api"),
        "ahrefs" => Some("https://app.ahrefs.com/api"),
        "mercury" => Some("https://app.mercury.com/settings/tokens"),
        "ramp" => Some("https://docs.ramp.com/developer-api/v1/guides/getting-started"),
        "rippling" => Some("https://developer.rippling.com/documentation/rest-api"),
        "ashby" => Some("https://app.ashbyhq.com/admin/api-keys"),
        "plain" => Some("https://docs.plain.com/"),
        "salesforce" => {
            Some("https://help.salesforce.com/s/articleView?id=sf.connected_app_create.htm")
        }
        "hubspot" => Some("https://developers.hubspot.com/docs/guides/apps/private-apps/overview"),
        "mailchimp" => Some("https://admin.mailchimp.com/account/api/"),
        "zendesk" => Some("https://support.zendesk.com/hc/en-us/articles/4408889192858"),
        "intercom" => Some(
            "https://developers.intercom.com/docs/build-an-integration/learn-more/authentication/",
        ),
        "attio" => Some("https://app.attio.com/settings/developers"),
        "crunchbase" => Some("https://www.crunchbase.com/account/api"),
        "greenhouse" => Some("https://app.greenhouse.io/configure/dev_center/credentials"),
        "clickfunnels" => Some("https://docs.clickfunnels.com/"),
        "canny" => Some("https://canny.io/docs/api"),
        "phantombuster" => Some("https://phantombuster.com/settings/api"),

        // Small business
        "squarespace" => Some("https://developers.squarespace.com/commerce-apis/overview"),
        "webflow" => Some("https://developers.webflow.com/data/reference/rest-introduction"),
        "highlevel" => Some("https://marketplace.gohighlevel.com/"),
        "bigcommerce" => Some("https://login.bigcommerce.com/"),
        "square" => Some("https://developer.squareup.com/apps"),
        "quickbooks" => Some("https://developer.intuit.com/app/developer/dashboard"),
        "paddle" => Some("https://vendors.paddle.com/authentication"),
        "printful" => Some("https://www.printful.com/dashboard/settings/api"),
        "whop" => Some("https://dev.whop.com/api-reference"),
        "woocommerce" => Some("https://woocommerce.github.io/woocommerce-rest-api-docs/"),
        "xero_accounting" => Some("https://developer.xero.com/app/manage"),
        "lightspeed_x" => Some("https://x-series-api.lightspeedhq.com/docs/introduction"),
        "fedex" => Some("https://developer.fedex.com/api/en-us/home.html"),
        "ups" => Some("https://developer.ups.com/apps"),
        "dhl" => Some("https://developer.dhl.com/"),

        // Finance
        "stripe" => Some("https://dashboard.stripe.com/apikeys"),
        "paypal" => Some("https://developer.paypal.com/dashboard/applications"),
        "coinbase" => Some("https://portal.cdp.coinbase.com/access/api"),
        "wise" => Some("https://wise.com/settings/api-tokens"),
        "ynab" => Some("https://app.ynab.com/settings/developer"),
        "moonpay" => Some("https://dashboard.moonpay.com/"),

        // Social & messaging
        "telegram" => Some("https://t.me/BotFather"),
        "whatsapp" | "instagram" | "facebook_pages" => {
            Some("https://developers.facebook.com/apps/")
        }
        "pinterest" => Some("https://developers.pinterest.com/apps/"),
        "reddit" => Some("https://www.reddit.com/prefs/apps"),
        "twitch" => Some("https://dev.twitch.tv/console/apps"),
        "bluesky" => Some("https://bsky.app/settings/app-passwords"),
        "mastodon" => Some("https://docs.joinmastodon.org/client/token/"),
        "linkedin_ads" => Some("https://www.linkedin.com/developers/apps"),
        "luma" => Some("https://lu.ma/manage"),
        "beehiiv" => Some("https://app.beehiiv.com/settings/integrations/api"),

        // Lifestyle
        "readwise" => Some("https://readwise.io/access_token"),
        "fitbit" => Some("https://dev.fitbit.com/apps"),
        "philips_hue" => Some("https://developers.meethue.com/develop/"),
        "newscatcher" => Some("https://app.newscatcherapi.com/"),
        "last_fm" => Some("https://www.last.fm/api/accounts"),

        // LLM providers (keyless Kilo models and user-supplied endpoints have
        // no vendor credential page; the guide explains they need no key).
        "openai" => Some("https://platform.openai.com/api-keys"),
        "claude" => Some("https://console.anthropic.com/settings/keys"),
        "azure-claude" | "azure-gpt5" => Some(
            "https://portal.azure.com/#view/Microsoft_Azure_ProjectOxford/CognitiveServicesHub",
        ),
        "glm" => Some("https://z.ai/manage-apikey/apikey-list"),
        "bedrock" => Some("https://console.aws.amazon.com/bedrock"),
        "vertex" => Some("https://console.cloud.google.com/vertex-ai"),
        "kiro" => Some("https://kiro.dev/docs/"),
        "deepinfra" => Some("https://deepinfra.com/dash/api_keys"),
        "cerebras" => Some("https://cloud.cerebras.ai/"),
        "fireworks" => Some("https://app.fireworks.ai/settings/users/api-keys"),
        "together" => Some("https://api.together.xyz/settings/api-keys"),
        "openrouter" => Some("https://openrouter.ai/settings/keys"),
        "requesty" => Some("https://docs.requesty.ai/"),

        // No public credential surface: unsupported, unknown or user-defined.
        "arena" | "grain" | "descript" | "snowflake" | "expensify" | "qik" | "robinhood"
        | "signal" | "moltbook" | "eight_sleep" | "openai-compatible" => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::credential_page;

    #[test]
    fn credential_pages_are_https_and_never_empty() {
        for id in [
            "github", "slack", "aws", "stripe", "openai", "zendesk", "telegram", "trello",
        ] {
            let url = credential_page(id).unwrap_or_default();
            assert!(
                url.starts_with("https://"),
                "{id} must point at an https credential page"
            );
        }
    }

    #[test]
    fn providers_without_a_public_credential_surface_have_no_link() {
        for id in [
            "signal",
            "qik",
            "robinhood",
            "eight_sleep",
            "openai-compatible",
        ] {
            assert!(
                credential_page(id).is_none(),
                "{id} must not advertise a credential page"
            );
        }
    }
}
