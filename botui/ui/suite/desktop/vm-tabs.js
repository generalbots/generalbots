/* #1843 — shared container/VM tab strip for the desktop Browser surface.
 *
 * Personal tabs stay local to the canvas app. Tabs opened inside the user's
 * agent VM are listed from the server, shared by every conversation and
 * badged green. Selecting one hands its session id back to browser.js, which
 * already streams the CDP frames over /api/browser/session/:id/stream.
 */
"use strict";

(function () {
    let vmName = 'default';
    let conversationId = null;
    let tabs = [];
    let callbacks = {};
    const POLL_MS = 5000;

    const strip = document.getElementById('vm-tabstrip');
    const list = document.getElementById('vm-tabstrip-list');
    const hint = document.getElementById('vm-tabstrip-hint');

    function authHeaders() {
        const token = (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) ||
            localStorage.getItem('gb-access-token') || sessionStorage.getItem('gb-access-token') || '';
        return token ? { 'Authorization': 'Bearer ' + token } : {};
    }

    function queryParam(name) {
        try { return new URLSearchParams(location.search).get(name) || ''; } catch (e) { return ''; }
    }

    function resolveVm() {
        const params = window.__gbAppParams__ || {};
        const explicit = params.vm || queryParam('vm');
        if (explicit) return Promise.resolve(String(explicit));
        return fetch('/api/agent/vm/status', { headers: authHeaders() })
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (data) {
                return data && data.has_vm && data.vm_name ? String(data.vm_name) : 'default';
            })
            .catch(function () { return 'default'; });
    }

    function resolveConversation() {
        const params = window.__gbAppParams__ || {};
        conversationId = params.session_id || params.conversation_id ||
            queryParam('session_id') || queryParam('conversation_id') ||
            (window.ChatState && window.ChatState.currentSessionId) || null;
    }

    function path(suffix) {
        return '/api/browser/vm/' + encodeURIComponent(vmName) + '/tabs' + (suffix || '');
    }

    function refresh() {
        return fetch(path(), { headers: authHeaders() })
            .then(function (r) {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.json();
            })
            .then(function (data) {
                tabs = data.tabs || [];
                render();
            })
            .catch(function () {
                if (strip) strip.style.display = 'none';
            });
    }

    function render() {
        if (!strip || !list) return;
        // Keep the strip visible even when empty so the VM surface is
        // discoverable and the user can open the first tab.
        strip.style.display = 'flex';
        list.textContent = '';
        const activeId = callbacks.getSessionId ? callbacks.getSessionId() : null;
        tabs.forEach(function (tab) {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'vm-tab' + (tab.id === activeId ? ' active' : '');
            btn.setAttribute('data-tab-id', tab.id);
            if (tab.vm_controlled) btn.setAttribute('data-vm-controlled', 'true');
            btn.title = (tab.title || tab.url || 'VM tab') + (tab.vm_controlled ? ' — VM-controlled' : '');

            if (tab.vm_controlled) {
                const badge = document.createElement('span');
                badge.className = 'vm-badge';
                badge.textContent = 'VM';
                btn.appendChild(badge);
            }

            const title = document.createElement('span');
            title.className = 'vm-tab-title';
            title.textContent = tab.title || tab.url || 'New Tab';
            btn.appendChild(title);

            if (tab.conversation_id && conversationId && tab.conversation_id !== conversationId) {
                const shared = document.createElement('span');
                shared.className = 'vm-tab-shared';
                shared.textContent = 'shared';
                btn.appendChild(shared);
            }

            btn.onclick = function () { attach(tab); };

            const close = document.createElement('span');
            close.className = 'vm-tab-close';
            close.textContent = '\u00d7';
            close.title = 'Close VM tab';
            close.onclick = function (ev) {
                ev.stopPropagation();
                closeTab(tab.id);
            };
            btn.appendChild(close);
            list.appendChild(btn);
        });
        if (hint) {
            hint.textContent = vmName === 'default' ? 'Shared across conversations' : 'VM ' + vmName;
        }
    }

    function attach(tab) {
        fetch(path('/' + encodeURIComponent(tab.id) + '/attach'), {
            method: 'POST',
            headers: Object.assign({ 'Content-Type': 'application/json' }, authHeaders()),
            body: JSON.stringify({ conversation_id: conversationId })
        }).catch(function () { /* the stream works without an attach record */ })
            .then(function () {
                if (callbacks.onAttach) callbacks.onAttach(tab.id, tab.title);
            });
    }

    function open() {
        return fetch(path(), {
            method: 'POST',
            headers: Object.assign({ 'Content-Type': 'application/json' }, authHeaders()),
            body: JSON.stringify({
                headless: true,
                vm: vmName,
                conversation_id: conversationId,
                vm_controlled: true,
                title: 'New Tab'
            })
        }).then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        }).then(function (data) {
            return refresh().then(function () {
                if (data && data.tab) attach(data.tab);
                return data;
            });
        });
    }

    function closeTab(id) {
        fetch(path('/' + encodeURIComponent(id)), { method: 'DELETE', headers: authHeaders() })
            .catch(function () { return null; })
            .then(function () {
                if (callbacks.onClosed) callbacks.onClosed(id);
                return refresh();
            });
    }

    function init(opts) {
        callbacks = opts || {};
        resolveConversation();
        resolveVm().then(function (name) {
            vmName = name;
            render();
            refresh();
            setInterval(refresh, POLL_MS);
        });
    }

    window.GBDesktopVMTabs = {
        init: init,
        refresh: refresh,
        open: open,
        close: closeTab,
        attach: attach,
        path: path,
        vm: function () { return vmName; },
        conversation: function () { return conversationId; }
    };
})();
