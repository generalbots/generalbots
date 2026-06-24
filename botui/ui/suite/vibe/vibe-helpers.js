function esc(text) {
var d = document.createElement("div");
d.textContent = text || "";
return d.innerHTML;
}

function vibeAddMsg(role, text) {
var box = document.getElementById("vibeChatMessages");
if (!box) return;
var div = document.createElement("div");
if (role === "user") {
div.style.cssText =
"align-self:flex-end;background:var(--accent);color:var(--surface);font-weight:500;padding:10px 14px;border-radius:12px 12px 0 12px;max-width:85%;word-wrap:break-word;";
div.textContent = text;
} else if (role === "system") {
div.style.cssText =
"align-self:center;background:rgba(132,214,105,0.12);color: var(--accent);padding:6px 12px;border-radius:8px;font-size:11px;text-align:center;";
div.innerHTML = text;
} else {
div.style.cssText =
"align-self:flex-start;background:var(--surface-hover);color:var(--text);padding:10px 14px;border-radius:12px 12px 12px 0;max-width:85%;word-wrap:break-word;";
div.className = "vibe-bot-msg";
if (typeof marked !== "undefined" && marked.parse) {
div.innerHTML = marked.parse(text);
} else {
div.textContent = text;
}
}
box.appendChild(div);
box.scrollTop = box.scrollHeight;
return div;
}

function vibeAddStreamStart() {
vibeStreamId = "vibe-stream-" + Date.now();
vibeStreamContent = "";
var el = vibeAddMsg("bot", "▍");
if (el) el.id = vibeStreamId;
return el;
}

function vibeUpdateStream(content) {
vibeStreamContent += content || "";
var el = document.getElementById(vibeStreamId);
if (!el) return;
if (typeof marked !== "undefined" && marked.parse) {
el.innerHTML = marked.parse(vibeStreamContent);
} else {
el.textContent = vibeStreamContent;
}
var box = document.getElementById("vibeChatMessages");
if (box) box.scrollTop = box.scrollHeight;
}

function vibeFinalizeStream() {
var el = document.getElementById(vibeStreamId);
if (el) {
if (typeof marked !== "undefined" && marked.parse) {
el.innerHTML = marked.parse(vibeStreamContent);
} else {
el.textContent = vibeStreamContent;
}
el.removeAttribute("id");
}
vibeStreamId = null;
vibeStreamContent = "";
vibeStreaming = false;
}

function setVibeStatus(status) {
var dot = document.getElementById("vibeChatStatusDot");
var badge = document.getElementById("vibeChatStatusBadge");
if (status === "connected") {
if (dot) {
dot.className = "as-status-dot green";
dot.style.boxShadow = "0 0 8px var(--accent)";
}
if (badge) {
badge.textContent = "EVOLVED";
badge.style.background = "var(--accent)";
badge.style.color = "var(--bg)";
}
} else if (status === "connecting") {
if (dot) {
dot.className = "as-status-dot yellow";
dot.style.boxShadow = "0 0 8px #f59e0b";
}
if (badge) {
badge.textContent = "CONNECTING…";
badge.style.background = "var(--surface-hover)";
badge.style.color = "var(--text-muted)";
}
} else {
if (dot) {
dot.className = "as-status-dot red";
dot.style.boxShadow = "0 0 8px #ef4444";
}
if (badge) {
badge.textContent = "OFFLINE";
badge.style.background = "var(--surface-hover)";
badge.style.color = "var(--text-muted)";
}
}
}
