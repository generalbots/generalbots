//! Numbered credential setup guides shared by the catalog auth profiles.
//!
//! #1854: every provider must explain, step by step, where to create the
//! credential it needs. The steps live here instead of inside `auth.rs` so
//! that file stays under the repository's size limit and the same guide is
//! reused by every provider that shares an auth method.

pub(crate) const OAUTH2_STEPS: &[&str] = &[
    "Open the vendor's developer console and create an OAuth 2.0 app, or use the managed app when the vendor offers one.",
    "Add this workspace's callback URL to the app's allowed redirect URIs.",
    "Copy the client ID and client secret into the fields above, then start the authorization flow and approve the requested scopes.",
    "Review the granted scopes after connecting; revoke and reconnect if broader access than needed was approved.",
];

pub(crate) const GOOGLE_OAUTH2_STEPS: &[&str] = &[
    "In Google Cloud Console create or select a project and enable the API for this provider.",
    "Configure the OAuth consent screen and create an OAuth 2.0 Client ID of type Web application.",
    "Add this workspace's callback URL to the authorized redirect URIs, then paste the client ID and secret here.",
    "Authorize with the Google account that owns the data, granting only the scopes required by the actions you enable.",
];

pub(crate) const MICROSOFT_OAUTH2_STEPS: &[&str] = &[
    "In the Microsoft Entra admin center open App registrations and create (or select) an application.",
    "Add a Web redirect URI for this workspace's callback URL under Authentication.",
    "Create a client secret, then paste the application (client) ID and secret here.",
    "Under API permissions add the delegated Microsoft Graph scopes needed by the enabled actions and grant consent for the tenant.",
];

pub(crate) const INSTAGRAM_OAUTH2_STEPS: &[&str] = &[
    "Create a Meta app at developers.facebook.com and add the Instagram Graph API product.",
    "Link a Business or Creator Instagram account to a Facebook Page and grant the app access to that Page.",
    "Add this workspace's callback URL, then paste the Meta app ID and secret here and authorize the Page.",
    "Request only instagram_basic, instagram_content_publish, pages_show_list and pages_read_engagement.",
];

pub(crate) const API_KEY_STEPS: &[&str] = &[
    "Open the vendor console through the documentation link below and sign in to the workspace that owns this integration.",
    "Create a dedicated API key with the narrowest scope required by the actions you enable.",
    "Paste the key here; it is stored in the secrets vault and is never sent to the language model.",
    "Record the key's expiry and rotation policy, then rotate it before it expires.",
];

pub(crate) const KEYLESS_STEPS: &[&str] = &[
    "No API key is required: this provider is served through a free gateway.",
    "Select the provider and save the connection; requests use the shared free-tier endpoint.",
    "If the free tier is rate limited, create a paid key from the vendor's documentation page and switch to that provider.",
];

pub(crate) const TOKEN_STEPS: &[&str] = &[
    "Open the vendor console through the documentation link below and sign in to the account that owns this integration.",
    "Create a dedicated access token with only the scopes required by the enabled actions.",
    "Set an expiry when the vendor supports one, then paste the token here; it is stored in the secrets vault.",
    "Rotate the token before it expires and revoke it immediately if it leaks.",
];

pub(crate) const BASIC_STEPS: &[&str] = &[
    "Create a dedicated integration or API user in the vendor's admin console.",
    "Grant that user only the permissions required by the enabled actions.",
    "Enter the username and password or API secret here; the credentials stay in the secrets vault.",
];

pub(crate) const IMAP_STEPS: &[&str] = &[
    "Confirm the mail provider's IMAP host and port (993 with TLS) from its help pages.",
    "Enable IMAP access and create an app password or OAuth credential for this mailbox.",
    "Enter the host, port and mailbox username, then paste the password or token here.",
    "Prefer a dedicated or read-only mailbox unless message mutation is explicitly enabled.",
];

pub(crate) const AWS_STEPS: &[&str] = &[
    "Prefer workload identity or an assumed STS role. Use an IAM user only when workload identity or an STS role is unavailable.",
    "Create or select an IAM principal and attach a minimum custom IAM policy scoped to the exact actions and resource ARNs required.",
    "Never use root or AdministratorAccess. Generate an access key (and optional session token) only for that least-privilege principal.",
    "Paste the access key ID and secret access key here; credentials stay in the secrets vault and never enter LLM context.",
];

pub(crate) const TELEGRAM_STEPS: &[&str] = &[
    "Open Telegram, message @BotFather and run /newbot to create a dedicated bot.",
    "Copy the bot token that @BotFather returns and paste it here; it is stored in the secrets vault.",
    "Optionally set a webhook secret so unsigned webhook deliveries are rejected.",
    "Disable unused commands and inline keyboards before going live.",
];

pub(crate) const WHATSAPP_STEPS: &[&str] = &[
    "Create a Meta app at developers.facebook.com and add the WhatsApp Business product.",
    "Register a WhatsApp Business phone number and copy its Phone Number ID and WABA ID.",
    "Create a permanent System User access token scoped to that phone number and paste it here.",
    "Set a webhook verify token, subscribe the callback URL and enable only the webhook fields you need.",
];

pub(crate) const ZENDESK_TOKEN_STEPS: &[&str] = &[
    "In Zendesk Admin Center open Apps and integrations, then APIs, and create a Zendesk API token.",
    "Copy the tenant subdomain (acme from acme.zendesk.com) and the account email that owns the token.",
    "Paste the subdomain, email and token here; they are used for Basic {email}/token authentication over TLS.",
    "Use a dedicated agent account whose role covers only the ticket operations you enable.",
];

pub(crate) const TRELLO_KEY_TOKEN_STEPS: &[&str] = &[
    "Open trello.com/power-ups/admin, create or select a Power-Up and generate its API key.",
    "Use the token link next to the key to generate a member token for that key.",
    "Paste both the key and the token here; the token is scoped to the boards used by the enabled actions.",
];

pub(crate) const OPENAI_COMPATIBLE_STEPS: &[&str] = &[
    "Read your provider's API documentation for its chat-completions URL, model name and authentication scheme.",
    "Enter the full endpoint URL (ending in /v1/chat/completions) and the exact model identifier.",
    "Add the API key if the endpoint requires one; leave it empty for keyless local or self-hosted endpoints.",
    "Save the connection and send a test prompt to confirm the endpoint supports tool calling.",
];

pub(crate) const UNKNOWN_STEPS: &[&str] = &[
    "This catalog entry has no verified authentication flow yet.",
    "Read the vendor documentation link below before collecting any credentials.",
    "Do not paste secrets until the supported auth method and minimum scopes are confirmed.",
];

pub(crate) const UNSUPPORTED_STEPS: &[&str] = &[
    "The vendor does not publish a supported public API for this integration.",
    "Use the documentation link below to check whether an official API or partner program has become available.",
    "Do not collect or store credentials for this provider.",
];
