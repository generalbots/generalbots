    // Vibe agent LLM slots: three combos (Reasoning / Agentic / Fast), each
    // selecting a connected LLM provider for this branch.
    (function () {
        "use strict";
        var AGENTS = [
            { id: "reasoning", label: "Reasoning", hint: "Deep planning, analysis and complex multi-step reasoning" },
            { id: "agentic", label: "Agentic", hint: "Autonomous tool-using agent loops (default)" },
            { id: "fast", label: "Fast", hint: "Low-latency quick turns and scaffolding" },
        ];
        var root = document.getElementById("vibe-agent-slots");
        if (!root) return;

        function apiFetch(url, opts) {
            var headers = Object.assign(
                { "Content-Type": "application/json" },
                (opts && opts.headers) || {}
            );
            var token = window.getGBAccessToken
                ? window.getGBAccessToken()
                : (localStorage.getItem("gb-access-token") ||
                    sessionStorage.getItem("gb-access-token") ||
                    localStorage.getItem("management_token") || "");
            if (token) headers["Authorization"] = "Bearer " + token;
            return fetch(url, Object.assign({ headers: headers }, opts || {}));
        }

        function escapeHtml(text) {
            return String(text == null ? "" : text)
                .replace(/&/g, "&amp;").replace(/</g, "&lt;")
                .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
        }

        function render() {
            var botId = null;
            apiFetch("/api/apps/integrations/context")
                .then(function (r) { return r.json(); })
                .then(function (ctx) {
                    botId = ctx.bot_id || "";
                    if (!botId) {
                        root.innerHTML = '<div class="empty-state"><p>No active bot — connect a bot first.</p></div>';
                        return null;
                    }
                    return apiFetch("/api/bots/" + encodeURIComponent(botId) + "/integration-connections")
                        .then(function (r) { return r.json(); })
                        .catch(function () { return { items: [] }; });
                })
                .then(function (j) {
                    if (!j) return;
                    var conns = (j.items || []).filter(function (c) {
                        return c.status !== "revoked" && /^(kilo-|openai|claude|azure|glm|bedrock|vertex|kiro|deepinfra|cerebras|fireworks|together|openrouter|requesty|opencode|openai-compatible)/.test(c.provider_slug || "");
                    });
                    var options = '<option value="">— none —</option>' + conns.map(function (c) {
                        return '<option value="' + escapeHtml(c.id) + '">' + escapeHtml(c.provider_slug) + "</option>";
                    }).join("");

                    var html = AGENTS.map(function (a) {
                        return (
                            '<div style="padding:14px 0;border-bottom:1px solid var(--border,#262b36)">' +
                            '<label style="display:block;font-weight:600;margin-bottom:2px">' + a.label + "</label>" +
                            '<div style="font-size:12px;color:var(--text-muted,#8b93a7);margin-bottom:8px">' + a.hint + "</div>" +
                            '<div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">' +
                            '<select class="vibe-agent-select" data-agent="' + a.id + '" data-bot="' + botId + '" style="padding:8px 10px;border:1px solid var(--border,#334155);border-radius:6px;background:var(--bg,#14142a);color:var(--text,#eee);min-width:220px">' + options + "</select>" +
                            '<button type="button" class="btn-primary btn-sm vibe-agent-save" data-agent="' + a.id + '" data-bot="' + botId + '">Assign</button>' +
                            '<span class="vibe-agent-status" data-status="' + a.id + '" role="status" aria-live="polite" aria-atomic="true" style="font-size:12px;color:var(--text-muted,#8b93a7)"></span>' +
                            "</div></div>"
                        );
                    }).join("");
                    root.innerHTML = html;

                    // Load current assignments from per-bot config.
                    var cfgUrl = "/api/bot/config?bot_name=" + encodeURIComponent(window.__INITIAL_BOT_NAME__ || "default");
                    apiFetch(cfgUrl)
                        .then(function (r) { return r.json().catch(function () { return {}; }); })
                        .then(function (cfg) {
                            AGENTS.forEach(function (a) {
                                var provider = cfg["vibe-llm-" + a.id + "-provider"] || "";
                                var model = cfg["vibe-llm-" + a.id + "-model"] || "";
                                var sel = root.querySelector('[data-agent="' + a.id + '"]');
                                if (!sel) return;
                                // Match by connection whose provider_slug equals the stored provider.
                                var match = conns.find(function (c) { return c.provider_slug === provider; });
                                if (match) sel.value = match.id;
                                var status = root.querySelector('[data-status="' + a.id + '"]');
                                if (status) status.textContent = provider ? (provider + (model ? " · " + model : "")) : "not assigned";
                            });
                            bindEvents(botId, conns);
                        })
                        .catch(function () { bindEvents(botId, conns); });
                })
                .catch(function (e) {
                    root.setAttribute("role", "alert");
                    root.setAttribute("aria-live", "assertive");
                    root.setAttribute("aria-atomic", "true");
                    root.innerHTML = '<div class="empty-state"><p>Vibe agents unavailable: ' + escapeHtml((e && e.message) || e) + "</p></div>";
                });
        }

        function bindEvents(botId, conns) {
            root.querySelectorAll(".vibe-agent-save").forEach(function (btn) {
                btn.addEventListener("click", function () {
                    var agent = btn.getAttribute("data-agent");
                    var sel = root.querySelector('[data-agent="' + agent + '"]');
                    var connId = sel ? sel.value : "";
                    var status = root.querySelector('[data-status="' + agent + '"]');
                    if (!connId) {
                        if (status) status.textContent = "select a provider first";
                        return;
                    }
                    btn.disabled = true;
                    btn.textContent = "Saving…";
                    apiFetch("/api/bots/" + encodeURIComponent(botId) + "/integration-connections/" + encodeURIComponent(connId) + "/use-for-vibe", {
                        method: "POST",
                        body: JSON.stringify({ agent: agent }),
                    })
                        .then(function (r) { return r.json(); })
                        .then(function (j) {
                            if (!j.success) throw new Error(j.error || "assign failed");
                            if (status) status.textContent = j.provider + " · " + j.model + " ✔";
                            var msg = "Vibe " + agent + " agent → " + j.provider + " (" + j.model + ")";
                            if (window.GBModal && typeof window.GBModal.alert === "function") {
                              window.GBModal.alert(msg, "Vibe Agent");
                            } else if (window.GBToasts && typeof window.GBToasts.show === "function") {
                              window.GBToasts.show("Vibe Agent", msg, "success");
                            }
                            render();
                        })
                        .catch(function (e) {
                            if (status) status.textContent = "error: " + ((e && e.message) || e);
                            render();
                        });
                });
            });
        }

        render();
    })();
