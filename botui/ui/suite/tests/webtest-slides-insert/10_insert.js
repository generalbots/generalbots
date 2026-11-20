/* Slides Insert hub — RUNTIME invariants from the four shipped IIFEs (#2428).
 * STATIC blocks only assert source/catalog facts; they never claim a runtime
 * result. Fake DOM nodes are host stubs, never copies of production logic. */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var env = h.env;
var s = env.sandbox;
var hub = env.hub;
var place = env.place;
var win = env.win;
var doc = env.doc;

function itemIds(items) {
  return items.map(function (item) { return item.id || item.dataset.itemId; }).join(",");
}

function categoryButtons() {
  return hub.cats.children.map(function (btn) { return btn.dataset.category; }).join(",");
}

function selectedCards() {
  return hub.cards().filter(function (card) {
    return card.getAttribute("aria-selected") === "true";
  }).map(function (card) { return card.dataset.itemId; }).join(",");
}

// ── RUNTIME: shipped surface + catalog ────────────────────────────────────
test("RUNTIME: four shipped modules expose their real helper surfaces", function () {
  ["siT", "siFind", "siLabel", "siMatches", "siItemsFor", "siRemember",
    "siFavorites"].forEach(function (name) {
    assert(typeof s[name] === "function", "catalog helper " + name + " missing");
  });
  ["open", "close", "build", "renderCategories", "renderGrid", "select",
    "doInsert", "onKeyDown", "onCardKey", "previewGlyph", "renderPreview",
    "renderOptions", "optionValues", "focusables", "adoptLegacy"].forEach(function (name) {
    assert(typeof hub[name] === "function", "hub method " + name + " missing");
  });
  ["place", "payload", "point", "typeOf", "post", "resolveDrive", "reuseItems",
    "reusePayload", "applyImageExtras", "diagram", "readImages"].forEach(function (name) {
    assert(typeof place[name] === "function", "placement helper " + name + " missing");
  });
});

test("RUNTIME: category rail is the shipped 13-entry ordered set", function () {
  assert(s.SI_CATEGORIES.length === 13, "unexpected category count");
  assert(s.SI_CATEGORIES.map(function (c) { return c.id; }).join(",") ===
    "recent,favorites,layouts,text,media,shapes,icons,charts,tables,diagrams,equations,fields,reuse",
    "category order changed");
  s.SI_CATEGORIES.forEach(function (c) {
    assert(c.key && c.fallback && c.id, "category missing identity/i18n metadata");
  });
});

test("RUNTIME: catalog has 55 unique ids across 11 content categories", function () {
  assert(s.SI_ITEMS.length === 55, "unexpected insertable count: " + s.SI_ITEMS.length);
  var ids = {};
  s.SI_ITEMS.forEach(function (item) {
    assert(!ids[item.id], "duplicate insertable id: " + item.id);
    ids[item.id] = true;
    assert(item.key && item.fallback && item.kind && item.glyph,
      "incomplete insertable metadata: " + item.id);
    assert(s.SI_CATEGORIES.some(function (c) { return c.id === item.category; }),
      "unknown category on " + item.id);
  });
});

test("RUNTIME: layouts are exactly the six server-supported layout ids", function () {
  var layouts = s.SI_ITEMS.filter(function (item) { return item.kind === "layout"; });
  assert(itemIds(layouts) ===
    "layout-title,layout-content,layout-two-column,layout-section,layout-image,layout-blank",
    "layout catalog ids changed");
  assert(layouts.map(function (i) { return i.layout; }).join(",") ===
    "title,content,two_column,section,image,blank", "server layout values changed");
  assert(layouts[0].w === 960 && layouts[0].h === 540, "title layout geometry changed");
});

test("RUNTIME: labels resolve through the injected i18t stub, not hardcoded text", function () {
  var item = s.siFind("layout-two-column");
  assert(s.siLabel(item) === "<slides-insert-layout-two-column>", "label ignored gbT key");
  assert(s.siT("x-unknown-key", "<fallback>") === "<x-unknown-key>", "siT fallback contract changed");
});

test("RUNTIME: search is case-insensitive, trimmed and spans id/keywords/label", function () {
  assert(itemIds(s.siItemsFor("layouts", " TWO  ")) === "layout-two-column",
    "trim/case-insensitive id search failed");
  assert(itemIds(s.siItemsFor("media", "drive")) === "media-drive",
    "keyword search failed");
  assert(itemIds(s.siItemsFor("layouts", "zzzz-no-match")) === "", "no-match search failed");
  assert(s.siItemsFor("layouts", "").length === 6, "empty query must not filter");
});

