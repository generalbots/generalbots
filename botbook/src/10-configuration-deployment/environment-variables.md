# Environment Variables 🟡 BETA

Every environment variable the product reads, derived from the source tree. This is a **name reference** — it never contains values. Bot-level settings belong in [`config.csv`](./config-csv.md) and are listed in [Bot Parameters](./parameters.md); credentials belong in Vault, not in `.env` (see [Secrets Management](./secrets-management.md)).

Resolution order for most secrets: **Vault → environment → built-in default**. A few desktop/offline paths intentionally read the environment first because no Vault is present. The canonical examples are `.env.example`, `botserver/.env.example` and `botmodels/.env.example`.

## Runtime essentials

| Variable | Purpose | Source |
|---|---|---|
| `VAULT_ADDR` | Vault server address used by the secret resolver. | `botserver/src/main_body.rs` |
| `VAULT_TOKEN` | Vault token for the local bootstrap process (never commit a value). | `botserver/src/main_body.rs` |
| `VAULT_CACERT` | Path to the Vault CA certificate used for TLS verification. | `botserver/src/main_module/wizard_provision.rs` |
| `VAULT_SKIP_VERIFY` | Disable Vault TLS verification (development only). | `botserver/crates/botcoresecrets/src/manager.rs` |
| `VAULT_CACHE_TTL` | Secret cache lifetime in seconds (default 300). | `botserver/crates/botcoresecrets/src/manager.rs` |
| `DATABASE_URL` | PostgreSQL connection string; production credentials come from Vault gbo/tables. | `botserver/crates/botmonitoring/src/lib.rs` |
| `CACHE_URL` | Valkey/Redis connection string; credentials come from Vault gbo/cache. | `botserver/src/main_module/cache.rs` |
| `JWT_SECRET` | Legacy/general JWT secret fallback when no SaaS secret is configured. | `botserver/crates/botsecurity-crypto/src/encryption/master_key_lock.rs` |
| `SAAS_JWT_SECRET` | Fallback HMAC secret for SaaS/cloud tokens; the persisted value wins. | `.env.example` |
| `GB_ALLOW_DEV_JWT_SECRET` | Opt into the public development JWT constant (debug builds only, never production). | `.env.example` |
| `BOTSERVER_STACK_PATH` | Root of the runtime service stack (desktop/offline installs). | `botserver/src/main_body.rs` |
| `BOTSERVER_BIN` | Explicit path to the botserver executable (desktop sidecar and tests). | `bottest/src/harness/start.rs` |
| `BOTSERVER_DISABLE_TLS` | Disable TLS in botserver (throwaway audit/offline profiles only). | `botserver/src/main_module/listen_impl.rs` |
| `BOTUI_PORT` | Suite static server port. | `botui/src/main.rs` |
| `BOTUI_CLOUD_PORT` | Cloud static server port. | `botui/src/main.rs` |
| `BOTUI_LOGIN_PORT` | Login static server port. | `botui/src/main.rs` |
| `GB_LOGIN_URL` | Login origin injected into suite and cloud pages. | `botserver/src/main_module/routes/cloud_sso_handler/pages.rs` |
| `GB_PUBLIC_URL` | Public origin advertised to clients. | `botserver/crates/botdrive/src/drive_handlers/share_email.rs` |
| `LLM_URL` | Chat-completions endpoint used when no Vault LLM entry exists. | `botserver/src/main_module/health.rs` |
| `LLM_MODEL` | Model identifier/path used when no Vault LLM entry exists. | `botserver/src/main_module/state_builder.rs` |
| `LLM_KEY` | API key used when no Vault LLM entry exists. | `botserver/src/main_module/state_builder.rs` |
| `RUST_LOG` | Rust log filter directive. | `botserver/src/main_body.rs` |

## Desktop, appliance and agent VMs

| Variable | Purpose | Source |
|---|---|---|
| `GB_DESKTOP_ALLOW_DOWNLOAD` | Allow first-run downloads in the desktop installer. | `botserver/crates/botcore/src/package_manager/facade/mod.rs` |
| `GB_DESKTOP_LOCAL_USER_ID` | User id used for desktop-local ownership. | `botserver/src/sheet/user_middleware.rs` |
| `GB_DESKTOP_MODE` | Enable desktop-local product behaviour. | `botapp/src/desktop/wizard/paths.rs` |
| `GB_DESKTOP_OFFLINE` | Refuse network downloads in desktop mode. | `botapp/src/desktop/sidecar.rs` |
| `GB_DESKTOP_SUITE_URL` | Suite origin used by the desktop webview. | `botapp/src/desktop/wizard/window.rs` |
| `GBO_NO_DOTENV` | Gbo no dotenv. | `botserver/src/main_body.rs` |
| `GBO_SKIP_DIRECTORY_WAIT` | In through admin-credentials. | `botserver/src/main_module/state_builder.rs` |
| `GBO_SKIP_LOCAL_DIRECTORY` | Gbo skip local directory. | `botserver/src/main_module/state_builder.rs` |
| `GBO_STACK_PATH` | Alternate stack root used by the offline installer. | `botserver/crates/botcoresecrets/src/manager.rs` |
| `GBO_WORK_PATH` | In production (system container): /opt/gbo/work in development (with botserver-stack directory):. | `botserver/crates/botcore/src/shared/utils/init_secrets_manager.rs` |
| `GBO_WSL_DISTRO` | Gbo wsl distro. | `botserver/src/api/terminal/session.rs` |
| `BOTSERVER_INSTALLERS_PATH` | Cache directory of vendored installer archives for offline provisioning. | `botserver/crates/botcorepkg/src/cache/mod.rs` |
| `AGENT_ALLOW_UNREGISTERED_IDENTITIES` | Allow unregistered identities in single-user/desktop installs. | `botserver/crates/botagent/src/vm/identity.rs` |
| `AGENT_IDLE_TIMEOUT_SECS` | Seconds an idle agent VM stays alive. | `botserver/crates/botagent/src/lib.rs` |
| `AGENT_INCUS_BIN` | Path to the Incus CLI used by the agent VM manager. | `botserver/crates/botagent/src/codex/bridge_live_tests.rs` |
| `AGENT_MAX_SNAPSHOTS_PER_SESSION` | Snapshot cap per agent VM session. | `botserver/crates/botagent/src/lib.rs` |
| `AGENT_VM_BRIDGE` | Incus bridge used by agent VMs. | `botserver/crates/botagent/src/vm/launch.rs` |
| `AGENT_VM_CPU` | CPU allocation for an agent VM. | `botserver/crates/botagent/src/vm/launch.rs` |
| `AGENT_VM_LAUNCH_WAIT_SECS` | Agent vm launch wait secs. | `botserver/crates/botagent/src/vm/incus.rs` |
| `AGENT_VM_MEMORY` | Memory allocation for an agent VM. | `botserver/crates/botagent/src/vm/launch.rs` |
| `GB_KIOSK` | Run the kiosk/appliance shell profile. | `botapp/src/desktop/mod.rs` |
| `GB_OS_CHANNEL` | Update channel recorded for the OS appliance. | `botserver/crates/botsettings/src/os_appliance.rs` |
| `GB_UPDATE_CHANNEL` | Update channel used by the updater/provisioner. | `botserver/crates/botsettings/src/os_appliance.rs` |

## BotModels service

These are read by the Python BotModels service from its own `.env`; the server reaches that service through `BOTMODELS_URL` / `BOTMODELS_API_KEY`.

