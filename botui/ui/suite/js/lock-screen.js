"use strict";
/* Lock screen (#1158, hardened in #2446): a real session lock, not a cosmetic
 * div. Unlocking calls POST /api/auth/login; logind/resume events re-lock. */
(function () {
  if (window.LockScreen) return;

  var STATE_KEY = "gb-lock-state", IDLE_KEY = "gb-lock-idle-ms";
  var ATTEMPT_KEY = "gb-lock-attempts", AUDIT_KEY = "gb-lock-audit";
  var USER_KEY = "gb-user-data", TOKEN_KEY = "gb-access-token", LOGIN_URL = "/api/auth/login";
  var DEFAULT_IDLE_MS = 60 * 60 * 1000, MIN_IDLE_MS = 30 * 1000;
  var MAX_IDLE_MS = 24 * 60 * 60 * 1000, MAX_ATTEMPTS = 5;
  var COOLDOWN_MS = 30 * 1000, RESUME_THRESHOLD_MS = 1500, AUDIT_LIMIT = 50;

  var initialized = false, idleTimer = null, clockTimer = null, observer = null;
  var hiddenAt = 0, pendingUnlock = false, hiddenState = [];

  function logErr(error, context) {
    if (window.gbLogError) { try { window.gbLogError(error, context); return; } catch { /* console below */ } }
    if (window.console && console.error) console.error(context, error);
  }
  function t(key, fallback) {
    try { return window.gbT ? window.gbT(key, fallback) : fallback; }
    catch (error) { logErr(error, "lock:i18n"); return fallback; }
  }
  function esc(value) {
    return String(value == null ? "" : value).replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }
  function area(name) {
    try { return name === "session" ? window.sessionStorage : window.localStorage; }
    catch (error) { logErr(error, "lock:storage"); return null; }
  }
  function get(key, name) {
    var storage = area(name);
    if (!storage) return null;
    try { return storage.getItem(key); } catch (error) { logErr(error, "lock:read"); return null; }
  }
  function put(key, value, name) {
    var storage = area(name);
    if (!storage) return;
    try { storage.setItem(key, value); } catch (error) { logErr(error, "lock:write"); }
  }
  function del(key, name) {
    var storage = area(name);
    if (!storage) return;
    try { storage.removeItem(key); } catch (error) { logErr(error, "lock:remove"); }
  }
  function jsonGet(key, fallback, name) {
    var raw = get(key, name);
    if (!raw) return fallback;
    try { return JSON.parse(raw); } catch { return fallback; }
  }
  function jsonPut(key, value, name) { put(key, JSON.stringify(value), name); }

  function identityEmail() {
    var raw = get(USER_KEY) || get("gb-user") || "";
    try { var data = JSON.parse(raw); return typeof data.email === "string" ? data.email : ""; }
    catch { return ""; }
  }
  function getState() { return get(STATE_KEY, "session"); }
  function setState(value) { put(STATE_KEY, value, "session"); }
  function isLocked() { return document.getElementById("gb-lock-screen") !== null; }

  function audit(event) {
    var list = jsonGet(AUDIT_KEY, []);
    if (!Array.isArray(list)) list = [];
    list.push({ at: Date.now(), event: String(event) });
    if (list.length > AUDIT_LIMIT) list = list.slice(-AUDIT_LIMIT);
    jsonPut(AUDIT_KEY, list);
    if (window.console && console.info) console.info("[lock] " + String(event));
  }
  function getIdleMs() {
    var value = Number(get(IDLE_KEY));
    if (!isFinite(value) || value < MIN_IDLE_MS) return DEFAULT_IDLE_MS;
    return Math.min(value, MAX_IDLE_MS);
  }
  function setIdleMs(ms) {
    var value = Number(ms);
    if (!isFinite(value) || value < MIN_IDLE_MS) value = DEFAULT_IDLE_MS;
    put(IDLE_KEY, String(Math.min(value, MAX_IDLE_MS))); armIdle();
  }
  function reasonText(reason) {
    if (reason === "idle") return t("lock-reason-idle", "Locked after inactivity");
    if (reason === "resume" || reason === "gb-resume" || reason === "gb-lid-open" ||
        reason === "gb-session-switch" || reason === "org.freedesktop.login1.Lock")
      return t("lock-reason-resume", "Locked after resume");
    if (reason === "restore" || reason === "cross-tab") return t("lock-reason-restore", "Session locked");
    return t("lock-reason-manual", "Screen locked");
  }

  function ensureStyles() {
    if (document.getElementById("gb-lock-style")) return;
    var style = document.createElement("style");
    style.id = "gb-lock-style";
    style.textContent = [
      "body.gb-session-locked > *:not(#gb-lock-screen){visibility:hidden!important;pointer-events:none!important;user-select:none!important}",
      ".gb-lock-screen{cursor:default!important}.gb-lock-widget{max-width:380px;width:calc(100% - 40px)}",
      ".gb-lock-reason{font-size:var(--text-xs,12px);color:#94a3b8;margin-top:10px}", ".gb-lock-form{margin-top:28px;display:flex;flex-direction:column;gap:12px;text-align:left}",
      ".gb-lock-field{display:block}.gb-lock-label{display:block;font-size:12px;color:#cbd5e1;margin-bottom:5px}",
      ".gb-lock-field input{width:100%;box-sizing:border-box;padding:10px 12px;border-radius:8px;border:1px solid #475569;background:#0b1220;color:#e6e8ee;font-size:14px}",
      ".gb-lock-field input:focus{outline:2px solid #6366f1;border-color:#6366f1}", ".gb-lock-actions{display:flex;gap:10px;margin-top:6px}",
      ".gb-lock-actions button{flex:1;padding:10px 12px;border-radius:8px;border:1px solid #475569;background:#1e293b;color:#e6e8ee;font-size:14px;cursor:pointer}",
      ".gb-lock-actions button[type=submit]{background:#4f46e5;border-color:#4f46e5}",
      ".gb-lock-actions button:hover{filter:brightness(1.12)}.gb-lock-actions button:disabled{opacity:.6;cursor:wait}",
      ".gb-lock-status{min-height:1.3em;margin:6px 0 0;font-size:12px;color:#fca5a5}", '.gb-lock-status[data-level="info"]{color:#cbd5e1}.gb-lock-error-input{border-color:#ef4444!important}'
    ].join("");
    (document.head || document.body).appendChild(style);
  }
  function remember(child) {
    hiddenState.push({ el: child, inert: child.hasAttribute("inert"), aria: child.getAttribute("aria-hidden") });
    child.setAttribute("inert", ""); child.setAttribute("aria-hidden", "true");
  }
  function setContentHidden(hidden) {
    var body = document.body;
    if (!body) return;
    if (hidden) {
      if (body.classList.contains("gb-session-locked")) return;
      body.classList.add("gb-session-locked"); hiddenState = [];
      Array.prototype.forEach.call(body.children, function (child) {
        if (child.id !== "gb-lock-screen") remember(child);
      });
      return;
    }
    body.classList.remove("gb-session-locked");
    hiddenState.forEach(function (item) {
      if (!item.el || !item.el.isConnected) return;
      if (!item.inert) item.el.removeAttribute("inert");
      if (item.aria == null) item.el.removeAttribute("aria-hidden"); else item.el.setAttribute("aria-hidden", item.aria);
    });
    hiddenState = [];
  }
  function hideNewChildren() {
    if (!isLocked() || !document.body) return;
    Array.prototype.forEach.call(document.body.children, function (child) {
      if (child.id === "gb-lock-screen" || child.id === "gb-lock-style") return;
      if (!child.hasAttribute("inert")) remember(child);
      else if (child.getAttribute("aria-hidden") !== "true") child.setAttribute("aria-hidden", "true");
    });
  }
  function stopWatch() { if (observer) { observer.disconnect(); observer = null; } }
  function watchTamper() {
    stopWatch();
    if (typeof MutationObserver !== "function") return;
    observer = new MutationObserver(function () {
      if (!isLocked()) return;
      var overlay = document.getElementById("gb-lock-screen");
      if (!overlay || !document.body.classList.contains("gb-session-locked")) {
        // Deleting the node or marker class cannot dismiss a session lock.
        audit("tamper:rebuild"); if (overlay) overlay.remove(); lock("restore"); return;
      }
      hideNewChildren();
    });
    observer.observe(document.body, { childList: true, attributes: true, attributeFilter: ["id", "class"] });
  }
  function focusPassword() {
    var field = document.getElementById("gbLockPassword");
    if (field) { try { field.focus(); } catch { /* hidden webview */ } }
  }
  function tick() {
    var timeEl = document.getElementById("gbLockTime"), dateEl = document.getElementById("gbLockDate");
    if (!timeEl && !dateEl) return;
    var now = new Date();
    if (timeEl) timeEl.textContent = window.gbFormatTime ? gbFormatTime(now, { hour: "2-digit", minute: "2-digit" })
      : [String(now.getHours()).padStart(2, "0"), String(now.getMinutes()).padStart(2, "0")].join(":");
    if (dateEl) dateEl.textContent = window.gbFormatDate ? gbFormatDate(now, { weekday: "long", month: "long", day: "numeric" }) : now.toDateString();
  }
  function setStatus(element, message, level) {
    if (!element) return; element.textContent = message; if (level) element.setAttribute("data-level", level);
  }
  function trapFocus(event, overlay) {
    var items = Array.prototype.slice.call(overlay.querySelectorAll("input, button, [href]"))
      .filter(function (element) { return !element.disabled; });
    if (!items.length) return; var first = items[0], last = items[items.length - 1];
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }
  function blockWhileLocked(event) {
    if (!isLocked()) return;
    if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); return; }
    var overlay = document.getElementById("gb-lock-screen");
    if (overlay && overlay.contains(event.target)) return; event.preventDefault(); event.stopPropagation();
  }
  function onBeforeUnload(event) {
    if (!isLocked()) return; event.preventDefault(); event.returnValue = "";
  }
  function buildOverlay(reason) {
    var overlay = document.createElement("div");
    overlay.id = "gb-lock-screen"; overlay.className = "gb-lock-screen";
    overlay.setAttribute("role", "dialog"); overlay.setAttribute("aria-modal", "true");
    overlay.setAttribute("aria-label", t("taskbar-lock", "Lock screen"));
    overlay.innerHTML =
      '<div class="gb-lock-widget"><div class="gb-lock-time" id="gbLockTime">--:--</div>' + '<div class="gb-lock-date" id="gbLockDate"></div>' +
      '<div class="gb-lock-reason" id="gbLockReason">' + esc(reasonText(reason)) + '</div>' + '<form class="gb-lock-form" id="gbLockForm" autocomplete="off">' +
      '<label class="gb-lock-field"><span class="gb-lock-label">' + esc(t("auth-login-email", "Email Address")) + '</span>' +
      '<input id="gbLockEmail" type="email" name="email" autocomplete="username" required value="' + esc(identityEmail()) + '" /></label>' +
      '<label class="gb-lock-field"><span class="gb-lock-label">' + esc(t("auth-login-password", "Password")) + '</span>' +
      '<input id="gbLockPassword" type="password" name="password" autocomplete="current-password" required /></label>' +
      '<div class="gb-lock-actions"><button type="submit" id="gbLockSubmit">' + esc(t("auth-login-submit", "Sign In")) + '</button>' +
      '<button type="button" id="gbLockSignout">' + esc(t("user-menu-sign-out", "Sign out")) + '</button></div>' +
      '<p class="gb-lock-status" id="gbLockStatus" role="status" aria-live="polite"></p>' + '</form></div>';
    return overlay;
  }
  function wireOverlay(overlay) {
    var form = document.getElementById("gbLockForm"), signout = document.getElementById("gbLockSignout");
    // The backdrop is never an unlock gesture (#2446).
    overlay.addEventListener("click", function (event) { if (event.target === overlay) focusPassword(); });
    overlay.addEventListener("keydown", function (event) {
      if (event.key === "Tab") trapFocus(event, overlay);
      if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); }
    });
    if (form) form.addEventListener("submit", function (event) { event.preventDefault(); attemptUnlock(); });
    if (signout) signout.addEventListener("click", signOut);
  }
  function attemptsState() {
    var value = jsonGet(ATTEMPT_KEY, null, "session");
    if (!value || typeof value !== "object" || typeof value.count !== "number" || typeof value.until !== "number") return { count: 0, until: 0 };
    if (value.until && Date.now() >= value.until) return { count: 0, until: 0 };
    return value;
  }
  function secondsLeft(value) { return Math.max(0, Math.ceil((value.until - Date.now()) / 1000)); }

  async function attemptUnlock() {
    if (pendingUnlock) return; var status = document.getElementById("gbLockStatus"), emailEl = document.getElementById("gbLockEmail");
    var passwordEl = document.getElementById("gbLockPassword"), submit = document.getElementById("gbLockSubmit");
    if (!emailEl || !passwordEl || !status || !isLocked()) return;
    var attempts = attemptsState();
    if (attempts.until && Date.now() < attempts.until) {
      setStatus(status, t("auth-error-too-many-attempts", "Too many failed attempts. Please try again later.") + " (" + secondsLeft(attempts) + "s)", "error"); return;
    }
    var email = emailEl.value.trim(), password = passwordEl.value;
    if (!email || !password) {
      setStatus(status, t("auth-login-subtitle", "Enter your credentials to continue"), "info"); return;
    }
    pendingUnlock = true; if (submit) submit.disabled = true;
    setStatus(status, t("auth-login-loading", "Signing in..."), "info");
    try {
      var response = await fetch(LOGIN_URL, {
        method: "POST", headers: { "Content-Type": "application/json" },
        credentials: "same-origin", body: JSON.stringify({ email: email, password: password })
      });
      var body = {}; try { body = await response.json(); } catch { /* non-JSON error body */ }
      if (response.ok && body && body.access_token) {
        // Only a verified credential from the login endpoint reaches here.
        jsonPut(ATTEMPT_KEY, { count: 0, until: 0 }, "session"); var local = area("local"), session = area("session");
        try {
          if (local) local.setItem(TOKEN_KEY, body.access_token);
          if (session) session.setItem(TOKEN_KEY, body.access_token);
          // Collaboration WebSockets need the JWT, not the opaque REST token.
          var wsToken = body.session_token || body.token || "";
          if (wsToken.indexOf("eyJ") === 0) {
            if (local) local.setItem("gb-ws-token", wsToken);
            if (session) session.setItem("gb-ws-token", wsToken);
          }
        } catch (error) { logErr(error, "lock:token"); }
        audit("unlock:ok"); unlock(); return;
      }
      var count = (attempts.count || 0) + 1, next = { count: count, until: 0 };
      if (response.status === 429) {
        var wait = Number(body.retry_after || body.wait || 30);
        if (!isFinite(wait) || wait < 1) wait = 30;
        next.until = Date.now() + Math.min(wait, 300) * 1000; next.count = MAX_ATTEMPTS;
      } else if (count >= MAX_ATTEMPTS) { next.until = Date.now() + COOLDOWN_MS; }
      jsonPut(ATTEMPT_KEY, next, "session"); audit("unlock:fail:" + response.status);
      passwordEl.value = ""; passwordEl.classList.add("gb-lock-error-input");
      var message;
      if (next.until) message = t("auth-error-too-many-attempts", "Too many failed attempts. Please try again later.");
      else if (response.status === 401 || response.status === 403) message = t("auth-error-invalid-credentials", "Invalid email or password");
      else if (response.status >= 500) message = t("auth-error-server", "Server error. Please try again later.");
      else message = t("auth-error-unknown", "An unknown error occurred");
      var left = MAX_ATTEMPTS - next.count;
      if (left > 0) message += " (" + t("lock-attempts-left", "Attempts remaining:") + " " + left + ")";
      setStatus(status, message, "error"); passwordEl.focus();
    } catch (error) {
      logErr(error, "lock:verify"); audit("unlock:error"); setStatus(status, t("auth-error-network", "Network error. Please check your connection."), "error");
      passwordEl.value = ""; passwordEl.focus();
    } finally {
      pendingUnlock = false; if (submit) submit.disabled = false;
    }
  }

  async function signOut() {
    audit("signout:from-lock");
    window.removeEventListener("beforeunload", onBeforeUnload);
    del(TOKEN_KEY, "session"); del(TOKEN_KEY); setState("unlocked");
    try { await fetch("/api/auth/logout", { method: "POST", credentials: "same-origin" }); }
    catch (error) { logErr(error, "lock:logout"); }
    window.location.href = window.GB_LOGIN_URL || "/login";
  }
  function unlock() {
    var overlay = document.getElementById("gb-lock-screen");
    if (overlay) overlay.remove();
    var style = document.getElementById("gb-lock-style"); if (style) style.remove();
    setContentHidden(false);
    document.removeEventListener("keydown", blockWhileLocked, true);
    window.removeEventListener("beforeunload", onBeforeUnload);
    stopWatch(); if (clockTimer) { clearInterval(clockTimer); clockTimer = null; }
    setState("unlocked"); hiddenAt = 0; armIdle();
  }
  function lock(reason) {
    if (isLocked()) return;
    setState("locked"); audit("lock:" + String(reason || "manual"));
    ensureStyles(); setContentHidden(true);
    var overlay = buildOverlay(reason);
    document.body.appendChild(overlay); wireOverlay(overlay); tick();
    clockTimer = setInterval(tick, 1000);
    document.addEventListener("keydown", blockWhileLocked, true);
    window.addEventListener("beforeunload", onBeforeUnload);
    watchTamper(); armIdle(); focusPassword();
  }
  function armIdle() {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = setTimeout(function () { if (!isLocked()) lock("idle"); }, getIdleMs());
  }
  function resetIdle() { if (!isLocked()) armIdle(); }
  function onVisibility() {
    if (isLocked()) { hiddenAt = 0; return; }
    if (document.hidden) { hiddenAt = Date.now(); return; }
    var resume = hiddenAt && Date.now() - hiddenAt >= RESUME_THRESHOLD_MS;
    hiddenAt = 0; if (resume) lock("resume");
  }
  function onStorage(event) {
    if (event && event.key === STATE_KEY && event.newValue === "locked" && !isLocked()) lock("cross-tab");
  }
  function listenTauri() {
    var tauri = window.GBTAURI;
    if (!tauri || !tauri.event || typeof tauri.event.listen !== "function") return;
    ["gb-lock", "gb-lock-screen", "gb-suspend", "gb-resume", "gb-lid-open", "gb-session-switch", "org.freedesktop.login1.Lock"].forEach(function (name) {
      tauri.event.listen(name, function () { lock(name); }).catch(function (error) { logErr(error, "lock:listen"); });
    });
    ["gb-unlock", "org.freedesktop.login1.Unlock"].forEach(function (name) {
      // logind Unlock means "session active again", not "user authenticated".
      tauri.event.listen(name, function () { hiddenAt = 0; audit("logind:unlock-signal"); }).catch(function (error) { logErr(error, "lock:listen"); });
    });
  }
  function init() {
    if (initialized) return;
    initialized = true; listenTauri();
    document.addEventListener("visibilitychange", onVisibility); window.addEventListener("storage", onStorage);
    window.addEventListener("beforeunload", onBeforeUnload);
    ["mousemove", "keydown", "mousedown", "touchstart", "scroll"].forEach(function (name) {
      document.addEventListener(name, resetIdle, { passive: true });
    });
    armIdle();
    // A locked state survives a webview reload: re-arm before content is read.
    if (getState() === "locked") lock("restore");
  }
  window.LockScreen = { init: init, lock: lock, unlockWithCredential: attemptUnlock,
    isLocked: isLocked, getIdleMs: getIdleMs, setIdleMs: setIdleMs, audit: audit };
})();
