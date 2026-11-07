// #1551/#1642/#1836 GB6-16 — memory across conversations. The server copy
// (chat_preferences via /api/user/memory) is the source of truth the prompt
// reads; localStorage is only a cache for offline rendering.
// Reformulated: search, edit, sync status, per-bot scope; no bare key/value inputs.
(function () {
    function storageKey() {
        return "gb_user_memory_cache";
    }
    function load() {
        try { return JSON.parse(localStorage.getItem(storageKey()) || "[]"); }
        catch (e) { return []; }
    }
    function save(mem) {
        try { localStorage.setItem(storageKey(), JSON.stringify(mem)); } catch (e) {}
    }
    function token() {
        return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    }
    function headers(extra) {
        var h = extra || {};
        var t = token();
        if (t) h.Authorization = "Bearer " + t;
        return h;
    }
    async function serverList() {
        if (!token()) return null;
        try {
            var response = await fetch("/api/user/memory", { headers: headers() });
            if (!response.ok) return null;
            var payload = await response.json();
            return Array.isArray(payload.memories) ? payload.memories : [];
        } catch (e) {
            return null;
        }
    }
    async function serverPut(key, value, botId) {
        if (!token()) return;
        try {
            var response = await fetch("/api/user/memory", {
                method: "PUT",
                headers: headers({ "Content-Type": "application/json" }),
                body: JSON.stringify({ key: key, value: value, bot_id: botId || null }),
            });
            if (!response.ok) throw new Error("save failed");
        } catch (e) { /* offline: local cache still holds it */ throw e; }
    }
    async function serverDelete(key) {
        if (!token()) return;
        try {
            await fetch("/api/user/memory/" + encodeURIComponent(key), {
                method: "DELETE",
                headers: headers(),
            });
        } catch (e) { /* offline */ }
    }
    function esc(value) {
        return String(value == null ? "" : value)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }
    function currentBotName() {
        return window.__INITIAL_BOT_NAME__ || "default";
    }
    function setSyncStatus(status) {
        var el = document.getElementById("gbMemorySyncStatus");
        if (el) el.textContent = status;
    }
    var entries = [];
    var query = "";
    var scopeFilter = "all";
    var editing = null;
    var busy = false;

    function filteredEntries() {
        return entries.filter(function (e) {
            if (scopeFilter === "global" && e.bot_id) return false;
            if (scopeFilter === "bot" && !e.bot_id) return false;
            if (query) {
                var q = query.toLowerCase();
                if (String(e.key || "").toLowerCase().indexOf(q) === -1 &&
                    String(e.value || "").toLowerCase().indexOf(q) === -1) return false;
            }
            return true;
        }).sort(function (a, b) {
            return new Date(b.updated_at || 0) - new Date(a.updated_at || 0);
        });
    }
    function botLabel(botId) {
        return botId ? ("bot " + String(botId).slice(0, 8)) : "global";
    }
    function renderList() {
        var list = document.getElementById("gbMemoryList");
        if (!list) return;
        var botName = document.getElementById("gbMemoryBotName");
        if (botName) botName.textContent = currentBotName();
        var items = filteredEntries();
        if (!items.length) {
            list.innerHTML = '<p class="setting-desc" style="padding:8px 0;">Nothing remembered yet. The assistant adds facts as you chat; search or filter above.</p>';
            return;
        }
        list.innerHTML = items.map(function (e) {
            var isEditing = editing && editing.key === e.key &&
                String(editing.botId || "") === String(e.bot_id || "");
            if (isEditing) {
                return '<div class="setting-row" data-memory-key="' + esc(e.key) + '" data-memory-bot="' + esc(e.bot_id || "") + '">' +
                    '<div class="setting-info" style="flex:1;">' +
                        '<span class="setting-title">' + esc(e.key) + '</span>' +
                        '<textarea class="input-sm gb-memory-edit-value" aria-label="Edit memory value" style="width:100%;margin-top:6px;min-height:60px;">' + esc(e.value || "") + '</textarea>' +
                        '<div style="margin-top:6px;display:flex;gap:8px;align-items:center;flex-wrap:wrap;">' +
                            '<select class="input-sm gb-memory-edit-scope" aria-label="Memory scope">' +
                                '<option value="global"' + (e.bot_id ? "" : " selected") + '>Global (all bots)</option>' +
                                '<option value="bot"' + (e.bot_id ? " selected" : "") + '>Bot-specific</option>' +
                            '</select>' +
                            '<span class="setting-desc">[' + esc(botLabel(e.bot_id)) + ']</span>' +
                        '</div>' +
                    '</div>' +
                    '<div style="display:flex;gap:8px;align-items:flex-start;">' +
                        '<button type="button" class="btn btn-primary btn-sm gb-memory-save">Save</button>' +
                        '<button type="button" class="btn btn-secondary btn-sm gb-memory-cancel">Cancel</button>' +
                    '</div>' +
                '</div>';
            }
            return '<div class="setting-row" data-memory-key="' + esc(e.key) + '" data-memory-bot="' + esc(e.bot_id || "") + '">' +
                '<div class="setting-info">' +
                    '<span class="setting-title">' + esc(e.key) + '</span>' +
                    '<span class="setting-desc">' + esc(e.value || "") + '</span>' +
                    '<span class="setting-desc" style="margin-left:8px;opacity:.7;">[' + esc(botLabel(e.bot_id)) + ']</span>' +
                '</div>' +
                '<div style="display:flex;gap:8px;">' +
                    '<button type="button" class="btn btn-secondary btn-sm gb-memory-edit">Edit</button>' +
                    '<button type="button" class="btn btn-secondary btn-sm gb-memory-forget">Forget</button>' +
                '</div>' +
            '</div>';
        }).join("");
    }
    async function refresh() {
        setSyncStatus("Syncing…");
        var cached = load();
        var remote = await serverList();
        if (remote) {
            entries = remote.map(function (r) {
                return {
                    key: r.key,
                    value: String(r.value || "").substring(0, 2000),
                    bot_id: r.bot_id || null,
                    updated_at: r.updated_at || new Date().toISOString(),
                };
            });
            save(entries);
            setSyncStatus("Synced");
        } else {
            entries = Array.isArray(cached) ? cached : [];
            if (token()) setSyncStatus("Offline");
            else setSyncStatus("Not signed in");
        }
        var search = document.getElementById("gbMemorySearch");
        if (search) query = search.value.trim();
        var scope = document.getElementById("gbMemoryScope");
        if (scope) scopeFilter = scope.value;
        renderList();
    }
    document.addEventListener("click", async function (e) {
        if (busy) return;
        var editBtn = e.target.closest && e.target.closest(".gb-memory-edit");
        if (editBtn) {
            var row = editBtn.closest("[data-memory-key]");
            editing = {
                key: row.getAttribute("data-memory-key"),
                botId: row.getAttribute("data-memory-bot") || null,
            };
            renderList();
            return;
        }
        var cancelBtn = e.target.closest && e.target.closest(".gb-memory-cancel");
        if (cancelBtn) {
            editing = null;
            renderList();
            return;
        }
        var saveBtn = e.target.closest && e.target.closest(".gb-memory-save");
        if (saveBtn) {
            var saveRow = saveBtn.closest("[data-memory-key]");
            var key = saveRow.getAttribute("data-memory-key");
            var oldBot = saveRow.getAttribute("data-memory-bot") || "";
            var valueEl = saveRow.querySelector(".gb-memory-edit-value");
            var scopeEl = saveRow.querySelector(".gb-memory-edit-scope");
            var value = valueEl ? valueEl.value.trim() : "";
            if (!value) return;
            var newBotId = null;
            if (scopeEl && scopeEl.value === "bot") newBotId = oldBot || null;
            busy = true;
            setSyncStatus("Syncing…");
            try {
                await serverPut(key, value.substring(0, 2000), newBotId);
                var idx = entries.findIndex(function (en) {
                    return en.key === key && String(en.bot_id || "") === String(oldBot || "");
                });
                if (idx !== -1) {
                    entries[idx].value = value.substring(0, 2000);
                    entries[idx].bot_id = newBotId;
                    entries[idx].updated_at = new Date().toISOString();
                } else {
                    entries.push({ key: key, value: value.substring(0, 2000), bot_id: newBotId, updated_at: new Date().toISOString() });
                }
                save(entries);
                editing = null;
                setSyncStatus("Synced");
                renderList();
            } catch (err) {
                setSyncStatus("Offline");
            } finally {
                busy = false;
            }
            return;
        }
        var forgetBtn = e.target.closest && e.target.closest(".gb-memory-forget");
        if (forgetBtn) {
            var fRow = forgetBtn.closest("[data-memory-key]");
            var fKey = fRow.getAttribute("data-memory-key");
            var fBot = fRow.getAttribute("data-memory-bot") || "";
            busy = true;
            setSyncStatus("Syncing…");
            var fIdx = entries.findIndex(function (en) {
                return en.key === fKey && String(en.bot_id || "") === String(fBot || "");
            });
            if (fIdx !== -1) entries.splice(fIdx, 1);
            save(entries);
            try { await serverDelete(fKey); setSyncStatus("Synced"); }
            catch (e2) { setSyncStatus("Offline"); }
            renderList();
            busy = false;
            return;
        }
        if (e.target && e.target.id === "gbMemoryClear") {
            if (!confirm("Forget every remembered fact (server and this browser)?")) return;
            busy = true;
            setSyncStatus("Syncing…");
            var all = entries.slice();
            entries = [];
            save(entries);
            for (var k = 0; k < all.length; k++) await serverDelete(all[k].key);
            renderList();
            setSyncStatus("Synced");
            busy = false;
        }
    });
    document.addEventListener("input", function (e) {
        if (e.target && e.target.id === "gbMemorySearch") {
            query = e.target.value.trim();
            renderList();
        }
    });
    document.addEventListener("change", function (e) {
        if (e.target && e.target.id === "gbMemoryScope") {
            scopeFilter = e.target.value;
            renderList();
        }
    });
    document.addEventListener("DOMContentLoaded", refresh);
    document.addEventListener("gb-memory-synced", refresh);
    if (document.readyState !== "loading") refresh();
})();
