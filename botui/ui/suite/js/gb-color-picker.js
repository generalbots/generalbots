"use strict";
/* GB Color Picker (#2291): richer colour affordance for office toolbars.
 * Replaces bare <input type="color"> with a trigger + popover offering
 * theme swatches, recent colours, hex entry and the native OS picker as a
 * last resort. Dispatches gb-color-preview (live) and gb-color-commit
 * (once per interaction) on the value element so apps can avoid flooding
 * the server on every pixel of a drag. */
(function () {
  if (window.GBColorPicker) return;

  var PALETTE = [
    "#000000", "#434343", "#666666", "#999999", "#b7b7b7", "#cccccc", "#efefef", "#ffffff",
    "#980000", "#ff0000", "#ff9900", "#ffff00", "#00ff00", "#00ffff", "#4a86e8", "#9900ff",
    "#e6b8af", "#f4cccc", "#fce5cd", "#fff2cc", "#d9ead3", "#d0e0e3", "#c9daf8", "#d9d2e9",
    "#dd7e6b", "#ea9999", "#f9cb9c", "#ffe599", "#b6d7a8", "#a2c4c9", "#a4c2f4", "#b4a7d6",
    "#cc4125", "#e06666", "#f6b26b", "#ffd966", "#93c47d", "#76a5af", "#6d9eeb", "#8e7cc3",
    "#a61c00", "#cc0000", "#e69138", "#f1c232", "#6aa84f", "#45818e", "#3c78d8", "#674ea7",
    "#5b0f00", "#660000", "#783f04", "#7f6000", "#274e13", "#0c343d", "#1c4587", "#20124d"
  ];
  var RECENT_KEY = "gb_recent_colors";
  var MAX_RECENT = 12;
  var popover = null;
  var active = null;

  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
  }

  function normHex(v) {
    if (!v) return null;
    v = String(v).trim();
    if (/^[0-9a-fA-F]{3}$/.test(v)) v = v[0] + v[0] + v[1] + v[1] + v[2] + v[2];
    if (/^[0-9a-fA-F]{6}$/.test(v)) return "#" + v.toLower();
    if (/^#[0-9a-fA-F]{6}$/.test(v)) return v.toLowerCase();
    return null;
  }

  function loadRecent() {
    try { return gbStoreGet(RECENT_KEY, []) || []; } catch (e) { return []; }
  }

  function saveRecent(color) {
    var hex = normHex(color);
    if (!hex) return;
    var list = loadRecent().filter(function (c) { return c !== hex; });
    list.unshift(hex);
    if (list.length > MAX_RECENT) list = list.slice(0, MAX_RECENT);
    try { localStorage.setItem(RECENT_KEY, JSON.stringify(list)); } catch (e) {}
  }

  function injectCss() {
    if (document.getElementById("gb-color-picker-css")) return;
    var style = document.createElement("style");
    style.id = "gb-color-picker-css";
    style.textContent = [
      ".gb-cp-trigger{display:inline-flex;align-items:center;justify-content:center;gap:2px;min-width:30px;height:26px;padding:0 5px;border:1px solid transparent;border-radius:6px;background:transparent;cursor:pointer;color:inherit;position:relative;}",
      ".gb-cp-trigger:hover{background:rgba(148,163,184,.14);}",
      ".gb-cp-swatch{width:14px;height:14px;border-radius:3px;border:1px solid rgba(0,0,0,.2);box-shadow:inset 0 0 0 1px rgba(255,255,255,.15);flex:0 0 auto;}",
      ".gb-cp-pop{position:fixed;z-index:10050;background:#1f2937;color:#f9fafb;border:1px solid #374151;border-radius:8px;box-shadow:0 10px 30px rgba(0,0,0,.35);padding:10px;width:236px;font:12px/1.4 system-ui,sans-serif;}",
      ".gb-cp-title{font-size:11px;color:#9ca3af;margin:4px 0 6px;text-transform:uppercase;letter-spacing:.04em;}",
      ".gb-cp-grid{display:grid;grid-template-columns:repeat(8,1fr);gap:4px;}",
      ".gb-cp-cell{width:22px;height:22px;border-radius:4px;border:1px solid rgba(255,255,255,.12);cursor:pointer;padding:0;}",
      ".gb-cp-cell:hover{transform:scale(1.08);border-color:#fff;}",
      ".gb-cp-row{display:flex;gap:6px;align-items:center;margin-top:8px;}",
      ".gb-cp-hex{flex:1;min-width:0;height:28px;border:1px solid #4b5563;border-radius:6px;background:#111827;color:#f9fafb;padding:0 8px;font:12px ui-monospace,monospace;}",
      ".gb-cp-hex:focus{outline:2px solid #60a5fa;border-color:#60a5fa;}",
      ".gb-cp-native{position:relative;overflow:hidden;height:28px;padding:0 10px;border:1px solid #4b5563;border-radius:6px;background:#111827;color:#e5e7eb;cursor:pointer;white-space:nowrap;}",
      ".gb-cp-native input{position:absolute;inset:0;opacity:0;cursor:pointer;}",
      ".gb-cp-recent{min-height:22px;}",
      ".gb-cp-empty{color:#6b7280;font-size:11px;padding:2px 0;}"
    ].join("");
    document.head.appendChild(style);
  }

  function ensurePopover() {
    if (popover) return popover;
    injectCss();
    popover = document.createElement("div");
    popover.className = "gb-cp-pop";
    popover.style.display = "none";
    popover.setAttribute("role", "dialog");
    popover.setAttribute("aria-label", "Choose colour");
    document.body.appendChild(popover);
    document.addEventListener("mousedown", function (e) {
      if (!active) return;
      if (popover.contains(e.target)) return;
      if (active.trigger && active.trigger.contains(e.target)) return;
      close();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && active) close();
    });
    window.addEventListener("scroll", function () { if (active) position(); }, true);
    window.addEventListener("resize", function () { if (active) position(); });
    return popover;
  }

  function position() {
    if (!active || !popover) return;
    var r = active.trigger.getBoundingClientRect();
    var top = r.bottom + 6;
    var left = r.left;
    var pw = 236, ph = popover.offsetHeight || 320;
    if (left + pw > window.innerWidth - 8) left = Math.max(8, window.innerWidth - pw - 8);
    if (top + ph > window.innerHeight - 8) top = Math.max(8, r.top - ph - 6);
    popover.style.top = top + "px";
    popover.style.left = left + "px";
  }

  function render() {
    var p = ensurePopover();
    var current = normHex(active.valueEl.value) || "#000000";
    var recent = loadRecent();
    var recentHtml = recent.length
      ? '<div class="gb-cp-grid gb-cp-recent">' + recent.map(function (c) {
          return '<button type="button" class="gb-cp-cell" data-color="' + esc(c) + '" title="' + esc(c) + '" style="background:' + esc(c) + '"></button>';
        }).join("") + "</div>"
      : '<div class="gb-cp-empty">No recent colours</div>';
    p.innerHTML =
      '<div class="gb-cp-title">Palette</div>' +
      '<div class="gb-cp-grid">' + PALETTE.map(function (c) {
        return '<button type="button" class="gb-cp-cell" data-color="' + c + '" title="' + c + '" style="background:' + c + '"></button>';
      }).join("") + "</div>" +
      '<div class="gb-cp-title" style="margin-top:8px">Recent</div>' +
      recentHtml +
      '<div class="gb-cp-row">' +
        '<input class="gb-cp-hex" type="text" value="' + esc(current) + '" aria-label="Hex colour" spellcheck="false" maxlength="7">' +
        '<label class="gb-cp-native">More…<input type="color" value="' + esc(current) + '" aria-label="More colours"></label>' +
      "</div>";

    p.querySelectorAll("[data-color]").forEach(function (btn) {
      btn.addEventListener("mousedown", function (e) { e.preventDefault(); });
      btn.addEventListener("click", function () {
        commit(btn.getAttribute("data-color"));
      });
    });
    var hex = p.querySelector(".gb-cp-hex");
    function commitHex() {
      var v = normHex(hex.value);
      if (v) { commit(v); }
      else { hex.value = normHex(active.valueEl.value) || "#000000"; }
    }
    hex.addEventListener("keydown", function (e) {
      if (e.key === "Enter") { e.preventDefault(); commitHex(); }
    });
    hex.addEventListener("blur", commitHex);
    var native = p.querySelector(".gb-cp-native input[type='color']");
    if (native) {
      native.addEventListener("input", function () {
        hex.value = native.value;
        preview(native.value);
      });
      native.addEventListener("change", function () {
        commit(native.value);
      });
    }
  }

  function setValue(color, fire) {
    var hex = normHex(color);
    if (!hex || !active) return;
    active.valueEl.value = hex;
    if (active.swatch) active.swatch.style.background = hex;
    if (fire === "preview") {
      active.valueEl.dispatchEvent(new CustomEvent("gb-color-preview", { detail: { value: hex } }));
    } else if (fire === "commit") {
      active.valueEl.dispatchEvent(new CustomEvent("gb-color-commit", { detail: { value: hex } }));
      active.valueEl.dispatchEvent(new Event("change", { bubbles: true }));
    }
  }

  function preview(color) { setValue(color, "preview"); }
  function commit(color) {
    var hex = normHex(color);
    if (!hex) return;
    saveRecent(hex);
    setValue(hex, "commit");
    close();
  }

  function open(inst) {
    active = inst;
    render();
    popover.style.display = "block";
    position();
    var hex = popover.querySelector(".gb-cp-hex");
    if (hex) { try { hex.focus(); hex.select(); } catch (e) {} }
  }

  function close() {
    if (popover) popover.style.display = "none";
    active = null;
  }

  function enhance(valueEl, opts) {
    if (!valueEl || valueEl._gbColorEnhanced) return valueEl._gbColorPicker || null;
    opts = opts || {};
    var trigger = opts.trigger;
    var created = false;
    if (!trigger) {
      trigger = document.createElement("button");
      trigger.type = "button";
      created = true;
    }
    if (!trigger.classList.contains("gb-cp-trigger")) trigger.classList.add("gb-cp-trigger");
    if (!trigger.getAttribute("type")) trigger.setAttribute("type", "button");
    if (created) {
      trigger.title = opts.title || valueEl.title || "Choose colour";
      trigger.setAttribute("aria-label", trigger.title);
      trigger.innerHTML = (opts.iconHtml || "") + '<span class="gb-cp-swatch"></span>';
      valueEl.parentNode.insertBefore(trigger, valueEl);
    }
    var swatch = trigger.querySelector(".gb-cp-swatch");
    if (!swatch) {
      swatch = document.createElement("span");
      swatch.className = "gb-cp-swatch";
      trigger.appendChild(swatch);
    }
    valueEl._gbColorEnhanced = true;
    var inst = { valueEl: valueEl, trigger: trigger, swatch: swatch, opts: opts };
    valueEl._gbColorPicker = inst;
    var initial = normHex(valueEl.value) || normHex(opts.defaultValue) || "#000000";
    valueEl.value = initial;
    swatch.style.background = initial;
    // Hide raw color input if still present; keep as value carrier
    if (valueEl.type === "color") {
      valueEl.style.position = "absolute";
      valueEl.style.opacity = "0";
      valueEl.style.pointerEvents = "none";
      valueEl.style.width = "0";
      valueEl.style.height = "0";
      valueEl.tabIndex = -1;
    }
    trigger.addEventListener("mousedown", function (e) { e.preventDefault(); });
    trigger.addEventListener("click", function (e) {
      e.preventDefault();
      e.stopPropagation();
      if (active === inst) { close(); } else { open(inst); }
    });
    // forward programmatic value changes to swatch
    valueEl.addEventListener("change", function () {
      var h = normHex(valueEl.value);
      if (h) swatch.style.background = h;
    });
    valueEl.addEventListener("gb-color-commit", function () {
      var h = normHex(valueEl.value);
      if (h) swatch.style.background = h;
    });
    valueEl.addEventListener("gb-color-preview", function () {
      var h = normHex(valueEl.value);
      if (h) swatch.style.background = h;
    });
    return inst;
  }

  window.GBColorPicker = {
    enhance: enhance,
    close: close,
    normHex: normHex,
    palette: PALETTE.slice()
  };
})();
