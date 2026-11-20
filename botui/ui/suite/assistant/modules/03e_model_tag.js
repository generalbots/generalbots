"use strict";
/**
 * 03e_model_tag.js - Names the model that wrote each answer.
 *
 * A small tag sits next to the "Concluído"/"Done" chip of the thinking card,
 * or at the start of the answer toolbar when the answer had no thinking card.
 * The server does not store the model per message yet, so the shell keeps
 * the name it sent with each turn (by a hash of the answer text) in this
 * browser profile. Older answers without a record show no tag.
 *
 * main keeps its renderer in 03_render_b.js at the strict 450-line CI
 * ceiling, so the two hooks below wrap the exported functions instead of
 * editing that file. This module is loaded after 03_render_b.js.
 */
(function () {
    var A = window.GBAssistant;
    var KEY = "gb-answer-models";
    var MAX = 400;

    function tr(pt, en) { return A.locale && A.locale.startsWith("pt") ? pt : en; }

    function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }
    function hash(text) {
        var s = String(text || ""), h = 5381;
        for (var i = 0; i < s.length; i++) { h = ((h << 5) + h + s.charCodeAt(i)) | 0; }
        return (h >>> 0).toString(36) + ":" + s.length;
    }
    function load() {
        var map = typeof gbStoreGet === "function" ? gbStoreGet(KEY, {}) : {};
        return map && typeof map === "object" && !Array.isArray(map) ? map : {};
    }
    function save(map) {
        var keys = Object.keys(map);
        if (keys.length > MAX) {
            keys.sort(function (a, b) { return (map[a].t || 0) - (map[b].t || 0); });
            keys.slice(0, keys.length - MAX).forEach(function (k) { delete map[k]; });
        }
        try { localStorage.setItem(KEY, JSON.stringify(map)); } catch (e) { /* storage blocked */ }
    }

    /** Model the shell sends with a turn right now. */
    A.turnModelNow = function () { return A.selectedModel || A.defaultModel || ""; };

    A.rememberAnswerModel = function (content, model) {
        if (!content || !model) { return; }
        var map = load();
        map[hash(content)] = { m: model, t: Date.now() };
        save(map);
    };

    A.answerModel = function (content) {
        var rec = load()[hash(content)];
        return rec ? rec.m : "";
    };

    function label(model) {
        var name = typeof A.shortModelName === "function" ? A.shortModelName(model) : model;
        return String(name || "").replace(/\.gguf$/i, "");
    }

    /** Puts the tag on an answer element; replaces an older tag. */
    A.attachModelTag = function (el, model) {
        if (!el || !model) { return; }
        var old = el.querySelector(".gb-model-line") || el.querySelector(".gb-model-tag");
        if (old) { old.remove(); }
        var tag = document.createElement("span");
        tag.className = "gb-model-tag";
        tag.title = tr("Resposta gerada por ", "Answer written by ") + model;
        tag.innerHTML = '<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">' +
            '<rect x="5" y="5" width="14" height="14" rx="2"/><path d="M9 1v4M15 1v4M9 19v4M15 19v4M1 9h4M1 15h4M19 9h4M19 15h4"/></svg>' +
            '<span>' + esc(label(model)) + '</span>';
        var thinking = el.querySelectorAll(".gb-block-stack .gb-block--done .gb-block-header");
        var hdr = thinking.length ? thinking[thinking.length - 1] : null;
        var chip = hdr ? hdr.querySelector(".gb-block-chip") : null;
        if (chip) { hdr.insertBefore(tag, chip); return; }
        /* No thinking card: a line of its own above the toolbar, which only
           shows on hover. */
        var line = document.createElement("div");
        line.className = "gb-model-line";
        line.appendChild(tag);
        var tb = el.querySelector(".gb-msg-toolbar");
        if (tb) { el.insertBefore(line, tb); } else { el.appendChild(line); }
    };

    /** A live answer just finished: record and show the model it came from. */
    A.tagAnswer = function (el, content, model) {
        var used = model || A.turnModelNow();
        A.rememberAnswerModel(content, used);
        A.attachModelTag(el, used);
    };

    /** Last direct assistant child of the live message list. */
    function lastAssistant() {
        var msgs = document.getElementById("gbMessages");
        if (!msgs) { return null; }
        var kids = msgs.children;
        for (var i = kids.length - 1; i >= 0; i--) {
            if (kids[i].classList && kids[i].classList.contains("gb-msg-assistant")) { return kids[i]; }
        }
        return null;
    }

    /* ---- Wrappers: install the hooks without growing 03_render_b.js ---- */

    var origAssistant = A.renderAssistantMessage;
    if (typeof origAssistant === "function") {
        A.renderAssistantMessage = function (text, reasoning, opts) {
            var result = origAssistant.apply(this, arguments);
            var o = opts || {};
            var raw = text || "";
            if (!o.isError && raw && typeof A.tagAnswer === "function") {
                var el = lastAssistant();
                if (el) { A.tagAnswer(el, raw, o.model); }
            }
            return result;
        };
    }

    var origHistory = A.renderHistory;
    if (typeof origHistory === "function") {
        A.renderHistory = function (messages) {
            var result = origHistory.apply(this, arguments);
            var msgs = document.getElementById("gbMessages");
            if (!msgs || typeof A.attachModelTag !== "function") { return result; }
            var list = messages || [];
            var assistants = [];
            for (var i = 0; i < list.length; i++) {
                if (list[i] && list[i].role === "assistant") { assistants.push(list[i]); }
            }
            /* Pair history records with rendered elements in DOM order. */
            var els = [];
            var kids = msgs.children;
            for (var k = 0; k < kids.length; k++) {
                if (kids[k].classList && kids[k].classList.contains("gb-msg-assistant")) { els.push(kids[k]); }
            }
            for (var j = 0; j < assistants.length && j < els.length; j++) {
                A.attachModelTag(els[j], assistants[j].model || A.answerModel(assistants[j].content));
            }
            return result;
        };
    }
})();
