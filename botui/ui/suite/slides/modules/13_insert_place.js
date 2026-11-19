"use strict";
/* Slides Insert hub — placement through the existing Slides JSON APIs (#2428).
 * Nothing here adds a server endpoint: every object goes through element/add,
 * layouts through slide/add. Image alt/fit are applied client-side and cached
 * locally because the element API has no fields for them yet. */

var SlidesInsertPlace = {
  pendingImages: {},
  imageStoreKey: "slides_insert_image_extras",
  _cascade: 0,
};

SlidesInsertPlace.slide = function () {
  var canvas = document.getElementById("slide-canvas");
  return canvas ? (parseInt(canvas.dataset.slideId, 10) || 0) : 0;
};

SlidesInsertPlace.deck = function () {
  return typeof getSlidesPresentationId === "function"
    ? getSlidesPresentationId()
    : (window.__SLIDES_PRESENTATION_ID || "current");
};

SlidesInsertPlace.theme = function () {
  var theme = { primary: "#3b82f6", bg: "#f8fafc" };
  var canvas = document.getElementById("slide-canvas");
  if (canvas && canvas.dataset.bg) theme.bg = canvas.dataset.bg;
  var dot = document.querySelector(".sl-status-bar span span");
  if (dot && window.getComputedStyle) {
    var color = window.getComputedStyle(dot).color;
    if (color && color !== "rgba(0, 0, 0, 0)") theme.primary = color;
  }
  return theme;
};

SlidesInsertPlace.point = function (item, at) {
  if (at && isFinite(at.x) && isFinite(at.y)) return { x: Math.round(at.x), y: Math.round(at.y) };
  var w = item.w || 200;
  var h = item.h || 120;
  var sw = typeof SLIDE_W === "number" ? SLIDE_W : 960;
  var sh = typeof SLIDE_H === "number" ? SLIDE_H : 540;
  var n = (this._cascade = (this._cascade + 1) % 8);
  return {
    x: Math.max(0, Math.round((sw - w) / 2 - 40 + n * 16)),
    y: Math.max(0, Math.round((sh - h) / 2 - 30 + n * 14)),
  };
};

SlidesInsertPlace.post = function (path, payload) {
  var headers = { "Content-Type": "application/json" };
  try {
    var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
      window.localStorage.getItem("gb-access-token") ||
      window.sessionStorage.getItem("gb-access-token");
    if (token) headers.Authorization = "Bearer " + token;
  } catch (e) { /* storage disabled */ }
  return fetch(path, {
    method: "POST",
    headers: headers,
    credentials: "same-origin",
    body: JSON.stringify(payload),
  }).then(function (response) {
    if (!response.ok) throw new Error("HTTP " + response.status);
    return response.json().catch(function () { return {}; });
  });
};

SlidesInsertPlace.refresh = function (index) {
  if (typeof refreshSlideView === "function") refreshSlideView(index);
};

SlidesInsertPlace.newId = function () {
  return "el_" + Date.now() + "_" + Math.random().toString(36).slice(2, 6);
};

SlidesInsertPlace.typeOf = function (item) {
  if (item.kind === "image" || item.kind === "video") return "image";
  if (item.kind === "chart") return "chart";
  if (item.kind === "table") return "table";
  if (item.kind === "shape" || item.kind === "diagram") return "shape";
  return "text";
};

SlidesInsertPlace.fieldValue = function (item) {
  switch (item.field) {
    case "slide-number": return String(this.slide() + 1);
    case "date": return typeof window.gbFormatDate === "function" ? window.gbFormatDate(new Date()) : "";
    default: return siT(item.valueKey, item.fallback);
  }
};

