"use strict";
/* Issue #2503 — browser-visible admin replay surface. */
(function (window) {
  "use strict";
  var NS = window.GBEvents;
  if (!NS || !NS.contract) return;
  var contract = NS.contract;

  function cell(row, value) {
    var td = document.createElement("td");
    td.textContent = value;
    row.appendChild(td);
    return td;
  }

  function render() {
    var tbody = document.getElementById("events-dlq");
    if (!tbody) return;
    tbody.textContent = "";
    var letters = NS.delivery.list();
    if (!letters.length) {
      var emptyRow = document.createElement("tr");
      var emptyCell = document.createElement("td");
      emptyCell.colSpan = 4;
      emptyCell.textContent = contract.tr("events-empty", "No dead letters.");
      emptyRow.appendChild(emptyCell);
      tbody.appendChild(emptyRow);
    }
    letters.forEach(function (letter) {
      var row = document.createElement("tr");
      row.dataset.key = letter.idempotency_key;
      cell(row, letter.event_type);
      cell(row, letter.idempotency_key);
      cell(row, String(letter.attempts));
      cell(row, contract.tr("events-status-dead", "Dead lettered"));
      tbody.appendChild(row);
    });
  }

  function message(result) {
    var status = document.getElementById("events-replay-status");
    if (!status) return;
    var out;
    if (result.status === "delivered") {
      out = contract.tr("events-replay-delivered", "Delivered once for {key} (total {count}).",
        { key: result.key, count: result.count });
    } else if (result.status === "idempotent") {
      out = contract.tr("events-replay-idempotent",
        "Already delivered for {key} — no second delivery (total {count}).",
        { key: result.key, count: result.count });
    } else {
      out = contract.tr("events-replay-not-found", "No dead letter for that key.");
    }
    status.textContent = out;
  }

  function updateCount(key) {
    var el = document.getElementById("events-delivery-count");
    if (!el) return;
    el.textContent = contract.tr("events-delivery-count", "Deliveries for this key: {count}",
      { count: NS.delivery.countFor(key) });
  }

  function keyToReplay() {
    var letters = NS.delivery.list();
    if (letters.length) return letters[0].idempotency_key;
    // After a replay the DLQ entry is gone, but the ledger keeps the key —
    // a second click must resolve to the same key to prove exactly-once.
    if (NS.delivery.lastKey) return NS.delivery.lastKey;
    var ledger = NS.delivery.ledger;
    if (ledger && typeof ledger.keys === "function") {
      var first = ledger.keys().next();
      return first.done ? "" : first.value;
    }
    return "";
  }

  function bind() {
    var button = document.getElementById("events-replay-btn");
    if (!button) return;
    button.addEventListener("click", function () {
      var key = keyToReplay();
      if (!key) {
        message({ status: "not_found", key: key, count: 0 });
        return;
      }
      // The live adapter calls the HTTP API (a promise); the offline fallback
      // is synchronous. Promise.resolve normalises both paths.
      Promise.resolve(NS.delivery.replay(key)).then(function (result) {
        message(result);
        updateCount(key);
        render();
      }).catch(function () {
        message({ status: "not_found", key: key, count: 0 });
      });
    });
  }

  NS.admin = { render: render, bind: bind };
})(window);
