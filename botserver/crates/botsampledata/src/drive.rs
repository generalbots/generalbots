//! Drive object seeding for the fiscal test scenarios (issues #722/#723/#724)
//! and the Pragmatismo Vibe bot payload (#750).
//!
//! Writes real MinIO objects into the default bot's bucket so the chat/API
//! flows can discover them exactly like a production user's Drive:
//!
//!   * `faturas/`  — invoice folder used by the guided upload flow (#723)
//!   * `financeiro/` — cash-flow CSV files used by the banking import (#724)
//!   * `{bot}.gbot/`, `{bot}.gbdialog/` — the Pragmatismo reference payload
//!     (start.bas showing VIBE RUN / tool usage, PROMPT.md, config.csv and
//!     MCP tool definitions) seeded to the `pragmatismo` bot bucket (#750)
//!
//! All inserts are guarded by `object_exists`, so re-running is safe.

use botlib::traits::DriveRepository;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;

fn resolve_default_bot_name(conn: &mut diesel::PgConnection) -> Result<Option<String>, String> {
    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = Text)]
        name: String,
    }
    let row: Option<BotRow> = sql_query(
        "SELECT name FROM bots WHERE is_default_for_branch = true ORDER BY created_at ASC LIMIT 1",
    )
    .get_result(conn)
    .ok();
    Ok(row.map(|r| r.name))
}

/// Builds a cash-flow CSV with entries for the given month (so diagnosis
/// filters by the current month prefix correctly). Uses the real-world
/// Brazilian column names (data/historico/valor/tipo) that the import
/// parsers accept.
fn build_cashflow_csv(year: u32, month_idx: u32) -> String {
    let last_day = if month_idx == 2 { 28 } else { 30 };
    let d = |day: u32| format!("{year:04}-{month_idx:02}-{day:02}");
    format!(
        "data,historico,valor,tipo\n\
         {},\"Client contract\",5000.00,receita\n\
         {},\"Consulting hours\",2500.00,receita\n\
         {},\"Cloud subscription\",-1200.00,despesa\n\
         {},\"Office rent\",-1800.00,despesa\n\
         {},\"Telecom\",-{last_day}.00,despesa\n\
         {},\"Marketing\",-600.00,despesa\n",
        d(2),
        d(5),
        d(8),
        d(12),
        d(15),
        d(20),
    )
}

async fn put_if_missing(
    drive: &dyn DriveRepository,
    bucket: &str,
    key: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    if drive.object_exists(bucket, key).await.unwrap_or(false) {
        return Ok(());
    }
    drive
        .put_object(bucket, key, body.to_vec(), Some(content_type))
        .await
        .map_err(|e| format!("put {key}: {e}"))
}

/// The reference `start.bas` for the Pragmatismo bot: introduces the
/// VIBE agent commands (VIBE RUN / VIBE STATUS / VIBE TOOLS) through
/// suggestions. Uses only existing BASIC keywords from the no-drive
/// pipeline.
fn pragmatismo_start_bas() -> String {
    r#"ADD SUGGESTION "VIBE RUN \"Criar um website de apresentação para a empresa\"" AS "Criar website com Vibe"
ADD SUGGESTION "VIBE TOOLS" AS "Ver ferramentas do Vibe"
ADD SUGGESTION "Quero agendar um batizado" AS "Agendar batizado"
ADD SUGGESTION "Quero uma demonstração do produto" AS "Demonstração"

SET CONTEXT "vibe" AS "You are the Pragmatismo sales assistant. You help users
understand the Vibe agent (create projects, run agents, deploy websites) and the
bots platform. When the user asks to create something with Vibe, use VIBE RUN.

VIBE RUN \"intent\" creates an autonomous agent run on the project (tools:
file read/write, git status/log/diff/commit, shell commands, logs, tests).
VIBE STATUS \"run_id\" shows the state and executed tools.
VIBE TOOLS lists the available tools.
VIBE APPROVE \"run_id\" approves pending tool calls.
VIBE CANCEL \"run_id\" aborts a run.
VIBE EVENTS \"run_id\" streams progress events.

Never invent run ids; ask the user to provide one when needed."