test("RUNTIME: recent ids are de-duplicated, LRU-ordered and capped at 12", function () {
  env.reset();
  ["layout-title", "layout-content", "layout-title"].forEach(function (id) { s.siRemember(id); });
  assert(s.siRecentIds().join(",") === "layout-title,layout-content", "recent order/dedupe wrong");
  var valid = ["rectangle", "rounded", "circle", "triangle", "star", "arrow", "line", "diamond"]
    .map(function (id) { return "shape-" + id; });
  valid = valid.concat(["check", "star", "heart", "arrow", "cloud"]
    .map(function (id) { return "icon-" + id; }));
  valid.push("chart-bar");
  valid.forEach(function (id) { s.siRemember(id); });
  assert(s.siRecentIds().length === 12, "recent cap not applied");
  assert(s.siRecentIds()[0] === "chart-bar", "newest recent not first");
  assert(itemIds(s.siItemsFor("recent", "")) === s.siRecentIds().join(","),
    "recent grid disagrees with stored order");
});

test("RUNTIME: unknown recent ids are ignored and favourites toggle both ways", function () {
  env.reset();
  win.localStorage.setItem("slides_insert_recent", JSON.stringify(["missing-id", "text-body"]));
  assert(itemIds(s.siItemsFor("recent", "")) === "text-body", "unknown recent id leaked");
  assert(s.siToggleFavorite("chart-bar") === true, "first favourite toggle should add");
  assert(s.siToggleFavorite("chart-bar") === false, "second favourite toggle should remove");
  s.siToggleFavorite("chart-bar");
  assert(itemIds(s.siItemsFor("favorites", "")) === "chart-bar", "favourite grid wrong");
  assert(itemIds(s.siItemsFor("favorites", "pie")) === "", "favourite search not applied");
  assert(s.siItemsFor("reuse", "").length === 0, "reuse must start empty in the catalog");
});

// ── RUNTIME: dialog shell, grid, keyboard ─────────────────────────────────
test("RUNTIME: open() builds an accessible, focus-managed dialog", function () {
  env.reset();
  var host = doc.createElement("div");
  host.id = "slides-modal-container";
  doc.body.appendChild(host);
  var opener = doc.createElement("button");
  hub.open(opener);
  assert(hub.dialog && hub.dialog.parentNode === host, "dialog not appended to modal host");
  var dialog = hub.dialog.querySelector(".sl-insert-dialog");
  assert(dialog.getAttribute("role") === "dialog", "missing dialog role");
  assert(dialog.getAttribute("aria-modal") === "true", "missing aria-modal");
  assert(dialog.getAttribute("aria-labelledby") === "sl-insert-title", "missing title link");
  assert(hub.status.getAttribute("role") === "status" &&
    hub.status.getAttribute("aria-live") === "polite", "status region not announced");
  assert(hub.insertBtn.disabled === true, "insert must start disabled");
  assert(hub.category === "recent" && hub.query === "", "open state not reset");
  assert(doc.activeElement === hub.search, "open did not focus search");
  assert(h.text(hub.status) === "<slides-insert-status-ready>", "wrong ready status");
  assert(categoryButtons().split(",").length === 13, "rail not rendered");
  assert(hub.cats.querySelectorAll('[aria-selected="true"]').length === 1, "one tab must be active");
  hub.close();
  assert(hub.dialog === null && doc.activeElement === opener, "close did not restore focus");
});

test("RUNTIME: setCategory updates tab state and renders catalog order", function () {
  env.reset();
  hub.open();
  hub.setCategory("layouts");
  assert(hub.category === "layouts", "category not changed");
  assert(hub.cats.querySelector('[data-category="layouts"]').getAttribute("aria-selected") === "true",
    "active tab not marked");
  assert(hub.cats.querySelector('[data-category="recent"]').tabIndex === -1, "inactive tab still tabbable");
  assert(itemIds(s.SI_ITEMS.filter(function (i) { return i.category === "layouts"; })) ===
    itemIds(hub.cards()), "grid order differs from catalog order");
  hub.close();
});

