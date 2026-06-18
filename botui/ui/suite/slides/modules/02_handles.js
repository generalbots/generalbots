"use strict";
/* slides handles — add/remove 8-point resize handles and rotation handle */

SlideCanvas.addHandles = function (el) {
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
};

SlideCanvas.removeHandles = function (el) {
  var c = el.querySelector(".sl-handles-container");
  if (c) c.remove();
};

SlideCanvas.makeResizeHandler = function (hd, el) {
  var pos = hd.dataset.handle;
  var self = this;
  hd.addEventListener("pointerdown", function (pe) {
    pe.stopPropagation();
    pe.preventDefault();
    var startL = parseFloat(el.style.left) || 0;
    var startT = parseFloat(el.style.top) || 0;
    var startW = el.offsetWidth;
    var startH = el.offsetHeight;
    var startX = pe.clientX;
    var startY = pe.clientY;

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
};

SlideCanvas.makeRotateHandler = function (rotEl, line, el) {
  var self = this;
  rotEl.addEventListener("pointerdown", function (pe) {
    pe.stopPropagation();
    pe.preventDefault();
    var rect = el.getBoundingClientRect();
    var cx = rect.left + rect.width / 2;
    var cy = rect.top + rect.height / 2;

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
};
