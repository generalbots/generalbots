"use strict";
/* Slides Insert hub — catalog of insertables and local preferences (#2428).
 * Every visible label is resolved through gbT; keywords are search metadata,
 * not UI chrome. Geometry defaults are canvas pixels at scale 1. */

function siT(key, fallback, params) {
  return typeof window.gbT === "function"
    ? window.gbT(key, fallback, params)
    : fallback;
}

var SI_CATEGORIES = [
  { id: "recent", key: "slides-insert-recent", fallback: "Recent" },
  { id: "favorites", key: "slides-insert-favorites", fallback: "Favourites" },
  { id: "layouts", key: "slides-insert-cat-layouts", fallback: "Layouts" },
  { id: "text", key: "slides-insert-cat-text", fallback: "Text" },
  { id: "media", key: "slides-insert-cat-media", fallback: "Media" },
  { id: "shapes", key: "slides-insert-cat-shapes", fallback: "Shapes" },
  { id: "icons", key: "slides-insert-cat-icons", fallback: "Icons" },
  { id: "charts", key: "slides-insert-cat-charts", fallback: "Charts" },
  { id: "tables", key: "slides-insert-cat-tables", fallback: "Tables" },
  { id: "diagrams", key: "slides-insert-cat-diagrams", fallback: "Diagrams" },
  { id: "equations", key: "slides-insert-cat-equations", fallback: "Equations" },
  { id: "fields", key: "slides-insert-cat-fields", fallback: "Fields" },
  { id: "reuse", key: "slides-insert-cat-reuse", fallback: "Reuse from deck" },
];

/* Local, vendored SVG path data — no CDN, no remote URL. */
var SI_GLYPHS = {
  check: "M20 6 9 17l-5-5",
  star: "m12 2 3.1 6.3 6.9 1-5 4.9 1.2 6.9L12 17.8 5.8 21l1.2-6.9-5-4.9 6.9-1z",
  heart: "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8z",
  arrow: "M5 12h14m-6-6 6 6-6 6",
  cloud: "M17.5 19a4.5 4.5 0 0 0 .5-9 6 6 0 0 0-11.6 1.5A3.8 3.8 0 0 0 7 19z",
  lock: "M5 11h14v10H5zM8 11V7a4 4 0 0 1 8 0v4",
  mail: "M3 5h18v14H3zM3 6l9 7 9-7",
  calendar: "M4 5h16v16H4zM8 3v4m8-4v4M4 10h16",
  user: "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 21a8 8 0 0 1 16 0",
  chart: "M4 20V10m6 10V4m6 16v-7m4 7H2",
  folder: "M3 6h6l2 2h10v12H3z",
  bolt: "m13 2-9 12h7l-1 8 9-12h-7z",
};

var SI_ITEMS = [];
function siAdd(id, category, key, fallback, kind, keywords, extra) {
  SI_ITEMS.push(Object.assign({
    id: id, category: category, key: key, fallback: fallback,
    kind: kind, keywords: keywords || "",
  }, extra || {}));
}

/* Layouts. The server accepts exactly these six ids (botslides::layouts). */
siAdd("layout-title", "layouts", "slides-insert-layout-title", "Title slide", "layout", "title cover", { glyph: "title", w: 960, h: 540, layout: "title" });
siAdd("layout-content", "layouts", "slides-insert-layout-content", "Title and content", "layout", "body content", { glyph: "content", layout: "content" });
siAdd("layout-two-column", "layouts", "slides-insert-layout-two-column", "Two columns", "layout", "two column split", { glyph: "two-col", layout: "two_column" });
siAdd("layout-section", "layouts", "slides-insert-layout-section", "Section header", "layout", "section divider", { glyph: "section", layout: "section" });
siAdd("layout-image", "layouts", "slides-insert-layout-image", "Image highlight", "layout", "photo picture", { glyph: "image", layout: "image" });
siAdd("layout-blank", "layouts", "slides-insert-layout-blank", "Blank", "layout", "empty", { glyph: "blank", layout: "blank" });

/* Text. Options are rendered data-driven by the preview pane. */
var SI_TEXT_OPTIONS = [
  { type: "textarea", name: "text", key: "slides-insert-text-label", fallback: "Text", placeholderKey: "slides-insert-text-placeholder", placeholderFallback: "Type text to insert", value: "" },
];
siAdd("text-body", "text", "slides-insert-text-body", "Body text", "text", "paragraph body", { glyph: "text", w: 360, h: 90, fontSize: 18, options: SI_TEXT_OPTIONS });
siAdd("text-heading", "text", "slides-insert-text-heading", "Heading", "text", "title heading", { glyph: "heading", w: 480, h: 80, fontSize: 32, weight: "bold", options: SI_TEXT_OPTIONS });
siAdd("text-caption", "text", "slides-insert-text-caption", "Caption", "text", "small note", { glyph: "caption", w: 260, h: 60, fontSize: 14, options: SI_TEXT_OPTIONS });
siAdd("text-quote", "text", "slides-insert-text-quote", "Quote", "text", "blockquote citation", { glyph: "quote", w: 420, h: 110, fontSize: 22, style: "italic", options: SI_TEXT_OPTIONS });
siAdd("text-code", "text", "slides-insert-text-code", "Code block", "text", "source monospace", { glyph: "code", w: 420, h: 150, fontSize: 14, mono: true, options: SI_TEXT_OPTIONS });