test("RUNTIME: selecting a layout previews it, enables insert and marks one card", function () {
  env.reset();
  hub.open();
  hub.setCategory("layouts");
  assert(hub.cards().every(function (card) { return card.getAttribute("aria-selected") === "false"; }),
    "cards must start unselected");
  hub.select("layout-two-column");
  assert(hub.selectedId === "layout-two-column" && hub.insertBtn.disabled === false,
    "selection did not enable insert");
  assert(selectedCards() === "layout-two-column", "selection marker wrong");
  assert(hub.preview.querySelector(".sl-insert-mini-two_column") !== null, "layout preview missing");
  assert(hub.options.querySelector(".sl-insert-layout-note") !== null, "layout note missing");
  assert(h.text(hub.status) === "<slides-insert-status-selected>", "selected status missing");
  hub.close();
});

test("RUNTIME: search input and search-keyboard handoff filter the live grid", function () {
  env.reset();
  hub.open();
  hub.setCategory("layouts");
  hub.search.value = "two";
  h.fire(hub.search, "input", {});
  assert(itemIds(hub.cards()) === "layout-two-column", "input event did not filter grid");
  var moved = h.fire(hub.search, "keydown", { key: "ArrowDown" });
  assert(moved.defaultPrevented === true && doc.activeElement === hub.cards()[0],
    "ArrowDown did not move focus to first card");
  hub.close();
});

test("RUNTIME: card arrow/Home/End/Enter keys operate on the real card list", function () {
  env.reset();
  hub.open();
  hub.setCategory("text");
  var cards = hub.cards();
  h.fire(cards[0], "keydown", { key: "ArrowDown" });
  assert(doc.activeElement === cards[1], "ArrowDown failed");
  h.fire(cards[1], "keydown", { key: "ArrowLeft" });
  assert(doc.activeElement === cards[0], "ArrowLeft failed");
  h.fire(cards[0], "keydown", { key: "End" });
  assert(doc.activeElement === cards[cards.length - 1], "End failed");
  h.fire(cards[cards.length - 1], "keydown", { key: "Home" });
  assert(doc.activeElement === cards[0], "Home failed");
  h.fire(cards[2], "keydown", { key: "Enter" });
  assert(hub.selectedId === cards[2].dataset.itemId, "Enter did not select the card");
  h.fire(cards[2], "keydown", { key: " " });
  assert(hub.selectedId === cards[2].dataset.itemId, "Space did not select the card");
  hub.close();
});

test("RUNTIME: category rail supports horizontal keyboard navigation", function () {
  env.reset();
  hub.open();
  h.fire(hub.cats.querySelector('[data-category="recent"]'), "keydown", { key: "ArrowRight" });
  assert(hub.category === "favorites", "ArrowRight did not advance category");
  h.fire(hub.cats.querySelector('[data-category="favorites"]'), "keydown", { key: "ArrowLeft" });
  assert(hub.category === "recent", "ArrowLeft did not go back");
  hub.close();
});

test("RUNTIME: Escape, backdrop click and Tab trap are active on the real dialog", function () {
  env.reset();
  hub.open();
  var focusables = hub.focusables();
  assert(focusables.length > 2, "focus trap found too few controls");
  var first = focusables[0];
  var last = focusables[focusables.length - 1];
  var dialogNode = hub.dialog.querySelector(".sl-insert-dialog");
  doc.activeElement = last;
  h.fire(dialogNode, "keydown", { key: "Tab" });
  assert(doc.activeElement === first, "forward Tab did not wrap");
  doc.activeElement = first;
  h.fire(dialogNode, "keydown", { key: "Tab", shiftKey: true });
  assert(doc.activeElement === last, "Shift+Tab did not wrap");
  var escape = h.fire(dialogNode, "keydown", { key: "Escape" });
  assert(escape.defaultPrevented === true && hub.dialog === null, "Escape did not close");
  hub.open();
  h.fire(hub.dialog, "pointerdown", { target: hub.dialog });
  assert(hub.dialog === null, "backdrop pointerdown did not close");
});

