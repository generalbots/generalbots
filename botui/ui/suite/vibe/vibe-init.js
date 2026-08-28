function handleVibeSubmit(e) {
e.preventDefault();
var input = document.getElementById("vibeChatInput");
if (!input) return;
var text = input.value.trim();
if (!text) return;
input.value = "";

vibeAddMsg("user", text);

if (
typeof window.VibeRun !== "undefined" &&
window.VibeRun &&
typeof window.VibeRun.start === "function"
) {
window.VibeRun.start(text).catch(function () {
callAutotask(text);
});
} else {
callAutotask(text);
}
}

function vibeQuickSubmit(text) {
    // The runner chat form is gone — a submitted prompt IS a run.
    if (!text || !text.trim()) return;
    if (
        typeof window.VibeRun !== "undefined" &&
        window.VibeRun &&
        typeof window.VibeRun.start === "function"
    ) {
        window.VibeRun.start(text.trim()).catch(function () {
            callAutotask(text.trim());
        });
    } else {
        callAutotask(text.trim());
    }
}

function setupPipelineTabs() {
// The ribbon is wired by vibe-pipeline.js (tab → command-group switching).
// Kept as a no-op entry point for backward compatibility.
}

function setupSidebarCollapse() {
var btn = document.getElementById("agentsSidebarCollapse");
var sidebar = document.getElementById("agentsSidebar");
if (!btn || !sidebar) return;
btn.addEventListener("click", function () {
sidebar.classList.toggle("collapsed");
btn.textContent = sidebar.classList.contains("collapsed")
? "▶"
: "◀";
});
}

function setupWorkspaceAccordions() {
    document.addEventListener("click", function (e) {
        var toggle = e.target.closest(".as-workspace-toggle");
        if (!toggle) return;
        var body = toggle.nextElementSibling;
        var arrow = toggle.querySelector(".as-workspace-arrow");
        if (body) {
            var isOpen = body.style.display !== "none";
            body.style.display = isOpen ? "none" : "";
            if (arrow) arrow.textContent = isOpen ? "▶" : "▼";
        }
    });
}

function setupSidebarActions() {
    // "+ Create a New Project" opens the real New Project modal (the old
    // handler created a DOM-only fake workspace — removed 2026-08-14).
    var wsBtn = document.getElementById("createWorkspaceBtn");
    if (wsBtn) {
        wsBtn.addEventListener("click", function () {
            if (window.VibeNewProject) window.VibeNewProject.open();
        });
    }
}

function initVibe() {
    setupPipelineTabs();
    setupSidebarCollapse();
    setupWorkspaceAccordions();
    setupSidebarActions();

    var form = document.getElementById("vibeChatForm");
    if (form) form.addEventListener("submit", handleVibeSubmit); // legacy guard — the form was removed with the runner chat

    // Make floating panels (chat, run dock, graph, metrics) draggable and
    // resizable. The run dock arrives via HTMX after load, so wire again on
    // swap; vibeWireWindowPanels() is idempotent (per-element data guard).
    if (typeof vibeWireWindowPanels === "function") vibeWireWindowPanels();
    if (window.htmx) {
        document.body.addEventListener("htmx:afterSwap", function () {
            if (typeof vibeWireWindowPanels === "function") vibeWireWindowPanels();
        });
    }

    connectVibeWs();
}

if (document.readyState === "loading") {
(function(){ var __cb = initVibe; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
} else {
initVibe();
}
