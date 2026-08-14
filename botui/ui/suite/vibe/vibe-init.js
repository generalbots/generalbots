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
    var input = document.getElementById("vibeChatInput");
    if (!input) return;
    input.value = text;
    var form = document.getElementById("vibeChatForm");
    if (!form) return;
    if (form.requestSubmit) {
        form.requestSubmit();
    } else {
        form.dispatchEvent(
            new Event("submit", { bubbles: true, cancelable: true }),
        );
    }
}

function setupPipelineTabs() {
var container = document.querySelector(".vibe-pipeline");
if (!container) return;
container.addEventListener("click", function (e) {
var tab = e.target.closest(".vibe-pipeline-tab");
if (!tab) return;
container
.querySelectorAll(".vibe-pipeline-tab")
.forEach(function (t) {
t.classList.remove("active");
});
tab.classList.add("active");
});
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
    if (form) form.addEventListener("submit", handleVibeSubmit);

    connectVibeWs();
}

if (document.readyState === "loading") {
(function(){ var __cb = initVibe; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
} else {
initVibe();
}
