"use strict";
/**
 * 07_protocol.js - Strip leaked protocol text so nothing raw ever shows.
 *
 * When agent mode is off (or when the model hallucinates a tool call for a
 * surface that this build does not compile, such as terminal, research or
 * office), the raw JSON or XML tag leaks into the answer text. This module
 * catches every known marker and replaces it with a quiet notice card --
 * never with the raw text.
 *
 * It also strips the consent-request base64 marker (handled separately by
 * 08_consent.js) so no path prints it.
 */

(function () {
    var A = window.GBAssistant;

    /* ---- Tool-call JSON in text ---- */
    var TOOL_NAMES = "propose_plan|show_app|ask_user|show_surface|classify_media|vm_exec|execute_code|run_terminal|web_search|web_fetch|research";
    var TOOL_RE = new RegExp('"tool_call"\\s*:\\s*true|"__tool_call__"\\s*:|"name"\\s*:\\s*"(' + TOOL_NAMES + ')"');

    function stripToolCallJson(text) {
        var raw = String(text || "");
        for (var guard = 0; guard < 5; guard++) {
            var match = TOOL_RE.exec(raw);
            if (!match) { break; }
            var start = raw.lastIndexOf("{", match.index);
            if (start === -1) { break; }
            var depth = 0, inStr = false, esc = false, end = -1;
            for (var i = start; i < raw.length; i++) {
                var c = raw[i];
                if (inStr) {
                    if (esc) { esc = false; }
                    else if (c === "\\") { esc = true; }
                    else if (c === '"') { inStr = false; }
                    continue;
                }
                if (c === '"') { inStr = true; }
                else if (c === "{") { depth++; }
                else if (c === "}") { depth--; if (depth === 0) { end = i + 1; break; } }
            }
            if (end === -1) { raw = raw.slice(0, start).trim(); break; }
            raw = (raw.slice(0, start) + raw.slice(end)).replace(/\n{3,}/g, "\n\n");
        }
        return raw.trim();
    }

    /* ---- UI-plan JSON ---- */
    function stripClientPlan(text) {
        var raw = String(text || "");
        var marker = raw.search(/"_{0,2}ui_plan_{0,2}"\s*:/);
        if (marker === -1) { return raw; }
        var start = raw.lastIndexOf("{", marker);
        if (start === -1) { return raw; }
        var depth = 0, inStr = false, esc = false, end = -1;
        for (var i = start; i < raw.length; i++) {
            var c = raw[i];
            if (inStr) {
                if (esc) { esc = false; }
                else if (c === "\\") { esc = true; }
                else if (c === '"') { inStr = false; }
                continue;
            }
            if (c === '"') { inStr = true; }
            else if (c === "{") { depth++; }
            else if (c === "}") { depth--; if (depth === 0) { end = i + 1; break; } }
        }
        if (end === -1) { return raw.slice(0, start).trim(); }
        return (raw.slice(0, start) + raw.slice(end)).replace(/\n{3,}/g, "\n\n").trim();
    }

    /* ---- XML-like tags the server did not consume ---- */
    /* <vm_exec">, <execute_code>, <terminal>, etc. */
    var TAG_RE = /<\/?(vm_exec|execute_code|terminal|run_terminal|web_search|research|tool_call|function_call|mcp_call)\b[^>]*>/gi;
    /* Also catch an unclosed tag that streams in */
    var PARTIAL_TAG_RE = /<(vm_exec|execute_code|terminal|run_terminal|web_search|research|tool_call|function_call|mcp_call)\b[^>]*$/i;

    function stripLeakedTags(text) {
        var raw = String(text || "");
        raw = raw.replace(TAG_RE, "");
        raw = raw.replace(PARTIAL_TAG_RE, "");
        return raw;
    }

    /* ---- __api_call__ marker ---- */
    var API_CALL_RE = /"?__api_call__"?\s*:\s*\{[^}]*\}/g;

    /* ---- Consent marker (handled by 08_consent.js, strip here) ---- */
    var CONSENT_RE = /GB-CONSENT-REQUEST:[A-Za-z0-9+/=_-]+/g;

    /* ---- <think> tags from reasoning models ---- */
    function stripThinkTags(text) {
        return String(text || "").replace(/<think>[\s\S]*?(?:<\/think>|$)/gi, "");
    }

    /* ---- Public: clean one chunk or final text ---- */
    A.cleanProtocol = function (text) {
        if (!text) { return ""; }
        var cleaned = text;
        cleaned = stripThinkTags(cleaned);
        cleaned = stripToolCallJson(cleaned);
        cleaned = stripClientPlan(cleaned);
        cleaned = stripLeakedTags(cleaned);
        cleaned = cleaned.replace(API_CALL_RE, "");
        cleaned = cleaned.replace(CONSENT_RE, "");
        /* Remove "Copiar" that some models inject as a copy-button hallucination */
        cleaned = cleaned.replace(/^\s*Copiar\s*$/gm, "");
        return cleaned.replace(/\n{3,}/g, "\n\n").trim();
    };

    /**
     * Detect whether the raw text contained leaked protocol markup.
     * Returns an object { leaked: true, kind: "tool"|"tag"|"api"|... }
     * or null if the text is clean.
     */
    A.detectLeakedProtocol = function (raw) {
        if (!raw) { return null; }
        if (TOOL_RE.test(raw)) { TOOL_RE.lastIndex = 0; return { leaked: true, kind: "tool" }; }
        if (TAG_RE.test(raw)) { TAG_RE.lastIndex = 0; return { leaked: true, kind: "tag" }; }
        if (API_CALL_RE.test(raw)) { API_CALL_RE.lastIndex = 0; return { leaked: true, kind: "api" }; }
        if (PARTIAL_TAG_RE.test(raw)) { PARTIAL_TAG_RE.lastIndex = 0; return { leaked: true, kind: "tag" }; }
        return null;
    };
})();
