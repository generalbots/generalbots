/* GBIcons — one small, themeable SVG icon set for JS-rendered UI (#1978).
 *
 * Static markup carries its own inline `<svg>` (see slides/docs/player/plan/
 * photos/canvas), because those fragments are served as HTML. Controls built
 * from strings in JavaScript cannot, so they call `GBIcons.html(name)` and get
 * the same geometry: `currentColor` strokes, no fill, no emoji, so every icon
 * follows the active theme in light, dark and custom palettes.
 *
 *   GBIcons.html("trash")            → '<svg …>…</svg>'
 *   GBIcons.html("volume", 18)       → same icon at 18 px
 *   GBIcons.available                → ['trash', 'volume', …]
 */
"use strict";

(function (window) {
  var PATHS = {
    users:
      '<path d="M17 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9.5" cy="7" r="3.5"/>' +
      '<path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>',
    clock: '<circle cx="12" cy="12" r="9"/><polyline points="12 7 12 12 15.5 14"/>',
    history:
      '<path d="M3 12a9 9 0 1 0 3-6.7"/><polyline points="3 3 3 7 7 7"/>' +
      '<polyline points="12 8 12 13 15.5 15"/>',
    chat: '<path d="M21 15a2 2 0 0 1-2 2H8l-5 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>',
    mic:
      '<rect x="9" y="2.5" width="6" height="11" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0"/>' +
      '<line x1="12" y1="17.5" x2="12" y2="21"/>',
    film:
      '<rect x="2.5" y="4.5" width="19" height="15" rx="2.5"/><line x1="8" y1="4.5" x2="8" y2="19.5"/>' +
      '<line x1="16" y1="4.5" x2="16" y2="19.5"/><line x1="2.5" y1="9.5" x2="21.5" y2="9.5"/>' +
      '<line x1="2.5" y1="14.5" x2="21.5" y2="14.5"/>',
    music:
      '<path d="M9 18V6l10-2v12"/><circle cx="6.5" cy="18" r="2.5"/><circle cx="16.5" cy="16" r="2.5"/>',
    file:
      '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/>' +
      '<polyline points="14 3 14 8 19 8"/><line x1="9" y1="13" x2="15" y2="13"/>' +
      '<line x1="9" y1="17" x2="13" y2="17"/>',
    image:
      '<rect x="3" y="3.5" width="18" height="17" rx="2.5"/><circle cx="8.5" cy="9" r="1.6"/>' +
      '<polyline points="21 16 16 11 5 20.5"/>',
    trash:
      '<polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>' +
      '<path d="M10 11v6"/><path d="M14 11v6"/><path d="M9 6V4a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"/>',
    save:
      '<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>' +
      '<polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>',
    upload:
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/>' +
      '<line x1="12" y1="3" x2="12" y2="15"/>',
    download:
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/>' +
      '<line x1="12" y1="15" x2="12" y2="3"/>',
    pencil:
      '<path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>',
    eye:
      '<path d="M1.5 12S5.5 5 12 5s10.5 7 10.5 7-4 7-10.5 7S1.5 12 1.5 12z"/>' +
      '<circle cx="12" cy="12" r="3"/>',
    eyeOff:
      '<path d="M17.9 17.9A10.4 10.4 0 0 1 12 19.5C5.5 19.5 1.5 12 1.5 12a19 19 0 0 1 5.1-5.6"/>' +
      '<path d="M9.9 4.7A10.4 10.4 0 0 1 12 4.5c6.5 0 10.5 7.5 10.5 7.5a19 19 0 0 1-2.3 3.3"/>' +
      '<line x1="2" y1="2" x2="22" y2="22"/>',
    lock:
      '<rect x="4" y="10" width="16" height="10" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>',
    unlock:
      '<rect x="4" y="10" width="16" height="10" rx="2"/>' +
      '<path d="M8 10V7a4 4 0 0 1 7.6-1.7"/>',
    settings:
      '<circle cx="12" cy="12" r="3"/>' +
      '<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
    wand:
      '<path d="M4 20l10-10"/><path d="M14 4l1 2 2 1-2 1-1 2-1-2-2-1 2-1z"/>' +
      '<path d="M19 11l.7 1.4 1.4.7-1.4.7-.7 1.4-.7-1.4-1.4-.7 1.4-.7z"/>',
    brush:
      '<path d="M9.5 14.5l-4 4a2.1 2.1 0 0 0 3 3l4-4"/>' +
      '<path d="M14 10l6.5-6.5a2.1 2.1 0 0 0-3-3L11 7"/>',
    eraser:
      '<path d="M7 21h10"/><path d="M4.5 16.5l7-7a2 2 0 0 1 2.8 0l3.2 3.2a2 2 0 0 1 0 2.8l-4.5 4.5H8z"/>',
    note:
      '<rect x="4" y="3" width="16" height="18" rx="2"/><line x1="8" y1="8" x2="16" y2="8"/>' +
      '<line x1="8" y1="12" x2="16" y2="12"/><line x1="8" y1="16" x2="13" y2="16"/>',
    square: '<rect x="4" y="4" width="16" height="16" rx="2"/>',
    ban:
      '<circle cx="12" cy="12" r="9"/><line x1="6" y1="6" x2="18" y2="18"/>',
    volume:
      '<polygon points="11 5 6 9 3 9 3 15 6 15 11 19 11 5"/>' +
      '<path d="M15.5 8.5a5 5 0 0 1 0 7"/><path d="M18.5 5.5a9 9 0 0 1 0 13"/>',
    volumeOff:
      '<polygon points="11 5 6 9 3 9 3 15 6 15 11 19 11 5"/><line x1="16" y1="9" x2="22" y2="15"/>' +
      '<line x1="22" y1="9" x2="16" y2="15"/>',
    calendar:
      '<rect x="3" y="4.5" width="18" height="16.5" rx="2.5"/><line x1="3" y1="9.5" x2="21" y2="9.5"/>' +
      '<line x1="8" y1="2.5" x2="8" y2="6"/><line x1="16" y1="2.5" x2="16" y2="6"/>',
    robot:
      '<rect x="4" y="8" width="16" height="11" rx="3"/><line x1="12" y1="3" x2="12" y2="8"/>' +
      '<circle cx="9" cy="13.5" r="1.2" fill="currentColor" stroke="none"/>' +
      '<circle cx="15" cy="13.5" r="1.2" fill="currentColor" stroke="none"/>' +
      '<line x1="9" y1="16.5" x2="15" y2="16.5"/>',
    close: '<line x1="6" y1="6" x2="18" y2="18"/><line x1="18" y1="6" x2="6" y2="18"/>',
  };

  function html(name, size) {
    var body = PATHS[name];
    if (!body) {
      // Unknown icon: draw nothing rather than leaking the icon name into the
      // interface as text.
      return "";
    }
    var px = size || 15;
    return (
      '<svg viewBox="0 0 24 24" width="' + px + '" height="' + px + '" fill="none" ' +
      'stroke="currentColor" stroke-width="1.8" stroke-linecap="round" ' +
      'stroke-linejoin="round" aria-hidden="true">' + body + "</svg>"
    );
  }

  window.GBIcons = {
    html: html,
    available: Object.keys(PATHS),
  };
})(window);
