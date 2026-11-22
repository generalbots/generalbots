"use strict";
/* Issue #2503 — boot the Events app. */
(function (window) {
  "use strict";
  var NS = window.GBEvents;
  if (!NS || !NS.contract) return;
  var contract = NS.contract;

  function renderCatalog() {
    var list = document.getElementById("events-catalog");
    if (!list) return;
    list.textContent = "";
    contract.EVENT_NAMES.forEach(function (name) {
      var item = document.createElement("li");
      var label = document.createElement("code");
      label.textContent = name;
      var example = document.createElement("span");
      example.textContent = contract.tr("events-catalog-example",
        "example: evt_01J8Z3KQ2M · doc_456 · metadata only (no body)", {});
      item.appendChild(label);
      item.appendChild(example);
      list.appendChild(item);
    });
  }

  function renderVersion() {
    var el = document.getElementById("events-schema-version");
    if (el) {
      el.textContent = contract.tr("events-schema-version", "schema v{version}",
        { version: contract.SCHEMA_VERSION });
    }
  }

  function init() {
    if (!document.getElementById("events-app")) return;
    NS.queue = new contract.NativeQueue();
    NS.delivery = new contract.DeliveryEngine();

    // One realistic dead letter: a sheets.exported webhook that exhausted its
    // retry policy with a 5xx, retained for admin replay by idempotency key.
    var letter = contract.makeEvent("sheet.exported", {
      event_id: "evt_demo_001",
      doc_id: "sheet_456"
    });
    var key = contract.idempotencyKey(letter.event_id, "sub_webhook_catalog");
    NS.delivery.seedDeadLetter(letter, "sub_webhook_catalog", 5, "HTTP 503");

    if (NS.admin) { NS.admin.bind(); NS.admin.render(); }
    if (NS.automation) NS.automation.init();
    renderCatalog();
    renderVersion();

    var count = document.getElementById("events-delivery-count");
    if (count) {
      count.textContent = contract.tr("events-delivery-count", "Deliveries for this key: {count}",
        { count: NS.delivery.countFor(key) });
    }
    var poll = document.getElementById("events-poll-count");
    if (poll) {
      poll.textContent = contract.tr("events-poll-label", "Polls: {count} (event-driven)",
        { count: NS.queue.pollCount });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})(window);
