"use strict";
/**
 * settings-engine-names.js — one vocabulary for engine types across the
 * Settings app, ported from the gbgov assistant's 00_engine_names.js.
 *
 * Every surface that names an engine reads from window.GBSettingsEngineNames.
 * Three tiers per type:
 *   short  — one-word tags and status text
 *   group  — section headings, the Usage & limits chips
 *   detail — the sentence that explains the engine in full
 *
 * The detail tier deliberately reuses the existing mdl-local / mdl-free /
 * mdl-own keys so the Model panel and the limits panel say the same thing;
 * only the subscription (acp) engine needs a new detail key.
 */
(function () {
  var W = window;

  function t(key, fallback) {
    return typeof W.gbT === "function" ? W.gbT(key, fallback) : fallback;
  }

  var TYPES = ["local", "free", "own_key", "acp"];

  var KEYS = {
    local: { short: "eng-short-local", group: "eng-group-local", detail: "mdl-local" },
    free: { short: "eng-short-free", group: "eng-group-free", detail: "mdl-free" },
    own_key: { short: "eng-short-own-key", group: "eng-group-own-key", detail: "mdl-own" },
    acp: { short: "eng-short-acp", group: "eng-group-acp", detail: "eng-detail-acp" }
  };

  var FALLBACK = {
    local: { short: "Local", group: "Local models", detail: "On this computer" },
    free: { short: "Free", group: "Free model", detail: "Free hosted model" },
    own_key: { short: "Own key", group: "Own keys", detail: "Your server or provider key" },
    acp: { short: "Subscription", group: "Subscriptions", detail: "Subscription agent" }
  };

  /* Canonical kinds used by /api/llm/limits, mapped to the engine type. */
  var LIMIT_KEYS = { subscription: "acp", api_key: "own_key", local: "local", free: "free" };

  function engineName(type, tier) {
    tier = tier || "short";
    var keys = KEYS[type];
    var fallback = FALLBACK[type];
    if (!keys || !fallback) { return type || ""; }
    return t(keys[tier], fallback[tier]);
  }

  function engineNameByLimitKind(kind, tier) {
    var mapped = LIMIT_KEYS[kind];
    return mapped ? engineName(mapped, tier) : (kind || "");
  }

  W.GBSettingsEngineNames = {
    TYPES: TYPES,
    engineName: engineName,
    engineNameByLimitKind: engineNameByLimitKind
  };
})();
