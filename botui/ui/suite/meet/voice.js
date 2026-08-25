/* Voice + Telephony Console (#1184) — LiveKit voice sessions and trunk provisioning. */
(function () {
    'use strict';
    if (window.GBAppLifecycle) GBAppLifecycle.begin('meet');

    let voiceSessionId = null;
    const $ = (id) => document.getElementById(id);

    async function api(path, method, body) {
        const opts = { method: method || 'GET', headers: { 'Content-Type': 'application/json' } };
        if (body) opts.body = JSON.stringify(body);
        const resp = await fetch(path, opts);
        const data = await resp.json().catch(() => ({}));
        return { ok: resp.ok, data };
    }

    function setVoiceStatus(text) {
        $('voiceStatus').textContent = text;
    }

    async function loadTrunks() {
        const { data } = await api('/api/meet/trunks');
        const tbody = $('trunkList');
        tbody.innerHTML = '';
        const trunks = (data && data.trunks) || [];
        if (!trunks.length) {
            tbody.innerHTML = '<tr><td colspan="5" style="color:#888">No trunks provisioned yet.</td></tr>';
            return;
        }
        for (const t of trunks) {
            const tr = document.createElement('tr');
            tr.innerHTML = '<td>' + escapeHtml(t.name) + '</td>' +
                '<td>' + escapeHtml(t.phone_number) + '</td>' +
                '<td>' + escapeHtml(t.carrier) + '</td>' +
                '<td>' + escapeHtml(t.status) + '</td>' +
                '<td><button class="gb-voice-btn danger" data-id="' + t.id + '">Release</button></td>';
            tbody.appendChild(tr);
        }
    }

    function escapeHtml(s) {
        return String(s == null ? '' : s).replace(/[&<>"']/g, function (c) {
            return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c];
        });
    }

    function wire() {
        $('voiceStartBtn').addEventListener('click', async function () {
            const sessionId = $('voiceSessionId').value.trim() || null;
            const body = {};
            if (sessionId) body.session_id = sessionId;
            if (voiceSessionId) body.session_id = voiceSessionId;
            const { ok, data } = await api('/api/voice/start', 'POST', body);
            if (ok && data.token) {
                voiceSessionId = sessionId;
                $('voiceStartBtn').disabled = true;
                $('voiceStopBtn').disabled = false;
                setVoiceStatus('● Live — token issued');
                $('voiceMsg').textContent = 'Voice session started.';
            } else {
                $('voiceMsg').textContent = 'Start failed: ' + ((data && data.error) || 'unknown');
            }
        });

        $('voiceStopBtn').addEventListener('click', async function () {
            const body = {};
            if (voiceSessionId) body.session_id = voiceSessionId;
            const { ok, data } = await api('/api/voice/stop', 'POST', body);
            if (ok) {
                $('voiceStartBtn').disabled = false;
                $('voiceStopBtn').disabled = true;
                voiceSessionId = null;
                setVoiceStatus('Idle');
                $('voiceMsg').textContent = 'Voice session stopped.';
            } else {
                $('voiceMsg').textContent = 'Stop failed: ' + ((data && data.error) || 'unknown');
            }
        });

        $('trunkProvisionBtn').addEventListener('click', async function () {
            const name = $('trunkName').value.trim();
            const phone = $('trunkNumber').value.trim();
            if (!name || !phone) {
                $('trunkMsg').textContent = 'Name and number are required.';
                return;
            }
            const body = { name: name, phone_number: phone };
            const carrier = $('trunkCarrier').value.trim();
            if (carrier) body.carrier = carrier;
            const { ok, data } = await api('/api/meet/trunks', 'POST', body);
            $('trunkMsg').textContent = ok
                ? 'Trunk provisioned: ' + (data.trunk ? data.trunk.phone_number : '')
                : 'Provision failed: ' + ((data && data.error) || 'unknown');
            if (ok) {
                $('trunkName').value = '';
                $('trunkNumber').value = '';
                $('trunkCarrier').value = '';
                loadTrunks();
            }
        });

        $('trunkList').addEventListener('click', async function (ev) {
            const btn = ev.target.closest('button[data-id]');
            if (!btn) return;
            const { ok, data } = await api('/api/meet/trunks/' + btn.dataset.id, 'DELETE');
            $('trunkMsg').textContent = ok
                ? 'Trunk released.'
                : 'Release failed: ' + ((data && data.error) || 'unknown');
            if (ok) loadTrunks();
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () { wire(); loadTrunks(); });
    } else {
        wire();
        loadTrunks();
    }
    if (window.GBAppLifecycle) GBAppLifecycle.end('meet');
})();
