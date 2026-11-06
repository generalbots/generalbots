// #1551/#1642 GB6-16 — memory across conversations. The server copy
// (chat_preferences via /api/user/memory) is the source of truth the prompt
// reads; localStorage is only a cache for offline rendering.
(function () {
    function storageKey() {
        var bot = window.__INITIAL_BOT_NAME__ || "default";
        return "gb_chat_memory_" + bot;
    }
    function load() {
        try { return JSON.parse(localStorage.getItem(storageKey()) || "{}"); }
        catch (e) { return {}; }
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
    async function serverPut(key, value) {
        if (!token()) return;
        try {
            await fetch("/api/user/memory", {
                method: "PUT",
                headers: headers({ "Content-Type": "application/json" }),
                body: JSON.stringify({ key: key, value: value }),
            });
        } catch (e) { /* offline: local cache still holds it */ }
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
    function renderList(mem) {
        var list = document.getElementById("gbMemoryList");
        if (!list) return;
        var keys = Object.keys(mem);
        if (!keys.length) {
            list.innerHTML = '<p class="setting-desc" style="padding:8px 0;">Nothing remembered yet. Tell the assistant "remember …" or add a fact below.</p>';
            return;
        }
        list.innerHTML = keys.map(function (key) {
            return '<div class="setting-row" data-memory-key="' + esc(key) + '">' +
                '<div class="setting-info"><span class="setting-title">' + esc(key) + '</span>' +
                '<span class="setting-desc">' + esc(mem[key].value || "") + '</span></div>' +
                '<button type="button" class="btn btn-secondary btn-sm gb-memory-forget">Forget</button>' +
                "</div>";
        }).join("");
    }
    async function refresh() {
        var mem = load();
        var remote = await serverList();
        if (remote) {
            // The server copy wins for every entry it holds; entries only this
            // browser knew about are pushed once, then merged.
            for (var i = 0; i < remote.length; i++) {
                var entry = remote[i];
                if (!entry || !entry.key) continue;
                mem[entry.key] = {
                    value: String(entry.value || "").substring(0, 500),
                    _ts: Date.parse(entry.updated_at) || Date.now(),
                };
            }
            var missing = Object.keys(load()).filter(function (key) {
                return !remote.some(function (entry) { return entry.key === key; });
            });
            for (var j = 0; j < missing.length; j++) {
                var cached = load()[missing[j]];
                if (cached && cached.value) await serverPut(missing[j], cached.value);
            }
            save(mem);
        }
        renderList(mem);
    }
    var busy = false;
    document.addEventListener("click", async function (e) {
        if (busy) return;
        var forget = e.target.closest && e.target.closest(".gb-memory-forget");
        if (forget) {
            var row = forget.closest("[data-memory-key]");
            var key = row && row.getAttribute("data-memory-key");
            if (key) {
                busy = true;
                var mem = load();
                delete mem[key];
                save(mem);
                await serverDelete(key);
                renderList(mem);
                busy = false;
            }
            return;
        }
        if (e.target && e.target.id === "gbMemoryAdd") {
            var keyEl = document.getElementById("gbMemoryKey");
            var valueEl = document.getElementById("gbMemoryValue");
            var addKey = keyEl && keyEl.value.trim();
            var addValue = valueEl && valueEl.value.trim();
            if (!addKey || !addValue) return;
            busy = true;
            var current = load();
            current[addKey] = { value: addValue.substring(0, 500), _ts: Date.now() };
            save(current);
            await serverPut(addKey, addValue.substring(0, 500));
            keyEl.value = "";
            valueEl.value = "";
            renderList(current);
            busy = false;
            return;
        }
        if (e.target && e.target.id === "gbMemoryClear") {
            if (!confirm("Forget every remembered fact (server and this browser)?")) return;
            busy = true;
            var all = load();
            var keys = Object.keys(all);
            save({});
            for (var k = 0; k < keys.length; k++) await serverDelete(keys[k]);
            renderList({});
            busy = false;
        }
    });
    document.addEventListener("DOMContentLoaded", refresh);
    document.addEventListener("gb-memory-synced", refresh);
    if (document.readyState !== "loading") refresh();
})();
