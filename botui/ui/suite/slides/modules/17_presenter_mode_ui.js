"use strict";

/**
 * Module 17: Presenter Mode UI for Slides.
 * Connects the SlidesPresenter engine (module 12) to the actual UI.
 * Renders a presenter panel with: current slide, next-slide preview,
 * presenter notes, elapsed timer, navigation controls. Wires
 * keyboard shortcuts (arrow keys, Esc, B for black screen, W for
 * white screen, P for pen tool, L for laser). Calls the backend
 * /api/slides/presenter/start and /presenter/end.
 *
 * Public API: window.SlidesPresenterUI = { enter, exit, toggle, render,
 *   startSession, endSession, installKeyboard }.
 */

(function () {
  let panel = null;
  let active = false;
  let sessionId = null;
  let startedAt = null;
  let audienceWin = null;
  let timerInterval = null;

  function getState() { return window.state || null; }

  function ensurePanel() {
    if (panel) return panel;
    panel = document.createElement("div");
    panel.id = "slidesPresenterPanel";
    panel.style.cssText = "position:fixed;top:0;right:0;bottom:0;width:360px;background:#1a1a1a;color:#fff;z-index:9998;display:none;flex-direction:column;font-family:Arial,sans-serif;font-size:13px;border-left:1px solid #333;";
    panel.innerHTML = `
      <div style="padding:10px;background:#000;display:flex;align-items:center;gap:6px;">
        <strong>Presenter Mode</strong>
        <span id="spTimer" style="margin-left:auto;font-family:monospace;font-size:14px;">00:00</span>
        <button id="spExit" style="background:#c00;color:#fff;border:0;border-radius:3px;padding:3px 8px;cursor:pointer;">Exit</button>
      </div>
      <div style="padding:10px;background:#222;">
        <div style="margin-bottom:6px;font-size:11px;color:#aaa;">CURRENT SLIDE</div>
        <div id="spCurrent" style="background:#000;aspect-ratio:16/9;border:1px solid #444;display:flex;align-items:center;justify-content:center;color:#666;font-size:11px;">No slide</div>
      </div>
      <div style="padding:10px;background:#222;">
        <div style="margin-bottom:6px;font-size:11px;color:#aaa;">NEXT SLIDE</div>
        <div id="spNext" style="background:#000;aspect-ratio:16/9;border:1px solid #444;display:flex;align-items:center;justify-content:center;color:#666;font-size:11px;">End</div>
      </div>
      <div style="padding:10px;background:#222;flex:1;overflow:auto;">
        <div style="margin-bottom:6px;font-size:11px;color:#aaa;">NOTES</div>
        <div id="spNotes" style="white-space:pre-wrap;font-size:13px;line-height:1.5;color:#ddd;">(no notes)</div>
      </div>
      <div style="padding:10px;background:#000;display:flex;gap:6px;align-items:center;flex-wrap:wrap;">
        <button id="spPrev" style="background:#333;color:#fff;border:0;border-radius:3px;padding:6px 10px;cursor:pointer;">◀ Prev</button>
        <button id="spNextBtn" style="background:#333;color:#fff;border:0;border-radius:3px;padding:6px 10px;cursor:pointer;">Next ▶</button>
        <button id="spBlack" style="background:#000;color:#fff;border:1px solid #444;border-radius:3px;padding:6px 10px;cursor:pointer;">Black (B)</button>
        <button id="spWhite" style="background:#fff;color:#000;border:0;border-radius:3px;padding:6px 10px;cursor:pointer;">White (W)</button>
        <button id="spLaser" style="background:#333;color:#fff;border:0;border-radius:3px;padding:6px 10px;cursor:pointer;">Laser (L)</button>
        <button id="spAudience" style="background:#333;color:#fff;border:0;border-radius:3px;padding:6px 10px;cursor:pointer;">Audience</button>
        <button id="spReset" style="background:#333;color:#fff;border:0;border-radius:3px;padding:6px 10px;cursor:pointer;">Reset Timer</button>
      </div>
    `;
    document.body.appendChild(panel);
    panel.querySelector("#spExit").addEventListener("click", exit);
    panel.querySelector("#spPrev").addEventListener("click", () => { if (window.SlidesPresenter) window.SlidesPresenter.prevSlide(getState()); render(); });
    panel.querySelector("#spNextBtn").addEventListener("click", () => { if (window.SlidesPresenter) window.SlidesPresenter.nextSlide(getState()); render(); });
    panel.querySelector("#spBlack").addEventListener("click", () => { if (window.SlidesPresenter) window.SlidesPresenter.toggleBlackScreen(getState()); document.body.style.background = (getState().presenter && getState().presenter.blackScreen) ? "#000" : ""; });
    panel.querySelector("#spWhite").addEventListener("click", () => { if (window.SlidesPresenter) window.SlidesPresenter.toggleWhiteScreen(getState()); document.body.style.background = (getState().presenter && getState().presenter.whiteScreen) ? "#fff" : ""; });
    panel.querySelector("#spReset").addEventListener("click", () => { if (window.SlidesPresenter) window.SlidesPresenter.resetTimer(getState()); startedAt = Date.now(); });
    panel.querySelector("#spLaser").addEventListener("click", toggleLaser);
    panel.querySelector("#spAudience").addEventListener("click", openAudience);
    return panel;
  }

  function render() {
    if (!panel) return;
    const s = getState();
    if (!s) return;
    const slides = s.slides || [];
    const idx = s.currentSlide || 0;
    const current = slides[idx];
    const next = slides[idx + 1];
    const currentEl = panel.querySelector("#spCurrent");
    const nextEl = panel.querySelector("#spNext");
    if (current) {
      currentEl.innerHTML = "";
      currentEl.style.color = "#fff";
      const t = document.createElement("div");
      t.textContent = "Slide " + (idx + 1) + ": " + (current.title || "");
      currentEl.appendChild(t);
    } else {
      currentEl.textContent = "No slide";
    }
    if (next) {
      nextEl.innerHTML = "";
      const t = document.createElement("div");
      t.textContent = "Slide " + (idx + 2) + ": " + (next.title || "");
      nextEl.appendChild(t);
    } else {
      nextEl.textContent = "End of presentation";
    }
    const notes = (current && current.notes) || (s.notesBySlide && s.notesBySlide[idx]) || "";
    panel.querySelector("#spNotes").textContent = notes || "(no notes)";
  }

  function openAudience() {
    const s = getState();
    if (!s) return;
    const url = window.location.origin + "/audience?pid=" + (sessionId || "local");
    audienceWin = window.open(url, "audience", "width=1024,height=768");
    broadcastSlide();
  }

  function broadcastSlide() {
    if (!audienceWin || audienceWin.closed) return;
    try {
      const s = getState();
      audienceWin.postMessage({
        type: "slide-change",
        index: s.currentSlide || 0,
        slide: (s.slides || [])[s.currentSlide || 0] || null,
      }, "*");
    } catch (_e) { /* cross-origin / closed */ }
  }

  function toggleLaser() {
    let laser = document.getElementById("slidesLaser");
    if (laser) { laser.remove(); return; }
    laser = document.createElement("div");
    laser.id = "slidesLaser";
    laser.style.cssText = "position:fixed;width:16px;height:16px;border-radius:50%;background:radial-gradient(circle,#f00 0,#a00 60%,transparent 80%);z-index:9999;pointer-events:none;";
    document.body.appendChild(laser);
    document.addEventListener("mousemove", function onMove(e) {
      laser.style.left = (e.clientX - 8) + "px";
      laser.style.top = (e.clientY - 8) + "px";
    });
    document.addEventListener("keydown", function onKey(e) {
      if (e.key.toLowerCase() === "l") {
        laser.remove();
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("keydown", onKey);
      }
    });
  }

  function startSession() {
    const s = getState();
    if (!s || !s.botId) return null;
    return fetch("/api/slides/presenter/start", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        botId: s.botId,
        presentationId: s.presentationId || s.id,
        totalSlides: (s.slides || []).length,
      }),
    }).then(function (r) { return r.json(); }).then(function (data) {
      sessionId = data && data.sessionId ? data.sessionId : null;
      startedAt = Date.now();
      return sessionId;
    }).catch(function () { return null; });
  }

  function endSession() {
    if (!sessionId) return;
    const s = getState();
    fetch("/api/slides/presenter/end", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionId: sessionId }),
    }).catch(function () { /* offline */ });
    sessionId = null;
  }

  function startTimer() {
    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(function () {
      if (!panel) return;
      const elapsed = Date.now() - (startedAt || Date.now());
      panel.querySelector("#spTimer").textContent = window.SlidesPresenter ? window.SlidesPresenter.formatElapsed(elapsed) : "00:00";
    }, 1000);
  }

  function enter() {
    if (active) return;
    active = true;
    ensurePanel().style.display = "flex";
    if (window.SlidesPresenter) window.SlidesPresenter.enterPresenterMode(getState());
    startedAt = Date.now();
    startTimer();
    startSession();
    installKeyboard();
    render();
  }

  function exit() {
    if (!active) return;
    active = false;
    if (panel) panel.style.display = "none";
    if (window.SlidesPresenter) window.SlidesPresenter.exitPresenterMode(getState());
    if (timerInterval) clearInterval(timerInterval);
    timerInterval = null;
    if (audienceWin && !audienceWin.closed) audienceWin.close();
    endSession();
    document.body.style.background = "";
  }

  function toggle() {
    if (active) exit(); else enter();
  }

  function installKeyboard() {
    function onKey(e) {
      if (!active) return;
      const s = getState();
      if (!s || !window.SlidesPresenter) return;
      if (e.key === "ArrowRight" || e.key === " " || e.key === "PageDown") {
        e.preventDefault();
        window.SlidesPresenter.nextSlide(s);
        render();
        broadcastSlide();
      } else if (e.key === "ArrowLeft" || e.key === "PageUp") {
        e.preventDefault();
        window.SlidesPresenter.prevSlide(s);
        render();
        broadcastSlide();
      } else if (e.key === "Escape") {
        exit();
      } else if (e.key.toLowerCase() === "b") {
        e.preventDefault();
        window.SlidesPresenter.toggleBlackScreen(s);
        document.body.style.background = s.presenter.blackScreen ? "#000" : "";
      } else if (e.key.toLowerCase() === "w") {
        e.preventDefault();
        window.SlidesPresenter.toggleWhiteScreen(s);
        document.body.style.background = s.presenter.whiteScreen ? "#fff" : "";
      } else if (e.key.toLowerCase() === "l") {
        toggleLaser();
      }
    }
    document.addEventListener("keydown", onKey);
    if (active) document.removeEventListener("keydown", onKey);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      const btn = document.getElementById("presenterModeBtn");
      if (btn) btn.addEventListener("click", toggle);
    });
  }

  window.SlidesPresenterUI = {
    enter, exit, toggle, render,
    startSession, endSession, installKeyboard,
  };
})();
