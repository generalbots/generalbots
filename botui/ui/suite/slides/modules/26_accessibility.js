"use strict";
/* Slides accessibility panel (#2433): lists images/charts/tables without a text
 * alternative, jump-to-slide, mark decorative, local alt descriptions and a
 * text/background contrast warning. Descriptions are local-only metadata. */
(function () {
  var O = window.GBSlidesOutline;
  var ALT = "a11y-alt";
  var DECOR = "a11y-decorative";

  function altStore() { return O.load(ALT, {}); }
  function decorStore() { return O.load(DECOR, {}); }
  function refKey(item) {
    return [item.slideIndex, item.elementId, item.elementIndex, item.type, item.mediaIndex || 0].join("|");
  }

  function collect() {
    var items = [];
    O.slides().forEach(function (slide) {
      slide.elements.concat(slide.canvasElements).forEach(function (element, position) {
        var type = element.getAttribute("data-type") || "";
        var images = O.$$("img", element);
        if (type === "image" || images.length) {
          images.forEach(function (image, imageIndex) {
            var alt = image.getAttribute("alt") || "";
            items.push({
              kind: "image", type: type, slideIndex: slide.index,
              elementId: element.getAttribute("data-id") || "",
              elementIndex: position, mediaIndex: imageIndex,
              node: image, missing: !alt.trim(), labelKey: "slides-outline-a11y-image"
            });
          });
          if (type === "image" && !images.length) {
            items.push({
              kind: "image", type: type, slideIndex: slide.index,
              elementId: element.getAttribute("data-id") || "", elementIndex: position,
              node: element, missing: true, labelKey: "slides-outline-a11y-image"
            });
          }
        } else if (type === "chart" || type === "table") {
          items.push({
            kind: type, type: type, slideIndex: slide.index,
            elementId: element.getAttribute("data-id") || "", elementIndex: position,
            node: element, missing: true,
            labelKey: type === "chart" ? "slides-outline-a11y-chart" : "slides-outline-a11y-table"
          });
        }
      });
      if (slide.thumb) {
        O.$$("img.sl-media", slide.thumb).forEach(function (image, index) {
          var alt = image.getAttribute("alt") || "";
          items.push({
            kind: "image", type: "media", slideIndex: slide.index,
            elementId: "", elementIndex: -1, mediaIndex: index,
            node: image, missing: !alt.trim(), labelKey: "slides-outline-a11y-image-media"
          });
        });
      }
    });
    return items;
  }

  function parseColor(value) {
    var text = String(value || "").trim();
    var hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(text);
    if (hex) {
      var h = hex[1];
      if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
      return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
    }
    var rgb = /rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/i.exec(text);
    return rgb ? [parseInt(rgb[1], 10), parseInt(rgb[2], 10), parseInt(rgb[3], 10)] : null;
  }
  function luminance(rgb) {
    var channels = rgb.map(function (channel) {
      var value = channel / 255;
      return value <= 0.03928 ? value / 12.92 : Math.pow((value + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  }
  function contrast(foreground, background) {
    var a = luminance(foreground), b = luminance(background);
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
  }
  function contrastWarnings() {
    var warnings = [];
    var canvas = O.$("#slide-canvas") || document.getElementById("slide-canvas");
    var deckBg = canvas ? (canvas.getAttribute("data-bg") || "") : "";
    O.slides().forEach(function (slide) {
      slide.elements.concat(slide.canvasElements).forEach(function (element) {
        var type = element.getAttribute("data-type") || "";
        if (type !== "text" && type !== "title") return;
        var node = element.querySelector("p,h1,h2,h3") || element;
        var style = window.getComputedStyle ? window.getComputedStyle(element) : element.style;
        var foreground = parseColor(style.color || element.style.color);
        var behind = parseColor(element.style.background || element.style.backgroundColor) || parseColor(deckBg);
        if (!foreground || !behind) return;
        var value = contrast(foreground, behind);
        if (value < 4.5) {
          warnings.push({ slideIndex: slide.index, ratio: value.toFixed(2),
            sample: (node.textContent || "").trim().slice(0, 60) });
        }
      });
    });
    return warnings;
  }

  function applyDecor(item, decorative) {
    var store = decorStore();
    var key = refKey(item);
    if (decorative) store[key] = true; else delete store[key];
    O.save(DECOR, store);
    if (item.node && item.node.setAttribute) {
      if (decorative) {
        item.node.setAttribute("aria-hidden", "true");
        if (item.kind === "image") item.node.setAttribute("alt", "");
        else item.node.setAttribute("role", "presentation");
      } else {
        item.node.removeAttribute("aria-hidden");
        if (item.kind !== "image") item.node.removeAttribute("role");
      }
    }
  }
  function setAlt(item, text) {
    var store = altStore();
    var key = refKey(item);
    if (text) store[key] = text; else delete store[key];
    O.save(ALT, store);
    if (item.node && item.node.setAttribute) item.node.setAttribute("alt", text);
  }

  function describeDialog(item, render) {
    var dialog = O.dialog({
      titleKey: "slides-outline-a11y-describe-title", title: "Text alternative"
    });
    var label = O.el("label", "slides-tools-field-label",
      O.t("slides-outline-a11y-describe", "Description"), "slides-outline-a11y-describe");
    var input = document.createElement("input");
    input.type = "text";
    input.value = altStore()[refKey(item)] || "";
    input.setAttribute("data-i18n-placeholder", "slides-outline-a11y-describe");
    input.placeholder = O.t("slides-outline-a11y-describe", "Description");
    label.appendChild(input);
    dialog.body.appendChild(label);
    dialog.body.appendChild(O.localBadge());
    var cancel = O.el("button", "slides-tools-btn", O.t("slides-outline-cancel", "Cancel"), "slides-outline-cancel");
    var save = O.el("button", "slides-tools-btn primary", O.t("slides-outline-save", "Save"), "slides-outline-save");
    cancel.type = "button"; save.type = "button";
    cancel.addEventListener("click", function () { dialog.dismiss(false); });
    save.addEventListener("click", function () {
      var before = altStore()[refKey(item)] || "";
      var value = (input.value || "").trim();
      dialog.dismiss(true);
      setAlt(item, value); render();
      O.pushUndo({
        label: O.t("slides-outline-action-a11y-alt", "set description"),
        undo: function () { setAlt(item, before); render(); },
        redo: function () { setAlt(item, value); render(); }
      });
      O.status(O.t("slides-outline-a11y-saved", "Description saved locally"));
    });
    dialog.foot.appendChild(cancel); dialog.foot.appendChild(save);
  }

  function itemRow(item, host, render) {
    var key = refKey(item);
    var decorative = !!decorStore()[key];
    var row = O.el("div", "slides-a11y-row");
    var title = O.el("span", "slides-a11y-title",
      O.t(item.labelKey, "Item without a text alternative"), item.labelKey);
    row.appendChild(title);
    row.appendChild(O.el("span", "slides-a11y-slide",
      O.t("slides-outline-untitled", "Slide {n}", { n: item.slideIndex + 1 }), "slides-outline-untitled"));
    var jump = O.el("button", "slides-tools-btn", O.t("slides-outline-jump", "Go to slide"), "slides-outline-jump");
    jump.type = "button"; jump.addEventListener("click", function () { O.jump(item.slideIndex); });
    var describe = O.el("button", "slides-tools-btn", O.t("slides-outline-a11y-describe-btn", "Describe"), "slides-outline-a11y-describe-btn");
    describe.type = "button"; describe.addEventListener("click", function () { describeDialog(item, render); });
    var decorate = O.el("button", "slides-tools-btn",
      O.t(decorative ? "slides-outline-a11y-undecorate" : "slides-outline-a11y-decorate",
        decorative ? "Not decorative" : "Mark decorative"),
      decorative ? "slides-outline-a11y-undecorate" : "slides-outline-a11y-decorate");
    decorate.type = "button";
    decorate.addEventListener("click", function () {
      applyDecor(item, !decorative); render();
      O.pushUndo({
        label: O.t("slides-outline-action-a11y-decor", "mark decorative"),
        undo: function () { applyDecor(item, decorative); render(); },
        redo: function () { applyDecor(item, !decorative); render(); }
      });
      O.status(O.t("slides-outline-a11y-decorated", "Decorative flag saved locally"));
    });
    row.appendChild(jump); row.appendChild(describe); row.appendChild(decorate);
    host.appendChild(row);
  }

  function open() {
    var items = collect();
    var warnings = contrastWarnings();
    var panel = O.openPanel("slides-a11y-panel", function () {
      var root = O.el("section", "slides-tools-panel slides-a11y-panel");
      var head = O.el("div", "slides-tools-panel-head");
      head.appendChild(O.el("h2", "", O.t("slides-outline-a11y-title", "Accessibility"), "slides-outline-a11y-title"));
      head.appendChild(O.localBadge());
      var close = O.el("button", "slides-tools-close", O.t("slides-outline-close", "Close"), "slides-outline-close");
      close.type = "button"; close.addEventListener("click", O.closePanels);
      head.appendChild(close);
      root.appendChild(head);
      var summary = O.el("p", "slides-a11y-summary",
        O.t("slides-outline-a11y-summary", "{n} item(s) need a text alternative", { n: items.length }),
        "slides-outline-a11y-summary");
      root.appendChild(summary);
      var list = O.el("div", "slides-a11y-list");
      root.appendChild(list);
      var warningHost = O.el("div", "slides-a11y-warnings");
      root.appendChild(O.el("h3", "", O.t("slides-outline-a11y-contrast", "Contrast warnings"), "slides-outline-a11y-contrast"));
      root.appendChild(warningHost);
      return root;
    });
    var list = panel.querySelector(".slides-a11y-list");
    function render() {
      list.textContent = "";
      var shown = 0;
      items.forEach(function (item) {
        var key = refKey(item);
        var hasAlt = !item.missing || !!decorStore()[key] || !!altStore()[key];
        if (hasAlt) return;
        itemRow(item, list, render); shown += 1;
      });
      if (!shown) list.appendChild(O.el("p", "slides-tools-empty",
        O.t("slides-outline-a11y-clear", "No missing text alternatives"), "slides-outline-a11y-clear"));
    }
    render();
    var warningHost = panel.querySelector(".slides-a11y-warnings");
    if (!warnings.length) {
      warningHost.appendChild(O.el("p", "slides-tools-empty",
        O.t("slides-outline-a11y-no-contrast", "No low-contrast text detected"), "slides-outline-a11y-no-contrast"));
    } else {
      warnings.forEach(function (warning) {
        var row = O.el("div", "slides-a11y-row");
        row.appendChild(O.el("span", "slides-a11y-contrast-ratio",
          O.t("slides-outline-a11y-ratio", "{ratio}:1", { ratio: warning.ratio }), "slides-outline-a11y-ratio"));
        row.appendChild(O.el("span", "slides-a11y-slide",
          O.t("slides-outline-untitled", "Slide {n}", { n: warning.slideIndex + 1 }), "slides-outline-untitled"));
        row.appendChild(O.el("span", "slides-a11y-sample", warning.sample));
        var jump = O.el("button", "slides-tools-btn", O.t("slides-outline-jump", "Go to slide"), "slides-outline-jump");
        jump.type = "button";
        jump.addEventListener("click", function () { O.jump(warning.slideIndex); });
        row.appendChild(jump);
        warningHost.appendChild(row);
      });
    }
    O.status(O.t("slides-outline-a11y-status", "{n} item(s) need review", { n: items.length + warnings.length }));
    return panel;
  }

  window.GBSlidesAccessibility = { open: open };
})();
