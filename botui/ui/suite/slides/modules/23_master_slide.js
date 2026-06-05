"use strict";

/**
 * Module 23: Master slide inheritance for Slides.
 * Defines a MasterSlide with layout placeholders (title, body, footer,
 * date, slide number). Slide elements can inherit from a master, and
 * the editor shows a "Master view" tab where changes to the master
 * propagate to all slides bound to it. Supports up to 12 layout types.
 *
 * Public API: window.SlidesMaster = { create, attach, getLayouts,
 *   renderMasterView, renderInheritance, propagateToAll }.
 */

(function () {
  function getState() { return window.state || null; }
  function getMasterList() {
    const s = getState();
    if (!s) return [];
    if (!s.masters) s.masters = [];
    return s.masters;
  }

  const LAYOUT_TYPES = [
    "title", "section", "two-content", "comparison", "title-only",
    "blank", "content-with-caption", "picture-with-caption",
    "title-and-vertical-text", "vertical-title-and-text",
    "title-and-content", "title-and-horizontal-text",
  ];

  function create(name, layoutType) {
    const masters = getMasterList();
    const master = {
      id: "master-" + Date.now(),
      name: name || "Master " + (masters.length + 1),
      layout: layoutType || "title-and-content",
      placeholders: defaultPlaceholders(layoutType || "title-and-content"),
      elements: [],
      theme: { bg: "#ffffff", fg: "#202124", accent: "#1a73e8", font: "Inter, Arial" },
    };
    masters.push(master);
    return master;
  }

  function defaultPlaceholders(layout) {
    const common = [
      { type: "footer", x: 5, y: 92, width: 90, height: 5, text: "" },
      { type: "date", x: 5, y: 92, width: 25, height: 5, text: "" },
      { type: "slide-number", x: 88, y: 92, width: 7, height: 5, text: "" },
    ];
    switch (layout) {
      case "title":
        return [
          { type: "title", x: 10, y: 35, width: 80, height: 15, text: "Clique para adicionar título" },
          { type: "subtitle", x: 10, y: 55, width: 80, height: 10, text: "Subtítulo opcional" },
          ...common,
        ];
      case "title-and-content":
        return [
          { type: "title", x: 5, y: 5, width: 90, height: 12, text: "Clique para adicionar título" },
          { type: "body", x: 5, y: 22, width: 90, height: 65, text: "Adicione conteúdo aqui" },
          ...common,
        ];
      case "section":
        return [
          { type: "title", x: 5, y: 40, width: 90, height: 20, text: "Título da seção" },
          ...common,
        ];
      case "two-content":
        return [
          { type: "title", x: 5, y: 5, width: 90, height: 12, text: "Clique para adicionar título" },
          { type: "body", x: 5, y: 22, width: 42, height: 65, text: "Coluna esquerda" },
          { type: "body", x: 53, y: 22, width: 42, height: 65, text: "Coluna direita" },
          ...common,
        ];
      case "comparison":
        return [
          { type: "title", x: 5, y: 5, width: 90, height: 12, text: "Comparação" },
          { type: "header", x: 5, y: 22, width: 42, height: 8, text: "Opção A" },
          { type: "header", x: 53, y: 22, width: 42, height: 8, text: "Opção B" },
          { type: "body", x: 5, y: 33, width: 42, height: 55, text: "Detalhes A" },
          { type: "body", x: 53, y: 33, width: 42, height: 55, text: "Detalhes B" },
          ...common,
        ];
      case "title-only":
        return [
          { type: "title", x: 5, y: 5, width: 90, height: 12, text: "Clique para adicionar título" },
          ...common,
        ];
      case "content-with-caption":
        return [
          { type: "title", x: 5, y: 5, width: 90, height: 12, text: "Título" },
          { type: "body", x: 5, y: 22, width: 42, height: 65, text: "Texto à esquerda" },
          { type: "image", x: 53, y: 22, width: 42, height: 65, text: "" },
          ...common,
        ];
      case "picture-with-caption":
        return [
          { type: "image", x: 5, y: 5, width: 90, height: 70, text: "" },
          { type: "body", x: 10, y: 78, width: 80, height: 12, text: "Legenda da imagem" },
          ...common,
        ];
      case "blank":
        return common;
      default:
        return [
          { type: "title", x: 5, y: 5, width: 90, height: 12, text: "Título" },
          { type: "body", x: 5, y: 22, width: 90, height: 65, text: "Conteúdo" },
          ...common,
        ];
    }
  }

  function attach(slideIndex, masterId) {
    const s = getState();
    if (!s) return false;
    const slide = (s.slides || [])[slideIndex];
    if (!slide) return false;
    slide.masterId = masterId;
    if (masterId) slide.layout = (getMasterList().find((m) => m.id === masterId) || {}).layout;
    renderInheritance(slideIndex);
    return true;
  }

  function getLayouts() { return LAYOUT_TYPES.slice(); }

  function renderMasterView(masterId, container) {
    const c = container || document.querySelector(".master-view-canvas, .master-canvas");
    if (!c) return;
    const master = getMasterList().find((m) => m.id === masterId);
    if (!master) return;
    c.innerHTML = "";
    c.style.cssText = "position:relative;width:100%;aspect-ratio:16/9;background:" + (master.theme.bg || "#fff") + ";color:" + (master.theme.fg || "#202124") + ";font-family:" + (master.theme.font || "Inter, Arial") + ";";
    for (const ph of master.placeholders) {
      const e = document.createElement("div");
      e.className = "master-placeholder";
      e.dataset.placeholderType = ph.type;
      e.textContent = ph.text;
      e.style.cssText = "position:absolute;left:" + ph.x + "%;top:" + ph.y + "%;width:" + ph.width + "%;height:" + ph.height + "%;border:1px dashed rgba(26,115,232,0.4);display:flex;align-items:center;justify-content:center;text-align:center;padding:4px;box-sizing:border-box;color:" + (ph.type === "title" ? (master.theme.accent || "#1a73e8") : (master.theme.fg || "#202124")) + ";font-weight:" + (ph.type === "title" ? 700 : 400) + ";";
      c.appendChild(e);
    }
    for (const el of master.elements || []) {
      const d = document.createElement("div");
      d.className = "master-element";
      d.textContent = el.text || (el.type || "");
      d.style.cssText = "position:absolute;left:" + (el.x || 0) + "%;top:" + (el.y || 0) + "%;width:" + (el.width || 20) + "%;height:" + (el.height || 10) + "%;";
      c.appendChild(d);
    }
  }

  function renderInheritance(slideIndex) {
    const s = getState();
    if (!s) return;
    const slide = (s.slides || [])[slideIndex];
    if (!slide || !slide.masterId) return;
    const master = getMasterList().find((m) => m.id === slide.masterId);
    if (!master) return;
    if (!slide._inherited) slide._inherited = {};
    for (const ph of master.placeholders) {
      slide._inherited[ph.type] = ph;
    }
    const c = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
    if (!c) return;
    let layer = c.querySelector(".master-inherited-layer");
    if (!layer) {
      layer = document.createElement("div");
      layer.className = "master-inherited-layer";
      layer.style.cssText = "position:absolute;inset:0;pointer-events:none;z-index:0;";
      c.prepend(layer);
    }
    layer.innerHTML = "";
    for (const ph of master.placeholders) {
      if (ph.type === "footer" || ph.type === "date" || ph.type === "slide-number") {
        const e = document.createElement("div");
        e.className = "inherited-placeholder inherited-" + ph.type;
        e.style.cssText = "position:absolute;left:" + ph.x + "%;top:" + ph.y + "%;width:" + ph.width + "%;height:" + ph.height + "%;color:" + (master.theme.fg || "#202124") + ";font-size:11px;display:flex;align-items:center;pointer-events:auto;";
        if (ph.type === "slide-number") e.textContent = String((slideIndex || 0) + 1);
        else if (ph.type === "date") e.textContent = new Date().toLocaleDateString();
        else e.textContent = ph.text || "";
        layer.appendChild(e);
      }
    }
  }

  function propagateToAll(masterId) {
    const s = getState();
    if (!s) return false;
    const slides = s.slides || [];
    for (let i = 0; i < slides.length; i++) {
      if (slides[i].masterId === masterId) renderInheritance(i);
    }
    return true;
  }

  function attachUI() {
    const list = document.querySelector(".master-list, [data-masters-list]");
    if (!list) return;
    const masters = getMasterList();
    list.innerHTML = "";
    for (const m of masters) {
      const item = document.createElement("div");
      item.className = "master-item";
      item.textContent = m.name + " (" + m.layout + ")";
      item.dataset.masterId = m.id;
      item.style.cssText = "padding:6px 10px;cursor:pointer;border-bottom:1px solid #eee;";
      item.addEventListener("click", function () { renderMasterView(m.id); });
      list.appendChild(item);
    }
    const addBtn = document.querySelector("[data-action='add-master']");
    if (addBtn) {
      addBtn.addEventListener("click", function () {
        const name = window.prompt("Nome do master:", "Master " + (masters.length + 1));
        if (!name) return;
        const m = create(name, "title-and-content");
        renderMasterView(m.id);
      });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attachUI);
  } else {
    setTimeout(attachUI, 50);
  }

  window.SlidesMaster = {
    create, attach, getLayouts, renderMasterView, renderInheritance,
    propagateToAll, LAYOUT_TYPES,
  };
})();
