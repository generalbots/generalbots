/* #1843 — shared container/VM tab strip.
 *
 * The browser app keeps personal tabs locally (browser-inline.js). Tabs opened
 * inside the user's agent VM live on the server and are shared by every
 * conversation: this module lists them, badges them green ("VM"), attaches a
 * tab to the current conversation and streams its frames (VNC-like) over the
 * existing /api/browser/session/:id/stream socket.
 */
(function () {
    "use strict";

    var DEFAULT_VM = "default";
    var POLL_MS = 5000;
    var strip = document.getElementById("browser-vm-tabs");
    var list = document.getElementById("browser-vm-tabs-list");
    var hint = document.getElementById("browser-vm-tabs-hint");
    var viewport = document.getElementById("browser-viewport");
    var urlInput = document.getElementById("browser-url");
    var statusText = document.getElementById("browser-status-text");

    if (!strip || !list || !viewport) return;

    var vm = DEFAULT_VM;
    var conversationId = null;
    var tabs = [];
    var currentTabId = null;
    var ws = null;
    var vmPage = null;
    var canvas = null;
    var ctx = null;
    var stateEl = null;
    var timer = null;

    function authHeaders() {
        var token = "";
        if (typeof window.getGBAccessToken === "function") token = window.getGBAccessToken() || "";
        if (!token) token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
        return token ? { "Authorization": "Bearer " + token } : {};
    }

    function appParams() {
        return window.__gbAppParams__ || {};
    }

    function queryParam(name) {
        try { return new URLSearchParams(window.location.search).get(name) || ""; } catch (e) { return ""; }
    }

    function resolveConversation() {
        var p = appParams();
        conversationId = p.session_id || p.conversation_id || queryParam("session_id") || queryParam("conversation_id") || null;
        if (!conversationId && window.ChatState && window.ChatState.currentSessionId) {
            conversationId = window.ChatState.currentSessionId;
        }
    }

    function resolveVm() {
        var p = appParams();
        var explicit = p.vm || queryParam("vm");
        if (explicit) return Promise.resolve(String(explicit));
        // The agent workspace name is per user. Fall back to "default" for the
        // desktop-local build, where the VM browser is the only browser.
        return fetch("/api/agent/vm/status", { headers: authHeaders() })
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (data) {
                if (data && data.has_vm && data.vm_name) return String(data.vm_name);
                return DEFAULT_VM;
            })
            .catch(function () { return DEFAULT_VM; });
    }

    function api(path, method, body) {
        var opts = { method: method || "GET", headers: authHeaders() };
        if (body) {
            opts.headers = Object.assign({ "Content-Type": "application/json" }, opts.headers);
            opts.body = JSON.stringify(body);
        }
        return fetch(path, opts).then(function (r) {
            if (!r.ok) throw new Error("HTTP " + r.status);
            return r.json();
        });
    }

    function tabPath(suffix) {
        return "/api/browser/vm/" + encodeURIComponent(vm) + "/tabs" + (suffix || "");
    }

    function loadTabs() {
        return api(tabPath()).then(function (data) {
            tabs = (data && data.tabs) || [];
            render();
            if (currentTabId && !tabs.some(function (t) { return t.id === currentTabId; })) {
                stopStream();
                currentTabId = null;
            }
        }).catch(function () {
            // Offline / browser feature not compiled: hide the strip instead of
            // covering the user's personal browsing with an empty bar.
            strip.style.display = "none";
        });
    }

    function render() {
        // Keep the strip visible even when empty: it holds the "+ VM tab"
        // control that opens the first shared tab.
        strip.style.display = "flex";
        list.textContent = "";
        tabs.forEach(function (tab) {
            var el = document.createElement("button");
            el.type = "button";
            el.className = "browser-vm-tab" + (tab.id === currentTabId ? " active" : "");
            el.setAttribute("data-tab-id", tab.id);
            if (tab.vm_controlled) el.setAttribute("data-vm-controlled", "true");
            el.title = (tab.title || tab.url || "VM tab") + (tab.vm_controlled ? " — VM-controlled" : "");

            if (tab.vm_controlled) {
                var badge = document.createElement("span");
                badge.className = "browser-vm-badge";
                badge.textContent = "VM";
                el.appendChild(badge);
            }

            var title = document.createElement("span");
            title.className = "browser-vm-tab-title";
            title.textContent = tab.title || tab.url || "New Tab";
            el.appendChild(title);

            if (tab.conversation_id && conversationId && tab.conversation_id !== conversationId) {
                var shared = document.createElement("span");
                shared.className = "browser-vm-shared";
                shared.textContent = "shared";
                el.appendChild(shared);
            }

            el.addEventListener("click", function () { attachTab(tab); });

            var close = document.createElement("span");
            close.className = "browser-vm-tab-close";
            close.textContent = "\u00d7";
            close.title = "Close VM tab";
            close.addEventListener("click", function (ev) {
                ev.stopPropagation();
                closeTab(tab.id);
            });
            el.appendChild(close);
            list.appendChild(el);
        });

        var open = document.createElement("button");
        open.type = "button";
        open.className = "browser-vm-tab-open";
        open.textContent = "+ VM tab";
        open.title = "Open a tab inside the agent VM";
        open.addEventListener("click", openTab);
        list.appendChild(open);

        if (hint) hint.textContent = vm === DEFAULT_VM ? "Shared across conversations" : "VM " + vm;
    }

    function ensurePage() {
        if (vmPage && document.body.contains(vmPage)) return vmPage;
        vmPage = document.createElement("div");
        vmPage.className = "browser-page browser-vm-page";
        vmPage.id = "browser-vm-page";

        canvas = document.createElement("canvas");
        canvas.id = "browser-vm-canvas";
        canvas.addEventListener("click", function (ev) { clickAt(ev, false); });
        canvas.addEventListener("dblclick", function (ev) { clickAt(ev, true); });
        vmPage.appendChild(canvas);

        stateEl = document.createElement("div");
        stateEl.className = "browser-vm-state";
        stateEl.textContent = "Connecting to VM tab…";
        vmPage.appendChild(stateEl);

        viewport.appendChild(vmPage);
        ctx = canvas.getContext("2d");
        return vmPage;
    }

    function showPage(tab) {
        ensurePage();
        vmPage.classList.add("active");
        stateEl.textContent = "Connecting to VM tab…";
        if (urlInput) urlInput.value = tab.url || "";
        if (statusText) statusText.textContent = "VM tab: " + (tab.title || tab.url || "New Tab");
    }

    function hidePage() {
        if (vmPage) vmPage.classList.remove("active");
    }

    function attachTab(tab) {
        currentTabId = tab.id;
        render();
        var body = { conversation_id: conversationId };
        api(tabPath("/" + encodeURIComponent(tab.id) + "/attach"), "POST", body)
            .catch(function () { /* the stream still works without an attach */ })
            .then(function () {
                showPage(tab);
                startStream(tab.id);
            });
    }

    function openTab() {
        api(tabPath(), "POST", {
            headless: true,
            vm: vm,
            conversation_id: conversationId,
            vm_controlled: true,
            title: "New Tab"
        }).then(function (data) {
            var tab = data && data.tab;
            return loadTabs().then(function () {
                if (tab) attachTab(tab);
            });
        }).catch(function () {
            if (statusText) statusText.textContent = "Could not open a VM tab";
        });
    }

    function closeTab(id) {
        api(tabPath("/" + encodeURIComponent(id)), "DELETE")
            .catch(function () { return null; })
            .then(function () {
                if (currentTabId === id) {
                    stopStream();
                    currentTabId = null;
                    hidePage();
                }
                return loadTabs();
            });
    }

    function startStream(id) {
        stopStream();
        var proto = location.protocol === "https:" ? "wss:" : "ws:";
        ws = new WebSocket(proto + "//" + location.host + "/api/browser/session/" + encodeURIComponent(id) + "/stream");
        ws.binaryType = "arraybuffer";
        ws.onmessage = function (ev) {
            if (ev.data instanceof ArrayBuffer) renderFrame(ev.data);
        };
        ws.onopen = function () {
            if (stateEl) stateEl.textContent = "";
        };
        ws.onclose = function () {
            ws = null;
            if (currentTabId === id) {
                if (stateEl) stateEl.textContent = "VM tab disconnected — reconnecting…";
                timer = setTimeout(function () { if (currentTabId === id) startStream(id); }, 2000);
            }
        };
        ws.onerror = function () { };
    }

    function stopStream() {
        if (timer) { clearTimeout(timer); timer = null; }
        if (ws) { ws.close(); ws = null; }
    }

    function renderFrame(data) {
        if (!ctx || !canvas) return;
        var blob = new Blob([data], { type: "image/png" });
        var url = URL.createObjectURL(blob);
        var img = new Image();
        img.onload = function () {
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);
            URL.revokeObjectURL(url);
            if (stateEl) stateEl.textContent = "";
        };
        img.onerror = function () { URL.revokeObjectURL(url); };
        img.src = url;
    }

    function clickAt(ev, doubleClick) {
        if (!currentTabId || !canvas || !canvas.width || !canvas.height) return;
        var rect = canvas.getBoundingClientRect();
        var scaleX = canvas.width / rect.width;
        var scaleY = canvas.height / rect.height;
        var x = Math.round((ev.clientX - rect.left) * scaleX);
        var y = Math.round((ev.clientY - rect.top) * scaleY);
        var body = { x: x, y: y };
        if (doubleClick) body.click_count = 2;
        api("/api/browser/session/" + encodeURIComponent(currentTabId) + "/click", "POST", body)
            .catch(function () { /* keep the surface quiet; the frame shows the result */ });
    }

    function wrapLocalNavigation() {
        if (!window.GBBrowser) return;
        ["switchTab", "go", "addTab", "closeTab"].forEach(function (name) {
            var original = window.GBBrowser[name];
            if (typeof original !== "function" || original.__vmTabsWrapped) return;
            var wrapped = function () {
                hidePage();
                return original.apply(this, arguments);
            };
            wrapped.__vmTabsWrapped = true;
            window.GBBrowser[name] = wrapped;
        });
    }

    function wire() {
        resolveConversation();
        wrapLocalNavigation();

        var toolbar = document.querySelector(".browser-toolbar");
        if (toolbar) toolbar.addEventListener("click", hidePage);
        var localTabs = document.getElementById("browser-tabs-bar");
        if (localTabs) localTabs.addEventListener("click", hidePage);
        var input = document.getElementById("browser-url");
        if (input) input.addEventListener("focus", hidePage);

        resolveVm().then(function (name) {
            vm = name;
            render();
            loadTabs();
            setInterval(loadTabs, POLL_MS);
        });
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", wire);
    } else {
        wire();
    }

    window.GBVMTabs = {
        list: loadTabs,
        attach: function (id) {
            var tab = tabs.find(function (t) { return t.id === id; });
            if (tab) attachTab(tab);
        },
        open: openTab,
        close: closeTab,
        vm: function () { return vm; }
    };
})();
