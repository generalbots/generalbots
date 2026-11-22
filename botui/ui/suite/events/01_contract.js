"use strict";
/* Issue #2503 — client mirror of botevents::office (the Rust contract).
 * Semantics intentionally match NativeDispatcher / WebhookDispatcher:
 * push-based native fan-out, an idempotency ledger and a dead-letter queue.
 * No HTTP is used and there is no polling timer anywhere in this app. */
(function (window) {
  "use strict";
  var NS = (window.GBEvents = window.GBEvents || {});

  var SCHEMA_VERSION = 1;
  // The issue text uses the singular alias; the Rust taxonomy publishes the
  // plural wire name. Both resolve to one canonical name here.
  var ALIASES = { "sheet.exported": "sheets.exported" };
  var EVENT_NAMES = [
    "doc.created", "doc.updated", "doc.commented", "doc.review_requested",
    "doc.approved", "doc.published", "doc.exported", "doc.deleted",
    "doc.comment.added", "doc.comment.updated", "doc.comment.deleted",
    "doc.comment.mention.added", "slides.created", "slides.updated",
    "slides.presented", "slides.exported", "slides.deleted",
    "sheets.created", "sheets.updated", "sheets.refreshed",
    "sheets.exported", "sheets.deleted"
  ];

  function tr(key, fallback, params) {
    // Params are substituted centrally by the shared i18n helper: window.gbT
    // is tOr(), which substitutes into both the catalogue value (via t()) and
    // the authored fallback on a catalogue miss (i18n.js tOr, #2503/#2548).
    // A second local pass here would be redundant and weaker (exact {name}
    // only, plus a double-substitution hazard), so delegate directly.
    return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
  }
  function canonical(type) {
    return ALIASES[type] || type;
  }
  function idempotencyKey(eventId, subscriptionId) {
    return eventId + ":" + subscriptionId;
  }
  function nowIso() {
    return new Date().toISOString();
  }

  var sequence = 0;
  function makeEvent(type, overrides) {
    overrides = overrides || {};
    sequence += 1;
    return {
      event_id: overrides.event_id || "evt_demo_" + String(sequence).padStart(3, "0"),
      type: canonical(type),
      schema_version: SCHEMA_VERSION,
      app: overrides.app || "sheets",
      workspace_id: overrides.workspace_id || "ws_123",
      doc_id: overrides.doc_id || "doc_456",
      actor_id: overrides.actor_id || "usr_789",
      occurred_at: nowIso(),
      tags: overrides.tags || ["finance"],
      // Privacy default: IDs and metadata only — never a document body.
      metadata: overrides.metadata || { title: "Quarterly report" }
    };
  }

  /* Native queue — mirrors botevents::office::NativeDispatcher. Delivery is
   * synchronous (push). `pollCount` exists only to prove it stays zero. */
  function NativeQueue() {
    this.queue = [];
    this.listeners = [];
    this.enqueued = 0;
    this.pollCount = 0;
  }
  NativeQueue.prototype.subscribe = function (subscription) {
    this.listeners.push(subscription);
  };
  NativeQueue.prototype.publish = function (event) {
    var type = canonical(event.type);
    var self = this;
    var delivered = 0;
    this.listeners.forEach(function (subscription) {
      if (subscription.event_type !== type) return;
      var envelope = {
        id: "native-" + (self.enqueued + 1),
        bot_id: subscription.bot_id,
        subscription_id: subscription.id,
        event_id: event.event_id,
        event_type: type,
        app: event.app,
        payload: event,
        enqueued_at: nowIso()
      };
      self.enqueued += 1;
      self.queue.push(envelope);
      self.listeners.forEach(function (listener) {
        if (typeof listener.onEnvelope === "function") listener.onEnvelope(envelope);
      });
      delivered += 1;
    });
    return delivered;
  };
  NativeQueue.prototype.drain = function (botId) {
    var out = [];
    for (var i = this.queue.length - 1; i >= 0; i -= 1) {
      if (this.queue[i].bot_id === botId) out.push(this.queue.splice(i, 1)[0]);
    }
    return out;
  };
  NativeQueue.prototype.pending = function () {
    return this.queue.length;
  };

  /* Webhook replay engine — mirrors the ledger/DLQ part of WebhookDispatcher.
   * `replay` returns an idempotent receipt on a duplicate key, so a key can
   * never produce a second delivery. */
  function DeliveryEngine() {
    this.ledger = new Map();
    this.dlq = new Map();
    this.log = [];
  }
  DeliveryEngine.prototype.seedDeadLetter = function (event, subscriptionId, attempts, lastError) {
    var key = idempotencyKey(event.event_id, subscriptionId);
    this.dlq.set(key, {
      idempotency_key: key,
      event_id: event.event_id,
      event_type: event.type,
      subscription_id: subscriptionId,
      attempts: attempts,
      last_error: lastError,
      body: event
    });
    return this.dlq.get(key);
  };
  DeliveryEngine.prototype.list = function () {
    var out = [];
    this.dlq.forEach(function (letter) { out.push(letter); });
    return out.sort(function (a, b) { return a.event_id.localeCompare(b.event_id); });
  };
  DeliveryEngine.prototype.countFor = function (key) {
    var receipt = this.ledger.get(key);
    return receipt ? receipt.deliveryCount : 0;
  };
  DeliveryEngine.prototype.replay = function (key) {
    var receipt = this.ledger.get(key);
    if (receipt) {
      return { status: "idempotent", key: key, count: receipt.deliveryCount };
    }
    var letter = this.dlq.get(key);
    if (!letter) return { status: "not_found", key: key, count: 0 };
    receipt = {
      idempotency_key: key,
      event_id: letter.event_id,
      delivered_at: nowIso(),
      deliveryCount: 1
    };
    this.ledger.set(key, receipt);
    this.dlq.delete(key);
    this.log.push(receipt);
    return { status: "delivered", key: key, count: receipt.deliveryCount };
  };

  NS.contract = {
    SCHEMA_VERSION: SCHEMA_VERSION,
    EVENT_NAMES: EVENT_NAMES,
    ALIASES: ALIASES,
    tr: tr,
    canonical: canonical,
    idempotencyKey: idempotencyKey,
    makeEvent: makeEvent,
    NativeQueue: NativeQueue,
    DeliveryEngine: DeliveryEngine
  };
})(window);
