/* RDP adapter — mirrors the VNCAdapter interface over the desktop proxy
 * WebSocket tunnel (`/api/desktop/ws/proxy/{sessionId}`).
 *
 * The proxy is a transparent byte tunnel: RDP negotiation and the encrypted
 * session flow between the browser client and the target RDP server through
 * it. Authentication (including NLA) happens end-to-end between client and
 * server; the server never sees credentials.
 *
 * Protocol decode layer:
 *   RDP is not a raw frame stream like RFB — the client must implement the
 *   RDP protocol (X.224/MCS/security) to render the session. The decode
 *   layer is pluggable: assign `window.RDPDecoder` (a Guacamole-style
 *   tunnel client or the msts-client decoder) and the adapter hands it the
 *   connected socket. Without a decoder the adapter keeps the tunnel alive
 *   and reports the real connection state (status badge, byte counters),
 *   so the session registry, audit and kill switch stay accurate.
 *
 * Adapter contract (same as VNCAdapter):
 *   connect(), disconnect(), sendCtrlAltDel(), clipboardPaste(text),
 *   addEventListener("connect"|"disconnect"|"error", fn), .ws (raw socket)
 */
(function () {
    "use strict";

    function RDPAdapter(container, wsUrl, host, port) {
        this.container = container;
        this.wsUrl = wsUrl;
        this.host = host;
        this.port = port;
        this.ws = null;
        this.decoder = null;
        this._listeners = {};
    }

    RDPAdapter.prototype.connect = function () {
        var self = this;
        try {
            self.ws = new WebSocket(self.wsUrl);
            self.ws.binaryType = "arraybuffer";
        } catch (e) {
            self._emit("error", { detail: "WebSocket creation failed: " + e.message });
            return;
        }

        self.ws.onopen = function () {
            // Proxy handshake: identify the target host/port.
            self.ws.send(JSON.stringify({ host: self.host, port: self.port }));
        };

        self.ws.onmessage = function (event) {
            if (typeof event.data === "string") {
                try {
                    var msg = JSON.parse(event.data);
                    // The proxy notifies with {type:"connected",payload:{...}};
                    // older adapters used {status:"connected"} — accept both.
                    if (msg.status === "connected" || msg.type === "connected") {
                        self._onTunnelConnected();
                    } else if (msg.error) {
                        self._emit("error", { detail: msg.error });
                    }
                } catch (e) { /* ignore non-JSON frames */ }
                return;
            }
            // Binary frames are RDP protocol data; forward to the decoder.
            if (self.decoder && typeof self.decoder.onData === "function") {
                self.decoder.onData(event.data);
            }
        };

        self.ws.onclose = function () {
            self._emit("disconnect", {});
        };

        self.ws.onerror = function () {
            self._emit("error", { detail: "WebSocket error" });
        };
    };

    RDPAdapter.prototype._onTunnelConnected = function () {
        var self = this;
        var decoder = window.RDPDecoder;
        if (decoder) {
            try {
                self.decoder = decoder;
                if (typeof decoder.attach === "function") {
                    decoder.attach(self.ws);
                }
            } catch (e) {
                self._emit("error", { detail: "RDP decoder attach failed: " + e.message });
                return;
            }
        }
        self._emit("connect", { decoder: !!decoder });
    };

    RDPAdapter.prototype.disconnect = function () {
        if (this.decoder && typeof this.decoder.detach === "function") {
            try { this.decoder.detach(); } catch (e) { /* ignore */ }
            this.decoder = null;
        }
        if (this.ws) {
            try { this.ws.close(); } catch (e) { /* ignore */ }
            this.ws = null;
        }
    };

    RDPAdapter.prototype.sendCtrlAltDel = function () {
        // RDP secure access sequences (Ctrl+Alt+Del) are negotiated by the
        // client; forward to the decoder when present.
        if (this.decoder && typeof this.decoder.sendCtrlAltDel === "function") {
            this.decoder.sendCtrlAltDel();
        }
    };

    RDPAdapter.prototype.clipboardPaste = function (text) {
        // Clipboard sharing over RDP uses the RDPDR channel, which lives in
        // the decoder; without it the paste is a no-op.
        if (this.decoder && typeof this.decoder.clipboardPaste === "function") {
            this.decoder.clipboardPaste(text);
        }
    };

    RDPAdapter.prototype._emit = function (name, detail) {
        var callbacks = this._listeners[name] || [];
        for (var i = 0; i < callbacks.length; i++) {
            try { callbacks[i](detail); } catch (e) { /* ignore */ }
        }
    };

    RDPAdapter.prototype.addEventListener = function (name, fn) {
        if (!this._listeners[name]) this._listeners[name] = [];
        this._listeners[name].push(fn);
    };

    window.RDPAdapter = RDPAdapter;
})();