test("RUNTIME: media options, live preview and optionValues use real field handlers", function () {
  env.reset();
  hub.open();
  hub.setCategory("media");
  hub.select("media-image");
  ["url", "drivePath", "file", "alt", "fit"].forEach(function (name) {
    assert(hub.options.querySelector('[data-si-name="' + name + '"]') !== null,
      "missing option field " + name);
  });
  var fit = hub.options.querySelector('[data-si-name="fit"]');
  assert(fit.value === "contain" && fit.children.length === 3, "fit choices wrong");
  assert(hub.options.querySelector(".sl-insert-resolve") !== null, "Drive attach action missing");
  var url = hub.options.querySelector('[data-si-name="url"]');
  url.value = "http://localhost/pic.png";
  h.fire(url, "input", {});
  assert(hub.preview.querySelector(".sl-insert-preview-image") !== null, "live image preview missing");
  assert(hub.preview.querySelector(".sl-insert-preview-image").src === url.value, "preview src stale");
  assert(hub.optionValues().src === url.value, "optionValues ignored optionCache");
  hub.close();
});

test("RUNTIME: init adopts the legacy button and binds the Ctrl+Alt+I shortcut", function () {
  env.reset();
  var content = doc.createElement("div");
  content.id = "slides-content";
  doc.body.appendChild(content);
  var legacy = doc.createElement("button");
  legacy.setAttribute("hx-post", "/suite/slides/modals/insert-element");
  content.appendChild(legacy);
  env.runInit();
  assert(legacy.getAttribute("hx-post") === null && legacy.getAttribute("hx-vals") === null,
    "legacy post attributes were not removed");
  assert(h.text(legacy) === "<slides-insert-open>", "legacy label not localised");
  h.fire(legacy, "click", {});
  assert(hub.dialog !== null, "adopted legacy button did not open the hub");
  h.fire(doc, "keydown", { key: "i", ctrlKey: true, altKey: true });
  assert(hub.dialog === null, "Ctrl+Alt+I did not close the open hub");
  h.fire(doc, "keydown", { key: "i", ctrlKey: true, altKey: true });
  assert(hub.dialog !== null, "Ctrl+Alt+I did not reopen the hub");
  hub.close();
});

// ── RUNTIME: placement helpers (pure parts) ───────────────────────────────
test("RUNTIME: typeOf maps insert kinds onto the existing element API", function () {
  var expected = {
    image: "image", video: "image", chart: "chart", table: "table",
    shape: "shape", diagram: "shape", text: "text", icon: "text",
    equation: "text", field: "text", layout: "text", reuse: "text",
  };
  Object.keys(expected).forEach(function (kind) {
    assert(place.typeOf({ kind: kind }) === expected[kind], "typeOf(" + kind + ") changed");
  });
});

test("RUNTIME: point() cascades deterministically and honours explicit drops", function () {
  place._cascade = 0;
  var a = place.point({ w: 100, h: 50 });
  var b = place.point({ w: 100, h: 50 });
  place._cascade = 0;
  var again = place.point({ w: 100, h: 50 });
  assert(JSON.stringify(a) === JSON.stringify(again), "cascade is not reproducible");
  assert(b.x === a.x + 16 && b.y === a.y + 14, "cascade step changed");
  assert(JSON.stringify(place.point({ w: 100, h: 50 }, { x: 12.6, y: 8.4 })) ===
    JSON.stringify({ x: 13, y: 8 }), "explicit drop was not rounded");
});

test("RUNTIME: payload carries geometry, styling and type-specific fields", function () {
  env.reset();
  var text = place.payload(s.siFind("text-heading"), { text: "<body>" }, { x: 1, y: 2 });
  assert(text.element_type === "text" && text.text === "<body>" && text.font_size === 32,
    "text payload wrong");
  assert(text.font_weight === "bold" && text.x === 1 && text.y === 2 && text.rotation === 0,
    "text style/geometry wrong");
  var shape = place.payload(s.siFind("shape-star"), {});
  assert(shape.shape_type === "star" && shape.fill === "rgb(1, 2, 3)" && shape.color !== "",
    "shape payload wrong");
  var chart = place.payload(s.siFind("chart-pie"), {});
  assert(chart.chart_type === "pie" && JSON.parse(chart.data).datasets.length === 1,
    "chart payload wrong");
  var table = place.payload(s.siFind("table-grid"), { rows: "4", cols: "5" });
  assert(table.rows === "4" && table.cols === "5", "table dimensions not forwarded");
  var image = place.payload(s.siFind("media-image"), { src: "http://localhost/p.png", alt: "<alt>", fit: "crop" });
  assert(image.src && image.alt === "<alt>" && image.fit === "crop", "image fields not forwarded");
  assert(place.pendingImages[image.element_id].fit === "crop", "pending image extras not cached");
});

