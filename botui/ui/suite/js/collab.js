"use strict";
/* GBCollab — shared realtime collaboration client (WebSocket + presence).
 * Used by sheet, docs, slides, plan. Public API (window.GBCollab): connect({
 * app, docId, host?, callbacks... }), send*, disconnect, retry, getUser, helpers.
 * Auth: a SaaS JWT from localStorage.gb-access-token is sent as ?token=... on
 * the upgrade; Slides rejects an invalid token (401) and never upgrades.
 *
 * #2926 — reconnects use bounded exponential backoff and stop after a finite
 * number of attempts. A 1008 close (policy violation) or a 1006 whose reason
 * names a policy denial / HTTP 4xx is TERMINAL: browsers hide the status of a
 * denied upgrade and report only 1006, so 401/403/404/forbidden/denied in the
 * reason is treated conservatively as a rejection and never retried. A bare
 * 1006 is a network drop and gets bounded backoff — never a tab-lifetime loop.
 */
(function (window) {
  const RECONNECT_DELAY_MS = 2000;
  const BACKOFF_MAX_MS = 30000;
  const MAX_RECONNECT_ATTEMPTS = 5;
  const TERMINAL_REASON = /(?:policy|forbidden|unauthori[sz]ed|not found|denied|reject|\b(?:401|403|404|405)\b)/i;
  const HEARTBEAT_MS = 25000;
  const TYPING_AWAY_MS = 4000;
  const STATUS_ID = "gb-collab-status";
  const STORAGE_KEYS = { TOKEN: "gb-access-token", WS_TOKEN: "gb-ws-token", USER: "gb-user-data" };

  function readUser() {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.USER);
      if (raw) {
        const u = JSON.parse(raw);
        if (u && (u.id || u.sub || u.user_id)) return {
          id: u.id || u.sub || u.user_id,
          name: u.display_name || u.name || u.email || "User",
          color: u.color || pickColor(u.id || u.sub || u.user_id)
        };
      }
    } catch (_) { window.gbLogError ? gbLogError(_, "readUser") : console.error("readUser", _); }
    const id = "anon-" + Math.random().toString(36).slice(2, 10);
    return { id: id, name: "Guest", color: pickColor(id) };
  }

  function readToken() {
    // WebSockets verify a SaaS JWT; REST uses an opaque `gb_…` token. Mixing
    // them made every socket fail with malformed_token, so prefer WS_TOKEN.
    try {
      const ws = localStorage.getItem(STORAGE_KEYS.WS_TOKEN) ||
        sessionStorage.getItem(STORAGE_KEYS.WS_TOKEN) || "";
      if (ws) return ws;
      const rest = localStorage.getItem(STORAGE_KEYS.TOKEN) ||
        sessionStorage.getItem(STORAGE_KEYS.TOKEN) || "";
      return rest.indexOf("eyJ") === 0 ? rest : "";
    } catch (_) { return ""; }
  }

  window.setGBWSToken = function (token) {
    try {
      ["localStorage", "sessionStorage"].forEach(function (name) {
        const store = window[name];
        if (!store) return;
        if (token) store.setItem(STORAGE_KEYS.WS_TOKEN, token);
        else store.removeItem(STORAGE_KEYS.WS_TOKEN);
      });
    } catch (_) { /* storage disabled */ }
  };

  function pickColor(seed) {
    const palette = [
      "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD",
      "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA"
    ];
    let h = 0;
    for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) | 0;
    return palette[Math.abs(h) % palette.length];
  }

  function el(tag, attrs, children) {
    const node = document.createElement(tag);
    if (attrs) for (const k in attrs) {
      if (k === "style" && typeof attrs[k] === "object") Object.assign(node.style, attrs[k]);
      else if (k === "class") node.className = attrs[k];
      else if (k === "html") node.innerHTML = attrs[k];
      else if (k.startsWith("on") && typeof attrs[k] === "function") node.addEventListener(k.slice(2), attrs[k]);
      else if (k === "data" && typeof attrs[k] === "object") {
        for (const dk in attrs[k]) node.dataset[dk] = attrs[k][dk];
      } else node.setAttribute(k, attrs[k]);
    }
    if (children) {
      const list = Array.isArray(children) ? children : [children];
      list.forEach(function (c) {
        if (c == null) return;
        node.appendChild(typeof c === "string" ? document.createTextNode(c) : c);
      });
    }
    return node;
  }

  function avatar(user, size) {
    const px = size || 28;
    const initial = (user.user_name || user.name || "?").trim().charAt(0).toUpperCase();
    return el("span", {
      class: "gb-collab-avatar", title: user.user_name || user.name || user.user_id,
      data: { userId: user.user_id, color: user.user_color || user.color },
      style: {
        display: "inline-flex", alignItems: "center", justifyContent: "center",
        width: px + "px", height: px + "px", borderRadius: "50%",
        background: user.user_color || user.color || "#3b82f6", color: "#fff",
        fontWeight: "600", fontSize: Math.max(10, Math.floor(px * 0.45)) + "px",
        marginLeft: "-6px", border: "2px solid #0f172a",
        boxShadow: "0 1px 3px rgba(0,0,0,0.4)", position: "relative"
      }, html: escapeHtml(initial)
    });
  }

  function escapeHtml(s) {
    return String(s).replace(/[&<>"']/g, function (c) {
      return ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c];
    });
  }

  function renderAvatars(container, users) {
    if (!container) return;
    container.innerHTML = "";
    if (!users || !users.length) return;
    const max = container.dataset.maxAvatars ? parseInt(container.dataset.maxAvatars, 10) : 5;
    const overflow = users.length - max;
    const visible = users.slice(0, max);
    visible.forEach(function (u) { container.appendChild(avatar(u)); });
    if (overflow > 0) container.appendChild(el("span", {
      class: "gb-collab-avatar gb-collab-overflow",
      style: {
        display: "inline-flex", alignItems: "center", justifyContent: "center",
        width: "28px", height: "28px", borderRadius: "50%",
        background: "#334155", color: "#f8fafc", fontSize: "11px", fontWeight: "600",
        marginLeft: "-6px", border: "2px solid #0f172a"
      }, html: "+" + overflow
    }));
    container.dataset.userCount = String(users.length);
  }

  // The suite loads window.i18n with t(key); a missing key falls back to the
  // English default supplied here. #2926 uses window.gbT for the status text.
  function tr(key, fallback, vars) {
    var text = fallback;
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      if (value && value !== key) text = value;
    }
    if (vars) Object.keys(vars).forEach(function (name) {
      text = text.split("{" + name + "}").join(vars[name]);
    });
    return text;
  }

  function T(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }

  function renderTypingIndicator(container, users) {
    if (!container) return;
    container.innerHTML = "";
    if (!users || !users.length) return;
    const names = users.slice(0, 3).map(function (u) { return escapeHtml(u.user_name || u.name); });
    // #2029 — the indicator was hardcoded Portuguese next to English chrome.
    const text = names.length === 1
      ? tr("collab-typing-one", "{name} is typing…", { name: names[0] })
      : names.length === 2
        ? tr("collab-typing-two", "{first} and {second} are typing…", { first: names[0], second: names[1] })
        : tr("collab-typing-many", "{name} and {count} others are typing…", { name: names[0], count: users.length - 1 });
    container.appendChild(el("div", { class: "gb-typing-text", style: { color: "#94a3b8", fontSize: "12px", padding: "4px 8px", fontStyle: "italic" }, html: text }));
  }

  function defaultUrl(app, docId) {
    const loc = window.location;
    const proto = loc.protocol === "https:" ? "wss:" : "ws:";
    // Same-origin: botui (:3000) and prod Caddy both proxy /ws to botserver,
    // so the page host is correct everywhere; hardcoding :8080 breaks prod.
    return proto + "//" + loc.host + "/ws/" + app + "/" + encodeURIComponent(docId);
  }

  function GBCollab() {}

  GBCollab.prototype._init = function (opts) {
    this.opts = opts || {};
    this.app = this.opts.app;
    this.docId = this.opts.docId;
    this.url = this.opts.url || defaultUrl(this.app, this.docId);
    this.ws = null;
    this.connected = false;
    this.reconnectTimer = null;
    this.heartbeatTimer = null;
    this.typingTimer = null;
    this.reconnectAttempts = 0;
    this.terminal = false;
    this._manualClose = false;
    this.user = readUser();
    // Server-authoritative sequence cursor (#791): fetch every op after it
    // on reconnect before live messages are trusted, so nothing is lost.
    this.lastSeq = this.opts.lastSeq || 0;
    this.knownUsers = new Map();
    this.callbacks = {
      onPresence: this.opts.onPresence || function () {},
      onTyping: this.opts.onTyping || function () {},
      onSelection: this.opts.onSelection || function () {},
      onMessage: this.opts.onMessage || function () {},
      onEdit: this.opts.onEdit || function () {},
      onConnect: this.opts.onConnect || function () {},
      onDisconnect: this.opts.onDisconnect || function () {}
    };
    this.collaboratorsEl = this.opts.collaboratorsEl || document.getElementById("collaborators");
    this.typingEl = this.opts.typingEl || document.getElementById("typing-indicator");
  };

  GBCollab.prototype.connect = function (opts) {
    this._init(opts || this.opts);
    if (!this.app || !this.docId) {
      console.warn("[GBCollab] app and docId required");
      return null;
    }
    // #2029 — the old guard refused every connection without an explicit url.
    // botserver does serve /ws/docs, /ws/sheet and /ws/slides, which is what
    // defaultUrl builds; apps without a realtime route use REST presence only.
    var HAS_REALTIME = { docs: true, sheet: true, slides: true, canvas: true };
    if (!this.opts.url && HAS_REALTIME[this.app] !== true) {
      console.info("[GBCollab] no realtime route for '" + this.app + "'; using REST presence");
      this.callbacks.onDisconnect();
      return null;
    }
    this.reconnectAttempts = 0;
    this.terminal = false;
    this._manualClose = false;
    this._clearUnavailable();
    this._open();
    return this;
  };

  GBCollab.prototype._open = function () {
    this._manualClose = false;
    const token = readToken();
    const sep = this.url.indexOf("?") === -1 ? "?" : "&";
    const wsUrl = token ? this.url + sep + "token=" + encodeURIComponent(token) : this.url;
    let ws;
    try { ws = new WebSocket(wsUrl); }
    catch (e) { console.warn("[GBCollab] WebSocket ctor failed", e); this._scheduleReconnect(); return; }
    this.ws = ws;

    ws.addEventListener("open", function () {
      this.connected = true;
      this.reconnectAttempts = 0;
      this.terminal = false;
      this._clearUnavailable();
      this._sendRaw({ msg_type: "hello", user_id: this.user.id, user_name: this.user.name, user_color: this.user.color, doc_id: this.docId });
      this._startHeartbeat();
      this._catchUp();
      this.callbacks.onConnect(this.user);
    }.bind(this));

    ws.addEventListener("message", function (ev) {
      let msg = null;
      try { msg = JSON.parse(ev.data); } catch (_) { return; }
      if (!msg || !msg.msg_type) return;
      this._dispatch(msg);
    }.bind(this));

    ws.addEventListener("close", function (ev) {
      this.connected = false;
      this._stopHeartbeat();
      this.callbacks.onDisconnect();
      this.knownUsers.clear();
      renderAvatars(this.collaboratorsEl, []);
      if (!this._manualClose) this._scheduleReconnect(ev);
    }.bind(this));

    ws.addEventListener("error", function () {
      try { ws.close(); } catch (_) { window.gbLogError ? gbLogError(_, "_open") : console.error("_open", _); }
    });
  };
  GBCollab.prototype._isTerminalClose = function (ev) {
    if (!ev) return false;
    const code = ev.code;
    const reason = ev.reason || "";
    // 1008 = policy violation; a denial may report 1006 + reason; a bare 1006 is a network drop (bounded retries).
    return code === 1008 || (code === 1006 && TERMINAL_REASON.test(reason));
  };

  GBCollab.prototype._showUnavailable = function () {
    if (!document.body) return;
    let node = document.getElementById(STATUS_ID);
    if (!node) {
      // Exactly ONE status region for the whole tab, whatever the app.
      node = el("div", { id: STATUS_ID, class: "gb-collab-status", role: "status", "aria-live": "polite", style: { position: "fixed", right: "12px", bottom: "12px", zIndex: "9999", display: "flex", alignItems: "center", gap: "10px", padding: "10px 14px", borderRadius: "8px", background: "#7f1d1d", color: "#f8fafc", font: "13px sans-serif", boxShadow: "0 4px 16px rgba(0,0,0,.4)" } });
      document.body.appendChild(node);
    }
    node.innerHTML = "";
    node.appendChild(el("span", { class: "gb-collab-status-text" },
      T("collab-unavailable", "Collaboration unavailable")));
    node.appendChild(el("button", { type: "button", class: "gb-collab-retry",
      onclick: function () { this.retry(); }.bind(this) }, T("collab-retry", "Retry")));
  };

  GBCollab.prototype._clearUnavailable = function () {
    const node = document.getElementById(STATUS_ID);
    if (node && node.parentNode) node.parentNode.removeChild(node);
  };

  GBCollab.prototype.retry = function () {
    if (this.reconnectTimer) { clearTimeout(this.reconnectTimer); this.reconnectTimer = null; }
    this.reconnectAttempts = 0;
    this.terminal = false;
    this._clearUnavailable();
    this._open();
  };

  GBCollab.prototype._scheduleReconnect = function (ev) {
    if (this.reconnectTimer || this.terminal) return;
    if (this._isTerminalClose(ev) || this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      this.terminal = true;
      this._showUnavailable();
      return;
    }
    const wait = Math.min(RECONNECT_DELAY_MS * Math.pow(2, this.reconnectAttempts), BACKOFF_MAX_MS);
    this.reconnectAttempts += 1;
    this.reconnectTimer = setTimeout(function () {
      this.reconnectTimer = null;
      this._open();
    }.bind(this), wait);
  };

  GBCollab.prototype._startHeartbeat = function () {
    this._stopHeartbeat();
    this.heartbeatTimer = setInterval(function () {
      this._sendRaw({ msg_type: "ping" });
    }.bind(this), HEARTBEAT_MS);
  };

  GBCollab.prototype._stopHeartbeat = function () {
    if (this.heartbeatTimer) { clearInterval(this.heartbeatTimer); this.heartbeatTimer = null; }
  };

  GBCollab.prototype._catchUp = function () {
    // Session-oplog recovery (#789, #791): apply every op after lastSeq before
    // the live feed is trusted. Hooks: opLogUrl(docId, since), opSeqOf(op).
    if (!this.opts.opLogUrl || !this.callbacks.onEdit || !this.lastSeq) return;
    const url = this.opts.opLogUrl(this.docId, this.lastSeq);
    fetch(url)
      .then(function (r) { return r.ok ? r.json() : null; })
      .catch(function () { return null; })
      .then(function (data) {
        if (!data || !data.ops || !data.ops.length) return;
        const seqOf = this.opts.opSeqOf || function (op) { return op.seq; };
        data.ops.forEach(function (op) {
          const seq = seqOf(op);
          if (seq && seq <= this.lastSeq) return;
          if (seq) this.lastSeq = seq;
          this.callbacks.onEdit(op);
        }.bind(this));
      }.bind(this));
  };

  GBCollab.prototype._dispatch = function (msg) {
    const t = msg.msg_type;
    if (t === "join" || t === "leave" || t === "presence" || t === "pong") {
      if (msg.user_id && msg.user_id !== this.user.id) {
        if (t === "leave") this.knownUsers.delete(msg.user_id);
        else this.knownUsers.set(msg.user_id, msg);
      }
      this._renderPresence();
      this.callbacks.onPresence(Array.from(this.knownUsers.values()));
    } else if (t === "typing_start" || t === "typing_stop") this.callbacks.onTyping(msg);
    else if (t === "selection") this.callbacks.onSelection(msg);
    else if (t === "edit" || t === "cell_update" || t === "slide_update" || t === "plan_update") {
      if (typeof msg.seq === "number" && msg.seq > this.lastSeq) this.lastSeq = msg.seq;
      this.callbacks.onEdit(msg);
    } else if (t === "mention") this._toastMention(msg);
    else this.callbacks.onMessage(msg);
  };

  GBCollab.prototype._renderPresence = function () {
    const users = Array.from(this.knownUsers.values());
    if (this.collaboratorsEl) renderAvatars(this.collaboratorsEl, users);
  };

  GBCollab.prototype._toastMention = function (msg) {
    const text = (msg.from_user_name || msg.user_name || "Someone") + " mentioned you: " + (msg.content || msg.data || "");
    if (window.GBSuite && window.GBSuite.toast) window.GBSuite.toast(text, "info");
    else console.info("[GBCollab mention]", text);
  };

  GBCollab.prototype._sendRaw = function (obj) {
    if (!this.ws || this.ws.readyState !== 1) return false;
    try { this.ws.send(JSON.stringify(obj)); return true; }
    catch (e) { return false; }
  };

  GBCollab.prototype.send = function (type, payload) {
    const msg = Object.assign({ msg_type: type, user_id: this.user.id, user_name: this.user.name, user_color: this.user.color, doc_id: this.docId, timestamp: Date.now() }, payload || {});
    return this._sendRaw(msg);
  };
  function positionPayload(position, col) {
    const payload = {};
    if (col === undefined) payload.position = position;
    else { payload.row = position; payload.col = col; }
    return payload;
  }
  GBCollab.prototype.sendCursor = function (position, col) {
    if (position === undefined || position === null) return this.send("cursor", {});
    return this.send("cursor", positionPayload(position, col));
  };
  GBCollab.prototype.sendTypingStart = function (position, col) {
    this._typingActive = true;
    if (position === undefined || position === null) return this.send("typing_start", {});
    return this.send("typing_start", positionPayload(position, col));
  };
  GBCollab.prototype.sendTypingStop = function () {
    this._typingActive = false;
    return this.send("typing_stop");
  };
  GBCollab.prototype.debouncedTypingStart = function (position, col) {
    this.sendTypingStart(position, col);
    if (this.typingTimer) clearTimeout(this.typingTimer);
    this.typingTimer = setTimeout(function () { this.sendTypingStop(); }.bind(this), TYPING_AWAY_MS);
  };
  GBCollab.prototype.sendSelection = function (start, end, extra) {
    const data = Object.assign({ start: start, end: end }, extra || {});
    return this.send("selection", { position: start, content: JSON.stringify(data) });
  };
  GBCollab.prototype.sendMention = function (toUserId, message, position) {
    return this.send("mention", { position: position, content: JSON.stringify({ to_user_id: toUserId, message: message }) });
  };
  GBCollab.prototype.sendEdit = function (opts) {
    opts = opts || {};
    // #2447: send both `content` (legacy) and `value` (server field).
    if (opts.row !== undefined && opts.col !== undefined) {
      const payload = { row: opts.row, col: opts.col, content: opts.content, value: opts.content, length: opts.length, format: opts.format, removeLength: opts.removeLength };
      return this.send("edit", payload);
    }
    return this.send("edit", { position: opts.position, content: opts.content, value: opts.content, length: opts.length, format: opts.format, removeLength: opts.removeLength });
  };

  GBCollab.prototype.disconnect = function () {
    if (this.reconnectTimer) { clearTimeout(this.reconnectTimer); this.reconnectTimer = null; }
    this._manualClose = true; this.terminal = false;
    this._stopHeartbeat();
    if (this.ws) { try { this.ws.close(); } catch (_) { window.gbLogError ? gbLogError(_, "disconnect") : console.error("disconnect", _); } this.ws = null; }
  };

  GBCollab.prototype.getUser = function () { return this.user; };
  GBCollab.prototype.getKnownUsers = function () { return Array.from(this.knownUsers.values()); };
  GBCollab.prototype.isConnected = function () { return this.connected; };
  GBCollab.helpers = { renderAvatars: renderAvatars, renderTypingIndicator: renderTypingIndicator, avatar: avatar, pickColor: pickColor, escapeHtml: escapeHtml, el: el };
  window.GBCollab = new GBCollab();
})(window);
