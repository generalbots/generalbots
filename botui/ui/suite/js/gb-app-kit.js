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

  async function apiCall(appId, url, options) {
    try {
      var r = await fetch(url, options);
      if (!r.ok) throw new Error("HTTP " + r.status);
      var t = await r.text();
      return t ? JSON.parse(t) : null;
    } catch (e) {
      showFeedback(appId, "API error: " + e.message, "error");
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

  global.GBAppKit = {
    escapeHtml: escapeHtml,
    showFeedback: showFeedback,
    apiCall: apiCall,
    mountSignaturePad: mountSignaturePad,
    clearSignatureCanvas: clearSignatureCanvas
  };
})(window);
