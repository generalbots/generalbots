"use strict";
/* slides shell — sidebar tab switching + canvas pointer (drag/resize/select) + WS coord broadcast */

(function () {
  const SIDEBAR_TAB_KEY = "slides_sidebar_tab";
  const SLIDE_W = 960;
  const SLIDE_H = 540;
  const SCALE_MIN = 0.25;
  const SCALE_MAX = 2.0;

  function $(s, r) { return (r || document).querySelector(s); }
  function $$(s, r) { return Array.from((r || document).querySelectorAll(s)); }

  document.addEventListener("click", function (e) {
    const tab = e.target.closest("[data-sidebar-tab]");
    if (tab) {
      const which = tab.dataset.sidebarTab;
      $$(".sidebar-tab").forEach(function (b) {
        b.classList.toggle("active", b === tab);
        b.style.background = b === tab ? "#1e293b" : "#0f172a";
        b.style.color = b === tab ? "#f8fafc" : "#94a3b8";
      });
      $$(".sidebar-content").forEach(function (c) {
        c.style.display = c.dataset.sidebarContent === which ? "flex" : "none";
      });
      try { sessionStorage.setItem(SIDEBAR_TAB_KEY, which); } catch (_) {}
    }
  });

  function initSidebar() {
    let saved = null;
    try { saved = sessionStorage.getItem(SIDEBAR_TAB_KEY); } catch (_) {}
    if (saved) {
      const btn = document.querySelector('[data-sidebar-tab="' + saved + '"]');
      if (btn) btn.click();
    }
  }

  const SlideCanvas = {
    scale: 1.0,
    selectedId: null,
    canvas: null,
    elements: [],

    attach: function (host) {
      if (!host) return;
      const c = host.querySelector(".sl-canvas");
      if (!c) return;
      this.canvas = c;
      this.elements = $$(".sl-element", c);
      this.elements.forEach(function (el) { SlideCanvas.bindElement(el); });
      this.bindGlobalKeys();
      this.bindCanvasScroll(host);
    },

    bindElement: function (el) {
      el.addEventListener("pointerdown", function (e) {
        if (e.target.classList.contains("sl-resizer")) return;
        SlideCanvas.selectedId = el.dataset.id;
        $$(".sl-element", SlideCanvas.canvas).forEach(function (x) {
          const isSel = x === el;
          x.style.outline = isSel ? "2px solid #3b82f6" : "none";
          let r = x.querySelector(".sl-resizer");
          if (isSel) {
            if (!r) SlideCanvas.addResizer(x);
            else r.style.display = "block";
          } else {
            if (r) r.style.display = "none";
          }
        });
        const rect = el.getBoundingClientRect();
        const start = { x: e.clientX, y: e.clientY, left: rect.left - SlideCanvas.canvas.getBoundingClientRect().left, top: rect.top - SlideCanvas.canvas.getBoundingClientRect().top };
        const onMove = function (ev) {
          const dx = (ev.clientX - start.x) / SlideCanvas.scale;
          const dy = (ev.clientY - start.y) / SlideCanvas.scale;
          el.style.left = (start.left + dx) + "px";
          el.style.top = (start.top + dy) + "px";
        };
        const onUp = function () {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", onUp);
          SlideCanvas.persistElement(el);
        };
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
        e.stopPropagation();
      });
      el.addEventListener("dblclick", function (e) {
        if (el.dataset.kind === "text" || el.dataset.kind === "title") {
          const t = el.querySelector("p, h1, h2, h3");
          if (t) {
            t.contentEditable = "true";
            t.focus();
            t.addEventListener("blur", function () { t.contentEditable = "false"; SlideCanvas.persistElement(el); }, { once: true });
          }
        }
        e.stopPropagation();
      });
    },

    addResizer: function (el) {
      const resizer = document.createElement("div");
      resizer.className = "sl-resizer";
      resizer.addEventListener("pointerdown", function (pe) {
        const startW = el.offsetWidth;
        const startH = el.offsetHeight;
        const startX = pe.clientX;
        const startY = pe.clientY;
        const onResizeMove = function (ev) {
          const dw = (ev.clientX - startX) / SlideCanvas.scale;
          const dh = (ev.clientY - startY) / SlideCanvas.scale;
          el.style.width = Math.max(20, startW + dw) + "px";
          el.style.height = Math.max(20, startH + dh) + "px";
        };
        const onResizeUp = function () {
          window.removeEventListener("pointermove", onResizeMove);
          window.removeEventListener("pointerup", onResizeUp);
          SlideCanvas.persistElement(el);
        };
        window.addEventListener("pointermove", onResizeMove);
        window.addEventListener("pointerup", onResizeUp);
        pe.stopPropagation();
        pe.preventDefault();
      });
      el.appendChild(resizer);
    },

    bindGlobalKeys: function () {
      document.addEventListener("keydown", function (e) {
        if (e.target.isContentEditable) return;
        if (!SlideCanvas.selectedId) return;
        const el = document.querySelector('[data-id="' + SlideCanvas.selectedId + '"]');
        if (!el) return;
        const step = e.shiftKey ? 10 : 1;
        let x = parseInt(el.style.left, 10) || 0;
        let y = parseInt(el.style.top, 10) || 0;
        if (e.key === "ArrowLeft") { x -= step; e.preventDefault(); }
        else if (e.key === "ArrowRight") { x += step; e.preventDefault(); }
        else if (e.key === "ArrowUp") { y -= step; e.preventDefault(); }
        else if (e.key === "ArrowDown") { y += step; e.preventDefault(); }
        else if (e.key === "Delete" || e.key === "Backspace") {
          el.remove();
          SlideCanvas.selectedId = null;
          SlideCanvas.persistSlide();
          return;
        } else { return; }
        el.style.left = x + "px";
        el.style.top = y + "px";
        SlideCanvas.persistElement(el);
      });
    },

    bindCanvasScroll: function (host) {
      const c = host.querySelector(".sl-canvas-scroll");
      if (!c) return;
      c.addEventListener("wheel", function (e) {
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          const newScale = Math.max(SCALE_MIN, Math.min(SCALE_MAX, SlideCanvas.scale * (e.deltaY < 0 ? 1.1 : 0.9)));
          SlideCanvas.scale = newScale;
          const cv = c.querySelector(".sl-canvas");
          if (cv) {
            cv.style.width = (SLIDE_W * newScale) + "px";
            cv.style.height = (SLIDE_H * newScale) + "px";
            $$(".sl-element", cv).forEach(function (el) {
              el.style.transformOrigin = "0 0";
              el.style.transform = "scale(" + newScale + ")";
            });
          }
        }
      }, { passive: false });
    },

    persistElement: function (el) {
      const slide = this.canvas ? this.canvas.dataset.slideId : null;
      if (!slide) return;
      const payload = {
        presentation_id: "current",
        slide_id: slide,
        element_id: el.dataset.id,
        x: parseInt(el.style.left, 10) || 0,
        y: parseInt(el.style.top, 10) || 0,
        width: el.offsetWidth,
        height: el.offsetHeight,
        rotation: parseFloat(el.dataset.rotation || "0")
      };
      fetch("/api/slides/element", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload)
      }).catch(function () {});
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        window.GBCollab.send("slide_update", { content: JSON.stringify(payload), slide_index: parseInt(slide, 10) || 0, element_id: el.dataset.id });
      }
    },

    persistSlide: function () {
      if (!this.canvas) return;
      const slide = this.canvas.dataset.slideId;
      const elements = $$(".sl-element", this.canvas).map(function (el) { return el.dataset.id; });
      fetch("/api/slides/elements", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ presentation_id: "current", slide_id: slide, elements: elements })
      }).catch(function () {});
    }
  };

  function injectCanvasStyles() {
    if (document.getElementById("slides-canvas-styles")) return;
    const style = document.createElement("style");
    style.id = "slides-canvas-styles";
    style.textContent = ".sl-canvas-scroll{flex:1;display:flex;align-items:center;justify-content:center;overflow:auto;background:#020617;padding:32px;}.sl-canvas{position:relative;width:960px;height:540px;background:#f8fafc;border-radius:6px;box-shadow:0 8px 32px rgba(0,0,0,0.4);transform-origin:center center;}.sl-element{position:absolute;cursor:move;user-select:none;box-sizing:border-box;outline:none;}.sl-element[data-kind='title'] h1,.sl-element[data-kind='text'] p{margin:0;padding:4px 8px;}.sl-thumb{background:#1e293b;border:1px solid #334155;border-radius:4px;padding:8px;cursor:pointer;transition:border-color 0.15s;}.sl-thumb:hover{border-color:#3b82f6;}.sl-thumb.active{border-color:#3b82f6;background:#1e3a8a;}.sl-thumb-preview{position:relative;width:100%;aspect-ratio:16/9;background:#f8fafc;border-radius:2px;overflow:hidden;}.sl-thumb-num{font-size:11px;color:#94a3b8;margin-top:4px;text-align:center;}.sl-resizer{width:10px;height:10px;background:#3b82f6;position:absolute;right:-5px;bottom:-5px;cursor:se-resize;border-radius:50%;border:1px solid #ffffff;z-index:10;}";
    document.head.appendChild(style);
  }

  function initAuth() {
    if (window.GBAuthGuard) GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
  }

  function initCollab() {
    if (!window.GBCollab) return;
    const connStatus = document.getElementById("gb-conn-status");
    window.GBCollab.connect({
      app: "slides",
      docId: "current",
      collaboratorsEl: document.getElementById("collaborators"),
      onConnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status online"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "online"; }
      },
      onDisconnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status offline"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "offline"; }
      },
      onEdit: function (msg) {
        if (!msg || !msg.content) return;
        try {
          const update = JSON.parse(msg.content);
          if (update.element_id && update.x !== undefined && update.y !== undefined) {
            const el = document.querySelector('[data-id="' + update.element_id + '"]');
            if (el) {
              el.style.left = update.x + "px";
              el.style.top = update.y + "px";
            }
          }
        } catch (_) {}
      }
    });
  }

  document.addEventListener("htmx:afterSwap", function (e) {
    if (e.target.id === "slides-content" || (e.target.closest && e.target.closest("#slides-content"))) {
      SlideCanvas.attach(document.getElementById("slides-content"));
    }
    if (e.target.id === "sidebar-thumbs" || (e.target.closest && e.target.closest("#sidebar-thumbs"))) {
      $$(".sl-thumb", e.target).forEach(function (thumb) {
        thumb.addEventListener("click", function () {
          const slideId = thumb.dataset.slide;
          const view = document.getElementById("slides-content");
          if (view && slideId) {
            htmx.ajax("GET", "/suite/slides/fragments/presentation-view", {
              target: "#slides-content",
              swap: "innerHTML",
              values: { id: "current", slide: slideId }
            });
          }
        });
      });
    }
  });

  window.addEventListener("DOMContentLoaded", function () {
    injectCanvasStyles();
    initSidebar();
    initAuth();
    initCollab();
    window.SlidesCanvas = SlideCanvas;
  });
})();
