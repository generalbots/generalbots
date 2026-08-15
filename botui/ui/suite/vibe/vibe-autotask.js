/**
 * Legacy fallback for when the real AgentLoop (window.VibeRun.start) is
 * unavailable. #820 — this must NOT fabricate a fake plan. It reports the
 * failure honestly so the user retries against the AgentLoop instead of
 * seeing invented "Project Setup" / "Database Schema" nodes.
 */
function callAutotask(intent) {
    vibeAddMsg("system", "🔌 AgentLoop unavailable — could not start a Vibe run.");
    vibeAddMsg(
        "bot",
        "I could not reach the agent backend for: **" + esc(intent) + "**.\n\n" +
            "This request was not executed. Please retry — if the problem persists, " +
            "the Vibe run service is down and needs a restart.",
    );
}