| Variable | Purpose | Source |
|---|---|---|
| `ACESTEP_API_KEY` | Acestep api key. | `botmodels/.env.example` |
| `ACESTEP_API_URL` | Acestep api url. | `botmodels/.env.example` |
| `ACESTEP_AUDIO_TIMEOUT` | Acestep audio timeout. | `botmodels/.env.example` |
| `ACESTEP_REQUEST_TIMEOUT` | Acestep request timeout. | `botmodels/.env.example` |
| `ALLOWED_ORIGINS` | Comma-separated CORS origins for BotModels. | `botmodels/.env.example` |
| `ALLOW_REMOTE_SPEECH` | Opt in to remote speech providers (off by default). | `botmodels/.env.example` |
| `API_KEY` | BotModels API key (change from the example value in production). | `botmodels/.env.example` |
| `BOTMODELS_STT_ENGINE` | Botmodels stt engine. | `botmodels/.env.example` |
| `BOTMODELS_STT_MODEL` | Botmodels stt model. | `botmodels/.env.example` |
| `BOTMODELS_TTS_ENGINE` | Botmodels tts engine. | `botmodels/.env.example` |
| `BOTMODELS_TTS_MODEL` | Botmodels tts model. | `botmodels/.env.example` |
| `BOTMODELS_TTS_PATH` | Botmodels tts path. | `botmodels/.env.example` |
| `DEVICE` | BotModels compute device override (cuda/mps/cpu). | `botmodels/.env.example` |
| `ENV` | Env. | `botmodels/.env.example` |
| `HOST` | Host. | `botmodels/.env.example` |
| `IMAGE_HEIGHT` | Image height. | `botmodels/.env.example` |
| `IMAGE_MODEL_PATH` | Image model path. | `botmodels/.env.example` |
| `IMAGE_STEPS` | Image steps. | `botmodels/.env.example` |
| `IMAGE_WIDTH` | Image width. | `botmodels/.env.example` |
| `LOG_LEVEL` | Log level. | `botmodels/.env.example` |
| `MODE` | BotModels model tier: auto, min or max. | `botmodels/.env.example` |
| `OCR_MODEL_PATH` | Ocr model path. | `botmodels/.env.example` |
| `OUTPUT_DIR` | BotModels generated-file output directory. | `botmodels/.env.example` |
| `PORT` | Port. | `botserver/crates/botcore/src/config/load.rs` |
| `REALTIME_AUDIO_MODEL_PATH` | Realtime audio model path. | `botmodels/.env.example` |
| `STT_MODEL_PATH` | Stt model path. | `botmodels/.env.example` |
| `TTS_MODEL_PATH` | Tts model path. | `botmodels/.env.example` |
| `VIDEO_FPS` | Video fps. | `botmodels/.env.example` |
| `VIDEO_FRAMES` | Video frames. | `botmodels/.env.example` |
| `VIDEO_HEIGHT` | Video height. | `botmodels/.env.example` |
| `VIDEO_MODEL_PATH` | Video model path. | `botmodels/.env.example` |
| `VIDEO_STEPS` | Video steps. | `botmodels/.env.example` |
| `VIDEO_WIDTH` | Video width. | `botmodels/.env.example` |
| `VISION_MODEL_PATH` | Vision model path. | `botmodels/.env.example` |

## Complete index

349 names read in Rust plus the BotModels Python service. The source column is the first read site; grep the repository for the name to see every consumer.

