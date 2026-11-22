"use strict";
/* #2455 — sound controls for the OS appliance. Gated OS API calls plus local
 * WebAudio test tone; DOM events mirror hardware volume/mute keys for UAT. */
(function (window, document) {
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function id(x) { return document.getElementById(x); }
  function headers() {
    var h = { "Content-Type": "application/json", Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function set(setting, value) {
    if (setting === "volume") {
      var out = id("osVolumeValue");
      if (out) out.textContent = value + "%";
    }
    fetch("/api/os/sound/controls", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ setting: setting, value: String(value) })
    }).then(function (r) { return r.json(); }).then(function (body) {
      document.dispatchEvent(new CustomEvent("gb:sound-change", {
        detail: { setting: setting, value: value, ok: !!body.ok }
      }));
    }).catch(function () { /* web mode */ });
  }
  function testTone() {
    try {
      var Ctx = window.AudioContext || window.webkitAudioContext;
      if (!Ctx) { document.dispatchEvent(new CustomEvent("gb:sound-tone", { detail: { ok: false } })); return; }
      var ctx = new Ctx();
      var osc = ctx.createOscillator();
      var gain = ctx.createGain();
      osc.frequency.value = 440;
      gain.gain.value = 0.08;
      osc.connect(gain); gain.connect(ctx.destination);
      osc.start();
      setTimeout(function () { osc.stop(); ctx.close(); document.dispatchEvent(new CustomEvent("gb:sound-tone", { detail: { ok: true } })); }, 400);
    } catch (e) { document.dispatchEvent(new CustomEvent("gb:sound-tone", { detail: { ok: false } })); }
  }
  function inputLevel(value) {
    var meter = id("osInputMeter");
    if (meter) meter.value = Math.max(0, Math.min(100, Number(value) || 0));
  }
  function init() {
    document.addEventListener("gb:sound-input-level", function (e) { inputLevel(e.detail && e.detail.level); });
    document.addEventListener("gb:sound-key", function (e) {
      var d = e.detail || {};
      if (d.action === "volume-up" || d.action === "volume-down") {
        var slider = id("osVolume");
        var next = Math.max(0, Math.min(100, Number(slider ? slider.value : 70) + (d.action === "volume-up" ? 5 : -5)));
        if (slider) slider.value = next;
        set("volume", next);
      } else if (d.action === "mute") {
        var mute = id("osMute");
        if (mute) mute.checked = !mute.checked;
        set("mute", mute && mute.checked ? "on" : "off");
      }
    });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBSoundControls = { set: set, testTone: testTone, inputLevel: inputLevel };
})(window, document);
