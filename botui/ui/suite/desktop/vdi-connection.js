/* VDI companion module (#1313).
 *
 * vdi.js owns the VDI window: connection list, quick connect, saved
 * connections and the noVNC session lifecycle. It publishes that surface on
 * window.VDI.
 *
 * This file used to re-implement the same handlers while calling symbols that
 * live inside vdi.js's IIFE (`loadConnections`, `CONNECTIONS`,
 * `renderConnections`, `toast`). Each call threw a ReferenceError, which
 * aborted initialisation — so every VDI control (including Quick Connect to a
 * VNC host) appeared dead. It is now a thin adapter over window.VDI, kept for
 * the callers and markup that still reference the old entry points.
 */
(function () {
    'use strict';

    function api() {
        return window.VDI && typeof window.VDI === 'object' ? window.VDI : null;
    }

    function notify(message, kind) {
        var host = api();
        if (host && typeof host.toast === 'function') {
            host.toast(message, kind || 'info');
        }
    }

    async function deleteSavedConnection(id) {
        if (!confirm('Delete this connection?')) return;
        var host = api();
        // Prefer the owner module's implementation (it keeps its internal list
        // and re-renders); only fall back to the endpoint when absent.
        if (host && typeof host.deleteSaved === 'function' && host.deleteSaved !== deleteSavedConnection) {
            host.deleteSaved(id);
            return;
        }
        try {
            await fetch('/api/desktop/connections/' + encodeURIComponent(id), { method: 'DELETE' });
            if (host && typeof host.removeConnection === 'function') host.removeConnection(id);
            notify('Connection deleted', 'info');
        } catch (e) {
            notify('Delete failed', 'error');
        }
    }

    function connectSaved(id) {
        var host = api();
        var connections = host && typeof host.getConnections === 'function' ? host.getConnections() : [];
        var conn = connections.find(function (c) { return c.id === id; });
        if (host && typeof host.connectSaved === 'function' && host.connectSaved !== connectSaved) {
            host.connectSaved(id);
            return;
        }
        if (!conn) return;
        var hostName = conn.host || conn.target_host || '';
        var port = conn.port || conn.target_port || (conn.protocol === 'rdp' ? 3389 : 5900);
        if (!hostName) {
            notify('Connection has no host', 'error');
            return;
        }
        if (host && typeof host.startSession === 'function') {
            host.startSession(hostName, port, conn.protocol || 'vnc');
            return;
        }
        notify('VDI module unavailable', 'error');
    }

    function quickConnect() {
        var host = api();
        if (host && typeof host.connectQuick === 'function' && host.connectQuick !== quickConnect) {
            host.connectQuick();
            return;
        }
        var hostEl = document.getElementById('quick-host');
        var portEl = document.getElementById('quick-port');
        var address = (hostEl ? hostEl.value : '').trim();
        var port = parseInt(portEl ? portEl.value : '5900', 10) || 5900;
        if (!address) {
            notify('Enter a host address', 'error');
            return;
        }
        if (host && typeof host.startSession === 'function') {
            host.startSession(address, port, 'vnc');
            return;
        }
        notify('VDI module unavailable', 'error');
    }

    function init() {
        // vdi.js binds the window controls on its own load; only wire the
        // fallbacks when it is not present, so the two never double-bind.
        var host = api();
        if (!(host && typeof host.connectQuick === 'function')) {
            var btnConnect = document.getElementById('btn-quick-connect');
            if (btnConnect) btnConnect.onclick = quickConnect;
        }
        var hostInput = document.getElementById('quick-host');
        if (hostInput) {
            hostInput.addEventListener('keydown', function (e) {
                if (e.key === 'Enter') quickConnect();
            });
        }
        var btnNew = document.getElementById('btn-new-connection');
        if (btnNew && !(host && typeof host.openNewConnection === 'function') && window.showNewConnectionForm) {
            btnNew.onclick = window.showNewConnectionForm;
        }
        var btnModalClose = document.getElementById('btn-modal-close');
        if (btnModalClose && !(host && typeof host.closeModal === 'function') && window.hideModal) {
            btnModalClose.onclick = window.hideModal;
        }
        var modalOverlay = document.getElementById('modal-overlay');
        if (modalOverlay && window.hideModal) {
            modalOverlay.addEventListener('click', function (e) {
                if (e.target === modalOverlay) window.hideModal();
            });
        }
        if (host && typeof host.loadConnections === 'function') host.loadConnections();
    }

    // Merge into the namespace vdi.js already created instead of replacing it.
    var ns = api() || {};
    ns.connectionInit = init;
    ns.deleteSaved = ns.deleteSaved || deleteSavedConnection;
    ns.connectSaved = ns.connectSaved || connectSaved;
    ns.connectQuick = ns.connectQuick || quickConnect;
    window.VDI = ns;

    (function () {
        var boot = init;
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', boot);
        } else {
            boot();
        }
    })();
})();
