"use strict";
/**
 * 10i_accounts.js - Provider accounts bridge.
 *
 * Reads /api/llm/accounts through the standard authenticated apiFetch helper.
 * The endpoint is not shipped by every backend yet, so a 404/500 leaves an
 * empty account list and the rest of the assistant keeps working unchanged.
 */
(function () {
    var A = window.GBAssistant;

    A.accounts = [];

    /* /api/llm/accounts returns {id, name, enabled:[string]}; the chat model
       menu expects {id, name, models:[{model, value}]}. Normalize here so a
       backend without the grouped /api/chat/models field still renders. */
    function normalize(accounts) {
        return (accounts || []).map(function (a) {
            var id = a.id || a.provider || "";
            var raw = a.models || a.enabled || a.model_ids || [];
            return {
                id: id,
                name: a.name || id,
                logo: a.logo || id,
                models: raw.map(function (m) {
                    var name = typeof m === "string" ? m : (m.model || m.id || m.name || "");
                    var value = m && typeof m === "object" && m.value ? m.value : "acct:" + id + "/" + name;
                    return { model: name, value: value };
                }).filter(function (m) { return m.model; })
            };
        }).filter(function (a) { return a.id && a.models.length; });
    }

    A.refreshAccounts = async function () {
        try {
            var r = await A.apiFetch("/api/llm/accounts");
            if (!r.ok) {
                console.debug("[assistant] /api/llm/accounts unavailable:", r.status);
                return A.accounts;
            }
            var data = await r.json();
            var grouped = normalize(data.accounts);
            if (grouped.length) { A.accounts = grouped; }
            return A.accounts;
        } catch (e) {
            console.debug("[assistant] accounts fetch failed:", e);
            return A.accounts;
        }
    };

    /** Resolve a saved model preference to the connected account that owns it. */
    A.accountForPreference = function (pref) {
        if (!pref) { return null; }
        var value = String(pref);
        var hit = A.accounts.filter(function (acct) {
            return acct && (acct.id === value || (acct.models || []).some(function (m) {
                return m.value === value || m.model === value;
            }));
        })[0];
        return hit || null;
    };

    /* A provider-backed model may arrive as "acct:<provider>/<model>".
       main's 05b helper predates that form, so strip the prefix here without
       replacing the function other modules captured at load time. */
    if (typeof A.shortModelName === "function" && !A._acctPrefixHandled) {
        var baseShort = A.shortModelName;
        A.shortModelName = function (model) {
            if (!model) { return ""; }
            return baseShort(String(model).replace(/^acct:[^/]+\//, ""));
        };
        A._acctPrefixHandled = true;
    }

    /* Best effort on boot: settings can read A.accounts as soon as it opens. */
    A.refreshAccounts();
})();