test("RUNTIME: image extras are cached locally and applied to the canvas image", function () {
  env.reset();
  var image = place.payload(s.siFind("media-image"), { src: "http://localhost/p.png", fit: "crop" });
  var stored = place.readImages();
  assert(stored[image.element_id] && stored[image.element_id].fit === "crop",
    "readImages did not see rememberImage write");
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  var holder = doc.createElement("div");
  holder.setAttribute("data-id", image.element_id);
  var img = doc.createElement("img");
  holder.appendChild(img);
  canvas.appendChild(holder);
  doc.body.appendChild(canvas);
  place.applyImageExtras();
  assert(img.getAttribute("alt") === "", "alt not applied");
  assert(img.style.objectFit === "cover" && img.style.transform === "scale(1.18)",
    "crop styling not applied");
  hub.close();
});

// ── STATIC: source/catalog facts, never runtime claims ────────────────────
test("STATIC: shipped insert modules use no CDN, innerHTML or legacy modal posts", function () {
  h.MODULE_RELS.forEach(function (rel) {
    var src = h.readSuite(rel);
    assert(src.indexOf("innerHTML") === -1, rel + " writes innerHTML");
    assert(!/<script|cdn\.|unpkg|jsdelivr|cdnjs/i.test(src), rel + " carries a remote asset");
    assert(src.indexOf('fetch("/suite/slides/modals/') === -1, rel + " posts to a legacy modal");
  });
});

test("STATIC: every shipped slides-insert key exists in all eight catalogs", function () {
  var used = {};
  h.MODULE_RELS.forEach(function (rel) {
    (h.readSuite(rel).match(/slides-insert-[a-z0-9-]+/g) || []).forEach(function (key) { used[key] = true; });
  });
  s.SI_CATEGORIES.forEach(function (c) { used[c.key] = true; });
  s.SI_ITEMS.forEach(function (item) {
    [item.key, item.formulaKey, item.valueKey].forEach(function (key) { if (key) used[key] = true; });
    (item.options || []).forEach(function (option) {
      if (option.key) used[option.key] = true;
      if (option.placeholderKey) used[option.placeholderKey] = true;
      (option.choices || []).forEach(function (choice) { used[choice[1]] = true; });
    });
  });
  var keys = Object.keys(used);
  assert(keys.length >= 120, "unexpectedly small insert key set: " + keys.length);
  ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"].forEach(function (locale) {
    var catalog = h.readRepo("botlib/locales/" + locale + "/slides-insert.ftl");
    keys.forEach(function (key) {
      assert(new RegExp("^" + key + "\\s*=", "m").test(catalog),
        locale + " missing catalog key " + key);
    });
  });
});

// ── STATIC SCOPE-GAPS: real, documented limits of the shipped code ────────
// These are source facts only. They are NOT runtime results and NOT claims
// that the corresponding issue acceptance item is complete.
test("STATIC SCOPE-GAP: legacy per-type modal removal is backend-deferred", function () {
  var actions = h.readSuite("slides/modules/12_insert_actions.js");
  assert(actions.indexOf("SlidesInsertHub.adoptLegacy = function") >= 0,
    "legacy adoption helper unexpectedly absent");
  assert(actions.indexOf("/suite/slides/modals/insert-element") >= 0,
    "legacy button selector unexpectedly absent");
  assert(actions.indexOf('fetch("/suite/slides/modals/') === -1,
    "shipped JS must never call a legacy modal endpoint");
});

test("STATIC SCOPE-GAP: image alt/fit/crop is a local cache until ElementStyle lands", function () {
  var source = h.readSuite("slides/modules/13_insert_place.js");
  assert(source.indexOf('imageStoreKey: "slides_insert_image_extras"') >= 0,
    "client-side image-extra cache disappeared");
  assert(source.indexOf("/api/slides/element/update") === -1,
    "no server-side style field is used by the shipped insert path");
});

test("STATIC SCOPE-GAP: layout previews are CSS miniatures, not rendered thumbnails", function () {
  var source = h.readSuite("slides/modules/11_insert_dialog.js");
  assert(source.indexOf("sl-insert-mini-layout") >= 0 &&
    source.indexOf("sl-insert-mini-a") >= 0, "miniature preview markup missing");
  assert(source.indexOf("<iframe") === -1 && source.indexOf("<canvas") === -1,
    "no rendered-slide preview engine is shipped");
});
