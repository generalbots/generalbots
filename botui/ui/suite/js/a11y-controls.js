"use strict";
/* #2469 — suite-side accessibility controls. These mirror the image's AT-SPI
 * packages (orca/squeekboard) in the web shell so the settings, lock screen and
 * tray are operable with touch or keyboard alone. All state persists in
 * localStorage, surviving a page reload / shell restart. */
(function (window, document) {
  var KEY = "gb-a11y-settings";
  var IDS = {
    osk: "a11yOnScreenKeyboard",
    zoom: "a11yMagnifier",
    pointer: "a11yLargePointer",
    contrast: "a11yHighContrast",
    sticky: "a11yStickyKeys",
    slow: "a11ySlowKeys",
    bounce: "a11yBounceKeys"
  };
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function load() {
    try { return JSON.parse(localStorage.getItem(KEY) || "{}") || {}; }
    catch (e) { return {}; }
  }
  function save(state) {
    try { localStorage.setItem(KEY, JSON.stringify(state)); } catch (e) { /* ignore */ }
  }
  function apply(state) {
    Object.keys(IDS).forEach(function (k) {
      var on = !!state[k];
      document.body.classList.toggle("gb-a11y-" + k, on);
      var el = document.getElementById(IDS[k]);
      if (el) el.checked = on;
    });
    document.documentElement.style.zoom = state.zoom ? "1.25" : "";
    if (keyboard) keyboard.hidden = !state.osk;
  }
  function setOption(name, value) {
    var state = load();
    state[name] = !!value;
    save(state);
    apply(state);
    document.dispatchEvent(new CustomEvent("gb:a11y-change", { detail: state }));
  }

  var keyboard = null;
  function ensureKeyboard() {
    if (keyboard) return keyboard;
    keyboard = document.createElement("div");
    keyboard.id = "gb-osk";
    keyboard.className = "gb-osk";
    keyboard.setAttribute("role", "dialog");
    keyboard.setAttribute("aria-label", t("os-a11y-osk", "On-screen keyboard"));
    keyboard.hidden = true;
    var rows = ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm,.-"];
    var html = "";
    rows.forEach(function (row) {
      html += '<div class="gb-osk-row">';
      row.split("").forEach(function (ch) {
        html += '<button type="button" data-key="' + ch + '">' + ch + "</button>";
      });
      html += "</div>";
    });
    html += '<div class="gb-osk-row"><button type="button" data-key="SPACE">Space</button>' +
      '<button type="button" data-key="BACKSPACE">Backspace</button>' +
      '<button type="button" data-key="ENTER">Enter</button>' +
      '<button type="button" data-key="CLOSE">×</button></div>';
    keyboard.innerHTML = html;
    keyboard.addEventListener("click", function (e) {
      var b = e.target.closest("button");
      if (!b) return;
      var key = b.getAttribute("data-key");
      if (key === "CLOSE") { keyboard.hidden = true; return; }
      var el = document.activeElement;
      if (!el || !/INPUT|TEXTAREA/.test(el.tagName)) return;
      if (key === "BACKSPACE") el.value = el.value.slice(0, -1);
      else if (key === "SPACE") el.value += " ";
      else if (key === "ENTER") return;
      else el.value += key;
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });
    document.body.appendChild(keyboard);
    return keyboard;
  }
  function toggleKeyboard(force) {
    var k = ensureKeyboard();
    k.hidden = force === undefined ? !k.hidden : !force;
  }

  function init() {
    if (load().osk) ensureKeyboard();
    apply(load());
    Object.keys(IDS).forEach(function (k) {
      var el = document.getElementById(IDS[k]);
      if (el) el.addEventListener("change", function () { setOption(k, el.checked); });
    });
    document.addEventListener("keydown", function (e) {
      if (e.altKey && e.key.toLowerCase() === "k") { e.preventDefault(); toggleKeyboard(); }
      if (e.altKey && e.key.toLowerCase() === "a") {
        e.preventDefault();
        if (window.openDeepLink) window.openDeepLink("settings", { section: "os-accessibility" });
      }
    });
    document.addEventListener("gb:osk", function (e) { toggleKeyboard(e.detail && e.detail.show); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBA11y = { set: setOption, state: load, keyboard: toggleKeyboard, apply: apply };
})(window, document);
