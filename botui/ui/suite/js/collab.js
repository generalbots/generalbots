"use strict";
/* GBCollab — shared real-time collaboration client (WebSocket + presence)
 * Used by sheet, docs, slides, plan, and any future app that needs it.
 * Public API (window.GBCollab):
 *   connect({ app, docId, host?, onPresence, onTyping, onSelection, onMessage, onEdit, onConnect, onDisconnect })
 *   send(type, payload)                       — send a typed collab message
 *   sendCursor(position)                      — convenience: cursor event
 *   sendTypingStart(position)                 — convenience: typing_start
 *   sendTypingStop()                          — convenience: typing_stop
 *   sendSelection(start, end)                 — convenience: selection event
 *   sendMention(toUserId, message, position)  — convenience: mention
 *   sendEdit({ content, position, length, format })
 *   disconnect()
 *   getUser()                                 — { id, name, color } for self
 *
 * Auth: pulls JWT from localStorage.gb-access-token; sends as ?token=... on
 * WebSocket upgrade. Server (Rust) validates with Zitadel provider; if absent,
 * server falls back to anonymous UUID so dev mode still works.
 */

(function (window) {
  const RECONNECT_DELAY_MS = 2000;
  const HEARTBEAT_MS = 25000;
  const TYPING_AWAY_MS = 4000;
  const STORAGE_KEYS = { TOKEN: "gb-access-token", USER: "gb-user-data" };

  function readUser() {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.USER);
      if (raw) {
        const u = JSON.parse(raw);
        if (u && (u.id || u.sub || u.user_id)) {
          return {
            id: u.id || u.sub || u.user_id,
            name: u.display_name || u.name || u.email || "User",
            color: u.color || pickColor(u.id || u.sub || u.user_id)
          };
        }
      }
    } catch (_) {}
    const id = "anon-" + Math.random().toString(36).slice(2, 10);
    return { id: id, name: "Guest", color: pickColor(id) };
  }

  function readToken() {
    try { return localStorage.getItem(STORAGE_KEYS.TOKEN) || ""; } catch (_) { return ""; }
  }

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
    if (attrs) {
      for (const k in attrs) {
        if (k === "style" && typeof attrs[k] === "object") Object.assign(node.style, attrs[k]);
        else if (k === "class") node.className = attrs[k];
        else if (k === "html") node.innerHTML = attrs[k];
        else if (k.startsWith("on") && typeof attrs[k] === "function") node.addEventListener(k.slice(2), attrs[k]);
        else if (k === "data" && typeof attrs[k] === "object") {
          for (const dk in attrs[k]) node.dataset[dk] = attrs[k][dk];
        } else node.setAttribute(k, attrs[k]);
      }
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
      class: "gb-collab-avatar",
      title: user.user_name || user.name || user.user_id,
      data: { userId: user.user_id, color: user.user_color || user.color },
      style: {
        display: "inline-flex", alignItems: "center", justifyContent: "center",
        width: px + "px", height: px + "px", borderRadius: "50%",
        background: user.user_color || user.color || "#3b82f6",
        color: "#fff", fontWeight: "600", fontSize: Math.max(10, Math.floor(px * 0.45)) + "px",
        marginLeft: "-6px", border: "2px solid #0f172a",
        boxShadow: "0 1px 3px rgba(0,0,0,0.4)", position: "relative"
      },
      html: escapeHtml(initial)
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
    if (overflow > 0) {
      container.appendChild(el("span", {
        class: "gb-collab-avatar gb-collab-overflow",
        style: {
          display: "inline-flex", alignItems: "center", justifyContent: "center",
          width: "28px", height: "28px", borderRadius: "50%",
          background: "#334155", color: "#f8fafc", fontSize: "11px", fontWeight: "600",
          marginLeft: "-6px", border: "2px solid #0f172a"
        },
        html: "+" + overflow
      }));
    }
    container.dataset.userCount = String(users.length);
  }

  function renderTypingIndicator(container, users) {
    if (!container) return;
    container.innerHTML = "";
    if (!users || !users.length) return;
    const names = users.slice(0, 3).map(function (u) { return escapeHtml(u.user_name || u.name); });
    const text = names.length === 1
      ? names[0] + " está digitando…"
      : names.length === 2
        ? names[0] + " e " + names[1] + " estão digitando…"
        : names[0] + " e " + (users.length - 1) + " outros estão digitando…";
    container.appendChild(el("div", { class: "gb-typing-text", style: { color: "#94a3b8", fontSize: "12px", padding: "4px 8px", fontStyle: "italic" }, html: text }));
  }

  function defaultUrl(app, docId) {
    const loc = window.location;
    const proto = loc.protocol === "https:" ? "wss:" : "ws:";
    // Same-origin: the suite server (botui, dev :3000) and Caddy (prod
    // 80/443) both proxy /ws to botserver, so the page host + /ws is the
    // correct address everywhere. Hardcoding botserver's :8080 breaks prod,
    // where the port is not exposed publicly (#913).
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
    this.user = readUser();
    // Server-authoritative sequence cursor (#791): the last op seq applied to
    // local state. On reconnect the client fetches every op after it from the
    // session oplog before live messages arrive, so nothing is lost.
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
    this._open();
    return this;
  };

  GBCollab.prototype._open = function () {
    const token = readToken();
    const sep = this.url.indexOf("?") === -1 ? "?" : "&";
    const wsUrl = token ? this.url + sep + "token=" + encodeURIComponent(token) : this.url;
    let ws;
    try { ws = new WebSocket(wsUrl); }
    catch (e) { console.warn("[GBCollab] WebSocket ctor failed", e); this._scheduleReconnect(); return; }
    this.ws = ws;

    ws.addEventListener("open", function () {
      this.connected = true;
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

    ws.addEventListener("close", function () {
      this.connected = false;
      this._stopHeartbeat();
      this.callbacks.onDisconnect();
      this.knownUsers.clear();
      renderAvatars(this.collaboratorsEl, []);
      this._scheduleReconnect();
    }.bind(this));

    ws.addEventListener("error", function () {
      try { ws.close(); } catch (_) {}
    });
  };

  GBCollab.prototype._scheduleReconnect = function () {
    if (this.reconnectTimer) return;
    this.reconnectTimer = setTimeout(function () {
      this.reconnectTimer = null;
      this._open();
    }.bind(this), RECONNECT_DELAY_MS);
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
    // Session-oplog recovery (#789, #791): asks the document's ops endpoint
    // for every op after our cursor and applies them through onEdit before
    // the live feed is trusted. The provider opts into it by passing
    // `opLogUrl(docId, since)` and `opSeqOf(op) -> number|undefined`.
    if (!this.opts.opLogUrl || !this.callbacks.onEdit) return;
    if (!this.lastSeq) return;
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
    } else if (t === "typing_start" || t === "typing_stop") {
      this.callbacks.onTyping(msg);
    } else if (t === "selection") {
      this.callbacks.onSelection(msg);
    } else if (t === "edit" || t === "cell_update" || t === "slide_update" || t === "plan_update") {
      if (typeof msg.seq === "number" && msg.seq > this.lastSeq) this.lastSeq = msg.seq;
      this.callbacks.onEdit(msg);
    } else if (t === "mention") {
      this._toastMention(msg);
    } else {
      this.callbacks.onMessage(msg);
    }
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

  GBCollab.prototype.sendCursor = function (position, col) {
    const payload = {};
    if (position === undefined || position === null) return this.send("cursor", payload);
    if (col === undefined) payload.position = position;
    else { payload.row = position; payload.col = col; }
    return this.send("cursor", payload);
  };
  GBCollab.prototype.sendTypingStart = function (position, col) {
    this._typingActive = true;
    const payload = {};
    if (position === undefined || position === null) return this.send("typing_start", payload);
    if (col === undefined) payload.position = position;
    else { payload.row = position; payload.col = col; }
    return this.send("typing_start", payload);
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
    // A1 addressing (#791): send row/col when the caller supplies them; the
    // position encoding is kept for backwards compatibility with text apps.
    if (opts.row !== undefined && opts.col !== undefined) {
      const payload = { row: opts.row, col: opts.col, content: opts.content, length: opts.length, format: opts.format, removeLength: opts.removeLength };
      return this.send("edit", payload);
    }
    return this.send("edit", { position: opts.position, content: opts.content, length: opts.length, format: opts.format, removeLength: opts.removeLength });
  };

  GBCollab.prototype.disconnect = function () {
    if (this.reconnectTimer) { clearTimeout(this.reconnectTimer); this.reconnectTimer = null; }
    this._stopHeartbeat();
    if (this.ws) { try { this.ws.close(); } catch (_) {} this.ws = null; }
  };

  GBCollab.prototype.getUser = function () { return this.user; };
  GBCollab.prototype.getKnownUsers = function () { return Array.from(this.knownUsers.values()); };
  GBCollab.prototype.isConnected = function () { return this.connected; };

  GBCollab.helpers = { renderAvatars: renderAvatars, renderTypingIndicator: renderTypingIndicator, avatar: avatar, pickColor: pickColor, escapeHtml: escapeHtml, el: el };

  window.GBCollab = new GBCollab();
})(window);