/* Media. Drive paths are resolved to a public link with the existing Files API. */
var SI_IMAGE_OPTIONS = [
  { type: "url", name: "url", key: "slides-insert-url-label", fallback: "Image URL", placeholderKey: "slides-insert-url-placeholder", placeholderFallback: "https://example.com/image.png", value: "" },
  { type: "drive", name: "drivePath", key: "slides-insert-drive-label", fallback: "Drive file path", placeholderKey: "slides-insert-drive-placeholder", placeholderFallback: "/folder/photo.png", value: "" },
  { type: "file", name: "file", key: "slides-insert-file-label", fallback: "Choose an image file", value: "" },
  { type: "text", name: "alt", key: "slides-insert-alt-label", fallback: "Alt text (description)", placeholderKey: "slides-insert-alt-placeholder", placeholderFallback: "Describe the image for screen readers", value: "" },
  { type: "select", name: "fit", key: "slides-insert-fit-label", fallback: "Image fit", value: "contain", choices: [
    ["contain", "slides-insert-fit-contain", "Fit"],
    ["cover", "slides-insert-fit-cover", "Fill"],
    ["crop", "slides-insert-fit-crop", "Crop"],
  ] },
];
siAdd("media-image", "media", "slides-insert-media-url", "Image from URL", "image", "photo picture url", { glyph: "image", w: 420, h: 260, options: SI_IMAGE_OPTIONS });
siAdd("media-drive", "media", "slides-insert-media-drive", "Image from Drive", "image", "drive file storage", { glyph: "folder", w: 420, h: 260, options: SI_IMAGE_OPTIONS });
siAdd("media-upload", "media", "slides-insert-media-upload", "Upload image", "image", "upload local file", { glyph: "upload", w: 420, h: 260, options: SI_IMAGE_OPTIONS });
siAdd("media-video", "media", "slides-insert-media-video", "Video from URL", "video", "movie mp4 link", { glyph: "video", w: 480, h: 270, options: SI_IMAGE_OPTIONS });

/* Shapes use the existing shape_type field of /api/slides/element/add. */
[["rectangle", "slides-insert-shape-rectangle", "Rectangle", "rect square"],
 ["rounded", "slides-insert-shape-rounded", "Rounded rectangle", "rounded corners"],
 ["circle", "slides-insert-shape-circle", "Ellipse", "circle oval"],
 ["triangle", "slides-insert-shape-triangle", "Triangle", "triangle"],
 ["star", "slides-insert-shape-star", "Star", "star"],
 ["arrow", "slides-insert-shape-arrow", "Arrow", "arrow right"],
 ["line", "slides-insert-shape-line", "Line", "line rule"],
 ["diamond", "slides-insert-shape-diamond", "Diamond", "diamond rhombus"],
].forEach(function (s) {
  siAdd("shape-" + s[0], "shapes", s[1], s[2], "shape", s[3], { glyph: s[0], w: 180, h: 110, shapeType: s[0] });
});

/* Icons are local SVG glyphs inserted as text so the server needs no new type. */
[["check", "slides-insert-icon-check", "Check", "check tick ok"],
 ["star", "slides-insert-icon-star", "Star icon", "star favourite"],
 ["heart", "slides-insert-icon-heart", "Heart", "heart like"],
 ["arrow", "slides-insert-icon-arrow", "Arrow right", "arrow next"],
 ["cloud", "slides-insert-icon-cloud", "Cloud", "cloud storage"],
 ["lock", "slides-insert-icon-lock", "Lock", "lock secure"],
 ["mail", "slides-insert-icon-mail", "Mail", "mail email"],
 ["calendar", "slides-insert-icon-calendar", "Calendar", "calendar date"],
 ["user", "slides-insert-icon-user", "User", "user person"],
 ["chart", "slides-insert-icon-chart", "Chart icon", "chart graph"],
 ["folder", "slides-insert-icon-folder", "Folder", "folder files"],
 ["bolt", "slides-insert-icon-bolt", "Bolt", "bolt flash"],
].forEach(function (s) {
  siAdd("icon-" + s[0], "icons", s[1], s[2], "icon", s[3], { glyph: s[0], w: 72, h: 72, fontSize: 44 });
});

/* Charts. */
[["bar", "slides-insert-chart-bar", "Bar chart", "bar columns"],
 ["line", "slides-insert-chart-line", "Line chart", "line trend"],
 ["pie", "slides-insert-chart-pie", "Pie chart", "pie circle"],
 ["doughnut", "slides-insert-chart-doughnut", "Doughnut chart", "donut ring"],
 ["area", "slides-insert-chart-area", "Area chart", "area fill"],
].forEach(function (s) {
  siAdd("chart-" + s[0], "charts", s[1], s[2], "chart", s[3], { glyph: s[0], w: 380, h: 260, chartType: s[0] });
});

