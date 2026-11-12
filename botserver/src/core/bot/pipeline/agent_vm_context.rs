//! Agent-VM prompt context (#1580 follow-up).
//!
//! The `agent-vm` feature is part of the desktop build, so this module compiles
//! unconditionally and gates its body internally: callers do not need `cfg`
//! blocks around the prompt assembly.

use serde_json::Value;

use super::types::SessionIdentity;
use super::workspace_progress::{WorkspaceProgress, PHASE_INFRA};

/// Injects the workspace contract for the user's VM when one is running.
///
/// #1807 — provisioning is visible in the chat run timeline. A cold Incus
/// start can take minutes before the model is even called; this is the first
/// stage the user needs to see, not a silent spinner.
pub(super) async fn inject(identity: &SessionIdentity<'_>, messages: &mut Vec<Value>) {
    // #2239/#2264 — the Agent (VM) switch is off: never provision the
    // workspace and never hand the model the VM contract.
    if !super::agent_turn::workspace_route_allowed(identity.agent_vm_enabled) {
        return;
    }
    #[cfg(feature = "agent-vm")]
    {
        let SessionIdentity {
            session_id,
            user_id,
            bot_uuid,
            ..
        } = *identity;
        let progress = WorkspaceProgress::new(identity);
        if let Some(progress) = progress.as_ref() {
            progress
                .start(
                    PHASE_INFRA,
                    "queued",
                    "Workspace queued",
                    "Checking your workspace",
                )
                .await;
            progress.finish(PHASE_INFRA, "queued", "ok", "").await;
            progress
                .start(
                    PHASE_INFRA,
                    "container_start",
                    "Starting the workspace",
                    "Image pull, container boot and bootstrap can take minutes on a cold machine",
                )
                .await;
        }

        let vm =
            crate::core::bot::agent_vm_hook::ensure_agent_vm_result(session_id, user_id, bot_uuid)
                .await;
        if let Some(progress) = progress.as_ref() {
            match &vm {
                Ok(_) => {
                    progress
                        .finish(PHASE_INFRA, "container_start", "ok", "Workspace ready")
                        .await;
                }
                Err((_, error)) => {
                    progress
                        .finish(PHASE_INFRA, "container_start", "error", error)
                        .await;
                }
            }
        }

        if let Err((_, error)) = &vm {
            botlog::safe_warn!("auto-provision VM for user {user_id}: {error}");
        }

        if let Ok(vm_session) = vm {
            let conv_id = vm_session
                .active_conversation_id
                .as_deref()
                .unwrap_or("default");
            let vm_name = &vm_session.vm_name;
            let status = &vm_session.status;
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!(
                    "You have a running VM workspace '{vm_name}' (status: {status}). \
                     The current conversation folder is /opt/vibe/conversations/{conv_id}. \
                     Subfolders: output/ for generated files, program/ for scripts, \
                     data/ for input data. \
                     You run code by calling the native tool `vm_exec`. Do not claim you \
                     ran something without calling it. Emit exactly one JSON object, on \
                     its own line:\n\
                     {{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": {{\"language\": \"python\", \"code\": \"<program>\"}}}}\n\
                     `language` is python, node or shell. The tool writes the program into \
                     program/, runs it in the conversation folder, and returns stdout, \
                     stderr and every file it finds in output/. \
                     Live terminal: when the user explicitly asks to watch a command or \
                     take over a terminal, call `terminal_run` with \
                     {{\"command\": \"<bash command>\", \"timeout_ms\": 60000}}. It runs in a \
                     real PTY inside this VM, streams terminal output to the run card and \
                     lets the user take over. Use vm_exec for scripts that produce files. \
                     Dependencies: prefer the language standard library. Declare EVERY \
                     third-party package the program imports in \\\"packages\\\" (pip for \
                     python, npm for node), EVEN the preinstalled ones: a declared package \
                     is installed before the program starts, so the first run on a cold \
                     workspace cannot race the background bootstrap. Never call pip/npm inside \
                     the program, e.g. \\\"packages\\\":[\\\"reportlab\\\"] for PDFs and \
                     \\\"packages\\\":[\\\"pandas\\\",\\\"openpyxl\\\"] for spreadsheets. Use the distribution \
                     name for the import (docx -> python-docx, PIL -> pillow, yaml -> pyyaml, \
                     cv2 -> opencv-python-headless, sklearn -> scikit-learn, bs4 -> \
                     beautifulsoup4). The workspace ships with pandas, numpy, openpyxl, \
                     python-pptx, pillow, matplotlib, requests, python-docx, reportlab, \
                     beautifulsoup4, lxml and tabulate; the background bootstrap installs them too. If a run \
                     reports a missing module, add it to packages and run again. \
                     BotModels gateway: host services (image generation, vision/BLIP, speech TTS/STT, \
                     music, video, OCR, scoring, anomaly) are reachable at \
                     http://host.docker.internal:8080/api/botmodels/<service>/<endpoint> with header \
                     X-API-Key from BOTMODELS_API_KEY; prefer them over PIL/numpy for media tasks. \
                     Add \\\"background\\\": true for long jobs (renders, downloads, training): \
                     the job keeps running while the user moves to another conversation and \
                     its result is saved into THIS conversation. \
                      Browser automation: to capture screenshots or scrape a website, use \
                      selenium, playwright or puppeteer inside the VM (declare the package in \
                      packages) and write the captured image to output/. Prefer the browser app \
                      for interactive browsing; use the VM for batch capture. \
                     Save generated artifacts into output/ so the user can preview them. \
                     Every new or changed file left in output/ is published to the user's \
                     Drive automatically when the turn ends and its vmfile link is attached \
                     to the answer, so you do not need to call drive_save just to make an \
                     artifact durable; call it only to publish a specific file on demand or \
                     into a different folder. \
                     WORKSPACE TRACE (mandatory for programs with more than one step): \
                     print one line to stdout when each meaningful block starts and when \
                     it ends, so the chat run card shows a live task table with timing. A \
                     marker is the literal prefix @@GB_TRACE@@ followed by one JSON object: \
                     print('@@GB_TRACE@@ {{\"task\": \"install deps\", \"detail\": \"pip install pandas\", \"status\": \"started\"}}') \
                     ...work... \
                     print('@@GB_TRACE@@ {{\"task\": \"install deps\", \"status\": \"ok\"}}') \
                     Use status \"started\" at the beginning and \"ok\" (or \"error\") at the \
                     end; keep the task text identical in both lines; detail is optional \
                     and says what the block is doing. Emit 3 to 8 tasks per program \
                     (install, generate, build, test, write). The markers are stripped from \
                     the visible output: they only feed the table. \
                     Files the user names may already be in data/ (fetched from their Drive); \
                     read them from there. Use `drive_list` to discover files in the bot Drive (newest first). \
                     Use `drive_fetch` to pull a named Drive file into data/ and `drive_save` to publish \
                     output/ files to the user's Drive under the conversation folder when asked. When the \
                     user asks to save to Google Drive or \
                     OneDrive, call `save_to_external_drive` with \"provider\": \"gdrive\" \
                     or \"onedrive\" (plus optional \"files\" and \"folder\"); it writes \
                     straight to the account the user connected and returns a share link. If \
                     the account is not connected or only allows reading, report that and tell \
                     the user to grant write access in Drive -> External; never claim a file \
                     was saved when it was not. \
                     After the tool returns, summarise the result and list each file as a \
                     markdown link in the form [name](vmfile:{conv_id}/output/name). \
                     `drive_save` already returns each Drive file as \
                     [name](app://drive?path=<key>&scope=bot): repeat those links verbatim. \
                     These two are the only link forms the chat renders — never emit a \
                     drive: URL, it stays dead text and the user cannot open the file. \
                     PROGRESS ETIQUETTE (mandatory): the user stares at a spinner while \
                     code runs, so keep them informed. Before every tool call write one \
                     short line, in the user's language, saying what you are about to do \
                     ('Vou montar o PDF com o layout pedido…'). After every tool result \
                     write one short line saying what changed ('PDF pronto (12 KB). Agora \
                     o PPTX…'). One line per step, under 15 words, no headings; never \
                     repeat the same sentence; never claim a file exists before the tool \
                     returned it; if a job is long (background: true), say it is running \
                     and that the user can keep chatting while it finishes. \
                     Background work continues when the user switches conversations; \
                     never assume a job was cancelled. \
                     For news, current events, weather, prices or any informational \
                     request about the outside world, do NOT use vm_exec: rely on \
                     the web grounding injected for the turn or call the web.search \
                     command and cite every source. \
                     Do NOT refuse tasks that require code execution — you have a \
                     full Linux VM available. Generate the program dynamically based \
                     on what the user needs."
                ),
            }));
        }
    }
    #[cfg(not(feature = "agent-vm"))]
    {
        let _ = (identity, messages);
    }
}