| Variable | Purpose | Source |
|---|---|---|
| `ACESTEP_API_KEY` | Acestep api key. | `botmodels/.env.example` |
| `ACESTEP_API_URL` | Acestep api url. | `botmodels/.env.example` |
| `ACESTEP_AUDIO_TIMEOUT` | Acestep audio timeout. | `botmodels/.env.example` |
| `ACESTEP_REQUEST_TIMEOUT` | Acestep request timeout. | `botmodels/.env.example` |
| `AGENT_ALLOW_UNREGISTERED_IDENTITIES` | Allow unregistered identities in single-user/desktop installs. | `botserver/crates/botagent/src/vm/identity.rs:91` |
| `AGENT_IDLE_TIMEOUT_SECS` | Seconds an idle agent VM stays alive. | `botserver/crates/botagent/src/lib.rs:149` |
| `AGENT_INCUS_BIN` | Path to the Incus CLI used by the agent VM manager. | `botserver/crates/botagent/src/codex/bridge_live_tests.rs:24` |
| `AGENT_MAX_SNAPSHOTS_PER_SESSION` | Snapshot cap per agent VM session. | `botserver/crates/botagent/src/lib.rs:145` |
| `AGENT_VM_BRIDGE` | Incus bridge used by agent VMs. | `botserver/crates/botagent/src/vm/launch.rs:43` |
| `AGENT_VM_CPU` | CPU allocation for an agent VM. | `botserver/crates/botagent/src/vm/launch.rs:41` |
| `AGENT_VM_LAUNCH_WAIT_SECS` | Agent vm launch wait secs. | `botserver/crates/botagent/src/vm/incus.rs:150` |
| `AGENT_VM_MEMORY` | Memory allocation for an agent VM. | `botserver/crates/botagent/src/vm/launch.rs:42` |
| `ALLOWED_ORIGINS` | Comma-separated CORS origins for BotModels. | `botmodels/.env.example` |
| `ALLOW_REMOTE_SPEECH` | Opt in to remote speech providers (off by default). | `botmodels/.env.example` |
| `ALM_TOKEN` | Legacy name for the ALM token. | `botserver/crates/botcoresecrets/src/tenant.rs:169` |
| `ALM_URL` | Legacy name for the ALM/Forgejo origin. | `botserver/crates/botcoresecrets/src/tenant.rs:164` |
| `API_KEY` | BotModels API key (change from the example value in production). | `botmodels/.env.example` |
| `AWS_ACCESS_KEY_ID` | Get image bytes check if aws credentials are configured. | `botserver/crates/botbasic_system/src/keywords/face_api/service/detect_faces.rs:279` |
| `AWS_REGION` | Get image bytes check if aws credentials are configured. | `botserver/crates/botbasic_system/src/keywords/face_api/service/detect_faces.rs:278` |
| `AWS_SECRET_ACCESS_KEY` | Check if aws credentials are configured. | `botserver/crates/botbasic_system/src/keywords/face_api/service/detect_faces.rs:281` |
| `AZURE_STORAGE_ACCOUNT` | Azure storage account. | `botserver/crates/botchannels-core/src/media_upload/upload_to_azure_blob.rs:9` |
| `AZURE_STORAGE_CONTAINER` | Azure storage container. | `botserver/crates/botchannels-core/src/media_upload/upload_to_azure_blob.rs:11` |
| `BILLING_WEBHOOK_URL` | Billing webhook url. | `botserver/crates/botbilling/src/alerts/channel.rs:25` |
| `BOOTSTRAP_DB_PASSWORD` | Bootstrap db password. | `botserver/crates/botcorepkg/src/facade_download/finalize.rs:141` |
| `BOTMODELS_API_KEY` | API key for BotModels calls. | `botserver/src/jukebox/mod.rs:44` |
| `BOTMODELS_HOST` | Bind host of the BotModels service. | `botserver/src/jukebox/mod.rs:36` |
| `BOTMODELS_PORT` | Port of the BotModels service. | `botserver/src/main_module/routes/speech_duplex.rs:60` |
| `BOTMODELS_STT_ENGINE` | Botmodels stt engine. | `botmodels/.env.example` |
| `BOTMODELS_STT_MODEL` | Botmodels stt model. | `botmodels/.env.example` |
| `BOTMODELS_TTS_ENGINE` | Botmodels tts engine. | `botmodels/.env.example` |
| `BOTMODELS_TTS_MODEL` | Botmodels tts model. | `botmodels/.env.example` |
| `BOTMODELS_TTS_PATH` | Botmodels tts path. | `botmodels/.env.example` |
| `BOTMODELS_URL` | Origin of the BotModels service. | `botserver/src/jukebox/mod.rs:37` |
| `BOTSERVER_BIN` | Explicit path to the botserver executable (desktop sidecar and tests). | `bottest/src/harness/start.rs:16` |
| `BOTSERVER_BUILD_DATE` | Botserver build date. | `botserver/build.rs:8` |
| `BOTSERVER_BUILD_VERSION` | Version of the running build: the release tag when the build injected one, otherwise the workspace `cargo. | `botserver/crates/botcore/src/product/build_info.rs:13` |
| `BOTSERVER_COMMIT` | Botserver commit. | `botserver/build.rs:14` |
| `BOTSERVER_DESKTOP_LOCAL_BIN` | Botserver desktop local bin. | `bottest/src/harness/bot_server_instance.rs:460` |
| `BOTSERVER_DISABLE_TLS` | Disable TLS in botserver (throwaway audit/offline profiles only). | `botserver/src/main_module/listen_impl.rs:65` |
| `BOTSERVER_HOST` | Hostname botserver binds or advertises. | `botserver/src/main_module/listen_impl.rs:16` |
| `BOTSERVER_INSTALLERS_PATH` | Cache directory of vendored installer archives for offline provisioning. | `botserver/crates/botcorepkg/src/cache/mod.rs:57` |
| `BOTSERVER_STACK_PATH` | Root of the runtime service stack (desktop/offline installs). | `botserver/src/main_body.rs:38` |
| `BOTSERVER_URL` | Origin used by the desktop sidecar and tests to reach botserver. | `botui/src/shared/state.rs:13` |
| `BOTUI_BIN` | Explicit path to the botui executable. | `bottest/src/harness/bot_server_instance.rs:247` |
| `BOTUI_CLOUD_PORT` | Cloud static server port. | `botui/src/main.rs:19` |
| `BOTUI_COMMIT` | Botui commit. | `botui/build.rs:6` |
| `BOTUI_LOGIN_PORT` | Login static server port. | `botui/src/main.rs:26` |
| `BOTUI_PATH` | Botui path. | `botserver/src/main_module/server/run_axum_server.rs:216` |
| `BOTUI_PORT` | Suite static server port. | `botui/src/main.rs:12` |
| `BOTUI_URL` | Origin used to reach the botui static server. | `bottest/tests/e2e/mod.rs:43` |
| `BOT_AI_PROMPT_MODEL` | Bot ai prompt model. | `botserver/crates/botsheet/src/handlers/ai.rs:348` |
| `BOT_AI_PROMPT_URL` | Bot ai prompt url. | `botserver/crates/botsheet/src/handlers/ai.rs:119` |
| `BOT_CALENDAR_API_URL` | Bot calendar api url. | `botserver/crates/botemail/src/handlers/integration.rs:297` |
| `BOT_EMAIL_LLM_MODEL` | Bot email llm model. | `botserver/crates/botemail/src/email_triage.rs:108` |
| `BOT_EMAIL_LLM_URL` | Bot email llm url. | `botserver/crates/botemail/src/email_triage.rs:81` |
| `BOT_TEMPLATES_DIR` | Local template source directory override. | `botserver/crates/botapi/src/system_setup.rs:238` |
| `BOT_TEMPLATES_REPO` | Git repository cloned for the built-in templates. | `botserver/src/main_module/init.rs:383` |
| `BROWSER_BINARY` | Browser binary. | `bottest/src/services/browser_service.rs:106` |
| `BROWSER_CDP_URL` | Browser cdp url. | `botserver/crates/botvibe/src/browser.rs:9` |
| `BROWSER_CDP_URL_TEMPLATE` | A vm attach to that vm's single chromium (x session inside the container) instead of launching a process per tab. | `botserver/crates/botbrowser/src/lib.rs:83` |
| `CACHE_URL` | Valkey/Redis connection string; credentials come from Vault gbo/cache. | `botserver/src/main_module/cache.rs:12` |
| `CADDYFILE_INCUS_CONTAINER` | From that container, the site block appended/updated, and pushed back. | `botserver/crates/botvibe/src/caddy.rs:136` |
| `CADDY_API_URL` | Caddy admin API origin for dynamic site configuration. | `botserver/crates/botvibe/src/caddy.rs:19` |
| `CDP_PORT` | Chrome DevTools Protocol port. | `bottest/tests/e2e/mod.rs:195` |
| `CHAT_URL` | Chat origin used by the desktop shell and tests. | `botui/src/ui_server/cloud.rs:20` |
| `CI_CD_ENABLED` | Enable CI/CD-triggered behaviour in the running process. | `botserver/crates/botbasic_system/src/keywords/create_site/llm_impl/create_site.rs:45` |
| `CLOUD_URL` | Cloud origin used by the desktop shell and tests. | `botui/src/ui_server/login.rs:13` |
| `CUDA_HOME` | Cuda home. | `botserver/crates/botcorepkg/src/installer/llama.rs:12` |
| `CUDA_PATH` | The cuda major version of the toolkit on this windows machine (12 or 13), when one is installed. | `botserver/crates/botcorepkg/src/installer/llama.rs:93` |
| `DATABASE_URL` | PostgreSQL connection string; production credentials come from Vault gbo/tables. | `botserver/crates/botmonitoring/src/lib.rs:237` |
| `DATA_DIR` | Data dir. | `botserver/crates/botcore/src/config/load.rs:60` |
| `DBUS_SESSION_BUS_ADDRESS` | Dbus session bus address. | `botserver/crates/botsettings/src/os_appliance.rs:885` |
| `DB_HOST` | Db host. | `bottest/src/harness.rs:127` |
| `DB_NAME` | Db name. | `bottest/src/harness.rs:134` |
| `DB_PASSWORD` | Db password. | `bottest/src/harness.rs:133` |
| `DB_PORT` | Db port. | `bottest/src/harness.rs:128` |
| `DB_USER` | Db user. | `bottest/src/harness.rs:132` |
| `DEFAULT_BOT_ID` | Default bot id. | `botserver/crates/botbasic_data/src/keywords/products.rs:278` |
| `DEPLOYMENT_ENV` | Deployment environment label. | `botserver/crates/botmonitoring/src/tracing/resource_attributes.rs:12` |
| `DEPLOY_GATEWAY_URL` | Deploy gateway url. | `botserver/crates/botdeployment/src/gateway.rs:22` |
| `DEPLOY_KEY` | Deploy key. | `botserver/crates/botdeployment/src/gateway.rs:24` |
| `DEVICE` | BotModels compute device override (cuda/mps/cpu). | `botmodels/.env.example` |
| `DRIVE_SCAN_INTERVAL` | Their own cadence, so this slower discovery loop costs freshness only for brand-new buckets. | `botserver/src/main_module/drive_monitors/start_drive_monitors.rs:61` |
| `EDITOR_WORKSPACE` | Editor workspace. | `botserver/crates/boteditor/src/handlers/mod.rs:89` |
| `EMBEDDING_KEY` | Start from the db-backed config (url, model, dimensions, etc. | `botserver/crates/botcore/src/kb/website_crawler_service/ensure_crawler_service_running.rs:33` |
| `ENV` | Env. | `botmodels/.env.example` |
| `EVAL_LLM_KEY` | API key for the evaluation gate. | `botserver/crates/boteval/src/live_agent.rs:119` |
| `EVAL_LLM_MODEL` | Model used by the evaluation gate. | `botserver/crates/boteval/src/live_agent.rs:123` |
| `EVAL_LLM_URL` | LLM endpoint used by the offline evaluation gate. | `botserver/crates/boteval/src/live_agent.rs:118` |
| `EXTERNAL_DRIVE_GOOGLE_TOKEN_URL` | Token endpoint; overridable for a local mock in tests or a proxy mirror. | `botserver/crates/botdrive/src/external/gdrive/auth.rs:13` |
| `EXTERNAL_DRIVE_SYNC_INTERVAL` | Recorded on the connection row, so a provider outage degrades to "the external tab shows the last error" instead of noisy logs every tick. | `botserver/crates/botdrive/src/external/engine.rs:385` |
| `EXTERNAL_DRIVE_SYNC_STARTUP_DELAY` | External drive sync startup delay. | `botserver/crates/botdrive/src/external/engine.rs:390` |
| `FORGEJO_DEFAULT_ORG` | Forgejo default org. | `botserver/crates/botdeployment/src/handlers.rs:166` |
| `FORGEJO_TOKEN` | Token used for Forgejo API calls. | `botserver/crates/botcoresecrets/src/tenant.rs:167` |
| `FORGEJO_URL` | Forgejo/ALM origin used by the sync and deploy paths. | `botserver/crates/botcoresecrets/src/tenant.rs:162` |
| `FRESH_STACK` | Fresh stack. | `bottest/tests/e2e/mod.rs:271` |
| `GBDRIVE_DIR` | Drive working directory for the materialized bot buckets. | `botserver/crates/botvideo/src/render.rs:347` |
| `GBO_NO_DOTENV` | Gbo no dotenv. | `botserver/src/main_body.rs:37` |
| `GBO_SKIP_DIRECTORY_WAIT` | In through admin-credentials. | `botserver/src/main_module/state_builder.rs:87` |
| `GBO_SKIP_LOCAL_DIRECTORY` | Gbo skip local directory. | `botserver/src/main_module/state_builder.rs:121` |
| `GBO_STACK_PATH` | Alternate stack root used by the offline installer. | `botserver/crates/botcoresecrets/src/manager.rs:325` |
| `GBO_WORK_PATH` | In production (system container): /opt/gbo/work in development (with botserver-stack directory):. | `botserver/crates/botcore/src/shared/utils/init_secrets_manager.rs:274` |
| `GBO_WSL_DISTRO` | Gbo wsl distro. | `botserver/src/api/terminal/session.rs:148` |
| `GB_ADVISORY_API` | Origin of the advisory feed. | `botserver/crates/botcorepkg/src/cli_audit.rs:57` |
| `GB_ADVISORY_BUNDLE` | Local advisory bundle path. | `botserver/src/main_module/routes/advisories.rs:58` |
| `GB_AGENT_JWT_SECRET` | Gb agent jwt secret. | `botserver/crates/botagent/src/api/auth.rs:72` |
| `GB_ALLOW_LAN` | Allow LAN exposure of the LAN-gated feature. | `botserver/src/main_module/listen_impl.rs:51` |
| `GB_ALLOW_SAMPLEDATA` | Production guard: demo seeding is opt-in only via gb_allow_sampledata=1. | `botserver/src/main_body.rs:240` |
| `GB_API_KEY_HASH_PEPPER` | Pepper applied when hashing API keys at rest. | `botserver/crates/botagent/src/keys.rs:32` |
| `GB_APPROVALS_SIGNING_KEY` | Signing key comes from the environment when configured; otherwise a fresh per-process random key is generated. | `botserver/crates/botconsent/src/approvals/store.rs:35` |
| `GB_APP_ORIGIN` | Gb app origin. | `botserver/src/core/bot/sandbox/routes.rs:238` |
| `GB_BOOT_DEADLINE_SECS` | The deadline for this run. | `botapp/src/desktop/boot_guard.rs:88` |
| `GB_BOT_ID` | Gb bot id. | `botserver/src/basic/keywords/invoke_action.rs:21` |
| `GB_CDP_URL` | Chrome DevTools Protocol origin used by browser automation. | `bottest/tests/integration/dialogue_corpus.rs:210` |
| `GB_CLAUDE_LIVE_VM` | Gb claude live vm. | `botserver/crates/botagent/src/claude/bridge/live_tests.rs:24` |
| `GB_CODEX_LIVE_VM` | Gb codex live vm. | `botserver/crates/botagent/src/codex/bridge_live_tests.rs:25` |
| `GB_COMPUTE_REGION` | Region is a deployment-wide default today; threading a per-request region through the assign endpoint is a follow-up, not a silent assumption. | `botserver/crates/botcloud/src/api/resources.rs:95` |
| `GB_DEPENDENCIES_DIR` | Gb dependencies dir. | `botapp/src/desktop/wizard/hostcheck/mod.rs:157` |
| `GB_DESKTOP_ALLOW_DOWNLOAD` | Allow first-run downloads in the desktop installer. | `botserver/crates/botcore/src/package_manager/facade/mod.rs:25` |
| `GB_DESKTOP_LOCAL_USER_ID` | User id used for desktop-local ownership. | `botserver/src/sheet/user_middleware.rs:35` |
| `GB_DESKTOP_MODE` | Enable desktop-local product behaviour. | `botapp/src/desktop/wizard/paths.rs:112` |
| `GB_DESKTOP_OFFLINE` | Refuse network downloads in desktop mode. | `botapp/src/desktop/sidecar.rs:274` |
| `GB_DESKTOP_SUITE_URL` | Suite origin used by the desktop webview. | `botapp/src/desktop/wizard/window.rs:180` |
| `GB_DRIVE_SYNC_DIR` | Local directory mirrored into the Drive bucket. | `botserver/src/main_module/routes/drive_sync/mod.rs:76` |
| `GB_DUPLEX_LIVE_VM` | Gb duplex live vm. | `botserver/crates/botagent/src/vm/incus_duplex_live_tests.rs:24` |
| `GB_EXTERNAL_IFACE` | #2617 — never create an unscoped dnat rule. | `botserver/crates/botcore/src/package_manager/container/configure_iptables_nat.rs:25` |
| `GB_FEATURE_SETUP_WIZARD` | Gb feature setup wizard. | `botserver/crates/botapi/src/system_setup.rs:121` |
| `GB_FS_ALLOW_ANY` | Allow unrestricted filesystem scope (development only). | `botapp/src/desktop/fs_scope.rs:38` |
| `GB_FS_SCOPES` | Filesystem scope allowlist. | `botapp/src/desktop/fs_scope.rs:83` |
| `GB_GITHUB_TOKEN` | Token used for GitHub release lookups. | `botapp/src/desktop/update.rs:308` |
| `GB_KIOSK` | Run the kiosk/appliance shell profile. | `botapp/src/desktop/mod.rs:10` |
| `GB_LLM_CACHE_WARM` | Warm the LLM cache at boot. | `botserver/src/main_body.rs:432` |
| `GB_LLM_PRICING` | Inline JSON with token pricing data. | `botserver/crates/botsettings/src/settings_usage.rs:98` |
| `GB_LLM_PROFILES` | Inline JSON with LLM profile definitions. | `botserver/crates/botllm/src/profiles.rs:115` |
| `GB_LLM_PROFILES_FILE` | File holding the LLM profile definitions. | `botserver/crates/botllm/src/profiles.rs:121` |
| `GB_LOCALE` | The locale this process renders native strings in, normalised to the codes used by the catalogue (`pt_br. | `botapp/src/desktop/i18n.rs:63` |
| `GB_LOCAL_FILES_ROOT` | Root for local file operations in the sandbox. | `botserver/crates/botdrive/src/local_fs.rs:84` |
| `GB_LOGIN_URL` | Login origin injected into suite and cloud pages. | `botserver/src/main_module/routes/cloud_sso_handler/pages.rs:40` |
| `GB_MOBILE_SERVER_URL` | Is a hard error — never silently fall back to a public server. | `botapp/src/mobile/config.rs:11` |
| `GB_MODEL_DOWNLOAD_BASE` | Download flow can be proved with a local http server and a stand-in file. | `botserver/crates/botsettings/src/on_device_ai/jobs.rs:261` |
| `GB_OAUTH_GOOGLE_CLIENT_ID` | Gb oauth google client id. | `botlib/src/managed_oauth.rs:75` |
| `GB_OAUTH_MICROSOFT_CLIENT_ID` | Gb oauth microsoft client id. | `botlib/src/managed_oauth.rs:81` |
| `GB_OAUTH_MICROSOFT_TENANT_ID` | Gb oauth microsoft tenant id. | `botlib/src/managed_oauth.rs:95` |
| `GB_OPENBAO_COPY` | A move of a copy of a real stack: `gb_openbao_copy=<stack copy>` whose `config. | `botserver/crates/botcorepkg/src/openbao_migration/rehearsal.rs:195` |
| `GB_OPENBAO_REHEARSAL` | Gb openbao rehearsal. | `botserver/crates/botcorepkg/src/openbao_migration/rehearsal.rs:122` |
| `GB_OPENBAO_REHEARSAL_PORT` | Gb openbao rehearsal port. | `botserver/crates/botcorepkg/src/openbao_migration/rehearsal.rs:49` |
| `GB_OPS_SLO_PATH` | Gb ops slo path. | `botserver/crates/botanalytics/src/ops_slo.rs:298` |
| `GB_OS_CHANNEL` | Update channel recorded for the OS appliance. | `botserver/crates/botsettings/src/os_appliance.rs:820` |
| `GB_PLATFORM_DOMAIN` | Platform domain used for bot subdomains and TLS ask. | `botserver/crates/botvibe/src/publish/site.rs:10` |
| `GB_PLUGIN_ALLOWED_HOSTS` | Allowed hosts for plugin network access. | `botserver/crates/botcorepkg/src/plugin.rs:224` |
| `GB_PLUGIN_SIGNING_KEY` | Key used to sign/verify Vibe plugins. | `botserver/crates/botcorepkg/src/plugin.rs:231` |
| `GB_PROMPT_API_CATALOG` | Is on; a single-user install that deliberately drives only the agent vm can opt out with gb_prompt_api_catalog=0. | `botserver/src/core/bot/pipeline/system_prompt/mod.rs:127` |
| `GB_PUBLIC_URL` | Public origin advertised to clients. | `botserver/crates/botdrive/src/drive_handlers/share_email.rs:90` |
| `GB_PUBLISH_LEDGER_TTL_DAYS` | TTL of the Vibe publish ledger. | `botserver/src/core/bot/pipeline/output_publish/ledger.rs:33` |
| `GB_RELEASE_TAG_PREFIX` | Prefix used when deriving release tags. | `botapp/src/desktop/update/feed.rs:24` |
| `GB_REQUIRE_CHECKSUMS` | Require checksum verification for downloaded artifacts. | `botserver/crates/botcorepkg/src/checksum.rs:30` |
| `GB_RUNBOOK_BASE_URL` | Runbook base url so the panel can deep-link without hardcoding a host. | `botserver/crates/botanalytics/src/ops_slo.rs:398` |
| `GB_RUN_EVENTS` | Enable the run-event stream. | `botserver/src/core/bot/pipeline/event_sink/redactor.rs:72` |
| `GB_RUN_EVENTS_RETENTION_DAYS` | Retention window for run events. | `botserver/src/core/bot/runs/store.rs:122` |
| `GB_SANDBOX_ORIGIN` | Origin of the code sandbox service. | `botserver/src/core/bot/sandbox/routes.rs:224` |
| `GB_SANDBOX_PORT` | Port of the code sandbox service. | `botserver/src/core/bot/sandbox/routes.rs:261` |
| `GB_SECRET_DIR` | Directory holding local secret files in desktop/offline mode. | `botserver/crates/botcorepkg/src/installer_vault/secret_store.rs:136` |
| `GB_SELF_URL` | Self URL used for callbacks and link building. | `botserver/src/vibe/resolve_llm_slot.rs:94` |
| `GB_SERVICES_ON` | Whether the product file in `stack` switches the directory service off. | `botapp/src/desktop/wizard/provision/components/directory_wait.rs:15` |
| `GB_SESSION_TTL_SECS` | Session lifetime in seconds. | `botserver/crates/botcoredirectory/src/auth_routes/mod.rs:38` |
| `GB_SIDECAR_PORT` | Port the sidecar listens on. | `botapp/src/desktop/mod.rs:33` |
| `GB_SIDECAR_UID` | Emit an audit record when a non-sidecar process opens the store. | `botserver/crates/botcorepkg/src/installer_vault/secret_store.rs:229` |
| `GB_SIDE_BY_SIDE_URL` | Gb side by side url. | `bottest/tests/integration/dialogue_corpus.rs:209` |
| `GB_SPLASH_MIN_MS` | Reads the minimum splash duration from the environment or the compile-time default. | `botapp/src/desktop/splash.rs:62` |
| `GB_SPLASH_PREVIEW` | Parses `gb_splash_preview`: a number of seconds, or absent/empty. | `botapp/src/desktop/splash.rs:339` |
| `GB_STACK_PATH` | Gb stack path. | `botserver/crates/botcorebot/src/utils.rs:40` |
| `GB_STALE_RUN_MINUTES` | Age at which a Vibe run is considered stale. | `botserver/src/core/bot/runs/observability.rs:122` |
| `GB_TENANT` | Default tenant identifier. | `botserver/src/main_module/wizard_provision.rs:364` |
| `GB_TERMINAL_AUDIT_PURGE_SECS` | #1645 — 90-day retention. | `botserver/src/main_module/routes/computer_use_routes/audit.rs:211` |
| `GB_TEST_RIG` | Off-screen so the webview2 engine still renders (devtools screenshots capture the render tree) but nothing appears on the owner's desktop. | `botapp/src/desktop/splash.rs:33` |
| `GB_TRASH_PURGE_SECS` | #1640 — hourly retention job. | `botserver/crates/botdrive/src/drive_handlers/list_trash.rs:415` |
| `GB_UPDATE_CHANNEL` | Update channel used by the updater/provisioner. | `botserver/crates/botsettings/src/os_appliance.rs:818` |
| `GB_VERIFY_TOKEN` | Generic webhook verification token. | `bottest/tests/integration/verification.rs:28` |
| `GB_VERIFY_URL` | URL used for verification callbacks. | `bottest/tests/integration/verification.rs:24` |
| `GB_VIBE_GIT_MONITOR_SECS` | Background loop: every tick syncs every monitored project. | `botserver/src/main_module/git_bot_monitor/loop_ops.rs:353` |
| `GB_VIBE_TLS_INTERNAL` | Gb vibe tls internal. | `botserver/crates/botvibe/src/proxy_sites/caddy.rs:422` |
| `GB_VIBE_VM_IDLE_SECS` | Running by a crashed session) and then every 10 minutes. | `botserver/src/vibe/configure_vibe_routes.rs:60` |
| `GB_WIZARD_DEBUG` | Gb wizard debug. | `botserver/src/main_module/wizard_provision.rs:167` |
| `GB_WIZARD_LLM_KEY` | Boot). | `botserver/src/main_module/wizard_provision.rs:318` |
| `GB_WIZARD_LLM_MODEL` | Gb wizard llm model. | `botserver/src/main_module/wizard_provision.rs:243` |
| `GB_WIZARD_LLM_URL` | Local mode is seeded by the installer; free/own-key/hosted write the global llm secret (a desktop install is single-user). | `botserver/src/main_module/wizard_provision.rs:239` |
| `GB_WIZARD_MODEL_MODE` | Local mode is seeded by the installer; free/own-key/hosted write the global llm secret (a desktop install is single-user). | `botserver/src/main_module/wizard_provision.rs:238` |
| `GB_WIZARD_WITH_BOTMODELS` | Gb wizard with botmodels. | `botserver/src/main_module/wizard_provision/component_plan.rs:22` |
| `GB_WIZARD_WORKSPACE` | Gb wizard workspace. | `botapp/src/desktop/wizard/paths.rs:82` |
| `GB_WORKSPACE_TIMEOUT_MAX_MS` | The effective `timeout_ms` cap, overridable per node. | `botserver/crates/botagent/src/sandbox.rs:42` |
| `GB_WORK_PATH` | Gb work path. | `botserver/crates/botcorepkg/src/plugin.rs:235` |
| `GCS_BUCKET` | Gcs bucket. | `botserver/crates/botchannels-core/src/media_upload/get_platform_limits.rs:264` |
| `HEADED` | Headed. | `bottest/src/cli/args.rs:49` |
| `HEADLESS` | Headless. | `bottest/tests/e2e/mod.rs:194` |
| `HOST` | Host. | `botmodels/.env.example` |
| `IMAGE_HEIGHT` | Image height. | `botmodels/.env.example` |
| `IMAGE_MODEL_PATH` | Image model path. | `botmodels/.env.example` |
| `IMAGE_STEPS` | Image steps. | `botmodels/.env.example` |
| `IMAGE_WIDTH` | Image width. | `botmodels/.env.example` |
| `INCUS_SOCKET` | Incus socket. | `botserver/crates/botdeployment/src/gateway_server/gateway_state.rs:42` |
| `INTEGRATION_AUTOMATIONS` | Spawns the background automation runner. | `botserver/crates/botintegrations/src/automations.rs:138` |
| `INTERNAL_API_TOKEN` | Every internal caller (x-internal-token) resolves through this single function so the token can live in vault instead of the systemd unit. | `botserver/crates/botcoresecrets/src/tenant.rs:410` |
| `INVITE_BASE_URL` | Value that is not absolute is rejected. | `botserver/crates/botcore/src/organization_invitations/mod.rs:30` |
| `INVOKE_API_URL` | Invoke api url. | `botserver/src/basic/keywords/invoke_action.rs:17` |
| `JWT_SECRET` | Legacy/general JWT secret fallback when no SaaS secret is configured. | `botserver/crates/botsecurity-crypto/src/encryption/master_key_lock.rs:63` |
| `KEEP_ENV` | Keep env. | `bottest/src/cli/args.rs:48` |
| `LC_ALL` | Used by the catalogue (`pt_br. | `botapp/src/desktop/i18n.rs:65` |
| `LD_LIBRARY_PATH` | Ld library path. | `botapp/src/desktop/wizard/media.rs:258` |
| `LLM_ENDPOINT` | Llm endpoint. | `botserver/crates/botbasic_data/src/keywords/use_account/imap_index.rs:79` |
| `LLM_KEY` | API key used when no Vault LLM entry exists. | `botserver/src/main_module/state_builder.rs:186` |
| `LLM_MODEL` | Model identifier/path used when no Vault LLM entry exists. | `botserver/src/main_module/state_builder.rs:167` |
| `LLM_READ_TIMEOUT_SECS` | First chunk arrives; `llm_read_timeout_secs` (default 120) is the operator knob for that case. | `botserver/crates/botllm/src/openai_stream.rs:256` |
| `LLM_TURN_TIMEOUT_SECS` | A cpu-only provider must prefill the whole prompt before the first token, so the ceiling is configurable: `llm_turn_timeout_secs` (default 300). | `botserver/src/core/bot/pipeline/enriched_llm/mod.rs:14` |
| `LLM_URL` | Chat-completions endpoint used when no Vault LLM entry exists. | `botserver/src/main_module/health.rs:200` |
| `LOCAL_STORAGE_PATH` | Local storage path. | `botserver/crates/botchannels-core/src/media_upload/upload_to_azure_blob.rs:60` |
| `LOCAL_STORAGE_URL` | Local storage url. | `botserver/crates/botchannels-core/src/media_upload/upload_to_azure_blob.rs:62` |
| `LOGIN_URL` | Login base url for unauthenticated visitors (env `login_url`, default the canonical login domain documented in agents. | `botserver/crates/botvibe/src/domain_auth.rs:37` |
| `LOGNAME` | ("expect user 'alm' but current user is:. | `botserver/crates/botcorepkg/src/alm_setup.rs:42` |
| `LOG_LEVEL` | Log level. | `botmodels/.env.example` |
| `MAIL_FROM` | Mail from. | `botserver/crates/botcore/src/organization_invitations/send_invitation_email.rs:42` |
| `MAIL_HOST` | Invite creation. | `botserver/crates/botcore/src/organization_invitations/send_invitation_email.rs:30` |
| `MAIL_IMAP_PORT` | Real mail server from env (vault-backed in prod); falls back to the old localhost stub only when unconfigured. | `botserver/crates/botcoredirectory/src/provisioning.rs:178` |
| `MAIL_OAUTH_REDIRECT_BASE` | Match the uri registered with the provider byte for byte, so the configured base takes precedence over the request headers. | `botserver/crates/botemail/src/handlers/oauth.rs:68` |
| `MAIL_OAUTH_STATE_SECRET` | Mail oauth state secret. | `botserver/crates/botemail/src/handlers/oauth.rs:59` |
| `MAIL_PASS` | The host/user check below warns once before returning. | `botserver/crates/botcore/src/organization_invitations/send_invitation_email.rs:38` |
| `MAIL_PORT` | Mail port. | `botserver/crates/botcore/src/organization_invitations/send_invitation_email.rs:31` |
| `MAIL_SMTP_PORT` | Old localhost stub only when unconfigured. | `botserver/crates/botcoredirectory/src/provisioning.rs:179` |
| `MAIL_USER` | Absence is expected for an unauthenticated/local smtp relay; the host/user check below warns once before returning. | `botserver/crates/botcore/src/organization_invitations/send_invitation_email.rs:34` |
| `MEDIA_STORAGE_TYPE` | Get storage configuration from environment. | `botserver/crates/botchannels-core/src/media_upload/get_platform_limits.rs:204` |
| `MEET_TURN_REALM` | Meet turn realm. | `botserver/crates/botmeet/src/turn.rs:38` |
| `MEET_TURN_SECRET` | Meet turn secret. | `botserver/crates/botmeet/src/turn.rs:37` |
| `MEET_TURN_URL` | Meet turn url. | `botserver/crates/botmeet/src/turn.rs:36` |
| `MESSAGE_ENCRYPTION_KEY` | 2. | `botserver/crates/botsecurity-crypto/src/encryption/master_key_lock.rs:47` |
| `MODE` | BotModels model tier: auto, min or max. | `botmodels/.env.example` |
| `OCR_MODEL_PATH` | Ocr model path. | `botmodels/.env.example` |
| `OC_ALIAS` | Oc alias. | `botserver/src/main_module/routes/feature_routers/make_saas_router.rs:41` |
| `OC_PATH` | Resolve the real oc client: a bare `oc` on path may be an unrelated tool, whose invocation silently broke bucket provisioning at signup (#1371). | `botserver/src/main_module/routes/feature_routers/make_saas_router.rs:31` |
| `OLLAMA_HOST` | Vault-backed llm settings are not exposed as a client on appstate; when no endpoint url is published via env we can only report "configured". | `botserver/src/main_module/health.rs:200` |
| `OPENAI_API_KEY` | Openai api key. | `botserver/src/main_module/state_builder.rs:187` |
| `OPENAI_OAUTH_CLIENT_ID` | Openai oauth client id. | `botserver/src/core/ai_accounts_chatgpt/mod.rs:63` |
| `OPENAI_OAUTH_PORT` | Openai oauth port. | `botserver/src/core/ai_accounts_chatgpt/mod.rs:71` |
| `OPENAI_OAUTH_REDIRECT` | Openai oauth redirect. | `botserver/src/core/ai_accounts_chatgpt/mod.rs:78` |
| `OPENAI_OAUTH_TOKEN_URL` | Token endpoint; override for a local mock or a regional mirror. | `botserver/src/core/ai_accounts_chatgpt/mod.rs:87` |
| `OPENAI_RESPONSES_BASE_URL` | Openai responses base url. | `botserver/src/core/ai_accounts_chatgpt/oauth.rs:70` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Otel exporter otlp endpoint. | `botserver/src/core/bot/runs/observability.rs:372` |
| `OTTERIO_ACCESS_KEY` | Não faz fallback para credenciais hardcoded — se as env vars não existirem, retorna `err`. | `botserver/crates/botcore/src/config/from_vault.rs:105` |
| `OTTERIO_BACKEND` | Select an external backend (#1468) without a vault round trip; all are optional so the otterio-only deployment is unchanged. | `botserver/crates/botcore/src/config/from_vault.rs:120` |
| `OTTERIO_BUCKET` | Otterio bucket. | `botserver/crates/botcore/src/config/from_vault.rs:111` |
| `OTTERIO_ENDPOINT` | Otterio endpoint. | `botserver/crates/botmonitoring/src/lib.rs:256` |
| `OTTERIO_FALLBACK_ENDPOINT` | Select an external backend (#1468) without a vault round trip; all are optional so the otterio-only deployment is unchanged. | `botserver/crates/botcore/src/config/from_vault.rs:121` |
| `OTTERIO_REGION` | Select an external backend (#1468) without a vault round trip; all are optional so the otterio-only deployment is unchanged. | `botserver/crates/botcore/src/config/from_vault.rs:119` |
| `OTTERIO_SECRET_KEY` | Não faz fallback para credenciais hardcoded — se as env vars não existirem, retorna `err`. | `botserver/crates/botcore/src/config/from_vault.rs:107` |
| `OTTERIO_SERVER` | Otterio server. | `botserver/crates/botcore/src/config/from_vault.rs:113` |
| `OTTERIO_URL` | Otterio url. | `botserver/crates/botmonitoring/src/lib.rs:254` |
| `OUTPUT_DIR` | BotModels generated-file output directory. | `botmodels/.env.example` |
| `PG_URL` | Fallback to environment variable. | `botserver/crates/botcore/src/shared/utils/init_secrets_manager.rs:118` |
| `PLATFORM_COMPANY` | Platform company. | `botlib/src/branding.rs:94` |
| `PLATFORM_DOMAIN` | Platform domain. | `botlib/src/branding.rs:97` |
| `PLATFORM_LOGO_URL` | Platform logo url. | `botlib/src/branding.rs:100` |
| `PLATFORM_NAME` | Platform name. | `botlib/src/branding.rs:87` |
| `PLATFORM_PRIMARY_COLOR` | Platform primary color. | `botlib/src/branding.rs:103` |
| `PLATFORM_SHORT_NAME` | Platform short name. | `botlib/src/branding.rs:91` |
| `PORT` | Port. | `botserver/crates/botcore/src/config/load.rs:45` |
| `PRODUCT_FILE` | Product file. | `botlib/src/branding.rs:75` |
| `PUBLIC_BASE_URL` | `. | `botserver/crates/botcore/src/product/get_product_name.rs:49` |
| `QDRANT_API_KEY` | Qdrant api key. | `botserver/src/core/bot/kb_context/kb_context_search.rs:24` |
| `QDRANT_URL` | Qdrant url. | `botserver/src/core/bot/kb_context/kb_context_search.rs:17` |
| `REALTIME_AUDIO_MODEL_PATH` | Realtime audio model path. | `botmodels/.env.example` |
| `REDIS_DB` | Check for redis database number for namespace isolation. | `botserver/src/main_module/cache.rs:35` |
| `REDIS_URL` | Try environment variables first. | `botserver/src/main_module/cache.rs:13` |
| `RUST_LOG` | Rust log filter directive. | `botserver/src/main_body.rs:87` |
| `S3_BUCKET` | S3 bucket. | `botserver/crates/botchannels-core/src/media_upload/get_platform_limits.rs:219` |
| `S3_ENDPOINT` | S3 endpoint. | `botserver/crates/botmonitoring/src/lib.rs:255` |
| `S3_REGION` | S3 region. | `botserver/crates/botchannels-core/src/media_upload/get_platform_limits.rs:221` |
| `SAAS_DISABLE_CAPACITY_CHECK` | Dev/testing escape hatch; mirrors the flag the cloud signup path honors (botcloud::api::handle_signup). | `botserver/crates/botvibe/src/metering.rs:348` |
| `SCIM_TOKENS` | Fails closed: an unset or malformed value yields an empty map, so every scim route rejects requests until an operator provisions a token. | `botserver/crates/botcoredirectory/src/scim/server/scim_auth.rs:46` |
| `SITES_ROOT` | Sites root. | `botserver/crates/botdeployment/src/gateway_server/gateway_state.rs:53` |
| `SITE_DOMAIN` | Then `gb. | `botserver/crates/botvibe/src/publish/site.rs:13` |
| `SITE_PATH` | Site path. | `botserver/crates/botcore/src/config/load.rs:58` |
| `SKIP_E2E_TESTS` | Skip e2e tests. | `bottest/tests/e2e/mod.rs:209` |
| `SKIP_INTEGRATION_TESTS` | Skip integration tests. | `bottest/tests/integration/mod.rs:46` |
| `SKIP_LLM_SERVER` | Skip llm server. | `botserver/crates/botllm/src/local/ensure_llama_servers_running.rs:18` |
| `SLO_SNAPSHOT_OUT` | Slo snapshot out. | `bottest/tests/slo_budget_test.rs:108` |
| `STACK_PATH` | Stack path. | `botserver/crates/botcorebot/src/utils.rs:39` |
| `STRIPE_SECRET_KEY` | Stripe secret key. | `botserver/src/main_module/routes/feature_routers/make_saas_router.rs:18` |
| `STT_MODEL_PATH` | Stt model path. | `botmodels/.env.example` |
| `STUN_URL` | Stun url. | `botserver/crates/botmeet/src/turn_config.rs:17` |
| `SUITE_URL` | Suite url. | `botui/src/ui_server/cloud.rs:281` |
| `SYSTEMROOT` | `safecommand::env_clear()` reconstructs a minimal environment that may not resolve the bare name. | `botserver/crates/botsettings/src/on_device_ai/hardware.rs:71` |
| `TAURI_CONFIG` | Patch the cli already set (`--config`) is kept and the identity is laid over it; the values are the same file's, so applying them twice is a no-op. | `botapp/build.rs:30` |
| `TEST_ADMIN_EMAIL` | Test admin email. | `bottest/tests/e2e/dashboard.rs:7` |
| `TEST_ADMIN_PASSWORD` | Test admin password. | `bottest/tests/e2e/dashboard.rs:8` |
| `TEST_ATTENDANT_EMAIL` | Test attendant email. | `bottest/tests/e2e/dashboard.rs:14` |
| `TEST_ATTENDANT_PASSWORD` | Test attendant password. | `bottest/tests/e2e/dashboard.rs:16` |
| `TINYFISH_API_KEY` | Reads the key at call time. | `botserver/crates/botresearch/src/web_search/handle_search_history/tinyfish.rs:59` |
| `TTS_MODEL_PATH` | Tts model path. | `botmodels/.env.example` |
| `TURN_CREDENTIAL` | Turn credential. | `botserver/crates/botmeet/src/turn_config.rs:16` |
| `TURN_URL` | Turn url. | `botserver/crates/botmeet/src/turn_config.rs:14` |
| `TURN_USERNAME` | Turn username. | `botserver/crates/botmeet/src/turn_config.rs:15` |
| `USER` | User. | `botserver/src/sheet/user_middleware.rs:46` |
| `USERNAME` | Username. | `botserver/src/sheet/user_middleware.rs:47` |
| `USERPROFILE` | Sources `/. | `botserver/src/api/terminal/session.rs:223` |
| `VALKEY_URL` | Try environment variables first. | `botserver/src/main_module/cache.rs:14` |
| `VAULT_ADDR` | Vault server address used by the secret resolver. | `botserver/src/main_body.rs:55` |
| `VAULT_CACERT` | Path to the Vault CA certificate used for TLS verification. | `botserver/src/main_module/wizard_provision.rs:294` |
| `VAULT_CACHE_TTL` | Secret cache lifetime in seconds (default 300). | `botserver/crates/botcoresecrets/src/manager.rs:50` |
| `VAULT_CLIENT_CERT` | Vault client cert. | `botserver/crates/botcoresecrets/src/manager.rs:60` |
| `VAULT_CLIENT_KEY` | Vault client key. | `botserver/crates/botcoresecrets/src/manager.rs:66` |
| `VAULT_SKIP_VERIFY` | Disable Vault TLS verification (development only). | `botserver/crates/botcoresecrets/src/manager.rs:49` |
| `VAULT_TOKEN` | Vault token for the local bootstrap process (never commit a value). | `botserver/src/main_body.rs:64` |
| `VAULT_TOKEN_FILE` | Vault token file. | `botserver/crates/botcore/src/config/from_vault.rs:13` |
| `VECTORDB_API_KEY` | Vectordb api key. | `botserver/src/core/bot/kb_context/kb_context_search.rs:26` |
| `VECTORDB_URL` | Vectordb url. | `botserver/src/core/bot/kb_context/kb_context_search.rs:19` |
| `VIBE_API_URL` | Vibe api url. | `botserver/src/basic/keywords/vibe_agent.rs:18` |
| `VIBE_APP_PORT` | `{container}. | `botserver/crates/botvibe/src/ops.rs:104` |
| `VIBE_BACKUP_DIR` | Vibe backup dir. | `botserver/crates/botvibe/src/backups.rs:74` |
| `VIBE_CODEGRAPH_INSTALL` | Without npm can provision containers normally. | `botserver/crates/botvibe/src/vm_lifecycle/tests.rs:154` |
| `VIBE_INCUS_FORCE_UNAVAILABLE` | Vibe incus force unavailable. | `botserver/crates/botvibe/src/vm_incus/windows.rs:185` |
| `VIBE_LLM_COST_PER_1M_TOKENS` | Vibe llm cost per 1m tokens. | `botserver/crates/botvibe/src/agent_loop/llm_payload.rs:132` |
| `VIBE_MAX_PROJECTS_PER_KIND` | Maximum projects of one kind per branch. | `botserver/crates/botvibe/src/eviction.rs:24` |
| `VIBE_PROBE_URL_TEMPLATE` | Dial used by #770). | `botserver/crates/botvibe/src/ops.rs:54` |
| `VIBE_PROD_APP_PORT` | Through the caddy domain route -> container ip:3000, and the old default of 80 collided with the container's listeners. | `botserver/crates/botvibe/src/publish/publish_flow.rs:177` |
| `VIBE_RUN_PORT_BASE` | Deterministic per-project host port (31000-31999) so re-runs and the browser button reuse the same proxy device instead of piling up devices. | `botserver/crates/botvibe/src/projects_api/projects.rs:121` |
| `VIBE_RUN_TIMEOUT_SECS` | Every scaffold run then dies as "agent loop timed out". | `botserver/crates/botvibe/src/agent_loop/llm.rs:10` |
| `VIBE_VM_IMAGE` | Vibe vm image. | `botserver/crates/botvibe/src/vm_incus/linux.rs:82` |
| `VIBE_WORKSPACE_ROOT` | Otherwise the platform stack data directory is used. | `botserver/crates/botgit/src/path_guard.rs:31` |
| `VIBE_WSL_APP_PORT` | Vibe wsl app port. | `botserver/crates/botvibe/src/publish/publish_entry.rs:208` |
| `VIDEO_AUDIO_DIR` | Video audio dir. | `botserver/crates/botvideo/src/handlers/audio.rs:81` |
| `VIDEO_FPS` | Video fps. | `botmodels/.env.example` |
| `VIDEO_FRAMES` | Video frames. | `botmodels/.env.example` |
| `VIDEO_HEIGHT` | Video height. | `botmodels/.env.example` |
| `VIDEO_MODEL_PATH` | Video model path. | `botmodels/.env.example` |
| `VIDEO_PREVIEW_DIR` | Video preview dir. | `botserver/crates/botvideo/src/handlers/media.rs:166` |
| `VIDEO_REFRAME_DIR` | Video reframe dir. | `botserver/crates/botvideo/src/handlers/ai.rs:146` |
| `VIDEO_STEPS` | Video steps. | `botmodels/.env.example` |
| `VIDEO_THUMBNAILS_DIR` | Video thumbnails dir. | `botserver/crates/botvideo/src/handlers/ai.rs:97` |
| `VIDEO_UPLOAD_DIR` | Video upload dir. | `botserver/crates/botvideo/src/handlers/media.rs:29` |
| `VIDEO_WIDTH` | Video width. | `botmodels/.env.example` |
| `VISION_MODEL_PATH` | Vision model path. | `botmodels/.env.example` |
| `VULKAN_SDK` | Vulkan sdk. | `botserver/crates/botcorepkg/src/installer/llama.rs:15` |
| `WEBA_BRANCH_ID` | Weba branch id. | `botserver/crates/botweba/src/weba_state.rs:250` |
| `WEBA_SNAPSHOT_DIR` | Weba snapshot dir. | `botserver/crates/botweba/src/weba_state.rs:35` |
| `WEBKIT_FORCE_SANDBOX` | #2464 — thermal + battery telemetry for support reports. | `botserver/crates/botsettings/src/os_appliance.rs:458` |
| `WORK_PATH` | Work path. | `botserver/crates/botcorepkg/src/plugin.rs:236` |
| `XDG_CONFIG_HOME` | XDG base directory for user configuration files. | `botserver/crates/botcore/src/os_abstraction/linux.rs:84` |
| `XDG_DATA_HOME` | XDG base directory for user data files. | `botserver/crates/botcore/src/os_abstraction/linux.rs:75` |
| `XDG_RUNTIME_DIR` | XDG base directory for runtime files. | `botserver/crates/botsettings/src/os_appliance.rs:878` |
| `ZITADEL_CLIENT_ID` | Zitadel OIDC client identifier. | `botserver/src/main_module/routes/security_setup.rs:180` |
| `ZITADEL_ISSUER_URL` | Zitadel OIDC issuer URL used by the directory service. | `botserver/src/main_module/routes/security_setup.rs:179` |

