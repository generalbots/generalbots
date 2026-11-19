"use strict";
/* Slides delivery mode (#2432) — caption show/hide toggle and a styling
 * preview. The overlay reuses the existing transcript-caption look. This is a
 * local preview only: no speech engine, no WebVTT upload and no server write. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var OVERLAY_ID = "sd-caption-overlay";

  function state() { return D.load().captions; }
  function isOn() { return !!state().on; }

  function safeColor(value, fallback) {
    return /^(#[0-9a-f]{3,8}|rgba?\([0-9.,\s%]+\))$/i.test(String(value || "")) ? value : fallback;
  }
  function style() {
    var stored = state().style || {};
    var size = parseInt(stored.size, 10);
    return {
      color: safeColor(stored.color, "#f8fafc"),
      background: safeColor(stored.background, "#111827"),
      size: Math.max(10, Math.min(48, isFinite(size) ? size : 20))
    };
  }
  function apply() {
    var on = isOn();
    var overlay = document.getElementById(OVERLAY_ID);
    if (!on) {
      if (overlay && overlay.parentNode) overlay.parentNode.removeChild(overlay);
      return false;
    }
    if (!overlay) {
      overlay = document.createElement("div");
      overlay.id = OVERLAY_ID;
      overlay.className = "sd-caption-overlay";
      var cue = document.createElement("span");
      cue.className = "stm-transcript-cue sd-caption-cue";
      D.labelled(cue, "data-i18n", "slides-delivery-captions-sample",
        "Caption text appears here during a local preview.");
      overlay.appendChild(cue);
      document.body.appendChild(overlay);
    }
    var css = style();
    overlay.style.color = css.color;
    overlay.style.background = css.background;
    overlay.style.fontSize = css.size + "px";
    if (D.console) D.console.render();
    return true;
  }
  function toggle() {
    var next = !isOn();
    D.patch(function (data) { data.captions.on = next; });
    apply();
    D.status(next ? "slides-delivery-captions-on" : "slides-delivery-captions-off",
      next ? "Captions on" : "Captions off");
    return next;
  }
  function setStyle(patch) {
    D.patch(function (data) {
      data.captions.style = Object.assign(data.captions.style || {}, patch || {});
    });
    apply();
    return style();
  }

  function open() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-captions-title",
      titleFallback: "Captions",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        api.body.appendChild(D.el("p", "sd-hint", "slides-delivery-captions-note",
          "Speech recognition is not available in this local preview."));
        var toggleButton = D.button("slides-delivery-captions-toggle", "Show captions", function () {
          toggle();
          D.labelled(toggleButton, "data-i18n", isOn()
            ? "slides-delivery-captions-on" : "slides-delivery-captions-toggle",
            isOn() ? "Captions on" : "Show captions");
          D.labelled(previewLabel, "data-i18n", isOn()
            ? "slides-delivery-captions-preview" : "slides-delivery-captions-off",
            isOn() ? "Caption preview" : "Captions off");
        }, "primary");
        api.body.appendChild(toggleButton);
        D.labelled(toggleButton, "data-i18n", isOn()
          ? "slides-delivery-captions-on" : "slides-delivery-captions-toggle",
          isOn() ? "Captions on" : "Show captions");

        var controls = D.el("div", "sd-caption-controls");
        var colorField = D.field("slides-delivery-captions-color", "Text colour", style().color);
        colorField.control.type = "color";
        var backgroundField = D.field("slides-delivery-captions-background", "Background", style().background);
        backgroundField.control.type = "color";
        var sizeField = D.field("slides-delivery-captions-size", "Size", style().size);
        sizeField.control.type = "number";
        sizeField.control.min = "10";
        sizeField.control.max = "48";
        [colorField, backgroundField, sizeField].forEach(function (field) {
          field.control.addEventListener("change", function () {
            setStyle({
              color: colorField.control.value,
              background: backgroundField.control.value,
              size: sizeField.control.value
            });
            api.setStatus("slides-delivery-captions-saved", "Caption style saved locally");
          });
          controls.appendChild(field);
        });
        api.body.appendChild(controls);

        var previewLabel = D.el("h3", "sd-subhead", "slides-delivery-captions-preview", "Caption preview");
        api.body.appendChild(previewLabel);
        if (!isOn()) D.labelled(previewLabel, "data-i18n", "slides-delivery-captions-off", "Captions off");
        var sample = D.el("div", "sd-caption-sample stm-transcript-cue", "slides-delivery-captions-sample",
          "Caption text appears here during a local preview.");
        var css = style();
        sample.style.color = css.color;
        sample.style.background = css.background;
        sample.style.fontSize = css.size + "px";
        api.body.appendChild(sample);
        api.addButton("slides-delivery-close", "Close", function () { api.close(); });
      }
    });
    return dialog;
  }

  D.captions = {
    state: state,
    style: style,
    isOn: isOn,
    toggle: toggle,
    setStyle: setStyle,
    apply: apply,
    open: open
  };
})(window);
