"use strict";
/* slides state — constants, helpers, shared SlideCanvas shell */

var SIDEBAR_TAB_KEY = "slides_sidebar_tab";
var SLIDE_W = 960;
var SLIDE_H = 540;
var SCALE_MIN = 0.25;
var SCALE_MAX = 2.0;
var H_SZ = 8;
var ROT_H_SZ = 10;
var ROT_OFF = 28;

function $(s, r) { return (r || document).querySelector(s); }
function $$(s, r) { return Array.from((r || document).querySelectorAll(s)); }

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

var SlideCanvas = {
  scale: 1.0,
  selectedId: null,
  canvas: null,
  elements: [],

  attach: function (host) {
    if (!host) return;
    var c = host.querySelector(".sl-canvas");
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
  }
};
