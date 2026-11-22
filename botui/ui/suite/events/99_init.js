"use strict";
/* Issue #2503 — boot the Events app against the real event API, falling back
 * to the in-page simulation only when the server cannot be reached. */
(function (window) {
  "use strict";
  var NS = window.GBEvents;
  if (!NS || !NS.contract) return;
  var contract = NS.contract;

  function tr(key, fallback, params) {
    return contract.tr(key, fallback, params);
  }

  function renderCatalog() {
    var list = document.getElementById("events-catalog");
    if (!list) return;
    list.textContent = "";
    contract.EVENT_NAMES.forEach(function (name) {
      var item = document.createElement("li");
      var label = document.createElement("code");
      label.textContent = name;
      var example = document.createElement("span");
      example.textContent = tr("events-catalog-example",
        "example: evt_01J8Z3KQ2M · doc_456 · metadata only (no body)", {});
      item.appendChild(label);
      item.appendChild(example);
      list.appendChild(item);
    });
  }

  function renderVersion() {
    var el = document.getElementById("events-schema-version");
    if (el) {
      el.textContent = tr("events-schema-version", "schema v{version}",
        { version: contract.SCHEMA_VERSION });
    }
  }

  function markSource(source) {
    var app = document.getElementById("events-app");
    if (app) app.dataset.source = source;
  }

  function renderCounts() {
    var count = document.getElementById("events-delivery-count");
    if (count && NS.delivery) {
      count.textContent = tr("events-delivery-count", "Deliveries for this key: {count}",
        { count: NS.delivery.countFor(NS.delivery.lastKey || "") });
    }
    var poll = document.getElementById("events-poll-count");
    if (poll && NS.queue) {
      poll.textContent = tr("events-poll-label", "Polls: {count} (event-driven)",
        { count: NS.queue.pollCount });
    }
  }

  function bindApp() {
    if (NS.admin) { NS.admin.bind(); NS.admin.render(); }
    if (NS.automation) NS.automation.init();
    renderCatalog();
    renderVersion();
    renderCounts();
  }

  /* Real path: the DLQ list came from GET /api/events/dead-letters and the
   * native queue is the authenticated /ws/events socket. */
  function useLive(letters) {
    markSource("live");
    NS.delivery = new NS.api.ApiDelivery();
    NS.delivery.cache = letters || [];
    if (NS.delivery.cache.length) {
      NS.delivery.lastKey = NS.delivery.cache[0].idempotency_key;
    }
    NS.queue = new NS.api.ApiQueue();
    bindApp();
  }

  /* Offline path: keep the original in-page mirror so the app still works
   * with no server (the desktop shell before botserver is ready). */
  function useSimulation() {
    markSource("simulation");
    NS.queue = new contract.NativeQueue();
    NS.delivery = new contract.DeliveryEngine();

    var letter = contract.makeEvent("sheet.exported", {
      event_id: "evt_demo_001",
      doc_id: "sheet_456"
    });
    NS.delivery.seedDeadLetter(letter, "sub_webhook_catalog", 5, "HTTP 503");
    bindApp();
  }

  function init() {
    if (!document.getElementById("events-app")) return;
    markSource("connecting");
    if (!NS.api || typeof NS.api.list !== "function") {
      useSimulation();
      return;
    }
    try {
      NS.api.list().then(function (letters) {
        useLive(letters);
      }).catch(function () {
        useSimulation();
      });
    } catch (error) {
      useSimulation();
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})(window);
