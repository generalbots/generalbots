function updateMantis1(status, detail) {
var card = document.querySelector(
'.as-agent-card[data-agent-id="1"]',
);
if (!card) return;
var bar = card.querySelector(".as-agent-bar .as-bar-fill");
if (status === "working") {
card.style.borderLeftColor = "#f59e0b";
if (!card.querySelector(".as-agent-bar")) {
var barWrapper = document.createElement("div");
barWrapper.className = "as-agent-bar";
barWrapper.innerHTML =
'<div class="as-bar-fill bred" style="width:0%;transition:width 0.5s;"></div>';
card.appendChild(barWrapper);
}
} else if (status === "done") {
card.style.borderLeftColor = "var(--accent)";
bar = card.querySelector(".as-bar-fill");
if (bar) bar.style.width = "100%";
setTimeout(function () {
var b = card.querySelector(".as-agent-bar");
if (b) b.remove();
}, 2000);
}
}

function updateAgentCard(agentId, status, detail) {
var card = document.querySelector(
'.as-agent-card[data-agent-id="' + agentId + '"]',
);
if (!card) return;
card.style.opacity = "1";

var badge = card.querySelector(".as-badge");
var dot = card.querySelector(".as-status-dot");

if (status === "WORKING") {
card.style.borderLeft = "3px solid #f59e0b";
if (dot) {
dot.className = "as-status-dot yellow";
}
if (badge) {
badge.textContent = "WORKING";
badge.className = "as-badge badge-bred";
}
if (!card.querySelector(".as-agent-bar")) {
var barWrapper = document.createElement("div");
barWrapper.className = "as-agent-bar";
barWrapper.innerHTML =
'<div class="as-bar-fill bred" style="width:0%;transition:width 0.5s;"></div>';
card.appendChild(barWrapper);
}
} else if (status === "EVOLVED" || status === "DONE") {
card.style.borderLeft = "3px solid var(--accent)";
if (dot) {
dot.className = "as-status-dot green";
}
if (badge) {
badge.textContent = "EVOLVED";
badge.className = "as-badge badge-evolved";
}
var agBar = card.querySelector(".as-bar-fill");
if (agBar) agBar.style.width = "100%";
setTimeout(function () {
var b = card.querySelector(".as-agent-bar");
if (b) b.remove();
}, 2000);
} else if (status === "BRED") {
card.style.borderLeft = "3px solid #f59e0b";
if (dot) {
dot.className = "as-status-dot yellow";
}
if (badge) {
badge.textContent = "BRED";
badge.className = "as-badge badge-bred";
}
} else if (status === "FAILED") {
card.style.borderLeft = "3px solid #ef4444";
if (dot) {
dot.className = "as-status-dot red";
}
if (badge) {
badge.textContent = "FAILED";
badge.className = "as-badge badge-bred";
badge.style.background = "#ef4444";
}
}

if (detail) {
var detailEl = card.querySelector(".as-agent-detail");
if (!detailEl) {
detailEl = document.createElement("span");
detailEl.className = "as-agent-detail";
detailEl.style.cssText =
"font-size:10px;color: var(--text-muted);display:block;padding:0 12px 4px;";
var body = card.querySelector(".as-agent-body");
if (body) body.after(detailEl);
}
detailEl.textContent = detail;
}
}