var SI_TABLE_OPTIONS = [
  { type: "number", name: "rows", key: "slides-insert-rows-label", fallback: "Rows", value: 3, min: 1, max: 20 },
  { type: "number", name: "cols", key: "slides-insert-cols-label", fallback: "Columns", value: 3, min: 1, max: 10 },
];
siAdd("table-grid", "tables", "slides-insert-table-grid", "Table", "table", "grid rows columns", { glyph: "table", w: 420, h: 220, options: SI_TABLE_OPTIONS });

/* Diagrams are composed from several shape elements in one placement. */
[["flow", "slides-insert-diagram-flow", "Flow chart", "flow process steps"],
 ["timeline", "slides-insert-diagram-timeline", "Timeline", "timeline dates"],
 ["org", "slides-insert-diagram-org", "Org chart", "organisation hierarchy"],
 ["mindmap", "slides-insert-diagram-mindmap", "Mind map", "mindmap branches"],
].forEach(function (s) {
  siAdd("diagram-" + s[0], "diagrams", s[1], s[2], "diagram", s[3], { glyph: s[0], w: 520, h: 260, diagram: s[0] });
});

/* Equations are plain text (Unicode maths), so no new renderer is needed. */
[["emc", "slides-insert-equation-emc", "Mass–energy", "E = mc^2", "E = m c²"],
 ["quadratic", "slides-insert-equation-quadratic", "Quadratic formula", "x = (-b ± √Δ)/2a", "x = (−b ± √Δ) / 2a"],
 ["pythagoras", "slides-insert-equation-pythagoras", "Pythagoras", "a^2 + b^2 = c^2", "a² + b² = c²"],
 ["sum", "slides-insert-equation-sum", "Summation", "sigma sum", "∑ i = 1…n"],
 ["integral", "slides-insert-equation-integral", "Integral", "integral calculus", "∫ f(x) dx"],
].forEach(function (s) {
  siAdd("equation-" + s[0], "equations", s[1], s[2], "equation", s[3], { glyph: "equation", w: 380, h: 80, fontSize: 26, formula: s[4], formulaKey: s[1] + "-value" });
});

/* Fields compute their value at insert time; no user-facing literal is hardcoded. */
[["slide-number", "slides-insert-field-slide-number", "Slide number", "number page"],
 ["date", "slides-insert-field-date", "Date", "date today"],
 ["author", "slides-insert-field-author", "Author", "author name"],
 ["footer", "slides-insert-field-footer", "Footer", "footer bottom"],
 ["logo", "slides-insert-field-logo", "Logo placeholder", "logo brand"],
].forEach(function (s) {
  siAdd("field-" + s[0], "fields", s[1], s[2], "field", s[3], { glyph: "field", w: 220, h: 50, fontSize: 14, field: s[0], valueKey: s[1] + "-value" });
});

/* Local preferences: recent ids and favourites (never stored in the deck). */
var SI_STORE_RECENT = "slides_insert_recent";
var SI_STORE_FAVORITES = "slides_insert_favorites";

function siReadStore(key) {
  try {
    var raw = window.localStorage.getItem(key);
    var parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed : [];
  } catch (e) {
    return [];
  }
}

function siWriteStore(key, values) {
  try { window.localStorage.setItem(key, JSON.stringify(values)); } catch (e) { /* storage disabled */ }
}

function siRecentIds() { return siReadStore(SI_STORE_RECENT); }

function siFavorites() { return siReadStore(SI_STORE_FAVORITES); }

function siRemember(id) {
  var recent = siRecentIds().filter(function (x) { return x !== id; });
  recent.unshift(id);
  siWriteStore(SI_STORE_RECENT, recent.slice(0, 12));
}

function siToggleFavorite(id) {
  var list = siFavorites();
  var at = list.indexOf(id);
  if (at < 0) list.push(id);
  else list.splice(at, 1);
  siWriteStore(SI_STORE_FAVORITES, list);
  return at < 0;
}

function siFind(id) {
  for (var i = 0; i < SI_ITEMS.length; i++) {
    if (SI_ITEMS[i].id === id) return SI_ITEMS[i];
  }
  return null;
}

function siLabel(item) { return siT(item.key, item.fallback); }

function siMatches(item, query) {
  if (!query) return true;
  var needle = query.toLowerCase().trim();
  return (siLabel(item) + " " + item.category + " " + item.keywords + " " + item.id)
    .toLowerCase().indexOf(needle) >= 0;
}

function siItemsFor(category, query) {
  if (category === "recent") {
    return siRecentIds().map(siFind).filter(function (x) { return !!x && siMatches(x, query); });
  }
  if (category === "favorites") {
    return siFavorites().map(siFind).filter(function (x) { return !!x && siMatches(x, query); });
  }
  if (category === "reuse") return [];
  return SI_ITEMS.filter(function (item) {
    return item.category === category && siMatches(item, query);
  });
}
