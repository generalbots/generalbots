/* Terminal app — module 1/2: VM picker and shared helpers (split from terminal/terminal.js for #1731) */

if (window.GBAppLifecycle) GBAppLifecycle.begin("terminal");

(function (app) {
    'use strict';

    function t(key, fallback) {
        return window.gbT ? window.gbT(key, fallback) : fallback;
    }

    function getWsUrl(sessionId) {
        const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
        return proto + '//' + location.host + '/api/terminal/ws?id=' + encodeURIComponent(sessionId);
    }

    // #1841 — the backend returns capability categories; turn them into
    // instructions. The raw payload is logged by the caller.
    function errorMessage(data, status) {
        const code = (data && (data.error || data.code)) || '';
        const detail = data && (data.message || data.error);
        if (code === 'agent_vm_unavailable') {
            return t('terminal-error-agent-vm-unavailable', 'Terminal requires the agent VM, which is not available in this build.');
        }
        if (code === 'no_workspace_vm') {
            return t('terminal-error-no-workspace-vm', 'No workspace VM is running. Send a message in chat to start your workspace, then reopen the terminal.');
        }
        if (code === 'vm_not_found' || code === 'vm_stopped') {
            return t('terminal-error-vm-stopped', 'Your workspace VM is not running. Send a message in chat to start it, then reopen the terminal.');
        }
        if (code === 'incus_unavailable') {
            return t('terminal-error-incus-unavailable', 'Terminal needs Incus, but it is not installed or not available on this machine.');
        }
        if (code === 'terminal_forbidden') {
            return t('terminal-error-forbidden', 'The terminal cannot access the workspace VM. Check the Incus permissions.');
        }
        if (code === 'terminal_unavailable') {
            return detail || t('terminal-error-unavailable', 'Terminal is not available in this build.');
        }
        if (code === 'not_found' || status === 404) {
            return t('terminal-error-api-missing', 'Terminal is not available in this build (the terminal API is not compiled).');
        }
        return detail || ('HTTP ' + status);
    }

    // The product endpoint reports whether this binary ships both the PTY
    // routes and the agent VM that hosts the workspace.
    function capabilityMessage() {
        const cfg = window.productConfig;
        if (cfg && cfg.terminal_available === false) {
            return t('terminal-error-desktop-build', 'Terminal requires the agent VM, which is not available in this desktop build.');
        }
        return '';
    }

    /* ---- VM picker: exec into a project VM (Incus container) ---- */
    // Deep-linked from the Vibe ribbon → resolve THAT project's VM.
    // Opened standalone → list every VM across the user's vibe projects.
    const vmSelect = document.getElementById('termVmSelect');
    let vmOptionsLoaded = false;

    function apiGet(path) {
        const token =
            (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) ||
            localStorage.getItem('token') ||
            localStorage.getItem('id_token') ||
            localStorage.getItem('gb-access-token') ||
            sessionStorage.getItem('gb-access-token') ||
            '';
        return fetch(path, { headers: { 'Authorization': 'Bearer ' + token } })
            .then(function (r) { return r.json().catch(function () { return { success: false }; }); });
    }

    function projectFromContext() {
        const params = window.__gbAppParams__ || {};
        const m = window.location.search.match(/[?&]project=([^&]+)/);
        return (params.project || (m && decodeURIComponent(m[1])) || '').toString();
    }

    function preferredVm(vms) {
        // #1288 — only a RUNNING VM may be attached: `incus exec` into a
        // stopped/creating container blocks forever, leaving the terminal
        // stuck at "Connecting...". A non-running VM must be surfaced as
        // "No VM — start it (Play)" instead of being selected.
        if (!Array.isArray(vms) || !vms.length) return null;
        return vms.find(function (v) {
            return String(v.env || '').indexOf('development') !== -1 && String(v.status || '').indexOf('run') !== -1;
        }) || vms.find(function (v) {
            return String(v.status || '').indexOf('run') !== -1;
        }) || null;
    }

    function fillVmSelect(options, preferred) {
        if (!vmSelect) return;
        vmSelect.innerHTML = '';
        // SECURITY: the backend refuses host shells (403) — only project VMs
        // are attachable, so the picker offers nothing else. An empty list
        // disables the selector and the terminal explains how to provision a VM.
        let preselected = false;
        options.forEach(function (o) {
            const opt = document.createElement('option');
            opt.value = o.container || '';
            opt.textContent = o.label;
            if (preferred && o.container === preferred) {
                opt.selected = true;
                preselected = true;
            }
            vmSelect.appendChild(opt);
        });
        vmSelect.disabled = options.length === 0;
        if (options.length && !preselected) {
            vmSelect.selectedIndex = 0;
        }
        vmOptionsLoaded = true;
        // A deep-linked project VM should be used by the (already created)
        // first tab — reconnect it into the container.
        if (preferred && window.GBTerminal) {
            setTimeout(function () {
                if (window.GBTerminal && window.GBTerminal.onVmChange) window.GBTerminal.onVmChange();
            }, 250);
        }
    }

    // #1841 — the chat workspace VM is the terminal's primary target. The
    // project list alone left desktop users (who have no Vibe project) with an
    // empty picker even though their agent VM existed.
    function agentVmOptions() {
        return apiGet('/api/agent/vm/status').then(function (data) {
            if (!data || !data.has_vm || !data.vm_name) return [];
            return [{
                container: data.vm_name,
                label: t('terminal-vm-workspace', 'Workspace VM') + (data.status ? ' (' + data.status + ')' : ''),
            }];
        }).catch(function () { return []; });
    }

    function loadVmOptions() {
        const projectId = projectFromContext();
        if (!projectId) {
            // Standalone: the workspace VM first, then every project VM.
            apiGet('/api/vibe/projects').then(function (data) {
                const projects = (data && (data.projects || (data.success && data.data && data.data.projects))) || [];
                const jobs = projects.slice(0, 10).map(function (p) {
                    const pid = p.project_id || p.id;
                    return apiGet('/api/vibe/projects/' + encodeURIComponent(pid) + '/vms').then(function (vd) {
                        const vms = (vd && (vd.vms || (vd.success && vd.data && vd.data.vms))) || [];
                        return vms.map(function (v) {
                            return {
                                container: v.container_name || '',
                                label: (p.name || pid).substring(0, 24) + ' · ' + (v.env || 'vm') + (v.status ? ' (' + v.status + ')' : ''),
                            };
                        });
                    }).catch(function () { return []; });
                });
                return Promise.all([agentVmOptions()].concat(jobs));
            }).then(function (results) {
                const agent = results[0] || [];
                const groups = results.slice(1);
                const flat = [];
                agent.forEach(function (o) { if (o.container) flat.push(o); });
                groups.forEach(function (g) { if (Array.isArray(g)) flat.push.apply(flat, g); });
                const list = flat.filter(function (o) { return o.container; });
                // Prefer the agent VM (it holds the conversation files), then a
                // running project VM, then the first project VM.
                const running = list.find(function (o) { return /run/i.test(o.label || ''); });
                const pref = agent[0] || running || list[0] || null;
                fillVmSelect(list, pref ? pref.container : null);
            }).catch(function () { vmOptionsLoaded = true; });
        } else {
            // From the Vibe ribbon: resolve that project's VM and preselect it.
            apiGet('/api/vibe/projects/' + encodeURIComponent(projectId) + '/vms').then(function (vd) {
                const vms = (vd && (vd.vms || (vd.success && vd.data && vd.data.vms))) || [];
                const pref = preferredVm(vms);
                fillVmSelect(vms.map(function (v) {
                    return {
                        container: v.container_name || '',
                        label: (v.env || 'vm') + (v.status ? ' (' + v.status + ')' : ''),
                    };
                }).filter(function (o) { return o.container; }), pref && pref.container_name);
            }).catch(function () { vmOptionsLoaded = true; /* picker stays on default */ });
        }
    }

    function getSelectedContainer() {
        if (!vmOptionsLoaded || !vmSelect) return '';
        return (vmSelect.value || '').trim();
    }

    // Shared with module 2 through the same GBTerminal object (internal keys).
    app._vm = {
        getWsUrl: getWsUrl,
        getSelectedContainer: getSelectedContainer,
        isReady: function () { return vmOptionsLoaded; },
        errorMessage: errorMessage,
        capabilityMessage: capabilityMessage
    };

    if (vmSelect) {
        vmSelect.addEventListener('change', function () { window.GBTerminal.onVmChange(); });
    }
    loadVmOptions();
})(window.GBTerminal = window.GBTerminal || {});
