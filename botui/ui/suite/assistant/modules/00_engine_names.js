"use strict";
/**
 * 00_engine_names.js - Single source of truth for engine type names.
 *
 * Every surface that names an engine type reads from A.ENGINE_NAMES.
 * The map has three tiers per type:
 *   short  — engine panel, status bar, one-word tags
 *   group  — model menu group headings, Limites, usage drawer
 *   detail — Settings cards, guide, detail titles
 *
 * Sentence case everywhere ("Modelos locais", not "Modelos Locais").
 */
(function () {
    window.GBAssistant = window.GBAssistant || {};
    var A = window.GBAssistant;

    var NAMES = {
        local: {
            short:  ["Local", "Local"],
            group:  ["Modelos locais", "Local models"],
            detail: ["Neste computador", "On this computer"]
        },
        free: {
            short:  ["Gratuito", "Free"],
            group:  ["Modelo gratuito", "Free model"],
            detail: ["Modelo gratuito", "Free model"]
        },
        own_key: {
            short:  ["Chave própria", "Own key"],
            group:  ["Chaves próprias", "Own keys"],
            detail: ["Servidor ou chave própria", "Own server or key"]
        },
        acp: {
            short:  ["Assinatura", "Subscription"],
            group:  ["Assinaturas", "Subscriptions"],
            detail: ["Agente por assinatura", "Subscription agent"]
        }
    };

    /* Canonical keys used by /api/llm/limits and the usage drawer. */
    var LIMIT_KEYS = {
        subscription: "acp",
        api_key: "own_key",
        local: "local",
        free: "free"
    };

    function pt() { return !!(A.locale && A.locale.startsWith("pt")); }

    /**
     * A.engineName(type, tier)
     *   type: "local" | "free" | "own_key" | "acp"
     *   tier: "short" | "group" | "detail"
     * Returns the localised string; falls back to the type key.
     */
    A.engineName = function (type, tier) {
        var entry = NAMES[type];
        if (!entry) { return type || ""; }
        var pair = entry[tier || "short"];
        if (!pair) { return type; }
        return pt() ? pair[0] : pair[1];
    };

    /**
     * A.engineNameByLimitKind(kind, tier)
     *   kind: the /api/llm/limits `kind` value ("subscription", "api_key", "local", "free")
     */
    A.engineNameByLimitKind = function (kind, tier) {
        var mapped = LIMIT_KEYS[kind];
        return mapped ? A.engineName(mapped, tier) : (kind || "");
    };

    A.ENGINE_NAMES = NAMES;
})();
