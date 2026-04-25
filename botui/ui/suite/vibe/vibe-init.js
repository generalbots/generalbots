function handleVibeSubmit(e) {
e.preventDefault();
var input = document.getElementById("vibeChatInput");
if (!input) return;
var text = input.value.trim();
if (!text) return;
input.value = "";

vibeAddMsg("user", text);

callAutotask(text);
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
var toggles = document.querySelectorAll(".as-workspace-toggle");
toggles.forEach(function (toggle) {
toggle.addEventListener("click", function () {
var body = this.nextElementSibling;
var arrow = this.querySelector(".as-workspace-arrow");
if (body) {
var isOpen = body.style.display !== "none";
body.style.display = isOpen ? "none" : "";
if (arrow) arrow.textContent = isOpen ? "▶" : "▼";
}
});
});
}

function initVibe() {
setupPipelineTabs();
setupSidebarCollapse();
setupWorkspaceAccordions();

var form = document.getElementById("vibeChatForm");
if (form) form.addEventListener("submit", handleVibeSubmit);

connectVibeWs();
}

if (document.readyState === "loading") {
document.addEventListener("DOMContentLoaded", initVibe);
} else {
initVibe();
}
