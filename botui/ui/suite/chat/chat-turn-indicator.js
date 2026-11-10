/* Chat turn progress element (#2098) — DOM and elapsed timer only.
 *
 * The state machine (chat-turn-state.js) decides WHEN the element is visible;
 * this module owns WHAT it looks like and where it lives. It is scoped to the
 * pane that owns the turn: the stock pane is #messages, a parallel conversation
 * tab renders into its own .chat-pane-messages. Querying the document globally
 * leaked tab A's spinner into tab B.
 *
 * Dependency-free on purpose: loaded by the chat partial and by the
 * zero-dependency DOM test (botui/tests/chat_turn_state.test.js).
 */
var GBTurnIndicator = (function () {
  "use strict";

  function label(key, fallback) {
    try {
      if (typeof window.gbT === "function") return window.gbT(key, fallback);
      if (window.i18n && typeof window.i18n.t === "function") {
        var value = window.i18n.t(key);
        return value === key ? fallback : value;
      }
    } catch (e) { /* i18n unavailable */ }
    return fallback;
  }

  function resolveContainer(opts) {
    opts = opts || {};
    var pane = opts.pane;
    if (pane && pane.querySelector) {
      var inner = pane.querySelector(".chat-pane-messages");
      if (inner) return inner;
      if (pane.id === "messages") return pane;
    }
    var tabId = opts.tabId || "tab-default";
    var multi = window.GBMultiChat;
    var tab = multi && multi.tabs ? multi.tabs[tabId] : null;
    if (tab && tab.pane && tab.pane.querySelector) {
      var holder = tab.pane.querySelector(".chat-pane-messages");
      if (holder) return holder;
    }
    return document.getElementById("messages");
  }

  function button(className, title, text, handler) {
    var el = document.createElement("button");
    el.type = "button";
    el.className = className;
    el.title = title;
    el.textContent = text;
    if (handler) el.addEventListener("click", handler);
    return el;
  }

  function build() {
    var wrap = document.createElement("div");
    wrap.id = "thinking-indicator";
    wrap.className = "message bot";
    var content = document.createElement("div");
    content.className = "message-content bot-message";
    var box = document.createElement("div");
    box.className = "thinking-indicator";
    box.setAttribute("role", "status");
    box.setAttribute("aria-live", "polite");
    var spinner = document.createElement("span");
    spinner.className = "thinking-spinner";
    spinner.setAttribute("aria-hidden", "true");
    var text = document.createElement("span");
    text.className = "thinking-label";
    var elapsed = document.createElement("span");
    elapsed.className = "thinking-elapsed";
    elapsed.setAttribute("aria-hidden", "true");
    elapsed.textContent = "0s";
    var stop = button("thinking-stop", label("chat-stop", "Stop"),
      label("chat-stop", "Stop"), function () {
        var owner = wrap.__gbTurn;
        if (owner && typeof owner.onStop === "function") owner.onStop();
      });
    var retry = button("thinking-retry", label("chat-retry", "Retry"),
      label("chat-retry", "Retry"), function () {
        var owner = wrap.__gbTurn;
        if (owner && typeof owner.onRetry === "function") owner.onRetry();
      });
    box.appendChild(spinner);
    box.appendChild(text);
    box.appendChild(elapsed);
    box.appendChild(stop);
    box.appendChild(retry);
    content.appendChild(box);
    wrap.appendChild(content);
    return wrap;
  }

  function indicator(turn) {
    var box = turn.container || resolveContainer(turn);
    if (!box) return null;
    turn.container = box;
    var el = box.querySelector("#thinking-indicator");
    if (!el) {
      el = build();
      box.appendChild(el);
    } else if (box.lastElementChild !== el) {
      // Keep the indicator at the live edge, after any streamed content.
      box.appendChild(el);
    }
    turn.el = el;
    el.__gbTurn = turn;
    el.setAttribute("data-turn-id", turn.id);
    el.setAttribute("data-turn-state", turn.state);
    if (box.dataset) box.dataset.turnState = turn.state;
    return el;
  }

  function remove(turn) {
    if (turn && turn.el && turn.el.parentNode) turn.el.parentNode.removeChild(turn.el);
    if (turn) turn.el = null;
  }

  function updateElapsed(turn) {
    if (!turn || !turn.el) return;
    var el = turn.el.querySelector(".thinking-elapsed");
    var seconds = Math.max(0, Math.floor((Date.now() - turn.startedAt) / 1000));
    if (el) el.textContent = seconds + "s";
    turn.el.setAttribute("data-turn-elapsed", String(seconds));
  }

  function paint(turn, text, extraClass) {
    var el = indicator(turn);
    if (!el) return;
    var box = el.querySelector(".thinking-indicator");
    if (box) {
      box.classList.remove("thinking-slow", "thinking-error", "thinking-done");
      if (extraClass) box.classList.add(extraClass);
    }
    var textEl = el.querySelector(".thinking-label");
    if (textEl) textEl.textContent = text || "";
    var ended = turn.state === "done" || turn.state === "error" || turn.state === "cancelled";
    var elapsed = el.querySelector(".thinking-elapsed");
    if (elapsed) elapsed.style.display = ended ? "none" : "";
    var stop = el.querySelector(".thinking-stop");
    if (stop) stop.style.display = ended ? "none" : "";
    var retry = el.querySelector(".thinking-retry");
    if (retry) retry.style.display = turn.state === "error" ? "" : "none";
    updateElapsed(turn);
    el.setAttribute("data-turn-state", turn.state);
    if (turn.container && turn.container.dataset) turn.container.dataset.turnState = turn.state;
  }

  function stopTimer(turn) {
    if (turn && turn.timer) {
      clearInterval(turn.timer);
      turn.timer = null;
    }
  }

  /* The timer outlives the DOM node: a streaming turn hides the spinner but
   * must still notice a provider that stalls mid-answer. `onStall` runs once
   * the turn has seen no frame for `stallMs`. */
  function startTimer(turn, stallMs, onStall) {
    stopTimer(turn);
    turn.timer = setInterval(function () {
      if (!turn.alive) { stopTimer(turn); return; }
      updateElapsed(turn);
      if (turn.state === "done" || turn.state === "error" || turn.state === "cancelled") {
        stopTimer(turn);
        return;
      }
      if (Date.now() - turn.lastFrameAt >= stallMs) onStall(turn);
    }, 1000);
  }

  function scroll() {
    try {
      if (typeof window.scrollToBottom === "function") window.scrollToBottom(true);
      if (typeof window.scheduleScrollbarMarkers === "function") window.scheduleScrollbarMarkers();
    } catch (e) { /* transcript not mounted */ }
  }

  return {
    label: label,
    resolveContainer: resolveContainer,
    paint: paint,
    remove: remove,
    updateElapsed: updateElapsed,
    startTimer: startTimer,
    stopTimer: stopTimer,
    scroll: scroll,
  };
})();

window.GBTurnIndicator = GBTurnIndicator;
