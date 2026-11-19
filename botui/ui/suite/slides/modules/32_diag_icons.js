"use strict";
/* Slides diagram authoring — local inline SVG icon library (#2488).
 * Every glyph is vendored path data: no CDN, no fetch. `currentColor` lets the
 * deck theme recolour icons; each SVG carries a <title> for alt text. */

var SD_ICONS = [
  { id: "check", key: "slides-diagrams-icon-check", fallback: "Check", words: "tick ok done", d: "M20 6 9 17l-5-5" },
  { id: "star", key: "slides-diagrams-icon-star", fallback: "Star", words: "favourite priority", d: "m12 2 3.1 6.3 6.9 1-5 4.9 1.2 6.9L12 17.8 5.8 21l1.2-6.9-5-4.9 6.9-1z" },
  { id: "arrow", key: "slides-diagrams-icon-arrow", fallback: "Arrow", words: "next right flow", d: "M5 12h14m-6-6 6 6-6 6" },
  { id: "cloud", key: "slides-diagrams-icon-cloud", fallback: "Cloud", words: "hosting saas", d: "M17.5 19a4.5 4.5 0 0 0 .5-9 6 6 0 0 0-11.6 1.5A3.8 3.8 0 0 0 7 19z" },
  { id: "lock", key: "slides-diagrams-icon-lock", fallback: "Lock", words: "secure private", d: "M5 11h14v10H5zM8 11V7a4 4 0 0 1 8 0v4" },
  { id: "mail", key: "slides-diagrams-icon-mail", fallback: "Mail", words: "email message", d: "M3 5h18v14H3zM3 6l9 7 9-7" },
  { id: "calendar", key: "slides-diagrams-icon-calendar", fallback: "Calendar", words: "date time schedule", d: "M4 5h16v16H4zM8 3v4m8-4v4M4 10h16" },
  { id: "user", key: "slides-diagrams-icon-user", fallback: "User", words: "person people role", d: "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 21a8 8 0 0 1 16 0" },
  { id: "chart", key: "slides-diagrams-icon-chart", fallback: "Chart", words: "graph data metrics", d: "M4 20V10m6 10V4m6 16v-7m4 7H2" },
  { id: "folder", key: "slides-diagrams-icon-folder", fallback: "Folder", words: "files group", d: "M3 6h6l2 2h10v12H3z" },
  { id: "bolt", key: "slides-diagrams-icon-bolt", fallback: "Bolt", words: "flash fast energy", d: "m13 2-9 12h7l-1 8 9-12h-7z" },
  { id: "heart", key: "slides-diagrams-icon-heart", fallback: "Heart", words: "like care", d: "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8z" },
  { id: "flag", key: "slides-diagrams-icon-flag", fallback: "Flag", words: "milestone goal", d: "M5 21V4m0 0h11l-2 4 2 4H5" },
  { id: "gear", key: "slides-diagrams-icon-gear", fallback: "Gear", words: "settings process config", d: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2v3m0 14v3M2 12h3m14 0h3M4.9 4.9 7 7m10 10 2.1 2.1M19.1 4.9 17 7M7 17l-2.1 2.1" },
  { id: "search", key: "slides-diagrams-icon-search", fallback: "Search", words: "find zoom magnifier", d: "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zm10 2-4.5-4.5" },
  { id: "target", key: "slides-diagrams-icon-target", fallback: "Target", words: "aim objective focus", d: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zm0 5a4 4 0 1 0 0 8 4 4 0 0 0 0-8zm0 3a1 1 0 1 0 0 2 1 1 0 0 0 0-2z" }
];

var SDIcons = {
  all: function () {
    return SD_ICONS.slice();
  },

  find: function (id) {
    for (var i = 0; i < SD_ICONS.length; i++) {
      if (SD_ICONS[i].id === id) return SD_ICONS[i];
    }
    return null;
  },

  label: function (icon) {
    return icon ? sdT(icon.key, icon.fallback) : "";
  },

  search: function (query) {
    var needle = String(query || "").toLowerCase().trim();
    if (!needle) return this.all();
    return SD_ICONS.filter(function (icon) {
      return (SDIcons.label(icon) + " " + icon.words + " " + icon.id).toLowerCase().indexOf(needle) >= 0;
    });
  },

  svg: function (id, alt) {
    var icon = this.find(id);
    if (!icon || typeof document === "undefined" || !document.createElementNS) return null;
    var ns = "http://www.w3.org/2000/svg";
    var svg = document.createElementNS(ns, "svg");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("width", "18");
    svg.setAttribute("height", "18");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "1.8");
    svg.setAttribute("stroke-linecap", "round");
    svg.setAttribute("stroke-linejoin", "round");
    svg.setAttribute("role", "img");
    svg.setAttribute("class", "sl-diag-icon");
    var title = document.createElementNS(ns, "title");
    title.textContent = alt || this.label(icon);
    svg.appendChild(title);
    var path = document.createElementNS(ns, "path");
    path.setAttribute("d", icon.d);
    svg.appendChild(path);
    return svg;
  }
};
