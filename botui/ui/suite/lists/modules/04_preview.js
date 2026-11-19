"use strict";
/* #2537 - preview transport. Tries the structured preview endpoint; when it
   is absent (404/405/501) or unreachable, degrades to the clearly-labelled
   offline estimate. Never presents sample numbers as server data. */
(function (global) {
  var ns = global.GBSegments;
  if (!ns) return;

  ns.PREVIEW_ENDPOINT = "/api/crm/segments/preview";
  ns.previewState = { serverAvailable: null };

  var activeController = null;

  ns.cancelPreview = function () {
    if (activeController && typeof activeController.abort === "function") {
      activeController.abort();
    }
    activeController = null;
  };

  function toRows(values) {
    if (!Array.isArray(values)) return [];
    return values.slice(0, 10).map(function (row) {
      if (!row || typeof row !== "object") return null;
      var name = row.name || [row.first_name, row.last_name].filter(Boolean).join(" ").trim();
      return {
        name: String(name || "—"),
        email: String(row.email || ""),
        status: String(row.lifecycle_status || row.status || "")
      };
    }).filter(Boolean);
  }

  function fromServer(payload) {
    if (!payload || typeof payload !== "object") return null;
    var matched = payload.matched != null ? payload.matched : payload.count;
    var suppressed = payload.suppressed != null ? payload.suppressed : 0;
    if (!isFinite(Number(matched)) || !isFinite(Number(suppressed))) return null;
    var rows = toRows(payload.matches || payload.items || payload.contacts || []);
    return {
      source: "server",
      matched: Number(matched),
      suppressed: Number(suppressed),
      items: rows
    };
  }

  function unavailableStatus(status) {
    return status === 404 || status === 405 || status === 501 || status === 0;
  }

  ns.requestPreview = function (model, done) {
    ns.cancelPreview();
    var fallback = function () { done(ns.offlinePreview(model)); };

    if (ns.previewState.serverAvailable === false || typeof global.fetch !== "function") {
      fallback();
      return;
    }

    var controller = typeof global.AbortController === "function" ? new AbortController() : null;
    activeController = controller;
    var options = {
      method: "POST",
      headers: { "Content-Type": "application/json", "Accept": "application/json" },
      body: JSON.stringify({ segment: ns.normalize(model) })
    };
    if (controller) options.signal = controller.signal;

    global.fetch(ns.PREVIEW_ENDPOINT, options).then(function (response) {
      if (activeController !== controller) return null; // stale request
      if (!response || !response.ok) {
        if (response && unavailableStatus(response.status)) ns.previewState.serverAvailable = false;
        return null;
      }
      return response.json();
    }).then(function (payload) {
      if (activeController !== controller) return;
      var result = payload ? fromServer(payload) : null;
      if (result) {
        ns.previewState.serverAvailable = true;
        done(result);
      } else {
        fallback();
      }
    }).catch(function () {
      if (activeController !== controller) return;
      ns.previewState.serverAvailable = false;
      fallback();
    });
  };

  ns.previewSourceLabel = function (result) {
    return result && result.source === "server"
      ? ns.t("segments-preview-source-server", "Live preview from server")
      : ns.t("segments-preview-source-sample", "SAMPLE DATA — OFFLINE ESTIMATE (not from server)");
  };
})(window);
