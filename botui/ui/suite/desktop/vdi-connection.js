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
        if (conn) startSession(conn.host, conn.port);
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

        var btnNew = document.getElementById("btn-new-connection");
        if (btnNew) btnNew.onclick = showNewConnectionForm;

        var btnModalClose = document.getElementById("btn-modal-close");
        if (btnModalClose) btnModalClose.onclick = hideModal;

        var modalOverlay = document.getElementById("modal-overlay");
        if (modalOverlay) {
            modalOverlay.addEventListener("click", function (e) {
                if (e.target === modalOverlay) hideModal();
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
        closeModal: hideModal,
    };

    document.addEventListener("DOMContentLoaded", init);
})();
