/* HR app — module 1/6: API base and shared helpers (split from hr/hr.js for #1731) */

(function (app) {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("hr");
const API_BASE = '/api/hr';

async function fetchApi(url, options) {
const res = await fetch(API_BASE + url, options);
if (res.status === 401) {
let tok = null;
try {
tok = (window.getGBAccessToken ? window.getGBAccessToken() : null) ||
localStorage.getItem('gb-access-token') || localStorage.getItem('management_token');
} catch (e) { tok = null; }
if (!tok) {
const loginUrl = window.GB_LOGIN_URL || '/login';
// Full URL — a relative redirect would resolve against the login host
// after auth and bounce back to login (login loop).
window.location.href = loginUrl + '?redirect=' + encodeURIComponent(window.location.href);
throw new Error('Not authenticated');
}
}
if (!res.ok) throw new Error('API error: ' + res.status);
return res.json();
}

function nameParts(name) {
const parts = (name || '').trim().split(/\s+/);
return { first: parts[0] || '', last: parts.slice(1).join(' ') };
}

function showFeedback(msg, type) {
  let el = document.getElementById('hr-feedback');
  if (!el) { el = document.createElement('div'); el.id = 'hr-feedback'; el.style.cssText = 'position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;color:#fff;transition:opacity .3s'; document.body.appendChild(el); }
  el.textContent = msg;
  el.style.background = type === 'error' ? '#ef4444' : '#22c55e';
  el.style.opacity = '1';
  setTimeout(() => { el.style.opacity = '0'; }, 3000);
}

function openModal(id) {
document.getElementById(id).classList.add('open');
}

function closeModal(id) {
document.getElementById(id).classList.remove('open');
}

function esc(str) {
if (str === null || str === undefined) return '';
const div = document.createElement('div');
div.textContent = String(str);
return div.innerHTML;
}

Object.assign(app, { fetchApi, nameParts, showFeedback, openModal, closeModal, esc });
})(window.__gbHr = window.__gbHr || {});