TALK "Olá! Sou o assistente da Pragmatismo. Posso ajudar a criar um projeto com o agente Vibe (VIBE RUN), consultar seus runs (VIBE STATUS), ou agendar uma conversa comercial."#.to_string()
}

/// PROMPT.md for the Pragmatismo reference bot.
fn pragmatismo_prompt_md() -> String {
    r#"## IDENTIDADE
Você é o assistente virtual da Pragmatismo — plataforma General Bots que cria
bots, websites e sistemas a partir de conversas em português.

## RECURSOS DISPONÍVEIS
- **VIBE RUN "<intent>"**: cria um run do agente Vibe para gerar/editar um projeto
- **VIBE STATUS "id"** / **VIBE EVENTS "id"**: acompanhar progresso
- **VIBE TOOLS**: lista de ferramentas (file, shell, git, logs, test)
- **VIBE APPROVE/CANCEL**: aprovar ou cancelar runs aguardando permissão

## REGRAS
- Responda em pt-BR, sucinto e organizado.
- Nunca invente run IDs: peça ao usuário que informe.
- Não execute tools de escrita sem confirmação explícita do usuário.
- Para agendamento de serviços (batizado, evento, etc.), colete os dados
  obrigatórios (nome, data, endereço, contato) e peça confirmação antes.
- Prefira bullets e emojis discretos; evite tabelas markdown.

## PERSONA
Tom acolhedor, produtivo, consultivo."#
        .to_string()
}

/// config.csv for the Pragmatismo bot (loaded by drive_monitor into the
/// config manager; secrets are never seeded).
fn pragmatismo_config_csv() -> String {
    r#"llm-model,openai/gpt-oss-120b
llm-provider,openai
history-limit,6
system-prompt,You are the Pragmatismo sales assistant. Respond in Portuguese unless asked otherwise.
"#
    .to_string()
}

/// MCP-style tool definitions placed under `{bot}.gbdialog/`.
fn pragmatismo_tool_json(name: &str, description: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "description": "{description}",
  "input_schema": {{
    "type": "object",
    "properties": {{}},
    "required": []
  }}
}}"#
    )
}

/// Seeds the invoice folder and cash-flow spreadsheets of the default bot.
pub async fn seed_drive_objects(
    pool: &botcore::shared::utils::DbPool,
    drive: &dyn DriveRepository,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
    let bot_name = resolve_default_bot_name(&mut conn)?.ok_or("No default bot found")?;
    drop(conn);

    let bucket = format!("{bot_name}.gbai");
    let prefix = format!("{bot_name}.gbdrive/");

    drive
        .create_bucket_if_not_exists(&bucket)
        .await
        .map_err(|e| format!("create bucket {bucket}: {e}"))?;

    let now = chrono::Utc::now();
    let current_month = now.format("%Y-%m").to_string();
    let prev = now
        .checked_sub_months(chrono::Months::new(1))
        .unwrap_or(now);
    let prev_month = prev.format("%Y-%m").to_string();
    let year = now.format("%Y").to_string().parse::<u32>().unwrap_or(2026);
    let month_idx = now.format("%m").to_string().parse::<u32>().unwrap_or(8);
    let prev_year = prev.format("%Y").to_string().parse::<u32>().unwrap_or(2026);
    let prev_month_idx = prev.format("%m").to_string().parse::<u32>().unwrap_or(7);

    let faturas_marker = format!("{prefix}faturas/.keep");
    if !drive.object_exists(&bucket, &faturas_marker).await.unwrap_or(false) {
        drive
            .put_object(&bucket, &faturas_marker, Vec::new(), Some("text/plain"))
            .await
            .map_err(|e| format!("create faturas folder: {e}"))?;
    }

    let invoice_key = format!("{prefix}faturas/telefonia-{current_month}.pdf");
    if !drive.object_exists(&bucket, &invoice_key).await.unwrap_or(false) {
        drive
            .put_object(
                &bucket,
                &invoice_key,
                format!("%PDF-1.4 sample invoice {current_month}").into_bytes(),
                Some("application/pdf"),
            )
            .await
            .map_err(|e| format!("seed invoice: {e}"))?;
    }

    let financeiro_marker = format!("{prefix}financeiro/.keep");
    if !drive.object_exists(&bucket, &financeiro_marker).await.unwrap_or(false) {
        drive
            .put_object(&bucket, &financeiro_marker, Vec::new(), Some("text/plain"))
            .await
            .map_err(|e| format!("create financeiro folder: {e}"))?;
    }

    let finance_key = format!("{prefix}financeiro/fluxo-caixa-{current_month}.csv");
    if !drive.object_exists(&bucket, &finance_key).await.unwrap_or(false) {
        drive
            .put_object(
                &bucket,
                &finance_key,
                build_cashflow_csv(year, month_idx).into_bytes(),
                Some("text/csv"),
            )
            .await
            .map_err(|e| format!("seed current cashflow: {e}"))?;
    }

    let prev_key = format!("{prefix}financeiro/fluxo-caixa-{prev_month}.csv");
    if !drive.object_exists(&bucket, &prev_key).await.unwrap_or(false) {
        drive
            .put_object(
                &bucket,
                &prev_key,
                build_cashflow_csv(prev_year, prev_month_idx).into_bytes(),
                Some("text/csv"),
            )
            .await
            .map_err(|e| format!("seed previous cashflow: {e}"))?;
    }

    Ok(())
}

