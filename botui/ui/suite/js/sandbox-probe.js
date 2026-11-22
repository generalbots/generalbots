"use strict";
/* #2500 — prove the web process cannot read the host stack via file://.
 * A page served over http(s) must be refused a file:// fetch; if one succeeds
 * the WebView sandbox is not isolating the process. */
(function (window, document) {
  function t(key, fallback, params) {
    var value = window.gbT ? window.gbT(key, fallback, params) : fallback;
    if (!value || value.charAt(0) === "[") value = String(fallback);
    return String(value).replace(/\{(\w+)\}/g, function (_, k) {
      return params && params[k] != null ? String(params[k]) : "{" + k + "}";
    });
  }
  function run() {
    var out = document.getElementById("osSandboxProbeResult");
    if (!out) return;
    var targets = ["file:///opt/gbo/data/", "file:///opt/gbo/bin/botserver"];
    Promise.all(targets.map(function (url) {
      return fetch(url).then(function () { return false; }).catch(function () { return true; });
    })).then(function (blocked) {
      var ok = blocked.every(Boolean);
      out.textContent = t("os-sandbox-result", "Web process sandboxed: {state}", {
        state: ok ? "yes" : "no"
      });
      out.setAttribute("data-sandboxed", ok ? "true" : "false");
      document.dispatchEvent(new CustomEvent("gb:sandbox-probe", { detail: { sandboxed: ok, blocked: blocked } }));
    });
  }
  window.GBSandboxProbe = { run: run };
})(window, document);
