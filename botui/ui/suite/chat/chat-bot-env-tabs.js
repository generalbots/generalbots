/**
 * Vibe bot two-environment chat tabs (#1504).
 *
 * When the Chat window is deep-linked from a Vibe bot project the params
 * carry: bot (initial identity), botTest ({bot}-test), botProd ({bot}),
 * botEnv ("test"|"production") and botLabel (display name). This module
 * renders a slim TEST/PROD tab strip above the conversation and switches the
 * live session via ChatSwitchBot — one window, two environments:
 *
 *   Run (vibe)   → botEnv=test        → TEST tab active
 *   Deploy       → botEnv=production  → PROD tab active
 *
 * The strip only appears for bot deep-links with both identities present;
 * every other chat use keeps the plain window (no tabs).
 */
(function () {
    "use strict";

    function params() {
        return window.__gbAppParams__ || {};
    }

    function isTwoEnv() {
        var p = params();
        return !!(p.botTest && p.botProd);
    }

    function labels() {
        var p = params();
        var label = p.botLabel || p.botProd || "bot";
        return {
            test: label + " (TEST)",
            prod: label + " (PROD)",
        };
    }

    function activeEnv() {
        return params().botEnv === "production" ? "production" : "test";
    }

    function botFor(env) {
        var p = params();
        return env === "production" ? (p.botProd || p.bot) : (p.botTest || p.bot);
    }

    function buildStrip() {
        if (document.getElementById("vibeBotEnvTabs")) return;
        var strip = document.createElement("div");
        strip.id = "vibeBotEnvTabs";
        strip.className = "vibe-bot-env-tabs";

        var names = labels();
        [["test", names.test], ["production", names.prod]].forEach(function (pair) {
            var env = pair[0];
            var btn = document.createElement("button");
            btn.type = "button";
            btn.className = "vibe-bot-env-tab" + (env === activeEnv() ? " active" : "");
            btn.dataset.env = env;
            btn.textContent = pair[1];
            btn.addEventListener("click", function () { switchEnv(env); });
            strip.appendChild(btn);
        });

        var host = document.getElementById("messages") || document.body;
        var anchor = host.parentElement || host;
        anchor.insertBefore(strip, host);
    }

    function switchEnv(env) {
        var tabs = document.querySelectorAll(".vibe-bot-env-tab");
        tabs.forEach(function (t) {
            t.classList.toggle("active", t.dataset.env === env);
        });
        if (env === activeEnv()) return;
        window.__gbAppParams__.botEnv = env;
        window.__gbAppParams__.bot = botFor(env);
        if (typeof window.ChatSwitchBot === "function") {
            window.ChatSwitchBot(botFor(env));
        }
    }

    function init() {
        if (!isTwoEnv()) return;
        buildStrip();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
