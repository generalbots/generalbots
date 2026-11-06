/* Terminal app — module 1/2: VM picker and shared helpers (split from terminal/terminal.js for #1731) */

if (window.GBAppLifecycle) GBAppLifecycle.begin("terminal");

(function (app) {
    'use strict';

    function getWsUrl(sessionId) {
        const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
        return proto + '//' + location.host + '/api/terminal/ws?id=' + encodeURIComponent(sessionId);
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

    function loadVmOptions() {
        const projectId = projectFromContext();
        if (!projectId) {
            // Standalone: list VMs across all of the user's projects.
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
                return Promise.all(jobs);
            }).then(function (groups) {
                const flat = [];
                groups.forEach(function (g) { if (Array.isArray(g)) flat.push.apply(flat, g); });
                const list = flat.filter(function (o) { return o.container; });
                // Prefer a running VM (label carries the status), else the first.
                const running = list.find(function (o) { return /run/i.test(o.label || ''); });
                const pref = running || list[0] || null;
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
        isReady: function () { return vmOptionsLoaded; }
    };

    if (vmSelect) {
        vmSelect.addEventListener('change', function () { window.GBTerminal.onVmChange(); });
    }
    loadVmOptions();
})(window.GBTerminal = window.GBTerminal || {});
