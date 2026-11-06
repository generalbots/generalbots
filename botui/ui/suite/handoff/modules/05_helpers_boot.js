/* Handoff app — module 5/5: helpers and boot (split from handoff/handoff.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    refresh() { this.loadAll(); },

    fmtTime(s) {
        if (!s && s !== 0) return '-';
        s = Math.round(s);
        if (s < 60) return s + 's';
        return Math.floor(s / 60) + 'm ' + (s % 60) + 's';
    },
    esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; },
    showFeedback(msg, type) {
        let el = document.getElementById('handoff-feedback');
        if (!el) { el = document.createElement('div'); el.id = 'handoff-feedback'; el.style.cssText = 'position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;color:#fff;transition:opacity .3s'; document.body.appendChild(el); }
        el.textContent = msg;
        el.style.background = type === 'error' ? '#ef4444' : '#22c55e';
        el.style.opacity = '1';
        setTimeout(() => { el.style.opacity = '0'; }, 3000);
    }
});

(function(){ var __cb = () => app.init(); if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
})(window.HandoffApp);
