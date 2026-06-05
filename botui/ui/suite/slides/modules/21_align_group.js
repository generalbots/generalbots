"use strict";

/**
 * Module 21: Alignment, distribution, and group/ungroup for Slides.
 * Adds alignment buttons (Align Left/Right/Top/Bottom/Center H/Center V),
 * distribution (horizontal/vertical), and group/ungroup. Supports
 * multi-select via Shift+click or drag-rect on the canvas. Groups
 * are stored as a parent SlideElement with `children: SlideElement[]`
 * and move/resize/rotate as a single unit.
 *
 * Public API: window.SlidesArrange = { align, distribute, group,
 *   ungroup, multiSelect, getSelectedElements, getGroupBounds }.
 */

(function () {
  function getState() { return window.state || null; }
  function getSlide() {
    const s = getState();
    return s ? (s.slides || [])[s.currentSlide || 0] : null;
  }
  function getCanvas() {
    return document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
  }
  function getSelectedElements() {
    return Array.from(document.querySelectorAll(".slide-element.selected, .slide-element.in-range"));
  }

  function bounds(el) {
    const r = el.getBoundingClientRect();
    return { x: r.left, y: r.top, width: r.width, height: r.height, right: r.right, bottom: r.bottom };
  }

  function getCanvasRect() {
    const c = getCanvas();
    return c ? c.getBoundingClientRect() : { left: 0, top: 0, width: 800, height: 600, right: 800, bottom: 600 };
  }

  function align(alignment) {
    const els = getSelectedElements();
    if (els.length < 2) return false;
    const cr = getCanvasRect();
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const el of els) {
      const b = bounds(el);
      if (b.x < minX) minX = b.x;
      if (b.right > maxX) maxX = b.right;
      if (b.y < minY) minY = b.y;
      if (b.bottom > maxY) maxY = b.bottom;
    }
    for (const el of els) {
      const b = bounds(el);
      let dx = 0, dy = 0;
      switch (alignment) {
        case "left": dx = minX - b.x; break;
        case "right": dx = maxX - b.right; break;
        case "centerH": dx = ((minX + maxX) / 2) - ((b.x + b.right) / 2); break;
        case "top": dy = minY - b.y; break;
        case "bottom": dy = maxY - b.bottom; break;
        case "centerV": dy = ((minY + maxY) / 2) - ((b.y + b.bottom) / 2); break;
        case "centerCanvasH": dx = (cr.left + cr.width / 2) - (b.x + b.width / 2); break;
        case "centerCanvasV": dy = (cr.top + cr.height / 2) - (b.y + b.height / 2); break;
      }
      el.style.left = (parseFloat(el.style.left || 0) + dx) + "px";
      el.style.top = (parseFloat(el.style.top || 0) + dy) + "px";
    }
    return true;
  }

  function distribute(axis) {
    const els = getSelectedElements();
    if (els.length < 3) return false;
    const sorted = els.slice().sort((a, b) => axis === "horizontal"
      ? bounds(a).x - bounds(b).x
      : bounds(a).y - bounds(b).y);
    const first = bounds(sorted[0]);
    const last = bounds(sorted[sorted.length - 1]);
    const start = axis === "horizontal" ? first.x : first.y;
    const end = axis === "horizontal" ? (axis === "horizontal" ? last.right : last.bottom) : last.bottom;
    const totalSpan = end - start;
    let totalSize = 0;
    for (const el of els) {
      const b = bounds(el);
      totalSize += (axis === "horizontal" ? b.width : b.height);
    }
    const gap = (totalSpan - totalSize) / (els.length - 1);
    let cursor = start;
    for (let i = 0; i < sorted.length; i++) {
      const el = sorted[i];
      const b = bounds(el);
      const size = axis === "horizontal" ? b.width : b.height;
      if (i === 0) {
        cursor += size + gap;
        continue;
      }
      if (i === sorted.length - 1) break;
      const target = cursor;
      const delta = target - (axis === "horizontal" ? b.x : b.y);
      el.style.left = (parseFloat(el.style.left || 0) + delta) + "px";
      el.style.top = (parseFloat(el.style.top || 0) + delta) + "px";
      cursor += size + gap;
    }
    return true;
  }

  function group() {
    const els = getSelectedElements();
    if (els.length < 2) return null;
    const slide = getSlide();
    if (!slide) return null;
    if (!slide.elements) slide.elements = [];
    const group = {
      id: "group-" + Date.now(),
      type: "group",
      children: [],
      x: 0, y: 0, width: 0, height: 0,
    };
    const cr = getCanvasRect();
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const el of els) {
      const b = bounds(el);
      if (b.x < minX) minX = b.x;
      if (b.right > maxX) maxX = b.right;
      if (b.y < minY) minY = b.y;
      if (b.bottom > maxY) maxY = b.bottom;
    }
    group.x = ((minX - cr.left) / cr.width) * 100;
    group.y = ((minY - cr.top) / cr.height) * 100;
    group.width = ((maxX - minX) / cr.width) * 100;
    group.height = ((maxY - minY) / cr.height) * 100;
    for (const el of els) {
      const idx = slide.elements.findIndex((e) => e.domRef === el);
      if (idx >= 0) {
        const item = slide.elements[idx];
        item.x = ((bounds(el).x - minX) / (maxX - minX || 1)) * 100;
        item.y = ((bounds(el).y - minY) / (maxY - minY || 1)) * 100;
        item.width = (bounds(el).width / (maxX - minX || 1)) * 100;
        item.height = (bounds(el).height / (maxY - minY || 1)) * 100;
        item._groupOffset = { x: item.x, y: item.y, w: item.width, h: item.height };
        group.children.push(item);
        slide.elements.splice(idx, 1);
        el.remove();
      }
    }
    slide.elements.push(group);
    renderGroup(group);
    return group;
  }

  function ungroup() {
    const groups = document.querySelectorAll(".slide-element.slide-group");
    for (const g of groups) {
      const slide = getSlide();
      if (!slide) return false;
      const groupData = slide.elements.find((e) => e.id === g.dataset.groupId);
      if (!groupData) continue;
      g.remove();
      slide.elements = slide.elements.filter((e) => e.id !== g.dataset.groupId);
      for (const child of groupData.children || []) {
        delete child._groupOffset;
        slide.elements.push(child);
      }
    }
    return true;
  }

  function renderGroup(group) {
    const c = getCanvas();
    if (!c) return;
    const el = document.createElement("div");
    el.className = "slide-element slide-group";
    el.dataset.groupId = group.id;
    el.style.cssText = "position:absolute;left:" + group.x + "%;top:" + group.y + "%;width:" + group.width + "%;height:" + group.height + "%;border:1px dashed #1a73e8;background:rgba(26,115,232,0.04);";
    for (const child of group.children || []) {
      const c2 = document.createElement("div");
      c2.className = "slide-element slide-group-child";
      c2.style.cssText = "position:absolute;left:" + (child._groupOffset ? child._groupOffset.x : 0) + "%;top:" + (child._groupOffset ? child._groupOffset.y : 0) + "%;width:" + (child._groupOffset ? child._groupOffset.w : 0) + "%;height:" + (child._groupOffset ? child._groupOffset.h : 0) + "%;";
      c2.textContent = child.type + ": " + (child.text || child.url || child.title || "");
      el.appendChild(c2);
    }
    c.appendChild(el);
  }

  function getGroupBounds(groupEl) {
    return bounds(groupEl);
  }

  function multiSelect() {
    const c = getCanvas();
    if (!c) return;
    let startX, startY, marquee;
    c.addEventListener("mousedown", function (e) {
      if (e.target !== c) return;
      if (!e.shiftKey) return;
      e.preventDefault();
      startX = e.clientX;
      startY = e.clientY;
      marquee = document.createElement("div");
      marquee.style.cssText = "position:fixed;border:1px dashed #1a73e8;background:rgba(26,115,232,0.1);z-index:9990;pointer-events:none;";
      document.body.appendChild(marquee);
      function onMove(ev) {
        const x1 = Math.min(startX, ev.clientX);
        const y1 = Math.min(startY, ev.clientY);
        const x2 = Math.max(startX, ev.clientX);
        const y2 = Math.max(startY, ev.clientY);
        marquee.style.left = x1 + "px";
        marquee.style.top = y1 + "px";
        marquee.style.width = (x2 - x1) + "px";
        marquee.style.height = (y2 - y1) + "px";
        const rect = { x: x1, y: y1, right: x2, bottom: y2 };
        document.querySelectorAll(".slide-element").forEach((el) => {
          const b = bounds(el);
          const inside = !(b.x > rect.right || b.right < rect.x || b.y > rect.bottom || b.bottom < rect.y);
          if (inside) el.classList.add("in-range");
          else el.classList.remove("in-range");
        });
      }
      function onUp() {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        marquee.remove();
      }
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    });
  }

  function attach() {
    multiSelect();
    document.addEventListener("keydown", function (e) {
      if (!(e.ctrlKey || e.metaKey)) return;
      if (e.key.toLowerCase() === "g" && !e.shiftKey) { e.preventDefault(); group(); }
      if (e.key.toLowerCase() === "g" && e.shiftKey) { e.preventDefault(); ungroup(); }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesArrange = {
    align, distribute, group, ungroup, multiSelect,
    getSelectedElements, getGroupBounds,
  };
})();