SlidesInsertPlace.payload = function (item, options, at, overrides) {
  var theme = this.theme();
  var point = this.point(item, at);
  var payload = {
    presentation_id: this.deck(),
    slide_index: this.slide(),
    slide_id: String(this.slide()),
    element_id: this.newId(),
    element_type: this.typeOf(item),
    x: point.x,
    y: point.y,
    width: item.w || 200,
    height: item.h || 120,
    rotation: 0,
  };
  var text = "";
  if (item.kind === "text") text = options.text || siT("slides-insert-sample-text", "Your text here");
  else if (item.kind === "icon") text = options.text || siT(item.key, item.fallback);
  else if (item.kind === "equation") text = siT(item.formulaKey, item.formula);
  else if (item.kind === "field") text = this.fieldValue(item);
  else if (item.kind === "shape") text = options.text || "";
  if (text) payload.text = text;
  if (item.fontSize) payload.font_size = item.fontSize;
  if (item.weight) payload.font_weight = item.weight;
  if (item.style === "italic") payload.font_style = "italic";
  if (item.mono) payload.font_family = "monospace";
  if (item.shapeType) payload.shape_type = item.shapeType;
  if (item.kind === "shape" || item.kind === "diagram" || item.kind === "icon") {
    payload.fill = theme.primary;
    payload.color = "#ffffff";
  }
  if (item.kind === "chart") {
    payload.chart_type = item.chartType || "bar";
    payload.data = JSON.stringify({
      datasets: [{ label: "", data: [1, 2, 3], color: theme.primary }],
    });
  }
  if (item.kind === "table") {
    payload.rows = options.rows || 3;
    payload.cols = options.cols || 3;
  }
  if (item.kind === "image" || item.kind === "video") {
    payload.src = options.src || "";
    payload.alt = options.alt || "";
    payload.fit = options.fit || "contain";
    if (payload.src) {
      this.pendingImages[payload.element_id] = { alt: payload.alt, fit: payload.fit };
      this.rememberImage(payload.element_id, payload.alt, payload.fit);
    }
  }
  return Object.assign(payload, overrides || {});
};

SlidesInsertPlace.rememberImage = function (id, alt, fit) {
  var all = {};
  try { all = gbStoreGet(this.imageStoreKey, {}) || {}; } catch (e) { all = {}; }
  all[id] = { alt: alt || "", fit: fit || "contain" };
  try { window.localStorage.setItem(this.imageStoreKey, JSON.stringify(all)); } catch (e) { /* storage disabled */ }
};

SlidesInsertPlace.applyImageExtras = function () {
  var ids = Object.keys(this.pendingImages).concat(Object.keys(this.readImages()));
  var seen = {};
  ids.forEach(function (id) {
    if (seen[id]) return;
    seen[id] = true;
    var extra = SlidesInsertPlace.pendingImages[id] || SlidesInsertPlace.readImages()[id];
    if (!extra) return;
    var img = document.querySelector('#slide-canvas [data-id="' + id + '"] img');
    if (!img) return;
    img.setAttribute("alt", extra.alt || "");
    var fit = extra.fit || "contain";
    img.style.objectFit = fit === "contain" ? "contain" : "cover";
    img.style.objectPosition = "center";
    img.style.transform = fit === "crop" ? "scale(1.18)" : "none";
    img.style.transformOrigin = "center";
  });
};

SlidesInsertPlace.readImages = function () {
  try { return gbStoreGet(this.imageStoreKey, {}) || {}; }
  catch (e) { return {}; }
};

SlidesInsertPlace.diagram = function (item, options, at) {
  var kind = item.diagram;
  var layouts = {
    flow: [[0, 0], [210, 0], [420, 0]],
    timeline: [[0, 60], [180, 60], [360, 60]],
    org: [[200, 0], [40, 130], [360, 130]],
    mindmap: [[220, 80], [0, 0], [0, 160], [440, 0], [440, 160]],
  };
  var cells = layouts[kind] || layouts.flow;
  var origin = this.point(item, at);
  var self = this;
  return cells.map(function (cell, index) {
    var chip = { w: 150, h: 80, kind: "shape" };
    return self.post("/api/slides/element/add", self.payload(chip, {
      text: siT("slides-insert-diagram-step", "Step {n}", { n: index + 1 }),
    }, { x: origin.x + cell[0], y: origin.y + cell[1] }, {
      element_type: "shape",
      shape_type: kind === "timeline" && index === 0 ? "circle" : "rounded",
      width: 150,
      height: 80,
    }));
  });
};

