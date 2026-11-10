/* Chat turn state machine (#2098) — one visible progress element per turn.
 *
 * The indicator used to be shown by the send path and hidden by the first
 * BOT_RESPONSE frame, so a turn with a tool call or a slow provider looked
 * dead until the answer landed. Visibility is now derived from the turn's
 * state instead of ad-hoc show/hide calls:
 *
 *   queued -> thinking -> (tool) -> streaming -> done
 *                  \-> error / cancelled
 *
 * Every turn is keyed by conversation tab and carries its own id, start time
 * and last-frame time, so a late frame from the previous turn (or from
 * another conversation) can never clear the live one. The DOM lives in
 * chat-turn-indicator.js; this file owns the transitions only.
 */
var GBTurn = (function () {
  "use strict";

  var DEFAULT_TAB = "tab-default";
  var SLOW_AFTER_MS = 12000;
  // Prefill on a CPU-only model can keep the socket silent for minutes before
  // the first token; `window.GB_STALL_MS` lets a deployment widen the window
  // without changing the default for hosted providers.
  var STALL_AFTER_MS = Number(window.GB_STALL_MS) > 0 ? Number(window.GB_STALL_MS) : 90000;
  var DONE_LINGER_MS = 1600;
  var DEDUPE_MS = 60000;

  var turns = {};
  var turnSeq = 0;
  var lastEnded = null;
  // Turns that ended but still show a "Done"/"Stopped" element for a moment.
  var lingering = [];

  function ind() {
    return window.GBTurnIndicator || null;
  }

  function label(key, fallback) {
    var dom = ind();
    return dom ? dom.label(key, fallback) : fallback;
  }

  function tabOf(opts) {
    opts = opts || {};
    return opts.tabId || DEFAULT_TAB;
  }

  function chatState() {
    return window.ChatState || null;
  }

  function active(opts) {
    return turns[tabOf(opts)] || null;
  }

  function state(opts) {
    var turn = active(opts);
    return turn ? turn.state : "idle";
  }

  /* ── state transitions ─────────────────────────────────────────────────── */

  function start(text, opts) {
    opts = opts || {};
    var tabId = tabOf(opts);
    var previous = turns[tabId];
    if (previous) {
      previous.alive = false;
      var dom = ind();
      if (dom) dom.stopTimer(previous);
    }
    var cs = chatState();
    var turn = {
      id: "turn-" + (++turnSeq) + "-" + Date.now(),
      tabId: tabId,
      sessionId: opts.sessionId || (cs && cs.currentSessionId) || "",
      text: text || "",
      state: "queued",
      startedAt: Date.now(),
      lastFrameAt: Date.now(),
      frames: 0,
      alive: true,
      container: null,
      el: null,
      timer: null,
      onStop: function () { cancel(tabId); },
      onRetry: function () { retry(turn); },
    };
    turn.container = ind() ? ind().resolveContainer({ tabId: tabId, pane: opts.pane }) : null;
    // A new turn owns its pane: forget the previous turn's "Done" element.
    lingering = lingering.filter(function (entry) { return entry.container !== turn.container; });
    turns[tabId] = turn;
    paint(turn, label("status-sending", "Sending…"));
    if (ind()) {
      ind().startTimer(turn, STALL_AFTER_MS, function (stalled) {
        fail("The model stopped answering.", { tabId: stalled.tabId });
      });
      ind().scroll();
    }
    return turn.id;
  }

  function paint(turn, text, extraClass) {
    if (!ind()) return;
    ind().paint(turn, text, extraClass);
  }

  function remove(turn) {
    if (ind()) ind().remove(turn);
  }

  function note(turn) {
    turn.lastFrameAt = Date.now();
    turn.frames += 1;
  }

  function touch(opts) {
    var turn = active(opts);
    if (!turn) return false;
    note(turn);
    return true;
  }

  function thinking(opts) {
    var turn = active(opts);
    if (!turn) {
      // A frame without a local send (reconnect, another device, start.bas):
      // the turn is running, so show progress instead of a dead chat. A frame
      // arriving right after a completed turn must not resurrect the spinner.
      var ended = lastEnded && lastEnded.tabId === tabOf(opts) &&
        Date.now() - lastEnded.at < DONE_LINGER_MS;
      if (ended) return false;
      start("", opts);
      turn = active(opts);
    }
    if (!turn) return false;
    note(turn);
    if (turn.state !== "queued" && turn.state !== "thinking" && turn.state !== "tool") {
      // streaming/done/error: never downgrade a visible answer.
      return turn.state === "streaming";
    }
    turn.state = "thinking";
    paint(turn, label("status-thinking", "Thinking…"));
    return true;
  }

  function progress(stepText, opts) {
    var turn = active(opts);
    if (!turn) return false;
    if (turn.state === "done" || turn.state === "error" || turn.state === "cancelled") return false;
    note(turn);
    turn.state = "tool";
    paint(turn, stepText || label("status-working", "Working…"));
    if (ind()) ind().scroll();
    return true;
  }

  function visible(opts) {
    // First visible content of the turn: the bubble itself is the progress
    // element now, so the spinner steps aside — but the turn stays open until
    // the completion frame arrives (a tool can still run after the first
    // chunk). A later tool frame re-paints the indicator below the bubble.
    var turn = active(opts);
    if (!turn) return false;
    if (turn.state === "done" || turn.state === "error" || turn.state === "cancelled") return false;
    note(turn);
    turn.state = "streaming";
    remove(turn);
    if (turn.container && turn.container.dataset) turn.container.dataset.turnState = "streaming";
    return true;
  }

  function slow(opts) {
    var turn = active(opts);
    if (!turn) return false;
    if (turn.state === "streaming" || turn.state === "done" ||
        turn.state === "error" || turn.state === "cancelled") return false;
    paint(turn, label("status-slow", "Provider is slow… still waiting"), "thinking-slow");
    return true;
  }

  function contentOf(frame) {
    return frame && typeof frame.content === "string" ? frame.content : "";
  }

  function reasoningOf(frame) {
    return frame && typeof frame.reasoning === "string" ? frame.reasoning : "";
  }

  /* A frame may not end (or steer) the live turn when it belongs to another
   * conversation, arrives after the turn already ended, or repeats the answer
   * of the previous turn (server retries). With no live turn there is nothing
   * to protect: the frame renders normally (reconnect / pending reply). */
  function stale(frame, opts) {
    var turn = active(opts);
    if (!turn) return false;
    if (turn.state === "done" || turn.state === "cancelled") return true;
    var sessionId = frame && frame.session_id;
    if (sessionId && turn.sessionId && sessionId !== turn.sessionId) return true;
    // A duplicate of the previous answer only counts while this turn has not
    // heard anything yet: a real answer is preceded by a thinking/keepalive
    // frame, and a legitimate repeat of the same text must still land.
    var untouched = turn.frames === 0;
    if (untouched && lastEnded && lastEnded.tabId === turn.tabId &&
        Date.now() - lastEnded.at < DEDUPE_MS &&
        contentOf(frame) === lastEnded.content &&
        reasoningOf(frame) === lastEnded.reasoning &&
        (lastEnded.content !== "" || lastEnded.reasoning !== "")) {
      return true;
    }
    return false;
  }

  function complete(frame, opts) {
    var turn = active(opts);
    if (!turn) return false;
    if (turn.state === "done" || turn.state === "cancelled") return false;
    turn.alive = false;
    turn.state = "done";
    if (ind()) ind().stopTimer(turn);
    var answer = opts && typeof opts.content === "string" ? opts.content : contentOf(frame);
    var reason = opts && typeof opts.reasoning === "string" ? opts.reasoning : reasoningOf(frame);
    lastEnded = { id: turn.id, tabId: turn.tabId, content: answer, reasoning: reason, at: Date.now() };
    delete turns[turn.tabId];
    paint(turn, label("status-done", "Done"), "thinking-done");
    var container = turn.container;
    lingering.push(turn);
    window.setTimeout(function () {
      remove(turn);
      lingering = lingering.filter(function (entry) { return entry !== turn; });
      if (container && container.dataset && container.dataset.turnState === "done") {
        container.dataset.turnState = "idle";
      }
    }, DONE_LINGER_MS);
    return true;
  }

  function fail(message, opts) {
    var turn = active(opts);
    if (!turn) return false;
    if (turn.state === "done" || turn.state === "cancelled") return false;
    turn.alive = false;
    turn.state = "error";
    if (ind()) ind().stopTimer(turn);
    paint(turn, message || label("status-error", "The model did not answer."), "thinking-error");
    return true;
  }

  function retry(turn) {
    if (!turn) return;
    var text = turn.text;
    forceHide({ tabId: turn.tabId });
    if (text && typeof window.sendMessage === "function") window.sendMessage(text);
  }

  function socketFor(tabId) {
    if (tabId === DEFAULT_TAB) {
      var cs = chatState();
      return cs && cs.ws;
    }
    var multi = window.GBMultiChat;
    var tab = multi && multi.tabs ? multi.tabs[tabId] : null;
    return tab && tab.ws;
  }

  function cancel(tabId) {
    var turn = active({ tabId: tabId });
    if (!turn) return false;
    var socket = socketFor(turn.tabId);
    try {
      if (socket && socket.readyState === 1) {
        var cs = chatState();
        socket.send(JSON.stringify({
          type: "run_cancel",
          session_id: turn.sessionId || (cs && cs.currentSessionId) || "",
        }));
      }
    } catch (e) { /* socket already gone */ }
    turn.alive = false;
    if (ind()) ind().stopTimer(turn);
    turn.state = "cancelled";
    lastEnded = { id: turn.id, tabId: turn.tabId, content: "", reasoning: "", at: Date.now() };
    delete turns[turn.tabId];
    paint(turn, label("status-stopped", "Stopped"));
    lingering.push(turn);
    window.setTimeout(function () {
      remove(turn);
      lingering = lingering.filter(function (entry) { return entry !== turn; });
    }, 900);
    if (typeof window.clearChatPending === "function") window.clearChatPending();
    return true;
  }

  function forceHide(opts) {
    var only = opts && opts.tabId;
    Object.keys(turns).forEach(function (tabId) {
      if (only && tabId !== only) return;
      var turn = turns[tabId];
      turn.alive = false;
      if (ind()) ind().stopTimer(turn);
      remove(turn);
      if (turn.container && turn.container.dataset) turn.container.dataset.turnState = "idle";
      delete turns[tabId];
    });
    // Ended turns keep a "Done"/"Stopped" element for a moment; a forced end
    // (bot switch, consent card, cancel ack) removes it right away.
    lingering = lingering.filter(function (turn) {
      if (only && turn.tabId !== only) return true;
      turn.alive = false;
      remove(turn);
      if (turn.container && turn.container.dataset) turn.container.dataset.turnState = "idle";
      return false;
    });
    if (typeof window.clearChatPending === "function") window.clearChatPending();
  }

  // A tool finished: back to "thinking" only if the turn was waiting on it.
  function settle(turn) {
    if (turn.state !== "tool") return true;
    turn.state = "thinking";
    paint(turn, label("status-thinking", "Thinking…"));
    return true;
  }

  /* Progress frames (AG-UI run events and VM tool progress) keep the turn
   * alive and label it with the running step. */
  function observe(frame, opts) {
    if (!frame || typeof frame !== "object") return false;
    if (frame.thinking) return thinking(opts);
    var turn = active(opts);
    if (!turn) return false;
    if (frame.message_type === 2) touch(opts);
    var type = frame.type || frame.event_type || "";
    var data = frame.data || {};
    if (type === "TOOL_CALL_START") {
      return progress("Running " + (data.name || data.call_id || "tool") + "…", opts);
    }
    if (type === "STEP_STARTED") {
      var stepName = data.detail || data.name || data.step || data.step_id || "";
      return progress(stepName ? "Working: " + stepName : "", opts);
    }
    if (type === "TOOL_CALL_RESULT" || type === "STEP_FINISHED") {
      touch(opts);
      return settle(turn);
    }
    if (type === "CUSTOM" && data.name === "gb.tool_progress") {
      var payload = data.payload || {};
      if (payload.phase === "start") {
        return progress("Running " + (payload.tool || "tool") + "…", opts);
      }
      touch(opts);
      return settle(turn);
    }
    if (type === "CUSTOM" && data.name === "gb.question") {
      return progress(label("status-waiting-input", "Waiting for your answer"), opts);
    }
    return false;
  }

  return {
    DEFAULT_TAB: DEFAULT_TAB,
    SLOW_AFTER_MS: SLOW_AFTER_MS,
    STALL_AFTER_MS: STALL_AFTER_MS,
    start: start,
    thinking: thinking,
    progress: progress,
    visible: visible,
    slow: slow,
    touch: touch,
    stale: stale,
    complete: complete,
    fail: fail,
    cancel: cancel,
    forceHide: forceHide,
    observe: observe,
    state: state,
    active: active,
  };
})();

window.GBTurn = GBTurn;
