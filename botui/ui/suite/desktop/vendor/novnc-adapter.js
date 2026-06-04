/* noVNC adapter — bridges noVNC RFB to our WebSocket TCP proxy.
 *
 * Our proxy expects:
 *   1. First WS message: JSON {"host":"...","port":...}
 *   2. After status reply: binary RFB protocol data
 *
 * noVNC RFB expects:
 *   - A WebSocket URL that speaks RFB directly
 *
 * This adapter connects to our proxy, sends the JSON handshake,
 * waits for "connected" status, then initializes noVNC on the same socket.
 */
(function () {
    "use strict";

    function VNCAdapter(container, wsUrl, host, port) {
        this.container = container;
        this.wsUrl = wsUrl;
        this.host = host;
        this.port = port;
        this.rfb = null;
        this.ws = null;
        this._listeners = {};
    }

    VNCAdapter.prototype.connect = function () {
        var self = this;
        try {
            self.ws = new WebSocket(self.wsUrl);
            self.ws.binaryType = "arraybuffer";
        } catch (e) {
            self._emit("error", { detail: "WebSocket creation failed: " + e.message });
            return;
        }

        self.ws.onopen = function () {
            self.ws.send(JSON.stringify({ host: self.host, port: self.port }));
        };

        self.ws.onmessage = function (event) {
            if (typeof event.data === "string") {
                try {
                    var msg = JSON.parse(event.data);
                    if (msg.status === "connected") {
                        self._initNoVNC();
                    } else if (msg.error) {
                        self._emit("error", { detail: msg.error });
                    }
                } catch (e) { /* ignore parse errors */ }
                return;
            }
        };

        self.ws.onclose = function () {
            self._emit("disconnect", {});
        };

        self.ws.onerror = function () {
            self._emit("error", { detail: "WebSocket error" });
        };
    };

    VNCAdapter.prototype._initNoVNC = function () {
        var self = this;
        if (typeof noVNC === "undefined" || !noVNC.RFB) {
            self._emit("error", {
                detail:
                    "noVNC library not loaded. Place noVNC core/ files in " +
                    "/suite/desktop/vendor/novnc/core/",
            });
            return;
        }

        try {
            var rfb = new noVNC.RFB(self.container, self.ws, {
                wsProtocols: [],
            });

            rfb.addEventListener("connect", function () {
                self.rfb = rfb;
                self._emit("connect", {});
            });

            rfb.addEventListener("disconnect", function (e) {
                self._emit("disconnect", e);
            });

            rfb.addEventListener("error", function (e) {
                self._emit("error", e);
            });
        } catch (e) {
            self._emit("error", { detail: "noVNC init failed: " + e.message });
        }
    };

    VNCAdapter.prototype.disconnect = function () {
        if (this.rfb) {
            try { this.rfb.disconnect(); } catch (e) { /* ignore */ }
            this.rfb = null;
        }
        if (this.ws) {
            try { this.ws.close(); } catch (e) { /* ignore */ }
            this.ws = null;
        }
    };

    VNCAdapter.prototype.sendCtrlAltDel = function () {
        if (this.rfb) this.rfb.sendCtrlAltDel();
    };

    VNCAdapter.prototype.clipboardPaste = function (text) {
        if (this.rfb) this.rfb.clipboardPaste(text);
    };

    VNCAdapter.prototype._emit = function (name, detail) {
        var callbacks = this._listeners[name] || [];
        for (var i = 0; i < callbacks.length; i++) {
            try { callbacks[i](detail); } catch (e) { /* ignore */ }
        }
    };

    VNCAdapter.prototype.addEventListener = function (name, fn) {
        if (!this._listeners[name]) this._listeners[name] = [];
        this._listeners[name].push(fn);
    };

    window.VNCAdapter = VNCAdapter;
})();