SlidesInsertPlace.resolveDrive = function (path) {
  if (!path) return Promise.resolve("");
  if (/^https?:\/\//i.test(path)) return Promise.resolve(path);
  var headers = { "Content-Type": "application/json" };
  try {
    var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
      window.localStorage.getItem("gb-access-token") || "";
    if (token) headers.Authorization = "Bearer " + token;
  } catch (e) { /* storage disabled */ }
  var scope = "user";
  try { scope = window.sessionStorage.getItem("drive-scope") || "user"; } catch (e) { /* storage disabled */ }
  return fetch("/api/files/share-link", {
    method: "POST",
    headers: headers,
    credentials: "same-origin",
    body: JSON.stringify({ bucket: "", path: path, scope: scope }),
  }).then(function (response) {
    if (!response.ok) throw new Error("HTTP " + response.status);
    return response.json();
  }).then(function (data) {
    var url = String((data && data.url) || "");
    if (!url) return "";
    return /^https?:\/\//i.test(url) ? url : window.location.origin + url;
  });
};

SlidesInsertPlace.reuseItems = function () {
  var self = this;
  return fetch("/api/slides/load?id=" + encodeURIComponent(this.deck()), {
    headers: { Accept: "application/json" },
    credentials: "same-origin",
  }).then(function (response) {
    if (!response.ok) throw new Error("HTTP " + response.status);
    return response.json();
  }).then(function (deck) {
    var items = [];
    (deck && deck.slides || []).forEach(function (slide, index) {
      (slide.elements || []).forEach(function (element, at) {
        items.push({
          id: "reuse-" + index + "-" + at,
          category: "reuse",
          key: "slides-insert-reuse-item",
          fallback: "Reuse element",
          kind: "reuse",
          keywords: "reuse copy " + (element.element_type || ""),
          glyph: element.element_type || "text",
          source: element,
          slideNumber: index + 1,
          labelSuffix: (siT("slides-insert-reuse-slide", "Slide {n}", { n: index + 1 })),
        });
      });
    });
    return items;
  });
};

SlidesInsertPlace.reusePayload = function (item, at) {
  var source = item.source || {};
  var content = source.content || {};
  var style = source.style || {};
  var overrides = {
    element_type: source.element_type || "text",
    text: content.text || "",
    src: content.src || "",
    shape_type: content.shape_type || "",
    fill: style.fill || "",
    color: style.color || "",
    font_size: style.font_size || 0,
  };
  if (content.chart_data) {
    overrides.chart_type = content.chart_data.chart_type || "bar";
    overrides.data = JSON.stringify(content.chart_data);
  }
  if (content.table_data) {
    overrides.rows = content.table_data.rows || 3;
    overrides.cols = content.table_data.cols || 3;
    overrides.headers = (content.table_data.cells || [])
      .map(function (row) { return row[0] && row[0].content || ""; })
      .filter(Boolean).join(",");
  }
  return this.payload({ w: source.width || 200, h: source.height || 120, kind: "reuse" }, {}, at, overrides);
};

SlidesInsertPlace.place = function (item, options, at) {
  options = options || {};
  var self = this;
  if (item.kind === "layout") {
    return this.post("/api/slides/slide/add", {
      presentation_id: this.deck(),
      layout: item.layout || "content",
      position: this.slide() + 1,
    }).then(function () { self.refresh(self.slide() + 1); });
  }
  if (item.kind === "reuse") {
    return this.post("/api/slides/element/add", this.reusePayload(item, at))
      .then(function () { self.refresh(); });
  }
  if (item.kind === "diagram") {
    return Promise.all(this.diagram(item, options, at)).then(function () { self.refresh(); });
  }
  var src = options.src || "";
  if (options.drivePath && !src) {
    return this.resolveDrive(options.drivePath).then(function (url) {
      if (!url) throw new Error(siT("slides-insert-drive-link-failed", "Could not create a Drive link for this file."));
      options.src = url;
      return self.placeSingle(item, options, at);
    });
  }
  return this.placeSingle(item, options, at);
};

SlidesInsertPlace.placeSingle = function (item, options, at) {
  var self = this;
  return this.post("/api/slides/element/add", this.payload(item, options, at))
    .then(function () { self.refresh(); });
};
