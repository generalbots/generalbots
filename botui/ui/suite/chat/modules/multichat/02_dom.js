"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — pane DOM and message appending.
// Split out of the former 22_multichat module for #1731 (450-line limit). Mechanical split:
// same functions, globals and DOM/network behavior. Load order matters — the
// module namespace lives on window.GBMultiChat, created by 01_state.js.

(function () {
  var MC = window.GBMultiChat;

  function makePane() {
    var pane = document.createElement("div");
    pane.className = "chat-pane";
    pane.style.cssText =
      "display:flex;flex-direction:column;flex:1;min-height:0;overflow:hidden;";
    var msgs = document.createElement("main");
    msgs.className = "chat-pane-messages";
    msgs.style.cssText = window.getComputedStyle(MC.q("messages") || pane).cssText;
    // Minimal self-contained styles (computed cssText can be empty pre-layout):
    msgs.style.cssText =
      "flex:1;overflow-y:auto;overflow-x:hidden;padding:16px;display:flex;" +
      "flex-direction:column;gap:16px;scrollbar-width:thin;justify-content:flex-start;align-items:stretch;";
    var sugg = document.createElement("div");
    sugg.className = "suggestions-container";
    sugg.style.cssText = "display:flex;flex-wrap:wrap;gap:8px;";
    // #1753 — each pane owns its response-format switchers. The container is
    // parked with the pane; only the mounted tab carries the canonical ids.
    var switchers = document.createElement("div");
    switchers.className = "switchers-container";
    switchers.style.cssText = "display:none;";
    var switchersLabel = document.createElement("div");
    switchersLabel.className = "switchers-label";
    switchersLabel.textContent = "Format:";
    var switcherChips = document.createElement("div");
    switcherChips.className = "switchers-chips";
    switchers.appendChild(switchersLabel);
    switchers.appendChild(switcherChips);
    pane.appendChild(msgs);
    pane.appendChild(sugg);
    pane.appendChild(switchers);
    return pane;
  }

  function appendToContainer(container, sender, content, msgId) {
    if (!container) return null;
    var msgs = container.__msgs || container.querySelector(".chat-pane-messages");
    if (!msgs) return null;
    // Parallel tabs and history replay render here, bypassing addMessage and
    // therefore the consent-card hook: the raw GB-CONSENT-REQUEST payload used
    // to leak into those bubbles (#2194/#2203/#2204/#2216). Decode it and swap
    // in the interactive card, and always strip the payload from the text.
    var consentReq = null;
    if (sender !== "user" && typeof window.GBConsentExtract === "function") {
      consentReq = window.GBConsentExtract(content);
    }
    var displayContent = consentReq && typeof window.GBConsentStrip === "function"
      ? window.GBConsentStrip(content)
      : content;
    var div = document.createElement("div");
    div.className = "message " + (sender === "user" ? "user" : "bot");
    if (msgId) div.id = msgId;
    var inner = document.createElement("div");
    inner.className =
      "message-content " + (sender === "user" ? "user-message" : "bot-message");
    if (sender === "user") {
      var processed = typeof renderMentionInMessage === "function"
        ? renderMentionInMessage(MC.esc(content))
        : MC.esc(content);
      inner.innerHTML = '<div class="gb-message-text">' + processed + "</div>";
    } else if (typeof marked !== "undefined" && marked.parse) {
      try {
        inner.innerHTML = marked.parse(displayContent || "");
        // Replayed messages must render vmfile links as attachment cards too;
        // the stock addMessage path does this conversion, appendToContainer
        // (used by history replay and parallel tabs) did not.
        if (typeof renderVmFileLinks === "function") {
          inner.innerHTML = renderVmFileLinks(inner.innerHTML);
        }
        if (typeof renderDeepLinks === "function") {
          inner.innerHTML = renderDeepLinks(inner.innerHTML);
        }
      } catch (e) {
        inner.textContent = displayContent;
      }
    } else {
      inner.textContent = displayContent;
    }
    div.appendChild(inner);
    if (consentReq && typeof window.GBBuildConsentCard === "function") {
      inner.appendChild(window.GBBuildConsentCard(consentReq));
    }
    if (sender === "user" && typeof collapseUserMessageIfNeeded === "function") {
      collapseUserMessageIfNeeded(div, content);
    }
    if (!msgId || String(msgId).indexOf("streaming-") !== 0) {
      if (typeof attachMessageTimestamp === "function") {
        attachMessageTimestamp(div, Date.now());
      }
    }
    // Parallel tabs render through this helper, so they must add the same
    // per-message actions the stock addMessage path attaches.
    if (sender !== "user" && displayContent && displayContent.trim()) {
      if (typeof createCopyButton === "function") div.appendChild(createCopyButton(displayContent));
      if (typeof createSpeakButton === "function") div.appendChild(createSpeakButton(displayContent));
    }
    msgs.appendChild(div);
    if (typeof mergeWorkspaceOutput === "function") mergeWorkspaceOutput(div);
    // Merge runs first (#2008): polish moves the raw output into a details
    // card, after which mergeWorkspaceOutput can no longer find the `<pre>`
    // and a duplicate "workspace run" card survives.
    if (sender !== "user" && typeof polishOutputBlocks === "function") {
      polishOutputBlocks(inner);
    }
    msgs.scrollTop = msgs.scrollHeight;
    if (sender !== "user") {
      if (typeof hydrateVmAttachments === "function") {
        // Converts raw `[name](vmfile:...)` text (marked does not always
        // produce an anchor) and binds every card on this message.
        hydrateVmAttachments(div);
      } else {
        var cards = div.querySelectorAll(".gb-attachment");
        for (var ci = 0; ci < cards.length; ci++) {
          if (typeof vmBindAttachment === "function") vmBindAttachment(cards[ci]);
        }
      }
    }
    return div;
  }
  /* ── Published for the sibling multichat modules ── */

  MC.makePane = makePane;
  MC.appendToContainer = appendToContainer;
})();
