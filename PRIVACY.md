# Privacy

## No outbound telemetry by default

General Bots does not include Google Analytics, Google Tag Manager, Segment,
Mixpanel, PostHog, Sentry, Datadog, Amplitude, Hotjar, Plausible, Umami,
Facebook Pixel, or any other third-party analytics/tracking SDK. No analytics,
usage report, crash report, or behavioral beacon is sent anywhere by default.

This is enforced by code, not only by policy:

- The suite privacy panel keeps `telemetry_enabled`, `share_usage_stats`, and
  `crash_reports` hard-wired to `false`; a stale or edited `localStorage` value
  cannot re-enable a sender.
- The first-run wizard has no telemetry/crash-report opt-in and sends no
  wizard telemetry environment variable.
- The browser extension has no analytics setting.
- The distributed tracing exporter defaults to `enabled: false` and
  `ExporterType::None`.
- `scripts/adequacy_guard.py` fails CI if a known third-party analytics
  snippet is reintroduced into first-party `.rs`, `.js`, `.html`, or `.css`.

## What is sent by default

Nothing. With the bundled local model, prompts, answers, files, memory, and
settings stay on the machine.

## Data that stays local

These features look like telemetry but never leave the machine:

| Data | Where it goes |
|------|---------------|
| Server/UI logs (`botserver.log`, `botui.log`) | Local files |
| Client error reports and navigation events (`/api/client-errors`) | Local botserver log (`CLIENT:` lines) |
| Analytics dashboard (`/api/ui/analytics/*`) | Local PostgreSQL |
| Vibe run telemetry (`vibe_telemetry`) | Local PostgreSQL |
| Monitoring metrics and traces | Local process memory / local PostgreSQL |
| Chat history, memory, exports | Local database and browser storage |

## Explicit opt-ins (off unless an operator configures them)

The following can send data off the machine, but only after an operator
explicitly configures them:

- **LLM provider**: user prompts and answers are sent to the configured model
  endpoint (OpenAI-compatible API, Anthropic, Kilo, a local llama.cpp server,
  etc.). Use a loopback/local model endpoint to keep prompts local.
- **Embeddings**: knowledge-base text is sent to the configured
  `embedding-url`. If it is empty, the local hash fallback is used.
- **OTLP tracing**: setting `OTEL_EXPORTER_OTLP_ENDPOINT` exports run spans
  (model name, run state, token counts, tool names/status) to that collector.
  Prompt/answer text is not included in these spans.
- **Integrations**: email, SMS, social, CRM, storage, and observability
  connectors (including Amplitude/PostHog/Sentry provider catalog entries)
  only act when an operator connects credentials and invokes them.
- **Weather widget**: entering a city calls `api.open-meteo.com` directly from
  the browser. It is user-initiated and not analytics.
- **Web search / research tools**: only run when the operator or user invokes
  them.

## Translation, suggestions, embeddings

- `TRANSLATE` uses the configured LLM; the text is not sent to a separate
  translation service.
- Suggestions come from the bot's `start.bas` / `ADD_SUGGESTION` data, not a
  third-party suggestion API.
- Embeddings use the operator-configured endpoint or the local fallback.
- The desktop wizard defaults to a keyless hosted free tier (Kilo) so first run
  works without a download; choose the bundled local model in the wizard to
  keep prompts on the machine. This is a model-inference choice, not
  analytics, and the wizard states the destination before install.

## How to verify

```bash
# No third-party analytics SDKs in first-party source
grep -rniE "googletagmanager|google-analytics|gtag\\(|mixpanel|posthog\\.init|sentry\\.init|hotjar|plausible\\.io/js|umami|fbq\\(|clarity\\.ms" \
  botserver botui bottemplates botapp desktop scripts \
  --include='*.rs' --include='*.js' --include='*.html' --include='*.css' \
  --exclude-dir=target --exclude-dir=node_modules --exclude-dir=vendor

# Privacy guard
python3 scripts/adequacy_guard.py --baseline issues/adequacy/baseline.json
```

If you find an outbound analytics call, treat it as a P1 privacy bug and report
it at https://github.com/generalbots/generalbots/issues.
