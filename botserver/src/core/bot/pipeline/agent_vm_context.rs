//! Agent-VM prompt context (#1580 follow-up).
//!
//! The `agent-vm` feature is part of the desktop build, so this module compiles
//! unconditionally and gates its body internally: callers do not need `cfg`
//! blocks around the prompt assembly.

use serde_json::Value;
use uuid::Uuid;

/// Injects the workspace contract for the user's VM when one is running.
pub(super) async fn inject(
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    messages: &mut Vec<Value>,
) {
    #[cfg(feature = "agent-vm")]
    {
        if let Some(vm_session) =
            crate::core::bot::agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await
        {
            let conv_id = vm_session.active_conversation_id.as_deref().unwrap_or("default");
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
                     Add \\\"background\\\": true for long jobs (renders, downloads, training): \
                     the job keeps running while the user moves to another conversation and \
                     its result is saved into THIS conversation. \
                     Save generated artifacts into output/ so the user can preview them. \
                     Files the user names may already be in data/ (fetched from their Drive); \
                     read them from there. Use `drive_fetch` to pull a Drive file into data/ \
                     and `drive_save` to publish output/ files to the user's Drive under the \
                     conversation folder. \
                     After the tool returns, summarise the result and list each file as a \
                     markdown link in the form [name](vmfile:{conv_id}/output/name). \
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
                     Do NOT refuse tasks that require code execution — you have a \
                     full Linux VM available. Generate the program dynamically based \
                     on what the user needs."
                ),
            }));
        }
    }
    #[cfg(not(feature = "agent-vm"))]
    {
        let _ = (session_id, user_id, bot_uuid, messages);
    }
}
