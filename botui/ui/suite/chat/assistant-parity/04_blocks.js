/**
 * 04_blocks.js - assistant-style activity blocks inside the classic chat app.
 *
 * Chat re-dispatches every WebSocket frame on `document` as
 * `CustomEvent("gb-ws-frame", { detail: data })` (20_tabs.js, 61_terminal_use.js).
 * This module listens for those frames and mirrors the assistant's activity
 * cards into a dedicated stack. It NEVER touches #messages, .message rows or
 * any other part of chat's message DOM: the stack is appended to
 * #chatContentWrapper as a sibling of the transcript.
 */
(function () {
"use strict";

var P = window.GBChatAssistantParity = window.GBChatAssistantParity || {};

var MAX_CARDS = 20;
var DISMISS_MS = 9000;
var mounted = false;
var stackEl = null;

/* i18n keys (all user-facing strings go through gbT; English is the fallback) */
var K = {
  thinking: "assistant-parity-block-thinking",
  attachment_read: "assistant-parity-block-attachment-read",
  drive_list: "assistant-parity-block-drive-list",
  drive_search: "assistant-parity-block-drive-search",
  drive_read: "assistant-parity-block-drive-read",
  kb_search: "assistant-parity-block-kb-search",
  api_command: "assistant-parity-block-api-command",
  tool_run: "assistant-parity-block-tool-run",
  permission: "assistant-parity-block-permission",
  notice: "assistant-parity-block-notice",
  error: "assistant-parity-block-error",
  retry: "assistant-parity-block-retry",
  step: "assistant-parity-block-step",
  running: "assistant-parity-block-status-running",
  done: "assistant-parity-block-status-done",
  failed: "assistant-parity-block-status-failed",
  needs: "assistant-parity-block-status-needs-you",
  allow: "assistant-parity-block-allow",
  deny: "assistant-parity-block-deny",
  allowed: "assistant-parity-block-allowed",
  denied: "assistant-parity-block-denied",
  decision_failed: "assistant-parity-block-decision-failed",
  file: "assistant-parity-block-file",
  type: "assistant-parity-block-type",
  size: "assistant-parity-block-size",
  path: "assistant-parity-block-path",
  query: "assistant-parity-block-query",
  total: "assistant-parity-block-total",
  result: "assistant-parity-block-result",
  detail: "assistant-parity-block-detail",
  reason: "assistant-parity-block-reason",
  command: "assistant-parity-block-command",
  tool: "assistant-parity-block-tool",
  read: "assistant-parity-block-read",
  chars: "assistant-parity-block-chars",
  elapsed: "assistant-parity-block-elapsed", seconds: "assistant-parity-block-seconds",
  truncated: "assistant-parity-block-truncated", yes: "assistant-parity-block-yes"
};

var F = {
  thinking: "Thinking", attachment_read: "Attachment read", drive_list: "Drive listing",
  drive_search: "Drive search", drive_read: "File read", kb_search: "Knowledge base",
  api_command: "Command", tool_run: "Tool", permission: "Permission", notice: "Notice",
  error: "Error", retry: "Retry", step: "Activity",
  running: "Running", done: "Done", failed: "Failed", needs: "Needs you",
  allow: "Allow", deny: "Deny", allowed: "Allowed", denied: "Denied",
  decision_failed: "Could not record your decision",
  file: "File", type: "Type", size: "Size", path: "Path", query: "Query", total: "Total",
  result: "Result", detail: "Detail", reason: "Reason", command: "Command", tool: "Tool",
  read: "Read", chars: "chars", elapsed: "Elapsed", seconds: "s",
  truncated: "Truncated", yes: "Yes"
};

var TYPES = {};
["thinking", "attachment_read", "drive_list", "drive_search", "drive_read", "kb_search",
 "api_command", "tool_run", "permission", "notice", "error", "retry"].forEach(function (x) { TYPES[x] = true; });

var ALIASES = {
  thought: "thinking", attachment: "attachment_read", attachmentread: "attachment_read",
  listing: "drive_list", search: "drive_search", read: "drive_read", fileread: "drive_read",
  kb: "kb_search", knowledge: "kb_search", command: "api_command", api: "api_command",
  tool: "tool_run", toolcall: "tool_run", consent: "permission",
  permissionrequest: "permission", info: "notice", warning: "notice",
  failed: "error", exception: "error"
};
/* ---------- helpers ---------- */
function tr(key, fallback) {
  try { if (typeof window.gbT === "function") return window.gbT(key, fallback); } catch (e) { /* unavailable */ }
  return fallback;
}
function label(which) { return tr(K[which] || which, F[which] || which); }

function asObject(v) { return v && typeof v === "object" && !Array.isArray(v) ? v : null; }

function firstTruthy() {
  for (var i = 0; i < arguments.length; i++) {
    var v = arguments[i];
    if (v !== undefined && v !== null && v !== "") return v;
  }
  return "";
}

function normalizeType(raw) {
  var v = String(raw == null ? "" : raw).toLowerCase().trim();
  v = v.replace(/^gb\.activity[:.]?/, "").replace(/^activity[:._-]?/, "").replace(/^custom[:.]?/, "").replace(/^gb[-_]?/, "").replace(/\s+/g, "_");
  if (TYPES[v]) return v;
  if (ALIASES[v]) return ALIASES[v];
  return "";
}

function readTypeFrom(obj) {
  if (!obj) return "";
  var keys = ["activity_type", "kind", "type", "event_type", "event", "name"];
  for (var i = 0; i < keys.length; i++) {
    var found = normalizeType(obj[keys[i]]);
    if (found) return found;
  }
  return "";
}

function mapStatus(s) {
  var v = String(s == null ? "" : s).toLowerCase().trim();
  if (v === "started" || v === "running" || v === "in_progress") return "running";
  if (v === "needs_input" || v === "needs-you" || v === "pending") return "needs-you";
  if (v === "failed" || v === "error") return "failed";
  return "done";
}

function fmtBytes(b) {
  var n = Number(b);
  if (!isFinite(n) || n < 0) return "";
  if (n < 1024) return n + " B";
  if (n < 1048576) return (n / 1024).toFixed(0) + " KB";
  if (n < 1073741824) return (n / 1048576).toFixed(1) + " MB";
  return (n / 1073741824).toFixed(1) + " GB";
}

function authHeaders(extra) {
  var h = extra || {};
  try {
    var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
      (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
      sessionStorage.getItem("gb-access-token") || "";
    if (token) h.Authorization = "Bearer " + token;
  } catch (e) { /* anonymous */ }
  return h;
}

/* ---------- frame parsing ---------- */

function pack(type, data, id, status) {
  return { type: type, data: asObject(data) || {}, id: id ? String(id) : "", status: status || "" };
}

function extractFrame(detail) {
  var root = asObject(detail);
  if (!root) return null;

  var consent = asObject(root.consent_request) || asObject(root.data && root.data.consent_request);
  if (consent && consent.request_id) {
    return pack("permission", consent, consent.request_id, "needs_input");
  }

  if (root.gb_event === true) {
    var dd = asObject(root.data);
    if (dd && dd.name === "gb.activity") {
      var pay = asObject(dd.payload) || asObject(dd.data) || dd;
      var payType = normalizeType(pay.kind || pay.type || pay.activity_type);
      if (payType) return pack(payType, pay, firstTruthy(pay.id, dd.id), pay.status);
    }
  }

  var obj = asObject(root.activity) || asObject(root.data && root.data.activity) || root;
  var type = readTypeFrom(obj);
  if (!type && obj !== root) {
    obj = asObject(root.data) || obj;
    type = readTypeFrom(obj);
  }
  if (!type) return null;

  var data = asObject(obj.summary) || asObject(obj.payload) || asObject(obj.data) || obj;
  var id = firstTruthy(obj.id, obj.activity_id, data.id, data.request_id, root.id);
  return pack(type, data, id, firstTruthy(obj.status, data.status));
}

/* ---------- card construction (DOM only; model data never hits innerHTML) ---------- */

function addRow(body, key, value) {
  if (value === undefined || value === null || value === "") return;
  var row = document.createElement("div");
  row.className = "gb-parity-block-row";
  var lab = document.createElement("span");
  lab.className = "gb-parity-block-row-label";
  lab.textContent = label(key);
  var val = document.createElement("span");
  val.className = "gb-parity-block-row-value";
  val.textContent = String(value);
  row.appendChild(lab);
  row.appendChild(val);
  body.appendChild(row);
}

function fillBody(type, d, body) {
  var n;
  switch (type) {
    case "thinking":
      addRow(body, "reason", d.reasoning || d.reason || "");
      if (d.elapsed) addRow(body, "elapsed", d.elapsed + " " + label("seconds"));
      break;
    case "attachment_read":
      addRow(body, "file", d.name || d.file || "");
      addRow(body, "type", d.type || d.content_type || "");
      n = firstTruthy(d.size, d.size_bytes);
      if (n !== "") addRow(body, "size", fmtBytes(n));
      if (d.chars_read) addRow(body, "read", d.chars_read + " " + label("chars"));
      if (d.truncated) addRow(body, "truncated", label("yes"));
      addRow(body, "detail", d.error || "");
      break;
    case "drive_list":
    case "drive_search":
      addRow(body, "path", d.path || "");
      addRow(body, "query", d.query || "");
      n = firstTruthy(d.total, (d.entries || d.items || d.results || []).length);
      if (n !== "" && n !== 0) addRow(body, "total", n);
      break;
    case "drive_read":
      addRow(body, "file", d.name || d.path || "");
      if (d.size) addRow(body, "size", fmtBytes(d.size));
      break;
    case "kb_search":
      addRow(body, "query", d.query || "");
      n = firstTruthy(d.total, (d.results || d.sources || []).length);
      if (n !== "" && n !== 0) addRow(body, "total", n + " " + label("result"));
      break;
    case "api_command":
      addRow(body, "command", d.command || "");
      addRow(body, "result", d.status || d.outcome || "");
      break;
    case "tool_run":
      addRow(body, "tool", d.tool || d.name || "");
      addRow(body, "result", d.outcome || d.result || "");
      break;
    case "permission":
      addRow(body, "detail", d.detail || d.description || d.action || "");
      addRow(body, "path", d.app_id || "");
      break;
    case "notice":
    case "error":
    case "retry":
      addRow(body, "detail", d.message || d.error || d.reason || "");
      break;
    default:
      addRow(body, "detail", d.message || d.summary || "");
  }
}

function sourceFor(type, d) {
  switch (type) {
    case "attachment_read": return firstTruthy(d.name, d.file);
    case "drive_read": return firstTruthy(d.name, d.path);
    case "drive_list": return firstTruthy(d.path, d.query);
    case "drive_search": case "kb_search": return d.query;
    case "api_command": return d.command;
    case "tool_run": return firstTruthy(d.tool, d.name);
    default: return firstTruthy(d.source, d.file);
  }
}

function setStatus(el, status) {
  var s = mapStatus(status);
  ["running", "done", "failed", "needs-you"].forEach(function (n) { el.classList.remove("gb-parity-block--" + n); });
  el.classList.add("gb-parity-block--" + s);
  el.setAttribute("data-status", s);
  var chip = el.querySelector(".gb-parity-block-chip");
  if (chip) {
    chip.className = "gb-parity-block-chip gb-parity-block-chip--" + s;
    chip.textContent = label(s === "needs-you" ? "needs" : s);
  }
}

function buildActions(type, d, el) {
  if (type !== "permission") return;
  var requestId = firstTruthy(d.request_id, d.id);
  if (!requestId) return;
  var actions = document.createElement("div");
  actions.className = "gb-parity-block-actions";
  var allow = document.createElement("button");
  allow.type = "button";
  allow.className = "gb-parity-block-btn gb-parity-block-btn--allow";
  allow.textContent = label("allow");
  var deny = document.createElement("button");
  deny.type = "button";
  deny.className = "gb-parity-block-btn gb-parity-block-btn--deny";
  deny.textContent = label("deny");
  allow.addEventListener("click", function () { resolveConsent(el, requestId, "allow_once"); });
  deny.addEventListener("click", function () { resolveConsent(el, requestId, "deny"); });
  actions.appendChild(allow);
  actions.appendChild(deny);
  el.appendChild(actions);
}

function buildCard(type, data, id, status) {
  var d = asObject(data) || {};
  var initial = mapStatus(status || (type === "permission" ? "needs_input" : type === "thinking" ? "started" : "done"));
  var el = document.createElement("div");
  el.className = "gb-parity-block gb-parity-block--" + initial;
  el.setAttribute("data-status", initial);
  if (id) el.setAttribute("data-activity-id", String(id));

  var head = document.createElement("div");
  head.className = "gb-parity-block-header";
  var title = document.createElement("span");
  title.className = "gb-parity-block-title";
  title.textContent = label(TYPES[type] ? type : "step");
  var chip = document.createElement("span");
  chip.className = "gb-parity-block-chip gb-parity-block-chip--" + initial;
  chip.textContent = label(initial === "needs-you" ? "needs" : initial);
  head.appendChild(title);
  head.appendChild(chip);
  var source = sourceFor(type, d);
  if (source) {
    var src = document.createElement("span");
    src.className = "gb-parity-block-source";
    src.textContent = String(source);
    head.appendChild(src);
  }
  var body = document.createElement("div");
  body.className = "gb-parity-block-body";
  body.setAttribute("role", "status");
  body.setAttribute("aria-live", "polite");
  fillBody(type, d, body);
  el.appendChild(head);
  el.appendChild(body);
  buildActions(type, d, el);
  return el;
}

/* ---------- consent resolution ---------- */

function resolveConsent(el, requestId, decision) {
  var buttons = el.querySelectorAll(".gb-parity-block-btn");
  Array.prototype.forEach.call(buttons, function (b) { b.disabled = true; });
  var headers = authHeaders({ "Content-Type": "application/json" });
  try {
    fetch("/api/consent/resolve", {
      method: "POST",
      headers: headers,
      body: JSON.stringify({ request_id: requestId, decision: decision })
    }).then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      setStatus(el, "done");
      addRow(el.querySelector(".gb-parity-block-body"), "result",
        label(decision === "deny" ? "denied" : "allowed"));
    }).catch(function () {
      setStatus(el, "failed");
      addRow(el.querySelector(".gb-parity-block-body"), "detail", label("decision_failed"));
    });
  } catch (e) { setStatus(el, "failed"); }
}

/* ---------- stack lifecycle ---------- */

function ensureStack() {
  var host = document.getElementById("chatContentWrapper");
  if (!host) return null;
  var el = host.querySelector(":scope > .gb-parity-block-stack");
  if (!el) {
    el = document.createElement("div");
    el.className = "gb-parity-block-stack";
    el.setAttribute("aria-label", label("step"));
    host.appendChild(el);
  }
  stackEl = el;
  return el;
}

function findCard(id) {
  if (!id || !stackEl) return null;
  for (var i = 0; i < stackEl.children.length; i++) {
    var c = stackEl.children[i];
    if (c.getAttribute && c.getAttribute("data-activity-id") === String(id)) return c;
  }
  return null;
}

function trimStack(stack) {
  while (stack.children.length > MAX_CARDS && stack.firstElementChild) {
    stack.removeChild(stack.firstElementChild);
  }
}

/* ---------- public API ---------- */

function render(type, data, id, status) {
  var stack = ensureStack();
  if (!stack) return null;
  var existing = findCard(id);
  if (existing) {
    setStatus(existing, status || "done");
    var body = existing.querySelector(".gb-parity-block-body");
    if (body) {
      while (body.firstChild) body.removeChild(body.firstChild);
      fillBody(type, asObject(data) || {}, body);
    }
    return existing;
  }
  var card = buildCard(type, data, id, status);
  stack.appendChild(card);
  trimStack(stack);
  if (type === "notice" || type === "error" || type === "retry") {
    window.setTimeout(function () {
      if (card.parentNode) card.parentNode.removeChild(card);
    }, DISMISS_MS);
  }
  return card;
}

function show(type, data) {
  var raw = asObject(data) || {};
  var info = null;
  if (raw.activity || raw.consent_request || raw.gb_event || readTypeFrom(raw)) {
    info = extractFrame(raw);
  }
  var resolvedType = normalizeType(type) || (info && info.type);
  if (!resolvedType) return null;
  var payload = info ? info.data : raw;
  var id = firstTruthy(payload.id, payload.activity_id, payload.request_id, info && info.id);
  var status = firstTruthy(payload.status, info && info.status);
  return render(resolvedType, payload, id, status);
}

function onFrame(event) {
  var info;
  try { info = extractFrame(event && event.detail); } catch (e) { info = null; }
  if (!info) return;
  try { render(info.type, info.data, info.id, info.status); } catch (e) { /* never break the socket */ }
}

function mount() {
  if (mounted) return;
  mounted = true;
  ensureStack();
  document.addEventListener("gb-ws-frame", onFrame);
}

P.blocks = { mount: mount, show: show };
})();
