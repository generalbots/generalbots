(function () {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("vdi");
var VDI = window.VDI = window.VDI || {};

    var CONNECTIONS = [];

    function esc(s) {
        var d = document.createElement("div");
        d.textContent = s || "";
        return d.innerHTML;
    }

    function toast(msg, type) {
        var container = document.getElementById("toast-container");
        if (!container) return;
        var el = document.createElement("div");
        el.className = "toast toast-" + (type || "info");
        el.textContent = msg;
        container.appendChild(el);
        setTimeout(function () {
            el.classList.add("toast-fade");
            setTimeout(function () { el.remove(); }, 300);
        }, 3000);
    }

    function showModal(title, bodyHtml) {
        var overlay = document.getElementById("modal-overlay");
        var titleEl = document.getElementById("modal-title");
        var bodyEl = document.getElementById("modal-body");
        if (titleEl) titleEl.textContent = title;
        if (bodyEl) bodyEl.innerHTML = bodyHtml;
        if (overlay) overlay.classList.remove("hidden");
    }

    function hideModal() {
        var overlay = document.getElementById("modal-overlay");
        if (overlay) overlay.classList.add("hidden");
    }

    function updateConnectionCount() {
        var badge = document.getElementById("connections-count");
        if (badge) badge.textContent = CONNECTIONS.length;
    }

    function switchView(view) {
        var dashboard = document.getElementById("vdi-dashboard");
        var canvas = document.getElementById("vdi-canvas");
        if (view === "canvas") {
            if (dashboard) dashboard.classList.add("hidden");
            if (canvas) canvas.classList.remove("hidden");
        } else {
            if (dashboard) dashboard.classList.remove("hidden");
            if (canvas) canvas.classList.add("hidden");
        }
    }

    async function loadConnections() {
        try {
            var resp = await fetch("/api/desktop/connections");
            if (!resp.ok) return;
            var body = await resp.json();
            CONNECTIONS = (body && body.data) || [];
            renderConnections();
        } catch (e) {
            console.error("Failed to load connections:", e);
        }
    }

    function renderConnections() {
        var grid = document.getElementById("connections-grid");
        if (!grid) return;
        updateConnectionCount();
        if (!CONNECTIONS.length) {
            grid.innerHTML =
                '<div class="vdi-empty-state">' +
                '<div class="empty-icon">&#128421;</div>' +
                '<h3>No connections yet</h3>' +
                '<p>Create a new connection to get started</p>' +
                "</div>";
            return;
        }
        grid.innerHTML = CONNECTIONS.map(function (c) {
            var host = c.host || c.target_host || "";
            var port = c.port || c.target_port || 5900;
            var name = c.name || "Desktop";
            return (
                '<div class="vdi-connection-card" data-id="' + esc(c.id) + '">' +
                '<div class="card-header">' +
                '<span class="card-icon">&#128187;</span>' +
                '<div class="card-info">' +
                '<div class="card-name">' + esc(name) + "</div>" +
                '<div class="card-host">' + esc(host) + ":" + port + "</div>" +
                "</div>" +
                "</div>" +
                '<div class="card-footer">' +
                '<span class="card-protocol">' + esc(c.protocol || "vnc").toUpperCase() + "</span>" +
                '<div class="card-actions">' +
                '<button class="btn btn-primary btn-xs" onclick="VDI.connectSaved(\'' + esc(c.id) + "')\">Connect</button>" +
                '<button class="btn btn-ghost btn-xs btn-danger" onclick="VDI.deleteSaved(\'' + esc(c.id) + "')\">&#10005;</button>" +
                "</div>" +
                "</div>" +
                "</div>"
            );
        }).join("");
    }

    function drawStatusScreen(canvas, host, port, statusText) {
        var ctx = canvas.getContext("2d");
        var w = canvas.width = canvas.parentElement.clientWidth || 800;
        var h = canvas.height = canvas.parentElement.clientHeight || 600;
        ctx.fillStyle = "#1a1b26";
        ctx.fillRect(0, 0, w, h);
        ctx.fillStyle = "#7aa2f7";
        ctx.font = "bold 20px monospace";
        ctx.textAlign = "center";
        ctx.fillText("Desktop VDI", w / 2, h / 2 - 60);
        ctx.fillStyle = "#565f89";
        ctx.font = "14px monospace";
        ctx.fillText("Target: " + host + ":" + port, w / 2, h / 2 - 20);
        ctx.fillStyle = "#9ece6a";
        ctx.font = "16px monospace";
        ctx.fillText(statusText || "Connecting...", w / 2, h / 2 + 20);
        ctx.fillStyle = "#565f89";
        ctx.font = "12px monospace";
        ctx.fillText("VNC rendering requires noVNC library", w / 2, h / 2 + 60);
        ctx.fillText("Place noVNC files in /suite/desktop/vendor/novnc/", w / 2, h / 2 + 80);
    }

    // Shared surface for the ordered vdi scripts (#1731).
    VDI.toast = toast;
    VDI.showModal = showModal;
    VDI.hideModal = hideModal;
    VDI.switchView = switchView;
    VDI.drawStatusScreen = drawStatusScreen;
    VDI.loadConnections = loadConnections;
    VDI.renderConnections = renderConnections;
    VDI.getConnections = function () { return CONNECTIONS; };
    VDI.removeConnection = function (id) {
        CONNECTIONS = CONNECTIONS.filter(function (c) { return c.id !== id; });
        renderConnections();
    };
})();
