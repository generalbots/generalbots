"use strict";
/* Issue #2503 — suite automation consuming the native queue (push, no polling). */
(function (window) {
  "use strict";
  var NS = window.GBEvents;
  if (!NS || !NS.contract) return;
  var contract = NS.contract;

  var SUBSCRIPTION = {
    id: "sub_native_audit_log",
    bot_id: "automation-bot",
    event_type: contract.canonical("sheet.exported")
  };

  // One log <li> is appended per native reaction, so the log is the live
  // reaction count in both the simulation and the WebSocket transport.
  function reactionCount() {
    var list = document.getElementById("events-automation-log");
    return list ? list.children.length : 0;
  }

  function logReaction(envelope) {
    var list = document.getElementById("events-automation-log");
    if (!list) return;
    var item = document.createElement("li");
    item.textContent = contract.tr("events-automation-log-line",
      "Automation reacted to {type} — event {id}",
      {
        event: envelope.event_type,
        type: envelope.event_type,
        id: envelope.event_id
      });
    list.appendChild(item);
    // The live socket delivers envelopes asynchronously, after the click
    // handler has already called setStatus(); refresh on each reaction too.
    setStatus();
  }

  function setStatus() {
    var status = document.getElementById("events-automation-status");
    if (status) {
      status.textContent = contract.tr("events-automation-status",
        "Reactions: {count}", { count: reactionCount() });
    }
    var poll = document.getElementById("events-poll-count");
    if (poll) {
      poll.textContent = contract.tr("events-poll-label", "Polls: {count} (event-driven)",
        { count: NS.queue.pollCount });
    }
  }

  function bind() {
    var button = document.getElementById("events-export-btn");
    if (!button) return;
    button.addEventListener("click", function () {
      var event = contract.makeEvent("sheet.exported", {
        doc_id: "sheet_456",
        metadata: { title: "Quarterly report" }
      });
      NS.queue.publish(event);
      setStatus();
    });
  }

  function init() {
    NS.queue.subscribe({
      id: SUBSCRIPTION.id,
      bot_id: SUBSCRIPTION.bot_id,
      event_type: SUBSCRIPTION.event_type,
      onEnvelope: logReaction
    });
    bind();
    setStatus();
  }

  NS.automation = { init: init };
})(window);
