"use strict";
/**
 * Chat — thumbs up/down feedback on assistant answers.
 *
 * Stored per conversation in localStorage, keyed by the bubble's
 * `data-message-id`, so a reload restores the vote. The control is appended to
 * the bubble itself (not to `.message-content`, which streaming rewrites).
 */

(function () {
  var STORE_KEY = "gb-chat-feedback";
  var MAX_COMMENT = 500;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function loadStore() {
    try {
      var raw = localStorage.getItem(STORE_KEY);
      return raw ? JSON.parse(raw) : {};
    } catch (error) {
      return {};
    }
  }

  function saveStore(store) {
    try {
      localStorage.setItem(STORE_KEY, JSON.stringify(store));
    } catch (error) {
      /* quota or private mode: the vote still shows for this session */
    }
  }

  function sessionId() {
    return (window.ChatState && ChatState.currentSessionId) || "default";
  }

  function bubbleId(bubble) {
    return (bubble && bubble.dataset && bubble.dataset.messageId) || "";
  }

  function storeKey(bubble) {
    return sessionId() + ":" + bubbleId(bubble);
  }

  function getFeedback(bubble) {
    var id = bubbleId(bubble);
    if (!id) return null;
    return loadStore()[storeKey(bubble)] || null;
  }

  function setFeedback(bubble, vote, comment) {
    if (!bubbleId(bubble)) return;
    var store = loadStore();
    var key = storeKey(bubble);
    if (!vote) {
      delete store[key];
    } else {
      store[key] = { vote: vote, ts: Date.now() };
      if (comment) store[key].comment = String(comment).slice(0, MAX_COMMENT);
    }
    saveStore(store);
  }

  function thumbSvg(up) {
    var path = up
      ? '<path d="M14 9V5a3 3 0 0 0-3-3l-4 9v11h11.28a2 2 0 0 0 2-1.7l1.38-9a2 2 0 0 0-2-2.3H14zM7 22H4a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2h3"/>'
      : '<path d="M10 15v4a3 3 0 0 0 3 3l4-9V2H5.72a2 2 0 0 0-2 1.7l-1.38 9a2 2 0 0 0 2 2.3H10zM17 2h2.67A2.31 2.31 0 0 1 22 4v7a2.31 2.31 0 0 1-2.33 2H17"/>';
    return '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" ' +
      'stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
      path + "</svg>";
  }

  function removeCommentRow(row) {
    var box = row.querySelector(".gb-feedback-comment");
    if (box) box.remove();
  }

  function showCommentRow(row, bubble) {
    if (row.querySelector(".gb-feedback-comment")) return;
    var box = document.createElement("div");
    box.className = "gb-feedback-comment";

    var input = document.createElement("input");
    input.type = "text";
    input.className = "gb-feedback-input";
    input.maxLength = MAX_COMMENT;
    input.placeholder = t("chat-feedback-comment", "What could be better?");
    input.setAttribute("aria-label", input.placeholder);

    var send = document.createElement("button");
    send.type = "button";
    send.className = "gb-feedback-send";
    send.textContent = t("chat-feedback-send", "Send");

    function submit() {
      setFeedback(bubble, "down", input.value.trim() || undefined);
      box.remove();
    }

    send.addEventListener("click", submit);
    input.addEventListener("keydown", function (event) {
      if (event.key === "Enter") submit();
      if (event.key === "Escape") box.remove();
    });

    box.appendChild(input);
    box.appendChild(send);
    row.appendChild(box);
    input.focus();
  }

  function paint(row, bubble) {
    var existing = getFeedback(bubble);
    var up = row.querySelector(".gb-feedback-up");
    var down = row.querySelector(".gb-feedback-down");
    if (up) {
      up.classList.toggle("active", !!existing && existing.vote === "up");
      up.setAttribute("aria-pressed", !!existing && existing.vote === "up" ? "true" : "false");
    }
    if (down) {
      down.classList.toggle("active", !!existing && existing.vote === "down");
      down.setAttribute("aria-pressed", !!existing && existing.vote === "down" ? "true" : "false");
    }
  }

  function buildRow(bubble) {
    var row = document.createElement("div");
    row.className = "gb-feedback-wrap";
    row.setAttribute("role", "group");
    row.setAttribute("aria-label", t("chat-feedback-label", "Answer feedback"));

    var up = document.createElement("button");
    up.type = "button";
    up.className = "gb-feedback-btn gb-feedback-up";
    up.innerHTML = thumbSvg(true);
    up.title = t("chat-feedback-good", "Good answer");
    up.setAttribute("aria-label", up.title);

    var down = document.createElement("button");
    down.type = "button";
    down.className = "gb-feedback-btn gb-feedback-down";
    down.innerHTML = thumbSvg(false);
    down.title = t("chat-feedback-bad", "Bad answer");
    down.setAttribute("aria-label", down.title);

    up.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      var current = getFeedback(bubble);
      setFeedback(bubble, current && current.vote === "up" ? null : "up");
      removeCommentRow(row);
      paint(row, bubble);
    });

    down.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      var current = getFeedback(bubble);
      if (current && current.vote === "down") {
        setFeedback(bubble, null);
        removeCommentRow(row);
        paint(row, bubble);
        return;
      }
      setFeedback(bubble, "down");
      paint(row, bubble);
      showCommentRow(row, bubble);
    });

    row.appendChild(up);
    row.appendChild(down);
    paint(row, bubble);
    return row;
  }

  /** Attach feedback to a finished assistant bubble (idempotent). */
  function attach(bubble) {
    if (!bubble || !bubble.dataset || bubble.dataset.gbFeedbackBound === "1") return;
    if (!bubble.classList || !bubble.classList.contains("bot")) return;
    if (!bubbleId(bubble)) return;
    // Streaming previews are re-rendered constantly; the final bubble is the
    // one that gets the vote controls.
    if (String(bubble.id || "").indexOf("streaming-") === 0) return;
    if (!bubble.querySelector(".message-content")) return;
    bubble.dataset.gbFeedbackBound = "1";
    bubble.appendChild(buildRow(bubble));
  }

  function scan(root) {
    if (!root || !root.querySelectorAll) return;
    var bubbles = root.querySelectorAll("#messages .message.bot");
    for (var index = 0; index < bubbles.length; index++) attach(bubbles[index]);
  }

  function boot() {
    scan(document);
    if (typeof MutationObserver !== "function") return;
    var observer = new MutationObserver(function (records) {
      for (var index = 0; index < records.length; index++) {
        var added = records[index].addedNodes;
        for (var node = 0; node < added.length; node++) {
          var element = added[node];
          if (!element || element.nodeType !== 1) continue;
          if (element.classList && element.classList.contains("message")) attach(element);
          scan(element);
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
  }

  window.GBChatFeedback = {
    get: getFeedback,
    set: setFeedback,
    attach: attach
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
