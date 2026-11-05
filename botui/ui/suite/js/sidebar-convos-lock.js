/* #1682 — locked conversations: sidebar actions split from sidebar-convos.js
   to keep both modules inside the repository size limit. Loaded right after
   the main module; shares its namespace and helpers. */
"use strict";

(function (mod) {
  /// Reports a failure without a blocking native alert.
  function report(title, message) {
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show(title, message, "error");
    } else {
      console.warn(title + ": " + message);
    }
  }

  /// In-app PIN prompt. Native window.prompt shows the PIN in plain text and
  /// is blocked inside sandboxed webviews, so the lock flow uses a masked
  /// modal with inline validation instead.
  mod.promptPin = function (title, submitLabel) {
    return new Promise(function (resolve) {
      // Only one PIN dialog at a time; a second trigger must not stack modals.
      [...document.querySelectorAll(".gb-pin-overlay")].forEach(function (el) {
        el.remove();
      });
      var overlay = document.createElement("div");
      overlay.className = "modal-overlay gb-pin-overlay";
      // Fully self-contained styling: the chat route does not load the
      // suite-wide components.css, so the dialog must not depend on it.
      overlay.style.cssText =
        "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;" +
        "padding:16px;background:rgba(0,0,0,.7);z-index:12000;" +
        "opacity:0;visibility:hidden;transition:opacity .18s ease,visibility .18s ease";
      overlay.innerHTML =
        '<div class="modal modal-sm" role="dialog" aria-modal="true" aria-labelledby="gbPinTitle" ' +
        'style="width:100%;max-width:360px;background:var(--surface,#161616);' +
        "border:1px solid var(--border,#2a2a2a);border-radius:14px;overflow:hidden;" +
        'box-shadow:0 24px 80px rgba(0,0,0,.5)">' +
        '<div class="modal-header" style="display:flex;align-items:center;justify-content:space-between;' +
        'padding:14px 18px;border-bottom:1px solid var(--border,#2a2a2a)">' +
        '<h3 id="gbPinTitle" style="margin:0;font-size:15px"></h3>' +
        '<button type="button" class="modal-close" aria-label="Close" ' +
        'style="background:none;border:0;color:inherit;font-size:16px;cursor:pointer;padding:4px 8px">\u2715</button></div>' +
        '<div class="modal-content" style="padding:16px 18px 20px">' +
        '<label for="gbPinInput" style="display:block;font-size:12px;opacity:.7;margin-bottom:8px">' +
        "4\u20138 digits</label>" +
        '<input id="gbPinInput" type="password" inputmode="numeric" pattern="[0-9]*" ' +
        'maxlength="8" autocomplete="off" ' +
        'style="width:100%;box-sizing:border-box;padding:10px 12px;border-radius:8px;' +
        "border:1px solid var(--border,#333);background:var(--surface2,#1e1e2e);color:inherit;" +
        'font-size:18px;letter-spacing:6px" />' +
        '<div class="gb-pin-error" role="alert" ' +
        'style="display:none;color:#f87171;font-size:12px;margin-top:8px"></div>' +
        '<div style="display:flex;gap:8px;justify-content:flex-end;margin-top:16px">' +
        '<button type="button" class="btn gb-pin-cancel" style="padding:8px 14px;border-radius:8px;' +
        'border:1px solid var(--border,#333);background:transparent;color:inherit;cursor:pointer">Cancel</button>' +
        '<button type="button" class="btn btn-primary gb-pin-ok" style="padding:8px 14px;border-radius:8px;' +
        'border:1px solid transparent;background:var(--accent,#2563eb);color:#fff;cursor:pointer"></button>' +
        "</div></div></div>";
      var titleEl = overlay.querySelector("#gbPinTitle");
      var submit = overlay.querySelector(".gb-pin-ok");
      var input = overlay.querySelector("#gbPinInput");
      var error = overlay.querySelector(".gb-pin-error");
      titleEl.textContent = title;
      submit.textContent = submitLabel || "OK";
      document.body.appendChild(overlay);

      function finish(value) {
        document.removeEventListener("keydown", onKey, true);
        overlay.remove();
        resolve(value);
      }
      function fail(message) {
        error.textContent = message;
        error.style.display = "block";
        input.focus();
        input.select();
      }
      function accept() {
        var value = String(input.value || "").trim();
        if (!/^[0-9]{4,8}$/.test(value)) {
          fail("PIN must be 4 to 8 digits.");
          return;
        }
        finish(value);
      }
      function onKey(event) {
        if (event.key === "Escape") { event.preventDefault(); finish(null); }
        if (event.key === "Enter") { event.preventDefault(); accept(); }
      }
      overlay.querySelector(".gb-pin-cancel").addEventListener("click", function () { finish(null); });
      overlay.querySelector(".modal-close").addEventListener("click", function () { finish(null); });
      submit.addEventListener("click", accept);
      input.addEventListener("input", function () { error.style.display = "none"; });
      overlay.addEventListener("mousedown", function (event) {
        if (event.target === overlay) finish(null);
      });
      document.addEventListener("keydown", onKey, true);
      requestAnimationFrame(function () {
        overlay.classList.add("show");
        overlay.style.opacity = "1";
        overlay.style.visibility = "visible";
        // Force the visibility transition to apply before focusing: a
        // visibility:hidden element cannot take focus.
        void overlay.offsetHeight;
        input.focus();
        // Some shell focus managers reclaim focus right after the frame;
        // restore it once if the dialog is still open.
        setTimeout(function () {
          if (overlay.isConnected && document.activeElement !== input) input.focus();
        }, 80);
      });
    });
  };

  /// Locks the conversation behind a 4-8 digit PIN (#1682). The item
  /// disappears from history; it is only reachable from "Locked".
  mod.lockConversation = function (sessionId) {
    if (!sessionId) return;
    mod.promptPin("Set a 4-8 digit PIN for this conversation", "Lock").then(function (pin) {
      if (pin === null) return;
      fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId) + "/lock", {
        method: "POST",
        headers: Object.assign({ "Content-Type": "application/json" }, mod.authHeaders()),
        body: JSON.stringify({ pin: pin }),
      })
        .then(function (r) {
          if (!r.ok) throw new Error("HTTP " + r.status);
          mod.loadHistory();
        })
        .catch(function (err) {
          report("Lock conversation", "Could not lock: " + err.message);
        });
    });
  };

  /// Lists locked conversations (ids and dates only — no titles) and opens one
  /// after the PIN is verified. The grant is short-lived; the conversation
  /// stays locked in storage.
  mod.showLocked = function () {
    var list = document.getElementById("chatConversations");
    if (!list) return;
    fetch(
      "/api/chat/history/sessions?locked=1&limit=50&bot_name=" +
        encodeURIComponent(mod.resolveBot()),
      { headers: mod.authHeaders() }
    )
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        var sessions = (data && data.sessions) || [];
        list.innerHTML = "";
        var back = document.createElement("button");
        back.type = "button";
        back.className = "chat-sidebar-conv-back";
        back.textContent = "\u2190 Back to conversations";
        back.addEventListener("click", function () {
          mod.loadHistory();
        });
        list.appendChild(back);
        if (!sessions.length) {
          var empty = document.createElement("div");
          empty.className = "chat-sidebar-history-empty";
          empty.textContent = "No locked conversations";
          list.appendChild(empty);
          return;
        }
        sessions.forEach(function (session) {
          var item = document.createElement("div");
          item.className = "chat-sidebar-conv-item locked";
          item.setAttribute("data-session-id", session.session_id);
          item.innerHTML =
            '<div class="chat-sidebar-conv-info">' +
            '<div class="chat-sidebar-conv-name">\u{1F512} Locked conversation</div>' +
            "</div>" +
            '<div class="chat-sidebar-conv-time"></div>';
          item.addEventListener("click", function () {
            mod.unlockConversation(session.session_id);
          });
          list.appendChild(item);
        });
      })
      .catch(function (err) {
        report("Locked conversations", "Could not list locked conversations: " + err.message);
      });
  };

  /// Verifies the PIN, stores the short-lived grant and opens the chat tab.
  mod.unlockConversation = function (sessionId) {
    if (!sessionId) return;
    mod.promptPin("Enter the conversation PIN", "Unlock").then(function (pin) {
      if (pin === null) return;
      fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId) + "/unlock", {
        method: "POST",
        headers: Object.assign({ "Content-Type": "application/json" }, mod.authHeaders()),
        body: JSON.stringify({ pin: String(pin).trim() }),
      })
        .then(function (r) {
          if (r.status === 403) throw new Error("Incorrect PIN");
          if (!r.ok) throw new Error("HTTP " + r.status);
          return r.json();
        })
        .then(function (data) {
          if (data && data.unlock_token) {
            sessionStorage.setItem("gb-unlock:" + sessionId, data.unlock_token);
          }
          mod.openConversation(sessionId);
        })
        .catch(function (err) {
          report("Unlock conversation", "Could not unlock: " + err.message);
        });
    });
  };
})(window.GBSidebarConvos = window.GBSidebarConvos || {});