/// Seeds the Pragmatismo reference bot payload (start.bas, PROMPT.md,
/// config.md, MCP tool definitions) into the `pragmatismo` bot bucket
/// (#750). The payload drives the Vibe agent demo: the start.bas covers
/// both the summary suggestions and the VIBE bridge keywords.
pub async fn seed_pragmatismo_payload(drive: &dyn DriveRepository) -> Result<(), String> {
    const BOT: &str = "pragmatismo";
    let bucket = format!("{BOT}.gbai");
    drive
        .create_bucket_if_not_exists(&bucket)
        .await
        .map_err(|e| format!("create bucket {bucket}: {e}"))?;

    let base = |folder: &str, name: &str| format!("{BOT}.{folder}/{name}");

    put_if_missing(drive, &bucket, &base("gbot", "config.csv"), "text/csv", pragmatismo_config_csv().as_bytes()).await?;
    put_if_missing(drive, &bucket, &base("gbot", "PROMPT.md"), "text/markdown", pragmatismo_prompt_md().as_bytes()).await?;
    put_if_missing(drive, &bucket, &base("gbdialog", "start.bas"), "text/plain", pragmatismo_start_bas().as_bytes()).await?;
    let vibe_run_bas = r##"' vibe_run.bas — demo bridge script: surfaces the VIBE toolset.
VIBE TOOLS
VIBE RUN "Criar um website institucional""##;
    put_if_missing(drive, &bucket, &base("gbdialog", "vibe_run.bas"), "text/plain", vibe_run_bas.as_bytes()).await?;

    put_if_missing(drive, &bucket, &base("gbdialog", "vibe-run.mcp.json"), "application/json",
        pragmatismo_tool_json("vibe-run", "Cria um run do agente Vibe (projeto, website ou aplicação)").as_bytes()).await?;
    put_if_missing(drive, &bucket, &base("gbdialog", "vibe-status.mcp.json"), "application/json",
        pragmatismo_tool_json("vibe-status", "Mostra o estado de um run do Vibe").as_bytes()).await?;
    put_if_missing(drive, &bucket, &base("gbdialog", "vibe-approve.mcp.json"), "application/json",
        pragmatismo_tool_json("vibe-approve", "Aprova os tool calls pendentes de um run").as_bytes()).await?;
    put_if_missing(drive, &bucket, &base("gbdialog", "vibe-cancel.mcp.json"), "application/json",
        pragmatismo_tool_json("vibe-cancel", "Cancela um run do Vibe").as_bytes()).await?;
    put_if_missing(drive, &bucket, &base("gbdialog", "vibe-events.mcp.json"), "application/json",
        pragmatismo_tool_json("vibe-events", "Stream de eventos de progresso de um run").as_bytes()).await?;

    Ok(())
}