(function () {
'use strict';
var VDI = window.VDI;
var toast = VDI.toast;
var showModal = VDI.showModal;
var hideModal = VDI.hideModal;
var loadConnections = VDI.loadConnections;
var startSession = VDI.startSession;
var startSessionWithId = VDI.startSessionWithId;

    function showNewConnectionForm() {
        var formHtml =
            '<div class="vdi-form">' +
            '<div class="form-group">' +
            '<label>Connection Name</label>' +
            '<input type="text" id="new-conn-name" class="input" placeholder="My Server">' +
            "</div>" +
            '<div class="form-group">' +
            '<label>Host</label>' +
            '<input type="text" id="new-conn-host" class="input" placeholder="10.0.0.1">' +
            "</div>" +
            '<div class="form-group">' +
            '<label>Port</label>' +
            '<input type="number" id="new-conn-port" class="input" value="5900">' +
            "</div>" +
            '<div class="form-group">' +
            '<label>Protocol</label>' +
            '<select id="new-conn-protocol" class="input" onchange="VDI.onProtocolChange()">' +
            '<option value="vnc">VNC</option>' +
            '<option value="rdp">RDP</option>' +
            "</select>" +
            "</div>" +
            '<div class="form-group" id="new-conn-rdp-fields" style="display:none;">' +
            '<label>RDP Password</label>' +
            '<input type="password" id="new-conn-password" class="input" placeholder="Target password (vaulted)" autocomplete="off">' +
            "</div>" +
            '<div class="form-group" id="new-conn-rdp-domain" style="display:none;">' +
            '<label>RDP Domain (optional)</label>' +
            '<input type="text" id="new-conn-domain" class="input" placeholder="CORP\\user">' +
            "</div>" +
            '<div class="form-actions">' +
            '<button class="btn btn-ghost" onclick="VDI.closeModal()">Cancel</button>' +
            '<button class="btn btn-primary" onclick="VDI.saveNewConnection()">Save & Connect</button>' +
            "</div>" +
            "</div>";
        showModal("New Connection", formHtml);
        window.VDI.onProtocolChange = function () {
            var proto = (document.getElementById("new-conn-protocol") || {}).value || "vnc";
            var portEl = document.getElementById("new-conn-port");
            var pwdRow = document.getElementById("new-conn-rdp-fields");
            var domRow = document.getElementById("new-conn-rdp-domain");
            var isRdp = proto === "rdp";
            if (portEl && isRdp && parseInt(portEl.value, 10) === 5900) portEl.value = "3389";
            if (portEl && !isRdp && parseInt(portEl.value, 10) === 3389) portEl.value = "5900";
            if (pwdRow) pwdRow.style.display = isRdp ? "" : "none";
            if (domRow) domRow.style.display = isRdp ? "" : "none";
        };
    }

    window.saveNewConnection = async function saveNewConnection() {
    window.showNewConnectionForm = showNewConnectionForm;
    window.hideModal = hideModal;
        var name = (document.getElementById("new-conn-name") || {}).value || "";
        var host = (document.getElementById("new-conn-host") || {}).value || "";
        var port = parseInt((document.getElementById("new-conn-port") || {}).value || "5900");
        var protocol = (document.getElementById("new-conn-protocol") || {}).value || "vnc";
        var password = (document.getElementById("new-conn-password") || {}).value || "";
        var domain = (document.getElementById("new-conn-domain") || {}).value || "";
        name = name.trim();
        host = host.trim();
        if (!name || !host) {
            toast("Name and host are required", "error");
            return;
        }
        try {
            var payload = { name: name, host: host, port: port, protocol: protocol, auth_type: "password" };
            if (protocol === "rdp") {
                if (password) payload.password = password;
                if (domain) payload.domain = domain;
            }
            var resp = await fetch("/api/desktop/connect", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            if (!resp.ok) throw new Error("Save failed");
            var saved = await resp.json();
            hideModal();
            toast("Connection saved", "success");
            await loadConnections();
            startSessionWithId(host, port, saved.data && saved.data.id, protocol);
        } catch (e) {
            toast("Failed to save: " + e.message, "error");
        }
    }

    // =============================================================================
    // VDI PUBLIC API (wires the vdi.html controls)
    // =============================================================================

    var pendingAuth = null;

    function openNewConnection() {
        var overlay = document.getElementById("modal-overlay");
        if (overlay) overlay.classList.remove("hidden");
    }
    function closeModal() {
        var overlay = document.getElementById("modal-overlay");
        if (overlay) overlay.classList.add("hidden");
    }
    function openAuth() {
        var overlay = document.getElementById("auth-modal-overlay");
        if (overlay) overlay.classList.remove("hidden");
    }
    function closeAuth() {
        var overlay = document.getElementById("auth-modal-overlay");
        if (overlay) overlay.classList.add("hidden");
    }

    function readConnForm() {
        var el = function (id) { return document.getElementById(id); };
        var protocol = (el("conn-protocol") || {}).value || "vnc";
        var portEl = el("conn-port");
        var port = parseInt((portEl || {}).value || (protocol === "rdp" ? "3389" : "5900"), 10);
        return {
            name: ((el("conn-name") || {}).value || "").trim(),
            host: ((el("conn-host") || {}).value || "").trim(),
            port: port,
            protocol: protocol,
            authType: (el("conn-auth-type") || {}).value || "none",
        };
    }

    function saveNewConnection() {
        var c = readConnForm();
        if (!c.name || !c.host) {
            toast("Name and host are required", "error");
            return;
        }
        if (c.authType === "password") {
            pendingAuth = c;
            closeModal();
            openAuth();
            return;
        }
        closeModal();
        startSession(c.host, c.port, c.protocol, null);
    }

    function connectQuick() {
        var raw = ((document.getElementById("quick-host") || {}).value || "").trim();
        if (!raw) {
            toast("Enter a host:port", "error");
            return;
        }
        var parts = raw.split(":");
        var host = parts[0].trim();
        var port = parseInt((parts[1] || (raw.indexOf("rdp") >= 0 ? "3389" : "5900")).trim(), 10) || 5900;
        startSession(host, port, "vnc", null);
    }

    function submitAuth() {
        var password = ((document.getElementById("auth-password") || {}).value || "").trim();
        var c = pendingAuth;
        pendingAuth = null;
        closeAuth();
        if (!c) return;
        startSession(c.host, c.port, c.protocol, password);
    }

    function connectSaved(id) {
        var c = VDI.getConnections().filter(function (x) { return x.id === id; })[0];
        if (!c) {
            toast("Connection not found", "error");
            return;
        }
        startSession(c.host, c.port, c.protocol || "vnc", null);
    }

    async function deleteSaved(id) {
        try {
            var resp = await fetch("/api/desktop/connections/" + encodeURIComponent(id), { method: "DELETE" });
            if (!resp.ok) throw new Error("delete failed");
            toast("Connection removed", "success");
            await loadConnections();
        } catch (e) {
            toast("Failed to remove: " + e.message, "error");
        }
    }

    function applyResolution(res) {
        var canvas = document.getElementById("vnc-canvas");
        if (!canvas) return;
        if (!res || res === "auto") {
            var wrap = document.getElementById("vnc-container");
            canvas.style.width = "100%";
            canvas.style.height = "100%";
            return;
        }
        var dims = res.split("x");
        if (dims.length === 2) {
            var w = parseInt(dims[0], 10), h = parseInt(dims[1], 10);
            canvas.width = w;
            canvas.height = h;
            canvas.style.width = w + "px";
            canvas.style.height = h + "px";
        }
    }

    function applyScaling(mode) {
        var canvas = document.getElementById("vnc-canvas");
        if (!canvas) return;
        if (mode === "none") {
            applyResolution((document.getElementById("resolution-select") || {}).value);
        } else {
            canvas.style.width = "100%";
            canvas.style.height = "100%";
        }
    }

    function onProtocolChange() {
        var proto = (document.getElementById("conn-protocol") || {}).value || "vnc";
        var portEl = document.getElementById("conn-port");
        if (!portEl) return;
        var v = parseInt(portEl.value, 10);
        if (proto === "rdp" && (isNaN(v) || v === 5900)) portEl.value = "3389";
        if (proto === "vnc" && (isNaN(v) || v === 3389)) portEl.value = "5900";
        var authRow = document.getElementById("conn-auth-type");
        if (authRow) authRow.value = "password";
    }

    Object.assign(window.VDI, {
        openNewConnection: openNewConnection,
        closeModal: closeModal,
        openAuth: openAuth,
        closeAuth: closeAuth,
        saveNewConnection: saveNewConnection,
        connectQuick: connectQuick,
        submitAuth: submitAuth,
        connectSaved: connectSaved,
        deleteSaved: deleteSaved,
        applyResolution: applyResolution,
        applyScaling: applyScaling,
        onProtocolChange: onProtocolChange,
        // #1313 — vdi-connection.js runs right after this file and used to
        // call these bare names, which are scoped to this IIFE: the resulting
        // ReferenceError aborted its init and no VDI button was ever wired.
        loadConnections: loadConnections,
        startSession: startSession,
        startSessionWithId: startSessionWithId,
        init: bindVdiControls,
        // #1313 — these were IIFE-private too, so the companion module threw
        // ReferenceErrors on delete/notify paths.
        toast: toast,
    });

    function bindVdiControls() {
        var bind = function (id, ev, fn) {
            var el = document.getElementById(id);
            if (el) el.addEventListener(ev, fn);
        };
        bind("btn-new-connection", "click", openNewConnection);
        bind("btn-modal-cancel", "click", closeModal);
        bind("btn-modal-close", "click", closeModal);
        bind("btn-modal-save", "click", saveNewConnection);
        bind("btn-quick-connect", "click", connectQuick);
        bind("btn-auth-submit", "click", submitAuth);
        bind("conn-protocol", "change", onProtocolChange);
        var resSel = document.getElementById("resolution-select");
        if (resSel) resSel.addEventListener("change", function () { applyResolution(resSel.value); });
        var scaleSel = document.getElementById("scaling-select");
        if (scaleSel) scaleSel.addEventListener("change", function () { applyScaling(scaleSel.value); });
        if (typeof loadConnections === "function") loadConnections();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", bindVdiControls);
    } else {
        bindVdiControls();
    }
})();
