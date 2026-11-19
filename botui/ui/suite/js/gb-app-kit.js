"use strict";
/* Shared app skeleton helpers for suite mini-apps (o365/kyc/retail/biometry).
 * Extracted to remove duplicated inline helpers (#2521). Behavior-preserving. */
(function (global) {
  if (global.GBAppKit) return;

  function escapeHtml(value) {
    if (global.GBShared && global.GBShared.html && typeof global.GBShared.html.escape === "function") {
      return global.GBShared.html.escape(value);
    }
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function t(key, fallback) {
    if (typeof global.gbT === "function") return global.gbT(key, fallback);
    return fallback;
  }

  function showFeedback(appId, msg, type) {
    var id = String(appId || "gb-app") + "-feedback";
    var el = document.getElementById(id);
    if (!el) {
      el = document.createElement("div");
      el.id = id;
      el.style.cssText = "position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s";
      document.body.appendChild(el);
    }
    el.textContent = msg;
    el.style.background = type === "error" ? "#ef4444" : "#22c55e";
    el.style.color = "#fff";
    el.style.opacity = "1";
    setTimeout(function () { el.style.opacity = "0"; }, 3000);
  }

  function getAuthToken() {
    try {
      if (global.GBSecurity && typeof global.GBSecurity.getToken === "function") {
        var t = global.GBSecurity.getToken();
        if (t) return t;
      }
      if (typeof global.getGBAccessToken === "function") {
        var t2 = global.getGBAccessToken();
        if (t2) return t2;
      }
    } catch (e) {}
    var keys = ["gb-access-token", "management_token", "token", "id_token", "gb_token"];
    for (var i = 0; i < keys.length; i++) {
      try {
        var v = localStorage.getItem(keys[i]) || sessionStorage.getItem(keys[i]);
        if (v) return v;
      } catch (e2) {}
    }
    return null;
  }

  // Raw-response sibling of apiCall: same auth header, no JSON parsing, so a
  // blob download or a binary response never needs its own Bearer logic.
  async function apiRaw(url, options) {
    var opts = options ? Object.assign({}, options) : {};
    var headers = Object.assign({}, (options && options.headers) || {});
    var hasAuth = Object.keys(headers).some(function (k) {
      return String(k).toLowerCase() === "authorization";
    });
    if (!hasAuth) {
      var token = getAuthToken();
      if (token) headers.Authorization = "Bearer " + token;
    }
    opts.headers = headers;
    var r = await fetch(url, opts);
    if (!r.ok) throw new Error("HTTP " + r.status);
    return r;
  }

  async function download(appId, url, fileName) {
    try {
      var r = await apiRaw(url);
      var blob = await r.blob();
      var href = URL.createObjectURL(blob);
      var a = document.createElement("a");
      a.href = href;
      a.download = fileName || "download";
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(function () { URL.revokeObjectURL(href); }, 2000);
      showFeedback(appId, t("app-kit-download-complete", "Download complete"), "success");
    } catch (e) {
      showFeedback(appId, t("app-kit-download-failed", "Download failed") + ": " + e.message, "error");
    }
  }

  async function apiCall(appId, url, options) {
    try {
      var opts = options ? Object.assign({}, options) : {};
      var headers = Object.assign({}, (options && options.headers) || {});
      var hasAuth = false;
      Object.keys(headers).forEach(function (k) {
        if (String(k).toLowerCase() === "authorization") hasAuth = true;
      });
      if (!hasAuth) {
        var token = getAuthToken();
        if (token) headers.Authorization = "Bearer " + token;
      }
      opts.headers = headers;
      var r = await fetch(url, opts);
      if (!r.ok) throw new Error("HTTP " + r.status);
      var t = await r.text();
      return t ? JSON.parse(t) : null;
    } catch (e) {
      showFeedback(appId, t("app-kit-api-error", "API error") + ": " + e.message, "error");
      return null;
    }
  }

  function mountSignaturePad(canvas, state, opts) {
    if (!canvas) return;
    opts = opts || {};
    var bg = opts.bg || "#0f172a";
    var stroke = opts.stroke || "#f8fafc";
    var lineWidth = opts.lineWidth || 2;
    var parent = canvas.parentElement;
    if (parent) {
      var w = Math.min(parent.clientWidth || 500, 800);
      canvas.width = Math.max(300, w);
      canvas.height = 200;
    }
    var ctx = canvas.getContext("2d");
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = stroke;
    ctx.lineWidth = lineWidth;
    ctx.lineCap = "round";
    if (state) state.hasDrawn = false;
    else canvas.hasDrawn = false;

    function setDrawn() {
      if (state) state.hasDrawn = true;
      else canvas.hasDrawn = true;
    }
    function getPos(e) {
      var rect = canvas.getBoundingClientRect();
      var clientX, clientY;
      if (e.touches && e.touches.length) { clientX = e.touches[0].clientX; clientY = e.touches[0].clientY; }
      else if (e.changedTouches && e.changedTouches.length) { clientX = e.changedTouches[0].clientX; clientY = e.changedTouches[0].clientY; }
      else { clientX = e.clientX; clientY = e.clientY; }
      return { x: clientX - rect.left, y: clientY - rect.top };
    }
    var drawing = false, lastX = 0, lastY = 0;
    function setDrawing(v) {
      drawing = v;
      if (state && "drawing" in state) state.drawing = v;
    }
    function getDrawing() {
      return state && "drawing" in state ? !!state.drawing : drawing;
    }
    function setLast(x, y) {
      lastX = x; lastY = y;
      if (state) { if ("lastX" in state) state.lastX = x; if ("lastY" in state) state.lastY = y; }
    }
    function getLast() {
      if (state && "lastX" in state && "lastY" in state) return { x: state.lastX, y: state.lastY };
      return { x: lastX, y: lastY };
    }

    canvas.onmousedown = function (e) { setDrawing(true); var p = getPos(e); setLast(p.x, p.y); setDrawn(); e.preventDefault(); };
    canvas.onmousemove = function (e) { if (!getDrawing()) return; var p = getPos(e); var lp = getLast(); ctx.beginPath(); ctx.moveTo(lp.x, lp.y); ctx.lineTo(p.x, p.y); ctx.stroke(); setLast(p.x, p.y); e.preventDefault(); };
    canvas.onmouseup = function () { setDrawing(false); };
    canvas.onmouseleave = function () { setDrawing(false); };
    canvas.ontouchstart = function (e) { setDrawing(true); var p = getPos(e); setLast(p.x, p.y); setDrawn(); e.preventDefault(); };
    canvas.ontouchmove = function (e) { if (!getDrawing()) return; var p = getPos(e); var lp = getLast(); ctx.beginPath(); ctx.moveTo(lp.x, lp.y); ctx.lineTo(p.x, p.y); ctx.stroke(); setLast(p.x, p.y); e.preventDefault(); };
    canvas.ontouchend = function (e) { setDrawing(false); e.preventDefault(); };
    canvas.ontouchcancel = function () { setDrawing(false); };
  }

  function clearSignatureCanvas(canvas, state, opts) {
    if (!canvas) return;
    opts = opts || {};
    var bg = opts.bg || "#0f172a";
    var ctx = canvas.getContext("2d");
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    if (state) state.hasDrawn = false;
    else canvas.hasDrawn = false;
  }

  function showModal(id, opts) {
    var el = typeof id === "string" ? document.getElementById(id) : id;
    if (!el) return;
    opts = opts || {};
    if (opts.className) {
      el.classList.add(opts.className);
    } else {
      el.style.display = opts.display || "flex";
    }
  }

  function hideModal(id, opts) {
    var el = typeof id === "string" ? document.getElementById(id) : id;
    if (!el) return;
    opts = opts || {};
    if (opts.className) {
      el.classList.remove(opts.className);
    } else {
      el.style.display = opts.hideDisplay || "none";
    }
  }

  function switchTab(tab, btn, opts) {
    opts = opts || {};
    var tabSelector = opts.tabSelector || '[role="tab"]';
    var panelSelector = opts.panelSelector || '[role="tabpanel"]';
    var activeClass = opts.activeClass || "active";
    var panelId = typeof opts.panelId === "function"
      ? opts.panelId(tab)
      : (opts.panelPrefix || "panel-") + tab;

    var tabs = Array.prototype.slice.call(document.querySelectorAll(tabSelector));
    tabs.forEach(function (t) {
      var active = t === btn;
      t.classList.toggle(activeClass, active);
      t.setAttribute("aria-selected", active ? "true" : "false");
      if ("tabIndex" in t) t.tabIndex = active ? 0 : -1;
    });

    document.querySelectorAll(panelSelector).forEach(function (p) {
      p.classList.remove(activeClass);
      if ("hidden" in p) p.hidden = true;
    });

    var panel = document.getElementById(panelId);
    if (panel) {
      panel.classList.add(activeClass);
      if ("hidden" in panel) panel.hidden = false;
      if (btn && btn.id) panel.setAttribute("aria-labelledby", btn.id);
    }
    return panel || null;
  }

  function setStatValues(map) {
    if (!map) return;
    Object.keys(map).forEach(function (id) {
      var el = document.getElementById(id);
      if (el) el.textContent = String(map[id] == null ? "" : map[id]);
    });
  }

  global.GBAppKit = {
    escapeHtml: escapeHtml,
    showFeedback: showFeedback,
    getAuthToken: getAuthToken,
    apiRaw: apiRaw,
    download: download,
    apiCall: apiCall,
    mountSignaturePad: mountSignaturePad,
    clearSignatureCanvas: clearSignatureCanvas,
    showModal: showModal,
    hideModal: hideModal,
    switchTab: switchTab,
    setStatValues: setStatValues
  };
})(window);
