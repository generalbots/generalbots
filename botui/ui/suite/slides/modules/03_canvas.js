"use strict";
/* slides canvas — selection, drag, insert, persistence, keyboard, scroll, rotation input */

SlideCanvas.bindCanvasClick = function () {
  var self = this;
  if (!this.canvas) return;
  this.canvas.addEventListener("pointerdown", function (e) {
    if (e.target === self.canvas || e.target.classList.contains("sl-canvas")) {
      self.deselectAll();
    }
  });
};

SlideCanvas.deselectAll = function () {
  var self = this;
  this.selectedId = null;
  $$(".sl-element", this.canvas).forEach(function (el) {
    el.style.outline = "none";
    self.removeHandles(el);
  });
  var rg = document.getElementById("rotation-group");
  if (rg) rg.style.display = "none";
  if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
    window.GBCollab.send("cursor", {
      slide_index: parseInt((this.canvas && this.canvas.dataset.slideId) || "0", 10) || 0,
      element_id: null
    });
  }
};

SlideCanvas.selectElement = function (el) {
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
  if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
    window.GBCollab.send("cursor", {
      slide_index: parseInt((this.canvas && this.canvas.dataset.slideId) || "0", 10) || 0,
      element_id: el.dataset.id
    });
  }
};

SlideCanvas.bindElement = function (el) {
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
      var slideIdx = parseInt((self.canvas && self.canvas.dataset.slideId) || "0", 10) || 0;
      t.addEventListener("input", function () {
        if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
          window.GBCollab.send("typing_start", { slide_index: slideIdx, element_id: el.dataset.id });
        }
      });
      t.addEventListener("blur", function () {
        if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
          window.GBCollab.send("typing_stop", {});
        }
        t.contentEditable = "false";
        self.persistElement(el);
      }, { once: true });
    }
    e.stopPropagation();
  });
};

SlideCanvas.bindGlobalKeys = function () {
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
};

SlideCanvas.bindCanvasScroll = function (host) {
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
};

SlideCanvas.bindQuickShapes = function () {
  var self = this;
  $$(".quick-shape").forEach(function (btn) {
    btn.addEventListener("click", function () {
      var type = btn.dataset.shape;
      self.insertShape(type);
    });
  });
};

SlideCanvas.insertShape = function (type) {
  if (!this.canvas) return;
  var slideId = this.canvas.dataset.slideId || "0";
  var shapeStyle = "";
  if (type === "rectangle") {
    shapeStyle = "border-radius:4px;";
  } else if (type === "circle") {
    shapeStyle = "border-radius:50%;";
  } else if (type === "triangle") {
    shapeStyle = "clip-path:polygon(50% 0%, 0% 100%, 100% 100%);";
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
  this.canvas.appendChild(el);
  this.bindElement(el);
  this.selectElement(el);
  var payload = {
    presentation_id: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current",
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
};

SlideCanvas.persistElement = function (el) {
  var slide = this.canvas ? this.canvas.dataset.slideId : null;
  if (!slide) return;
  var payload = {
    presentation_id: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current",
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
  if (window.recordSlidesEdit) window.recordSlidesEdit();
  if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
    window.GBCollab.send("slide_update", { content: JSON.stringify(payload), slide_index: parseInt(slide, 10) || 0, element_id: el.dataset.id });
  }
};

SlideCanvas.persistSlide = function () {
  if (!this.canvas) return;
  var slide = this.canvas.dataset.slideId;
  var elements = $$(".sl-element", this.canvas).map(function (el) { return el.dataset.id; });
  fetch("/api/slides/element/add", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ presentation_id: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current", slide_id: slide, elements: elements })
  }).catch(function () {});
};

SlideCanvas.bindRotationInput = function () {
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
};

SlideCanvas.syncRotationInput = function (el) {
  var inp = document.getElementById("rotationInput");
  if (!inp) return;
  var r = parseFloat(el.dataset.rotation) || 0;
  inp.value = r;
};

SlideCanvas.updateRotationInput = function (el) {
  var inp = document.getElementById("rotationInput");
  if (!inp) return;
  var r = parseFloat(el.dataset.rotation) || 0;
  inp.value = r;
};
