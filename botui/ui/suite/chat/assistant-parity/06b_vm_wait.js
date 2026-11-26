/* Assistant parity — while a VM permission card waits in the open chat turn.
 *
 * Ported from the assistant's 08d/08e/08f modules against Chat's own transport:
 * no "Thinking…" label while a card blocks the turn, a send is confirmed above
 * the composer (chat's busy-send guard stays in charge of streaming cleanup),
 * and open confirm cards move with the composer.
 */
"use strict";
(function () {
  var P = (window.GBChatAssistantParity = window.GBChatAssistantParity || {});
  var mounted = false;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function composer() {
    return document.getElementById("chatForm") || document.getElementById("messageInput");
  }
  function focusComposer() {
    var input = document.getElementById("messageInput");
    if (!input) return;
    try { input.focus({ preventScroll: true }); } catch (error) { input.focus(); }
  }
  function waitingCard() {
    return P.consentVm && typeof P.consentVm.waitingCard === "function"
      ? P.consentVm.waitingCard() : null;
  }
  function closeConfirm() {
    var box = document.querySelector(".gb-parity-vm-confirm-send");
    if (!box) return false;
    var hadFocus = box.contains(document.activeElement);
    if (box.parentNode) box.parentNode.removeChild(box);
    if (hadFocus) focusComposer();
    return true;
  }
  function toCard(card) {
    if (!card || !card.isConnected || card.classList.contains("resolved")) {
      focusComposer();
      return;
    }
    (card.lastElementChild || card).scrollIntoView({ block: "nearest" });
    var first = card.querySelector(".gb-parity-vm-btn:not(:disabled)");
    (first || card).focus();
  }
  function stopTurn() {
    var state = window.ChatState;
    if (state && state.isStreaming && typeof window.finalizeStreaming === "function") {
      window.finalizeStreaming();
    }
    if (window.GBTurn && typeof window.GBTurn.cancel === "function") window.GBTurn.cancel();
    if (state) {
      state.isStreaming = false;
      state.pendingResponseAt = null;
      state.streamingMessageId = null;
    }
    if (typeof window.clearChatPending === "function") window.clearChatPending();
  }
  function ask(card, send) {
    closeConfirm();
    var box = document.createElement("div");
    box.className = "gb-parity-vm-confirm-send";
    box.setAttribute("role", "alertdialog");
    box.setAttribute("aria-labelledby", "gbParityVmSendHead");
    box.setAttribute("aria-describedby", "gbParityVmSendText");
    box.innerHTML =
      '<div class="gb-parity-vm-confirm-head" id="gbParityVmSendHead"></div>' +
      '<p class="gb-parity-vm-confirm-text" id="gbParityVmSendText"></p>' +
      '<div class="gb-parity-vm-confirm-actions">' +
        '<button type="button" class="gb-parity-vm-btn" data-act="back"></button>' +
        '<button type="button" class="gb-parity-vm-btn gb-parity-vm-btn--primary" data-act="send"></button>' +
      "</div>";
    box.querySelector("#gbParityVmSendHead").textContent = t("chat-vm-send-head", "Confirmation");
    box.querySelector("#gbParityVmSendText").textContent = t("chat-vm-send-text",
      "Sending now cancels the open permission request. Send anyway?");
    var back = box.querySelector('[data-act="back"]');
    var go = box.querySelector('[data-act="send"]');
    back.textContent = t("chat-vm-send-back", "Back to the request");
    go.textContent = t("chat-vm-send-go", "Send");
    function done() {
      if (box.parentNode) box.parentNode.removeChild(box);
      document.removeEventListener("keydown", onKey, true);
    }
    function onKey(event) {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      done();
      focusComposer();
    }
    back.addEventListener("click", function () { done(); toCard(card); });
    go.addEventListener("click", function () { done(); focusComposer(); stopTurn(); send(); });
    box.addEventListener("keydown", onKey);
    var host = composer();
    if (host && host.parentNode) host.parentNode.insertBefore(box, host);
    else document.body.appendChild(box);
    document.addEventListener("keydown", onKey, true);
    back.focus();
    if (P.consentVm && P.consentVm.announce) {
      P.consentVm.announce(box.querySelector("#gbParityVmSendText").textContent);
    }
  }
  function markTurn() {
    var card = waitingCard();
    if (!card) return;
    var turn = window.GBTurn;
    if (turn && typeof turn.active === "function" && turn.active() &&
        typeof turn.progress === "function") {
      try { turn.progress(t("chat-vm-consent-waiting", "Waiting for your permission")); }
      catch (error) { /* the turn label is decoration only */ }
    }
    if (P.consentVm && P.consentVm.typing() === false && P.consentVm.announce &&
        card.dataset.announced !== "1") {
      card.dataset.announced = "1";
      P.consentVm.announce(t("chat-vm-consent-announce", "Permission request opened. Press Alt+P to answer."));
    }
  }
  function onDecision(event) {
    var detail = (event && event.detail) || {};
    if (detail.blocking) closeConfirm();
    window.setTimeout(markTurn, 60);
  }

  P.vmWait = {
    mount: function () {
      if (mounted) return P;
      mounted = true;
      document.addEventListener("gb-ws-frame", function () { window.setTimeout(markTurn, 100); });
      window.addEventListener("gb-vm-consent-decision", onDecision);

      var original = window.sendMessage;
      if (typeof original === "function" && !original.__gbVmSendGuard) {
        var guarded = function () {
          var card = waitingCard();
          if (!card) return original.apply(this, arguments);
          var self = this, args = arguments;
          ask(card, function () { return original.apply(self, args); });
        };
        guarded.__gbVmSendGuard = true;
        window.sendMessage = guarded;
      }
      markTurn();
      return P;
    },
    markTurn: markTurn,
    closeConfirm: closeConfirm
  };
})();
