"use strict";
/* Slides object model — media, link and slide-size dialog sections (#2430).
 * Registers into the shell created by 49_om_dialog.js. No hardcoded UI strings:
 * every label, placeholder and aria-label resolves through gbT. */

(function () {
  var OM = window.GBSlidesOM;
  var Ops = window.GBSlidesOMOps;
  var UI = window.GBSlidesOMDialogUI;
  var el = UI.el, button = UI.button, fieldset = UI.fieldset;
  var label = UI.label, announce = UI.announce, controls = UI.controls;
  var warned = false;

  function textInput(key, fallback) {
    var input = document.createElement("input");
    input.type = "text";
    input.setAttribute("aria-label", label(key, fallback));
    input.setAttribute("data-i18n-aria-label", key);
    input.setAttribute("placeholder", label(key, fallback));
    input.setAttribute("data-i18n-placeholder", key);
    return input;
  }
  function numberInput(key, fallback) {
    var input = textInput(key, fallback);
    input.type = "number";
    return input;
  }
  function option(select, value, key, fallback) {
    var node = document.createElement("option");
    node.value = value;
    node.textContent = label(key, fallback);
    select.appendChild(node);
  }

  function objectsSection(body) {
    var box = fieldset("slides-om-objects-label", "Objects");
    [["shape", "slides-om-add-rect", "Rectangle"],
      ["shape", "slides-om-add-ellipse", "Ellipse"],
      ["text", "slides-om-add-text", "Text box"],
      ["image", "slides-om-add-image", "Image"]].forEach(function (item) {
      box.appendChild(button(item[1], item[2], function () { Ops.addShape(item[0]); }));
    });
    controls.pen = button("slides-om-pen", "Freeform pen", function () {
      OM.penMode = !OM.penMode; OM.connectorMode = null;
      announce(label(OM.penMode ? "slides-om-status-pen-on" : "slides-om-status-pen-off",
        OM.penMode ? "Freeform pen on" : "Freeform pen off"));
    });
    box.appendChild(controls.pen);
    controls.connect = button("slides-om-add-connector", "Connector", function () {
      var chosen = OM.selection.slice(0, 2);
      if (chosen.length === 2) {
        Ops.addConnector(chosen[0], chosen[1]);
        announce(label("slides-om-status-connector-added", "Connector attached"));
        return;
      }
      OM.connectorMode = { from: chosen[0] || null, anchor: "center" };
      announce(label("slides-om-status-pen-hint", "Select two objects, or click the first endpoint then a second object"));
    });
    box.appendChild(controls.connect);
    body.appendChild(box);
    lineSection(body);
    imageSection(body);
    linkSection(body);
  }

  function lineSection(body) {
    var box = fieldset("slides-om-line-label", "Line & arrows");
    var style = document.createElement("select");
    style.setAttribute("aria-label", label("slides-om-line-style", "Line style"));
    style.setAttribute("data-i18n-aria-label", "slides-om-line-style");
    [["solid", "slides-om-line-solid", "Solid"],
      ["dashed", "slides-om-line-dashed", "Dashed"],
      ["dotted", "slides-om-line-dotted", "Dotted"]].forEach(function (item) {
      option(style, item[0], item[1], item[2]);
    });
    box.appendChild(style);
    box.appendChild(button("slides-om-line-apply", "Apply line style", function () {
      Ops.setLineStyle({ lineStyle: style.value });
    }));
    box.appendChild(button("slides-om-arrow-start", "Arrow at start", function () {
      Ops.setLineStyle({ arrowStart: true });
    }));
    box.appendChild(button("slides-om-arrow-end", "Arrow at end", function () {
      Ops.setLineStyle({ arrowEnd: true });
    }));
    body.appendChild(box);
  }

  function imageSection(body) {
    var box = fieldset("slides-om-image-label", "Image crop & mask");
    var width = numberInput("slides-om-crop-width", "Crop width");
    var height = numberInput("slides-om-crop-height", "Crop height");
    width.value = "1"; height.value = "1";
    box.appendChild(width); box.appendChild(height);
    box.appendChild(button("slides-om-crop", "Crop", function () {
      if (!OM.selection.length) return;
      Ops.crop(OM.selection[0], { w: Math.min(1, OM.num(width.value, 1)), h: Math.min(1, OM.num(height.value, 1)) });
    }));
    box.appendChild(button("slides-om-crop-reset", "Reset crop", function () {
      if (Ops.resetCrop(OM.selection[0])) announce(label("slides-om-status-crop-reset", "Crop reset"));
    }));
    var mask = document.createElement("select");
    mask.setAttribute("aria-label", label("slides-om-mask-label", "Mask to shape"));
    mask.setAttribute("data-i18n-aria-label", "slides-om-mask-label");
    [["none", "slides-om-mask-none", "None"], ["circle", "slides-om-mask-circle", "Circle"],
      ["rounded", "slides-om-mask-rounded", "Rounded rectangle"],
      ["triangle", "slides-om-mask-triangle", "Triangle"]].forEach(function (item) {
      option(mask, item[0], item[1], item[2]);
    });
    box.appendChild(mask);
    box.appendChild(button("slides-om-mask-apply", "Apply mask", function () {
      Ops.mask(OM.selection[0], mask.value);
    }));
    var src = textInput("slides-om-image-url", "Image URL");
    box.appendChild(src);
    box.appendChild(button("slides-om-image-replace", "Replace image", function () {
      Ops.replaceImage(OM.selection[0], src.value);
      announce(label("slides-om-status-image-replaced", "Image replaced"));
    }));
    body.appendChild(box);
  }

  function linkSection(body) {
    var box = fieldset("slides-om-hyperlink", "Hyperlink & alt text");
    var kind = document.createElement("select");
    kind.setAttribute("aria-label", label("slides-om-hyperlink", "Hyperlink"));
    kind.setAttribute("data-i18n-aria-label", "slides-om-hyperlink");
    [["none", "slides-om-link-none", "None"], ["url", "slides-om-link-url", "Web address"],
      ["slide", "slides-om-link-slide", "Slide"], ["drive", "slides-om-link-drive", "Drive file"]].forEach(function (item) {
      option(kind, item[0], item[1], item[2]);
    });
    var target = textInput("slides-om-link-target", "Target");
    box.appendChild(kind); box.appendChild(target);
    box.appendChild(button("slides-om-link-apply", "Save hyperlink", function () {
      Ops.hyperlink(OM.selection[0], kind.value, target.value);
      announce(label("slides-om-status-link-saved", "Hyperlink saved"));
    }));
    var alt = textInput("slides-om-alt-text", "Alt text");
    box.appendChild(alt);
    var decorative = document.createElement("input");
    decorative.type = "checkbox";
    decorative.setAttribute("aria-label", label("slides-om-decorative", "Decorative (skip from accessibility)"));
    decorative.setAttribute("data-i18n-aria-label", "slides-om-decorative");
    var decoWrap = el("label", "sl-om-check");
    decoWrap.appendChild(decorative);
    decoWrap.appendChild(el("span", "", label("slides-om-decorative", "Decorative (skip from accessibility)"), "slides-om-decorative"));
    box.appendChild(decoWrap);
    box.appendChild(button("slides-om-alt-apply", "Save alt text", function () {
      Ops.alt(OM.selection[0], alt.value, decorative.checked);
      announce(label("slides-om-status-alt-saved", "Alt text saved"));
    }));
    body.appendChild(box);
  }

  function sizeSection(body) {
    var box = fieldset("slides-om-size-label", "Slide size");
    var preset = document.createElement("select");
    preset.setAttribute("aria-label", label("slides-om-size-label", "Slide size"));
    preset.setAttribute("data-i18n-aria-label", "slides-om-size-label");
    OM.presets().forEach(function (p) {
      var node = document.createElement("option");
      node.value = p.id;
      node.textContent = label("slides-om-size-" + p.id.replace(":", "x"), p.id);
      preset.appendChild(node);
    });
    var custom = document.createElement("option");
    custom.value = "custom";
    custom.textContent = label("slides-om-size-custom", "Custom");
    preset.appendChild(custom);
    var width = numberInput("slides-om-size-width", "Width");
    var height = numberInput("slides-om-size-height", "Height");
    width.value = String(OM.slide().size.w); height.value = String(OM.slide().size.h);
    box.appendChild(preset); box.appendChild(width); box.appendChild(height);
    box.appendChild(button("slides-om-size-apply", "Apply slide size", function () {
      var target = preset.value === "custom"
        ? { id: "custom", w: OM.num(width.value, 960), h: OM.num(height.value, 540) }
        : preset.value;
      var resolved = typeof target === "string" ? OM.preset(target) : target;
      if (!resolved) return;
      var current = OM.slide().size;
      if (warned) {
        warned = false;
        Ops.setSlideSize(target);
        return;
      }
      warned = true;
      warnRescale(current, resolved, function (accepted) {
        warned = false;
        if (!accepted) return;
        Ops.setSlideSize(target);
        announce(label("slides-om-status-size-changed", "Slide size changed to {size}", { size: resolved.id }));
      });
    }));
    body.appendChild(box);
  }

  function warnRescale(from, to, done) {
    var dialog = window.GBSlidesOMDialog.makeDialog("slides-om-size-warn-title", "Rescale slide?");
    var message = el("p", "sl-om-message", label("slides-om-size-warn",
      "Changing the slide size from {from} to {to} rescales every object proportionally. Continue?",
      { from: from.id || "16x9", to: to.id || "custom" }));
    dialog.body.appendChild(message);
    var cancel = button("slides-om-cancel", "Cancel", function () { dialog.dismiss(false); });
    var apply = button("slides-om-size-apply", "Apply slide size", function () { dialog.dismiss(true); });
    dialog.foot.appendChild(cancel); dialog.foot.appendChild(apply);
    dialog.setOnClose(done);
  }

  UI.sections.objects = objectsSection;
  UI.sections.size = sizeSection;
}());
