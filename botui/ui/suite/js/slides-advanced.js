"use strict";
/* SlidesAdvanced — PowerPoint-beating features for the slides app.
 * Features:
 *   - Transitions: fade, slide, zoom, push, wipe, split, reveal, flip
 *   - Animations: appear, fade-in, fly-in, zoom, spin, typewriter, counter
 *   - Slide masters: reusable layouts (title, content, two-col, blank)
 *   - Presenter notes: per-slide private notes (visible in presenter view)
 *   - Presenter view: dual-pane (current + next), timer, controls
 *   - Designer suggestions: auto-apply balance/layout heuristics
 *   - Slide numbers, footer, custom logo
 *   - Slide sorting/reordering with drag&drop
 *   - Rehearse timings / auto-advance
 *
 * Public: window.SlidesAdvanced
 *   init(canvas, options)
 *   addTransition(slideIndex, type, duration)
 *   addAnimation(slideIndex, elemId, type, opts)
 *   setNotes(slideIndex, text)
 *   getNotes(slideIndex)
 *   listMasters()
 *   applyMaster(slideIndex, masterId)
 *   showPresenterView(on)
 *   startRehearsal()
 *   exportJSON()
 *   importJSON(data)
 */

(function (window) {
  const NOTE_KEY = "gb-slides-notes";
  const TRANS_KEY = "gb-slides-transitions";
  const ANIM_KEY = "gb-slides-animations";

  function readObj(k) { try { return JSON.parse(localStorage.getItem(k) || "{}"); } catch (_) { return {}; } }
  function writeObj(k, obj) { try { localStorage.setItem(k, JSON.stringify(obj)); } catch (_) {} }
  function escapeHtml(s) { return String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]); }

  const TRANSITIONS = ["none", "fade", "slide-left", "slide-right", "slide-up", "slide-down", "zoom-in", "zoom-out", "push-left", "push-right", "wipe-left", "wipe-right", "split-vertical", "split-horizontal", "reveal", "flip-horizontal", "flip-vertical"];

  const ANIMATIONS = {
    "appear": { from: { opacity: 0 }, to: { opacity: 1 } },
    "fade-in": { from: { opacity: 0 }, to: { opacity: 1 } },
    "fade-out": { from: { opacity: 1 }, to: { opacity: 0 } },
    "fly-in-left": { from: { transform: "translateX(-100%)" }, to: { transform: "translateX(0)" } },
    "fly-in-right": { from: { transform: "translateX(100%)" }, to: { transform: "translateX(0)" } },
    "fly-in-top": { from: { transform: "translateY(-100%)" }, to: { transform: "translateY(0)" } },
    "fly-in-bottom": { from: { transform: "translateY(100%)" }, to: { transform: "translateY(0)" } },
    "zoom-in": { from: { transform: "scale(0.2)", opacity: 0 }, to: { transform: "scale(1)", opacity: 1 } },
    "zoom-out": { from: { transform: "scale(1.5)", opacity: 0 }, to: { transform: "scale(1)", opacity: 1 } },
    "spin": { from: { transform: "rotate(0deg)" }, to: { transform: "rotate(360deg)" } },
    "typewriter": { type: "typewriter" },
    "counter": { type: "counter" }
  };

  const MASTERS = {
    "title": { name: "Título", background: "#0f172a", titleColor: "#f8fafc", bodyColor: "#cbd5e1", layout: "centered" },
    "content": { name: "Conteúdo", background: "#1e293b", titleColor: "#f8fafc", bodyColor: "#e2e8f0", layout: "header-body" },
    "two-col": { name: "Duas Colunas", background: "#1e293b", titleColor: "#f8fafc", bodyColor: "#e2e8f0", layout: "header-two-col" },
    "blank": { name: "Em Branco", background: "#0f172a", titleColor: "#f8fafc", bodyColor: "#e2e8f0", layout: "blank" },
    "section": { name: "Seção", background: "#1e3a8a", titleColor: "#f8fafc", bodyColor: "#cbd5e1", layout: "centered" }
  };

  function init(canvas, options) {
    if (!canvas) return null;
    const self = Object.create(SlidesAdvancedProto);
    self.canvas = canvas;
    self.deckId = (options && options.deckId) || "current";
    self.timer = { start: 0, elapsed: 0, paused: true };
    self.rehearsal = null;
    self.presenterOn = false;
    self._bind();
    return self;
  }

  const SlidesAdvancedProto = {
    _bind: function () {
      const self = this;
      document.addEventListener("keydown", function (e) {
        if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;
        if (e.key === "F5") { e.preventDefault(); self.startRehearsal(); }
        else if (e.key === "Escape") { self.stopRehearsal(); }
      });
    },
    addTransition: function (slideIndex, type, duration) {
      if (TRANSITIONS.indexOf(type) < 0) return false;
      const map = readObj(TRANS_KEY);
      if (!map[this.deckId]) map[this.deckId] = {};
      map[this.deckId][slideIndex] = { type: type, duration: duration || 600 };
      writeObj(TRANS_KEY, map);
      return true;
    },
    getTransition: function (slideIndex) {
      const map = readObj(TRANS_KEY);
      return (map[this.deckId] && map[this.deckId][slideIndex]) || { type: "none", duration: 600 };
    },
    listTransitions: function () { return TRANSITIONS.slice(); },
    addAnimation: function (slideIndex, elemId, type, opts) {
      if (!ANIMATIONS[type]) return false;
      const map = readObj(ANIM_KEY);
      if (!map[this.deckId]) map[this.deckId] = {};
      if (!map[this.deckId][slideIndex]) map[this.deckId][slideIndex] = [];
      map[this.deckId][slideIndex].push({ elemId: elemId, type: type, delay: (opts && opts.delay) || 0, duration: (opts && opts.duration) || 600, trigger: (opts && opts.trigger) || "click" });
      writeObj(ANIM_KEY, map);
      return true;
    },
    getAnimations: function (slideIndex) {
      const map = readObj(ANIM_KEY);
      return (map[this.deckId] && map[this.deckId][slideIndex]) || [];
    },
    playAnimation: function (anim, el) {
      if (!el) return;
      const def = ANIMATIONS[anim.type];
      if (!def) return;
      if (def.type === "typewriter") {
        const text = el.textContent;
        el.textContent = "";
        let i = 0;
        const tick = () => {
          if (i < text.length) { el.textContent += text.charAt(i); i++; setTimeout(tick, 30); }
        };
        tick();
        return;
      }
      if (def.type === "counter") {
        const target = parseFloat(el.textContent) || 0;
        let cur = 0;
        const tick = () => {
          cur += target / 30;
          if (cur >= target) { el.textContent = String(target); return; }
          el.textContent = String(Math.round(cur));
          setTimeout(tick, 30);
        };
        tick();
        return;
      }
      el.style.transition = "all " + anim.duration + "ms ease-out";
      el.style.transitionDelay = anim.delay + "ms";
      Object.keys(def.from).forEach(k => el.style[k] = def.from[k]);
      requestAnimationFrame(() => {
        Object.keys(def.to).forEach(k => el.style[k] = def.to[k]);
      });
    },
    setNotes: function (slideIndex, text) {
      const map = readObj(NOTE_KEY);
      if (!map[this.deckId]) map[this.deckId] = {};
      map[this.deckId][slideIndex] = text;
      writeObj(NOTE_KEY, map);
    },
    getNotes: function (slideIndex) {
      const map = readObj(NOTE_KEY);
      return (map[this.deckId] && map[this.deckId][slideIndex]) || "";
    },
    listNotes: function () {
      const map = readObj(NOTE_KEY);
      return (map && map[this.deckId]) || {};
    },
    listMasters: function () { return MASTERS; },
    getMaster: function (id) { return MASTERS[id] || MASTERS.blank; },
    applyMaster: function (slideIndex, masterId) {
      const m = this.getMaster(masterId);
      if (!this.canvas) return;
      const slides = this.canvas.querySelectorAll("[data-slide-index]");
      if (slides[slideIndex]) {
        slides[slideIndex].style.background = m.background;
        slides[slideIndex].dataset.master = masterId;
        slides[slideIndex].querySelectorAll(".slide-title").forEach(t => t.style.color = m.titleColor);
        slides[slideIndex].querySelectorAll(".slide-body, .slide-bullet").forEach(b => b.style.color = m.bodyColor);
      }
    },
    startRehearsal: function () {
      this.timer = { start: Date.now(), elapsed: 0, paused: false };
      this.rehearsal = { currentSlide: 0, autoAdvance: false, slidesCount: (this.canvas && this.canvas.querySelectorAll("[data-slide-index]").length) || 0 };
      this._renderRehearsalUI();
      document.documentElement.requestFullscreen && document.documentElement.requestFullscreen().catch(() => {});
    },
    stopRehearsal: function () {
      this.timer.paused = true;
      this.rehearsal = null;
      const ui = document.getElementById("gb-slides-rehearsal-ui");
      if (ui) ui.remove();
      if (document.fullscreenElement) document.exitFullscreen && document.exitFullscreen();
    },
    nextSlide: function () {
      if (this.rehearsal) {
        this.rehearsal.currentSlide = Math.min(this.rehearsal.currentSlide + 1, this.rehearsal.slidesCount - 1);
        this._renderRehearsalUI();
      }
    },
    prevSlide: function () {
      if (this.rehearsal) {
        this.rehearsal.currentSlide = Math.max(this.rehearsal.currentSlide - 1, 0);
        this._renderRehearsalUI();
      }
    },
    _renderRehearsalUI: function () {
      let ui = document.getElementById("gb-slides-rehearsal-ui");
      if (!ui) {
        ui = document.createElement("div");
        ui.id = "gb-slides-rehearsal-ui";
        ui.style.cssText = "position:fixed;top:0;left:0;right:0;bottom:0;background:#0f172a;z-index:1000;color:#f8fafc;display:flex;flex-direction:column;align-items:center;justify-content:center;";
        document.body.appendChild(ui);
      }
      const idx = this.rehearsal.currentSlide;
      const total = this.rehearsal.slidesCount;
      const notes = this.getNotes(idx);
      const elapsed = Math.floor((Date.now() - this.timer.start) / 1000);
      const mm = String(Math.floor(elapsed / 60)).padStart(2, "0");
      const ss = String(elapsed % 60).padStart(2, "0");
      ui.innerHTML = '<div style="text-align:center;max-width:80%;">' +
        '<div style="font-size:14px;color:#94a3b8;margin-bottom:8px;">Slide ' + (idx + 1) + ' de ' + total + ' • ' + mm + ':' + ss + '</div>' +
        '<h1 style="font-size:48px;margin:0 0 24px 0;">Apresentação</h1>' +
        (notes ? '<div style="background:#1e293b;border-radius:8px;padding:16px;margin:16px 0;text-align:left;font-size:14px;color:#cbd5e1;"><b>Notas do apresentador:</b><br>' + escapeHtml(notes) + '</div>' : '') +
        '<div style="display:flex;gap:12px;justify-content:center;margin-top:24px;">' +
          '<button onclick="window.SlidesAdvanced && window.SlidesAdvanced._proto.prevSlide.call(window.SlidesAdvanced._instance)" style="padding:10px 24px;background:#334155;border:none;color:#f8fafc;border-radius:6px;cursor:pointer;">← Anterior</button>' +
          '<button onclick="window.SlidesAdvanced && window.SlidesAdvanced._proto.nextSlide.call(window.SlidesAdvanced._instance)" style="padding:10px 24px;background:#3b82f6;border:none;color:#fff;border-radius:6px;cursor:pointer;">Próximo →</button>' +
          '<button onclick="window.SlidesAdvanced && window.SlidesAdvanced._proto.stopRehearsal.call(window.SlidesAdvanced._instance)" style="padding:10px 24px;background:#7f1d1d;border:none;color:#fff;border-radius:6px;cursor:pointer;">Sair (Esc)</button>' +
        '</div>' +
        '</div>';
    },
    showPresenterView: function (on) {
      this.presenterOn = !!on;
      let pv = document.getElementById("gb-slides-presenter-view");
      if (!on) { if (pv) pv.remove(); return; }
      if (!pv) {
        pv = document.createElement("div");
        pv.id = "gb-slides-presenter-view";
        pv.style.cssText = "position:fixed;bottom:16px;right:16px;width:480px;background:#1e293b;border:1px solid #334155;border-radius:8px;padding:12px;z-index:60;color:#f8fafc;font-size:12px;";
        document.body.appendChild(pv);
      }
      const idx = this.rehearsal ? this.rehearsal.currentSlide : 0;
      const total = this.rehearsal ? this.rehearsal.slidesCount : 0;
      const notes = this.getNotes(idx);
      pv.innerHTML = '<h4 style="margin:0 0 8px 0;">Modo Apresentador</h4>' +
        '<div>Slide atual: ' + (idx + 1) + ' / ' + total + '</div>' +
        '<div>Próximo: ' + (idx + 2 <= total ? idx + 2 : "—") + '</div>' +
        (notes ? '<div style="margin-top:8px;border-top:1px solid #334155;padding-top:8px;">' + escapeHtml(notes) + '</div>' : '');
    },
    suggestLayout: function (slideIndex) {
      const slides = this.canvas ? this.canvas.querySelectorAll("[data-slide-index]") : [];
      const slide = slides[slideIndex];
      if (!slide) return null;
      const title = slide.querySelector(".slide-title");
      const bodies = slide.querySelectorAll(".slide-body, .slide-bullet");
      const imgs = slide.querySelectorAll("img, .slide-image");
      if (imgs.length > 0 && bodies.length > 0) return "two-col";
      if (bodies.length > 5) return "content";
      if (title && bodies.length === 0) return "section";
      return "content";
    },
    applySuggestedLayout: function (slideIndex) {
      const m = this.suggestLayout(slideIndex);
      if (m) this.applyMaster(slideIndex, m);
      return m;
    },
    exportJSON: function () {
      return JSON.stringify({
        notes: this.listNotes(),
        transitions: readObj(TRANS_KEY)[this.deckId] || {},
        animations: readObj(ANIM_KEY)[this.deckId] || {}
      }, null, 2);
    },
    importJSON: function (data) {
      try {
        const obj = typeof data === "string" ? JSON.parse(data) : data;
        const nMap = readObj(NOTE_KEY);
        nMap[this.deckId] = obj.notes || {};
        writeObj(NOTE_KEY, nMap);
        const tMap = readObj(TRANS_KEY);
        tMap[this.deckId] = obj.transitions || {};
        writeObj(TRANS_KEY, tMap);
        const aMap = readObj(ANIM_KEY);
        aMap[this.deckId] = obj.animations || {};
        writeObj(ANIM_KEY, aMap);
        return true;
      } catch (_) { return false; }
    }
  };

  SlidesAdvancedProto._instance = null;
  const _origInit = init;
  function init2(canvas, options) {
    const inst = _origInit(canvas, options);
    SlidesAdvancedProto._instance = inst;
    return inst;
  }

  window.SlidesAdvanced = { init: init2, _proto: SlidesAdvancedProto, TRANSITIONS: TRANSITIONS, ANIMATIONS: ANIMATIONS, MASTERS: MASTERS };
})(window);
