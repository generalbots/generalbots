/* Chat messages module: Strip leaked plans/tool-call JSON and detect embedded HTML apps. */
// A model sometimes returns a complete HTML app instead of prose. It must
// never be injected into the chat DOM (styles leak, scripts execute, and the
// raw CSS/JS shows as garbage). Render it in an opaque sandboxed iframe.
// Remove a UI-plan JSON object the model printed instead of (or in addition
// to) the server consuming it. Handles both `__ui_plan__` and the common
// `ui_plan` misspelling, plus the plan tail if braces are split.
function stripClientPlan(text) {
  var raw = String(text || "");
  var marker = raw.search(/"_{0,2}ui_plan_{0,2}"\s*:/);
  if (marker === -1) return raw;
  var start = raw.lastIndexOf("{", marker);
  if (start === -1) return raw;
  var depth = 0, inString = false, escaped = false, end = -1;
  for (var i = start; i < raw.length; i++) {
    var c = raw[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') inString = true;
    else if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) { end = i + 1; break; }
    }
  }
  if (end === -1) return raw.slice(0, start).trim();
  return (raw.slice(0, start) + raw.slice(end)).replace(/\n{3,}/g, "\n\n").trim();
}

function looksLikeHtmlDocument(text) {
  var raw = String(text || "").trim();
  if (raw.length < 600) return false;
  if (/^<!doctype\s+html/i.test(raw) || /^<html[\s>]/i.test(raw) ||
      /^<head[\s>]/i.test(raw) || /^<body[\s>]/i.test(raw)) return true;
  if (/^<meta\s+charset/i.test(raw) && /<style[\s>]/i.test(raw)) return true;
  return /<style[\s>][\s\S]*<\/style>/i.test(raw) &&
    /<(div|section|canvas|svg)[\s>]/i.test(raw) && raw.length > 1500;
}

function inlineArtifactTitle(html) {
  var title = /<title[^>]*>([\s\S]*?)<\/title>/i.exec(html);
  if (title && title[1].trim()) return title[1].replace(/<[^>]*>/g, "").trim().slice(0, 80);
  var h1 = /<h1[^>]*>([\s\S]*?)<\/h1>/i.exec(html);
  if (h1 && h1[1].trim()) return h1[1].replace(/<[^>]*>/g, "").trim().slice(0, 80);
  return "Interactive app";
}

// Some models emit a `show_app` tool call as *text* instead of invoking it:
// a JSON object whose arguments carry the HTML app. Recover it and render the
// sandboxed app instead of printing the JSON (the "garbage on screen" case).
// The model sometimes prints a tool call as text instead of (or in addition
// to) the server executing it. `{"tool_call": true, "name": "propose_plan",
// "arguments": {...}}` must never reach the reader; the matching surface is
// already rendered from the CUSTOM frame.
function stripToolCallJson(text) {
  var raw = String(text || "");
  var names = "propose_plan|show_app|ask_user|show_surface|classify_media|vm_exec";
  var pattern = new RegExp('"tool_call"\\s*:\\s*true|"name"\\s*:\\s*"(' + names + ')"');
  for (var guard = 0; guard < 5; guard++) {
    var match = pattern.exec(raw);
    if (!match) break;
    var start = raw.lastIndexOf("{", match.index);
    if (start === -1) break;
    var depth = 0, inString = false, escaped = false, end = -1;
    for (var i = start; i < raw.length; i++) {
      var c = raw[i];
      if (inString) {
        if (escaped) escaped = false;
        else if (c === "\\") escaped = true;
        else if (c === '"') inString = false;
        continue;
      }
      if (c === '"') inString = true;
      else if (c === "{") depth++;
      else if (c === "}") {
        depth--;
        if (depth === 0) { end = i + 1; break; }
      }
    }
    if (end === -1) {
      raw = raw.slice(0, start).trim();
      break;
    }
    raw = (raw.slice(0, start) + raw.slice(end)).replace(/\n{3,}/g, "\n\n");
  }
  return raw.trim();
}

function extractEmbeddedApp(text) {
  var raw = String(text || "");
  var marker = raw.indexOf('"__tool_call__"');
  if (marker === -1) marker = raw.indexOf('"name":"show_app"');
  if (marker === -1) marker = raw.indexOf('"name": "show_app"');
  if (marker === -1) return null;
  var start = raw.lastIndexOf("{", marker);
  if (start === -1) return null;
  var depth = 0, inString = false, escaped = false, end = -1;
  for (var i = start; i < raw.length; i++) {
    var c = raw[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') inString = true;
    else if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) { end = i + 1; break; }
    }
  }
  if (end === -1) return null;
  var payload;
  try { payload = JSON.parse(raw.slice(start, end)); } catch (e) { return null; }
  var args = payload.arguments || payload.args || (payload.function && payload.function.arguments);
  if (typeof args === "string") {
    try { args = JSON.parse(args); } catch (e) { return null; }
  }
  if (!args || typeof args !== "object") return null;
  var html = args.html || args.content || args.body || args.app_html;
  if (!html || !looksLikeHtmlDocument(html)) return null;
  return {
    title: args.title || inlineArtifactTitle(html),
    html: html
  };
}
