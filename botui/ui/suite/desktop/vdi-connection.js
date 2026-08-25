(function() {
'use strict';
    async function deleteSavedConnection(id) {
        if (!confirm("Delete this connection?")) return;
        try {
            await fetch("/api/desktop/connections/" + id, { method: "DELETE" });
            CONNECTIONS = CONNECTIONS.filter(function (c) { return c.id !== id; });
            renderConnections();
            toast("Connection deleted", "info");
        } catch (e) {
            toast("Delete failed", "error");
        }
    }

    function connectSaved(id) {
        var conn = CONNECTIONS.find(function (c) { return c.id === id; });
        if (!conn) return;
        var host = conn.host || conn.target_host || "";
        var port = conn.port || conn.target_port || (conn.protocol === "rdp" ? 3389 : 5900);
        if (!host) {
            toast("Connection has no host", "error");
            return;
        }
        startSession(host, port, conn.protocol || "vnc");
    }

    function quickConnect() {
        var hostEl = document.getElementById("quick-host");
        var portEl = document.getElementById("quick-port");
        var host = (hostEl ? hostEl.value : "").trim();
        var port = parseInt(portEl ? portEl.value : "5900") || 5900;
        if (!host) {
            toast("Enter a host address", "error");
            return;
        }
        startSession(host, port);
    }

    function init() {
        var btnConnect = document.getElementById("btn-quick-connect");
        if (btnConnect) btnConnect.onclick = quickConnect;

        var hostInput = document.getElementById("quick-host");
        if (hostInput) {
            hostInput.addEventListener("keydown", function (e) {
                if (e.key === "Enter") quickConnect();
            });
        }

        var showNew = window.showNewConnectionForm;
        var hide = window.hideModal;
        var btnNew = document.getElementById("btn-new-connection");
        if (btnNew && showNew) btnNew.onclick = showNew;

        var btnModalClose = document.getElementById("btn-modal-close");
        if (btnModalClose && hide) btnModalClose.onclick = hide;

        var modalOverlay = document.getElementById("modal-overlay");
        if (modalOverlay && hide) {
            modalOverlay.addEventListener("click", function (e) {
                if (e.target === modalOverlay) hide();
            });
        }

        updateSessionBadge();
        loadConnections();
    }

    window.VDI = {
        init: init,
        connectSaved: connectSaved,
        deleteSaved: deleteSavedConnection,
        saveNewConnection: window.saveNewConnection,
        closeModal: window.hideModal,
    };

    (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
})();
