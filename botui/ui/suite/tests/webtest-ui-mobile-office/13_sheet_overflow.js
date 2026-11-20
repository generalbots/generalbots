/**
 * #2515 RUNTIME check for the phone toolbar overflow contract.
 *
 * Executes the REAL shipped sheet-inline/04_toolbar_overflow.js inside the
 * existing stubbed DOM. On a phone the format row is out of the layout, so the
 * main row's overflow menu must still expose its commands (hit-test proof of
 * "format row moved into the overflow sheet"). No production logic is copied.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var dom = require("./01_dom");

function menuLabels(width) {
  var fx = f.create({ width: width, apps: [] });
  var doc = fx.doc;
  var app = dom.el("div", { id: "sheet-app" });
  doc.body.appendChild(app);

  var main = dom.el("div", { class: "sheet-toolbar-row sheet-toolbar-row-main" });
  main.rect = { left: 8, right: 300, top: 8, bottom: 70, width: 292, height: 62 };
  app.appendChild(main);
  var undo = dom.el("button", { id: "undoBtn", title: "Undo" });
  undo.rect = { left: 16, right: 60, top: 12, bottom: 56, width: 44, height: 44 };
  main.appendChild(undo);

  var format = dom.el("div", { class: "sheet-toolbar-row sheet-toolbar-row-format" });
  app.appendChild(format);
  var bold = dom.el("button", { id: "boldBtn", title: "Bold" });
  format.appendChild(bold);

  h.vm.runInContext(h.readFile("sheet/sheet-inline/04_toolbar_overflow.js"), fx.ctx,
    { filename: "sheet/sheet-inline/04_toolbar_overflow.js" });
  fx.win.__sheetToolbarOverflowInit();

  var btn = main.querySelector(".toolbar-overflow-btn");
  h.assert(btn, "main row has an overflow affordance");
  btn.dispatchEvent(fx.event({ type: "click" }));
  var menu = doc.querySelector(".toolbar-overflow-menu");
  h.assert(menu, "overflow menu was built");
  return {
    visible: btn.classList.contains("visible"),
    labels: menu.children.map(function (item) { return item.textContent; }),
  };
}

h.test("#2515 RUNTIME: phone overflow carries the hidden Sheet format commands", function () {
  var phone = menuLabels(390);
  h.assertEqual(phone.visible, true,
    "phone overflow affordance is visible while the format row is off-layout");
  h.assert(phone.labels.indexOf("Bold") >= 0,
    "hidden format command appears in the phone overflow menu: " + phone.labels.join(","));
});

h.test("#2515 RUNTIME: desktop overflow does not borrow the format row", function () {
  var desktop = menuLabels(1440);
  h.assert(desktop.labels.indexOf("Bold") < 0,
    "desktop keeps its own per-row overflow: " + desktop.labels.join(","));
});
