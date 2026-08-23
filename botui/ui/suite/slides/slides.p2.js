
(function () {
  const SIDEBAR_TAB_KEY = "slides_sidebar_tab";
  const SLIDE_W = 960;
  const SLIDE_H = 540;
  const SCALE_MIN = 0.25;
  const SCALE_MAX = 2.0;
  const H_SZ = 8;
  const ROT_H_SZ = 10;
  const ROT_OFF = 28;

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

  var HANDLE_POSITIONS = [
    {pos:"nw",cursor:"nwse-resize",cx:0,cy:0},
    {pos:"n", cursor:"ns-resize",   cx:0.5,cy:0},
    {pos:"ne",cursor:"nesw-resize", cx:1,cy:0},
    {pos:"e", cursor:"ew-resize",   cx:1,cy:0.5},
    {pos:"se",cursor:"nwse-resize", cx:1,cy:1},
    {pos:"s", cursor:"ns-resize",   cx:0.5,cy:1},
    {pos:"sw",cursor:"nesw-resize", cx:0,cy:1},
    {pos:"w", cursor:"ew-resize",   cx:0,cy:0.5}
  ];

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
      var self = this;
      this.elements.forEach(function (el) { self.bindElement(el); });
      this.bindGlobalKeys();
      this.bindCanvasScroll(host);
      this.bindQuickShapes();
      this.bindRotationInput();
      this.bindCanvasClick();
    },

    bindCanvasClick: function () {
      var self = this;
      if (!this.canvas) return;
      this.canvas.addEventListener("pointerdown", function (e) {
        if (e.target === self.canvas || e.target.classList.contains("sl-canvas")) {
          self.deselectAll();
        }
      });
    },

    deselectAll: function () {
      var self = this;
      this.selectedId = null;
      $$(".sl-element", this.canvas).forEach(function (el) {
        el.style.outline = "none";
        self.removeHandles(el);
      });
      var rg = document.getElementById("rotation-group");
      if (rg) rg.style.display = "none";
    },

    selectElement: function (el) {
      var self = this;
      this.selectedId = el.dataset.id;
      $$(".sl-element", this.canvas).forEach(function (x) {
        var isSel = x === el;
        x.style.outline = isSel ? "2px solid #3b82f6" : "none";
        if (isSel) {
          self.addHandles(x);
        } else {
          self.removeHandles(x);
        }
      });
      this.syncRotationInput(el);
      var rg = document.getElementById("rotation-group");
      if (rg) rg.style.display = "inline-flex";
    },

    bindElement: function (el) {
      var self = this;
      el.addEventListener("pointerdown", function (e) {
        if (e.target.closest(".sl-handle") || e.target.closest(".sl-rotator")) return;
        self.selectElement(el);
        var rect = el.getBoundingClientRect();
        var cRect = self.canvas.getBoundingClientRect();
        var start = {
          x: e.clientX,
          y: e.clientY,
          left: rect.left - cRect.left,
          top: rect.top - cRect.top
        };
        var onMove = function (ev) {
          var dx = (ev.clientX - start.x) / self.scale;
          var dy = (ev.clientY - start.y) / self.scale;
          el.style.left = (start.left + dx) + "px";
          el.style.top = (start.top + dy) + "px";
        };
        var onUp = function () {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", onUp);
          self.persistElement(el);
        };
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
        e.stopPropagation();
      });
      el.addEventListener("dblclick", function (e) {
        if (el.dataset.type === "text" || el.dataset.type === "title") {
          var t = el.querySelector("p, h1, h2, h3");
          if (!t) t = el;
          t.contentEditable = "true";
          t.focus();
          t.addEventListener("blur", function () {
            t.contentEditable = "false";
            self.persistElement(el);
          }, { once: true });
        }
        e.stopPropagation();
      });
    },

    addHandles: function (el) {
      this.removeHandles(el);
      var container = document.createElement("div");
      container.className = "sl-handles-container";
      container.style.cssText = "position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:100;";
      var self = this;

      HANDLE_POSITIONS.forEach(function (h) {
        var hd = document.createElement("div");
        hd.className = "sl-handle";
        hd.dataset.handle = h.pos;
        var sz = H_SZ;
        hd.style.cssText = "position:absolute;width:"+sz+"px;height:"+sz+
          "px;background:#3b82f6;border:1px solid #fff;border-radius:2px;"+
          "cursor:"+h.cursor+";pointer-events:all;z-index:101;"+
          "top:calc("+(h.cy*100)+"% - "+(sz*h.cy)+"px - 1px);"+
          "left:calc("+(h.cx*100)+"% - "+(sz*h.cx)+"px - 1px);";
        self.makeResizeHandler(hd, el);
        container.appendChild(hd);
      });

      var rot = document.createElement("div");
      rot.className = "sl-rotator";
      rot.style.cssText = "position:absolute;top:-"+ROT_OFF+"px;left:calc(50% - "+ROT_H_SZ+"px + 4px);"+
        "width:10px;height:10px;border-radius:50%;background:#10b981;border:2px solid #fff;"+
        "cursor:grab;pointer-events:all;z-index:102;";
      var line = document.createElement("div");
      line.style.cssText = "position:absolute;top:-"+ROT_OFF+"px;left:calc(50% - 1px);width:2px;height:"+ROT_OFF+"px;background:#10b981;pointer-events:none;z-index:101;";
      container.appendChild(line);

      self.makeRotateHandler(rot, line, el);
      container.appendChild(rot);
      el.appendChild(container);
    },

    removeHandles: function (el) {
      var c = el.querySelector(".sl-handles-container");
      if (c) c.remove();
    },

    makeResizeHandler: function (hd, el) {
      var pos = hd.dataset.handle;
      var self = this;
      hd.addEventListener("pointerdown", function (pe) {
        pe.stopPropagation();
        pe.preventDefault();
        var rect = el.getBoundingClientRect();
        var cRect = self.canvas.getBoundingClientRect();
        var startL = parseFloat(el.style.left) || 0;
        var startT = parseFloat(el.style.top) || 0;
        var startW = el.offsetWidth;
        var startH = el.offsetHeight;
        var startX = pe.clientX;
        var startY = pe.clientY;
        var hx = pe.clientX;
        var hy = pe.clientY;

        var onMove = function (ev) {
          var dx = (ev.clientX - startX) / self.scale;
          var dy = (ev.clientY - startY) / self.scale;
          var nx = startL, ny = startT, nw = startW, nh = startH;

          if (pos.indexOf("e") >= 0) {
            nw = Math.max(20, startW + dx);
          }
          if (pos.indexOf("w") >= 0) {
            var dw2 = Math.min(startW - 20, dx);
            nx = startL + dw2;
            nw = startW - dw2;
          }
          if (pos.indexOf("s") >= 0) {
            nh = Math.max(20, startH + dy);
          }
          if (pos.indexOf("n") >= 0) {
            var dh2 = Math.min(startH - 20, dy);
            ny = startT + dh2;
            nh = startH - dh2;
          }

          el.style.left = nx + "px";
          el.style.top = ny + "px";
          el.style.width = nw + "px";
          el.style.height = nh + "px";
        };
        var onUp = function () {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", onUp);
          self.persistElement(el);
        };
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
      });
    },

    makeRotateHandler: function (rotEl, line, el) {
      var self = this;
      rotEl.addEventListener("pointerdown", function (pe) {
        pe.stopPropagation();
        pe.preventDefault();
        var rect = el.getBoundingClientRect();
        var cx = rect.left + rect.width / 2;
        var cy = rect.top + rect.height / 2;
        var startAngle = parseFloat(el.dataset.rotation) || 0;

        var onMove = function (ev) {
          var dx = ev.clientX - cx;
          var dy = ev.clientY - cy;
          var deg = (Math.atan2(dy, dx) * 180 / Math.PI) + 90;
          deg = Math.round(deg);
          el.dataset.rotation = deg;
          el.style.transform = "rotate(" + deg + "deg)";
          self.updateRotationInput(el);
        };
        var onUp = function () {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", onUp);
          self.persistElement(el);
        };
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
      });
    },

    bindRotationInput: function () {
      var inp = document.getElementById("rotationInput");
      var rst = document.getElementById("resetRotationBtn");
      var self = this;
      if (inp) {
        inp.addEventListener("input", function () {
          var el = document.querySelector('[data-id="' + self.selectedId + '"]');
          if (!el) return;
          var v = parseFloat(inp.value) || 0;
          el.dataset.rotation = v;
          el.style.transform = "rotate(" + v + "deg)";
          self.persistElement(el);
        });
      }
      if (rst) {
        rst.addEventListener("click", function () {
          var el = document.querySelector('[data-id="' + self.selectedId + '"]');
          if (!el) return;
          el.dataset.rotation = 0;
          el.style.transform = "rotate(0deg)";
          if (inp) inp.value = 0;
          self.persistElement(el);
        });
      }
    },

    syncRotationInput: function (el) {
      var inp = document.getElementById("rotationInput");
      if (!inp) return;
      var r = parseFloat(el.dataset.rotation) || 0;
      inp.value = r;
    },

    updateRotationInput: function (el) {
      var inp = document.getElementById("rotationInput");
      if (!inp) return;
      var r = parseFloat(el.dataset.rotation) || 0;
      inp.value = r;
    },

    bindQuickShapes: function () {
      var self = this;
      $$(".quick-shape").forEach(function (btn) {
        btn.addEventListener("click", function () {
          var type = btn.dataset.shape;
          self.insertShape(type);
        });
      });
    },

    insertShape: function (type) {
      if (!this.canvas) return;
      var slideId = this.canvas.dataset.slideId || "0";
      var shapeStyle = "";
      var label = "";
      if (type === "rectangle") {
        shapeStyle = "border-radius:4px;";
        label = "";
      } else if (type === "circle") {
        shapeStyle = "border-radius:50%;";
        label = "";
      } else if (type === "triangle") {
        shapeStyle = "clip-path:polygon(50% 0%, 0% 100%, 100% 100%);";
        label = "";
      }
      var id = "el_" + Date.now() + "_" + Math.random().toString(36).substr(2, 4);
      var el = document.createElement("div");
      el.className = "sl-element";
      el.dataset.id = id;
      el.dataset.type = "shape";
      el.dataset.rotation = 0;
      var x = 150 + Math.random() * 200;
      var y = 100 + Math.random() * 150;
      el.style.cssText = "position:absolute;left:"+x+"px;top:"+y+"px;width:120px;height:80px;"+
        "background:#3b82f6;"+shapeStyle+
        "border:1px solid #2563eb;color:#f8fafc;display:flex;align-items:center;justify-content:center;"+
        "font-size:14px;transform:rotate(0deg);";
      el.textContent = label;
      this.canvas.appendChild(el);
      this.bindElement(el);
      this.selectElement(el);
      var payload = {
        presentation_id: "current",
        slide_id: slideId,
        element_id: id,
        element_type: "shape",
        shape_type: type,
        x: x, y: y, width: 120, height: 80,
        rotation: 0,
        fill: "#3b82f6"
      };
      fetch("/api/slides/element/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload)
      }).catch(function () {});
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        window.GBCollab.send("slide_update", { content: JSON.stringify(payload), slide_index: parseInt(slideId, 10) || 0, element_id: id });
      }
    },

    bindGlobalKeys: function () {
      var self = this;
      document.addEventListener("keydown", function (e) {
        if (e.target.isContentEditable) return;
        if (!self.selectedId) return;
        var el = document.querySelector('[data-id="' + self.selectedId + '"]');
        if (!el) return;
        var step = e.shiftKey ? 10 : 1;
        var x = parseInt(el.style.left, 10) || 0;
        var y = parseInt(el.style.top, 10) || 0;
        if (e.key === "ArrowLeft") { x -= step; e.preventDefault(); }
        else if (e.key === "ArrowRight") { x += step; e.preventDefault(); }
        else if (e.key === "ArrowUp") { y -= step; e.preventDefault(); }
        else if (e.key === "ArrowDown") { y += step; e.preventDefault(); }
        else if (e.key === "Delete" || e.key === "Backspace") {
          el.remove();
          self.selectedId = null;
          var rg = document.getElementById("rotation-group");
          if (rg) rg.style.display = "none";
          self.persistSlide();
          return;
        } else { return; }
        el.style.left = x + "px";
        el.style.top = y + "px";
        self.persistElement(el);
      });
    },

    bindCanvasScroll: function (host) {
      var c = host.querySelector(".sl-canvas-scroll");
      if (!c) return;
      var self = this;
      c.addEventListener("wheel", function (e) {
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          var newScale = Math.max(SCALE_MIN, Math.min(SCALE_MAX, self.scale * (e.deltaY < 0 ? 1.1 : 0.9)));
          self.scale = newScale;
          var cv = c.querySelector(".sl-canvas");
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
      var slide = this.canvas ? this.canvas.dataset.slideId : null;
      if (!slide) return;
      var payload = {
        presentation_id: "current",
        slide_id: slide,
        element_id: el.dataset.id,
        x: parseInt(el.style.left, 10) || 0,
        y: parseInt(el.style.top, 10) || 0,
        width: el.offsetWidth,
        height: el.offsetHeight,
        rotation: parseFloat(el.dataset.rotation || "0")
      };
      fetch("/api/slides/element/update", {
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
      var slide = this.canvas.dataset.slideId;
      var elements = $$(".sl-element", this.canvas).map(function (el) { return el.dataset.id; });
      fetch("/api/slides/element/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ presentation_id: "current", slide_id: slide, elements: elements })
      }).catch(function () {});
    }
  };

  function injectCanvasStyles() {
    if (document.getElementById("slides-canvas-styles")) return;
    var style = document.createElement("style");
    style.id = "slides-canvas-styles";
    style.textContent =
      ".sl-canvas-scroll{flex:1;display:flex;align-items:center;justify-content:center;overflow:auto;background:#020617;padding:32px;}"+
      ".sl-canvas{position:relative;width:960px;height:540px;background:#f8fafc;border-radius:6px;box-shadow:0 8px 32px rgba(0,0,0,0.4);transform-origin:center center;}"+
      ".sl-element{position:absolute;cursor:move;user-select:none;box-sizing:border-box;outline:none;}"+
      ".sl-element[data-type='title'] h1,.sl-element[data-type='text'] p{margin:0;padding:4px 8px;}"+
      ".sl-thumb{background:#1e293b;border:1px solid #334155;border-radius:4px;padding:8px;cursor:pointer;transition:border-color 0.15s;}"+
      ".sl-thumb:hover{border-color:#3b82f6;}"+
      ".sl-thumb.active{border-color:#3b82f6;background:#1e3a8a;}"+
      ".sl-thumb-preview{position:relative;width:100%;aspect-ratio:16/9;background:#f8fafc;border-radius:2px;overflow:hidden;}"+
      ".sl-thumb-num{font-size:11px;color:#94a3b8;margin-top:4px;text-align:center;}";
    document.head.appendChild(style);
  }

  function initAuth() {
    if (window.GBAuthGuard) GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
  }

  function initCollab() {
    if (!window.GBCollab) return;
    var connStatus = document.getElementById("gb-conn-status");
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
          var update = JSON.parse(msg.content);
          if (update.element_id && update.x !== undefined && update.y !== undefined) {
            var el = document.querySelector('[data-id="' + update.element_id + '"]');
            if (el) {
              el.style.left = update.x + "px";
              el.style.top = update.y + "px";
              if (update.width) el.style.width = update.width + "px";
              if (update.height) el.style.height = update.height + "px";
              if (update.rotation !== undefined) {
                el.dataset.rotation = update.rotation;
                el.style.transform = "rotate(" + update.rotation + "deg)";
              }
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
          var slideId = thumb.dataset.slide;
          var view = document.getElementById("slides-content");
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
