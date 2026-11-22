"use strict";
/* Issue #2503 — real event API client.
 *
 * Talks to the botserver Office event surface: JSON endpoints for the DLQ /
 * replay / publish and a WebSocket for the native stream. Every helper settles
 * to a rejected promise when the server is offline, which lets 99_init.js fall
 * back to the in-page simulation. No polling timer exists anywhere here. */
(function (window) {
  "use strict";
  var NS = (window.GBEvents = window.GBEvents || {});

  function token() {
    if (typeof window.getGBAccessToken === "function") {
      return window.getGBAccessToken() || "";
    }
    return "";
  }

  function requestHeaders(withBody) {
    var value = {};
    var bearer = token();
    if (bearer) value.Authorization = "Bearer " + bearer;
    if (withBody) value["Content-Type"] = "application/json";
    return value;
  }

  function request(path, options) {
    var opts = options || {};
    return fetch(path, {
      method: opts.method || "GET",
      headers: requestHeaders(!!opts.body),
      body: opts.body ? JSON.stringify(opts.body) : undefined,
      cache: "no-store"
    }).then(function (response) {
      if (!response.ok) throw new Error("events API " + response.status);
      return response.json();
    });
  }

  function list() {
    return request("/api/events/dead-letters");
  }

  function replay(idempotencyKey) {
    return request("/api/events/replay", {
      method: "POST",
      body: { idempotency_key: idempotencyKey }
    });
  }

  function publish(eventType, options) {
    var body = { event_type: eventType };
    var opts = options || {};
    ["doc_id", "workspace_id", "actor_id", "webhook_url"].forEach(function (key) {
      if (opts[key]) body[key] = opts[key];
    });
    return request("/api/events/publish", { method: "POST", body: body });
  }

  /* The botui proxy (dev :3000) rewrites arbitrary /ws paths, so the native
   * socket is opened directly against botserver on :8080 in that one dev
   * configuration. Every other host (embedded UI on :8080, Caddy in prod)
   * uses the same origin. CORS does not apply to WebSocket handshakes. */
  function streamUrl(types) {
    var scheme = window.location.protocol === "https:" ? "wss:" : "ws:";
    var host = window.location.port === "3000" ? "localhost:8080" : window.location.host;
    var query = "?types=" + encodeURIComponent(types || "sheets.exported");
    var bearer = token();
    if (bearer) query += "&access_token=" + encodeURIComponent(bearer);
    return scheme + "//" + host + "/ws/events" + query;
  }

  function openStream(types, onEnvelope) {
    var socket;
    try {
      socket = new WebSocket(streamUrl(types));
    } catch (error) {
      return function () {};
    }
    socket.onmessage = function (message) {
      var envelope;
      try {
        envelope = JSON.parse(message.data);
      } catch (error) {
        return;
      }
      if (envelope && envelope.event_type) onEnvelope(envelope);
    };
    return function () {
      if (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING) {
        socket.close();
      }
    };
  }

  /* Live adapter with the same small surface the local simulation exposes,
   * so 02_admin.js and 03_automation.js do not branch on the transport. */
  function ApiDelivery() {
    this.cache = [];
    this.counts = {};
    this.lastKey = "";
  }
  ApiDelivery.prototype.list = function () {
    return this.cache;
  };
  ApiDelivery.prototype.refresh = function () {
    var self = this;
    return list().then(function (letters) {
      self.cache = letters || [];
      if (self.cache.length) self.lastKey = self.cache[0].idempotency_key;
      return self.cache;
    });
  };
  ApiDelivery.prototype.replay = function (idempotencyKey) {
    var self = this;
    return replay(idempotencyKey).then(function (result) {
      self.lastKey = idempotencyKey;
      if (!result.deduped) {
        self.counts[idempotencyKey] = 1 + (self.counts[idempotencyKey] || 0);
      }
      return self.refresh().then(function () {
        return {
          status: result.deduped ? "idempotent" : "delivered",
          key: idempotencyKey,
          count: self.counts[idempotencyKey]
        };
      });
    });
  };
  ApiDelivery.prototype.countFor = function (idempotencyKey) {
    return this.counts[idempotencyKey] || 0;
  };

  function ApiQueue() {
    this.listeners = [];
    this.pollCount = 0;
    this.closeStream = null;
  }
  ApiQueue.prototype.subscribe = function (subscription) {
    this.listeners.push(subscription);
    if (this.closeStream) return;
    var self = this;
    var types = this.listeners
      .map(function (listener) { return listener.event_type; })
      .join(",");
    this.closeStream = openStream(types, function (envelope) {
      self.listeners.forEach(function (listener) {
        if (listener.event_type === envelope.event_type &&
            typeof listener.onEnvelope === "function") {
          listener.onEnvelope(envelope);
        }
      });
    });
  };
  ApiQueue.prototype.publish = function (event) {
    return publish(event.type, {
      doc_id: event.doc_id,
      workspace_id: event.workspace_id,
      actor_id: event.actor_id
    });
  };
  ApiQueue.prototype.pending = function () {
    return 0;
  };

  NS.api = {
    list: list,
    replay: replay,
    publish: publish,
    streamUrl: streamUrl,
    openStream: openStream,
    ApiDelivery: ApiDelivery,
    ApiQueue: ApiQueue
  };
})(window);
