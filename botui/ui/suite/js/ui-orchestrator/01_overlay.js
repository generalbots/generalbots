/**
 * GBUiOrchestrator — Agentic UI driver.
 *
 * Executes validated UI plans (message_type 9) with visible animation:
 * ghost cursor, focus rings, typing effects, click pulses, and a step
 * checklist in the chat window. Generic across all suite apps: fields are
 * resolved by data-gb-field attribute, label text, placeholder or name —
 * never by hardcoded IDs.
 */
(function (window, document) {
  "use strict";
  if (window.GBUiOrchestrator) return;
  var N = (document.__gb_mod_ui_orchestrator = {});
  var S = (N.s = { ACTIVE: null, focusRing: null, currentApp: null });
  var ghostCursor = null;

  var STEP_TIMEOUT_MS = 8000;
  var TYPE_DELAY_MS = 45;

  /* ------------------------------------------------------------
     Animation primitives
     ------------------------------------------------------------ */

  function ensureOverlay() {
    if (ghostCursor) return;
    var host = document.createElement("div");
    host.id = "gb-orchestrator-overlay";
    host.style.cssText =
      "position:fixed;top:0;left:0;width:0;height:0;z-index:2147483000;" +
      "pointer-events:none;";
    ghostCursor = document.createElement("div");
    ghostCursor.id = "gb-ghost-cursor";
    ghostCursor.style.cssText =
      "position:fixed;width:24px;height:24px;left:-100px;top:-100px;" +
      "border:2px solid #22c55e;border-radius:50%;background:rgba(34,197,94,.25);" +
      "box-shadow:0 0 12px rgba(34,197,94,.8);transition:left .35s ease,top .35s ease;";
    S.focusRing = document.createElement("div");
    S.focusRing.id = "gb-focus-ring";
    S.focusRing.style.cssText =
      "position:fixed;border:2px solid #22c55e;border-radius:6px;display:none;" +
      "box-shadow:0 0 0 3px rgba(34,197,94,.3),0 0 18px rgba(34,197,94,.6);" +
      "transition:left .3s ease,top .3s ease,width .3s ease,height .3s ease;";
    host.appendChild(ghostCursor);
    host.appendChild(S.focusRing);
    document.body.appendChild(host);
  }

  function moveCursorTo(rect, onDone) {
    ensureOverlay();
    var x = rect.left + rect.width / 2;
    var y = rect.top + rect.height / 2;
    ghostCursor.style.left = x - 12 + "px";
    ghostCursor.style.top = y - 12 + "px";
    S.focusRing.style.display = "block";
    S.focusRing.style.left = rect.left - 4 + "px";
    S.focusRing.style.top = rect.top - 4 + "px";
    S.focusRing.style.width = rect.width + 8 + "px";
    S.focusRing.style.height = rect.height + 8 + "px";
    setTimeout(onDone, 400);
  }

  function pulseClick() {
    if (!S.focusRing) return;
    var ring = S.focusRing;
    var rect = ring.getBoundingClientRect();
    var pulse = document.createElement("div");
    pulse.style.cssText =
      "position:fixed;left:" + (rect.left + rect.width / 2 - 20) + "px;" +
      "top:" + (rect.top + rect.height / 2 - 20) + "px;width:40px;height:40px;" +
      "border-radius:50%;border:3px solid rgba(34,197,94,.9);" +
      "z-index:2147483001;pointer-events:none;animation:gb-pulse .5s ease-out;";
    if (!document.getElementById("gb-pulse-style")) {
      var st = document.createElement("style");
      st.id = "gb-pulse-style";
      st.textContent =
        "@keyframes gb-pulse{from{transform:scale(.4);opacity:1}" +
        "to{transform:scale(2);opacity:0}}";
      document.head.appendChild(st);
    }
    document.body.appendChild(pulse);
    setTimeout(function () { pulse.remove(); }, 600);
  }

  function highlightFlash(el) {
    if (!el) return;
    var original = el.style.boxShadow;
    el.style.boxShadow = "0 0 0 3px rgba(34,197,94,.6), 0 0 20px rgba(34,197,94,.5)";
    el.style.transition = "box-shadow .2s ease";
    setTimeout(function () { el.style.boxShadow = original; }, 1200);
  }

  /* ------------------------------------------------------------
     DOM resolution (generic across all apps)
     ------------------------------------------------------------ */

  function normalize(text) {
    return (text || "").toLowerCase().replace(/\s+/g, " ").trim();
  }

  function labelMatches(el, target) {
    if (!target) return false;
    var t = normalize(target);
    var label = normalize(el.getAttribute("data-gb-field"));
    if (label && (label === t || label.indexOf(t) !== -1)) return true;
    var aria = normalize(el.getAttribute("aria-label"));
    if (aria && (aria === t || aria.indexOf(t) !== -1)) return true;
    var name = normalize(el.getAttribute("name"));
    if (name && (name === t || name.indexOf(t) !== -1)) return true;
    var placeholder = normalize(el.getAttribute("placeholder"));
    if (placeholder && (placeholder === t || placeholder.indexOf(t) !== -1)) return true;
    var id = normalize(el.id);
    if (id && (id === t || id.indexOf(t) !== -1)) return true;
    return false;
  }

  function labelOf(element) {
    var labels = element.labels;
    if (labels && labels.length) {
      return labels[0].textContent;
    }
    if (element.closest && element.closest("label")) {
      return element.closest("label").textContent;
    }
    var wrapper = element.parentElement;
    if (wrapper && wrapper.querySelector && wrapper.querySelector("label")) {
      return wrapper.querySelector("label").textContent;
    }
    return element.placeholder || element.name || element.id || "";
  }

  function fieldScope() {
    if (S.currentApp) {
      var win = document.getElementById("window-body-" + S.currentApp) ||
        document.getElementById("window-" + S.currentApp);
      if (win) return win;
    }
    return document;
  }

  function findField(fieldLabel) {
    var t = normalize(fieldLabel);
    var roots = [fieldScope(), document];
    for (var r = 0; r < roots.length; r++) {
      var all = Array.prototype.slice.call(
        roots[r].querySelectorAll(
          "input, select, textarea, [contenteditable=true], [contenteditable=\"\"]"
        )
      );
      var candidates = [];
      for (var i = 0; i < all.length; i++) {
        if (all[i].type === "hidden" || all[i].disabled) continue;
        if (all[i].offsetParent === null && all[i].getClientRects().length === 0) continue;
        candidates.push(all[i]);
      }
      for (var j = 0; j < candidates.length; j++) {
        var label = labelOf(candidates[j]);
        if (label && normalize(label) === t) return candidates[j];
      }
      for (var k = 0; k < candidates.length; k++) {
        if (labelMatches(candidates[k], t)) return candidates[k];
      }
      for (var m = 0; m < candidates.length; m++) {
        var label2 = labelOf(candidates[m]);
        if (label2 && normalize(label2).indexOf(t) !== -1) return candidates[m];
      }
      if (r === 0 && S.currentApp) continue;
    }
    return null;
  }

  var CLICK_ALIASES = {
    "new email": ["compose", "nova"],
    "new message": ["compose", "nova"],
    "write email": ["compose", "nova"],
    "compose email": ["compose", "nova"],
    "create email": ["compose", "nova"],
    "new": ["compose", "nova"],
    "new spreadsheet": ["nova", "compose"],
    "new sheet": ["nova", "compose"],
    "new file": ["nova", "compose"],
  };

  function clickableText(el) {
    // Whitespace-only textContent (newlines from an SVG icon) is truthy, so the
    // || chain stopped there and every icon-only button — title="Merge Cells",
    // no aria-label, only an <svg> child — became invisible to findClickable
    // (#2155: "Click 'Merge cells' — element not found").
    var text = normalize(el.textContent || "");
    if (text) return text;
    var value = normalize(el.value || "");
    if (value) return value;
    var aria = normalize(el.getAttribute("aria-label") || "");
    if (aria) return aria;
    return normalize(el.title || "");
  }

  function clickTargets(label) {
    var t = normalize(label);
    var alias = CLICK_ALIASES[t];
    var targets = [t];
    if (Array.isArray(alias)) {
      for (var i = 0; i < alias.length; i++) targets.push(alias[i]);
    } else if (alias) {
      targets.push(alias);
    }
    return targets.filter(function (x) { return x; });
  }

  function findClickable(label) {
    var targets = clickTargets(label);
    var roots = [fieldScope(), document];
    for (var r = 0; r < roots.length; r++) {
      var candidates = roots[r].querySelectorAll(
        "button, a[href], [role=button], input[type=submit], input[type=button], .btn, .btn-primary"
      );
      for (var i = 0; i < candidates.length; i++) {
        var el = candidates[i];
        if (el.offsetParent === null && el.getClientRects().length === 0) continue;
        var text = clickableText(el);
        if (!text) continue;
        for (var j = 0; j < targets.length; j++) {
          if (text === targets[j]) return el;
        }
      }
      for (var k = 0; k < candidates.length; k++) {
        var el2 = candidates[k];
        if (el2.offsetParent === null && el2.getClientRects().length === 0) continue;
        var text2 = clickableText(el2);
        if (!text2) continue;
        for (var m = 0; m < targets.length; m++) {
          if (text2.indexOf(targets[m]) !== -1) return el2;
        }
      }
      if (r === 0 && S.currentApp) continue;
    }
    return null;
  }

  function visibleText(el) {
    var t = normalize(el.textContent || "");
    return t.length > 0 && t.length < 120 ? t : null;
  }

  /* ------------------------------------------------------------
     Step execution
     ------------------------------------------------------------ */

  function reportStep(text, status) {
    var evt = new CustomEvent("gb:ui-step", {
      detail: { text: text, status: status || "info" },
    });
    window.dispatchEvent(evt);
    renderStepCard(text, status);
    if (window.AgentMode && window.AgentMode.isActive && window.AgentMode.isActive()) {
      try {
        window.AgentMode.handleMessage({
          type: "step_progress",
          current: S.ACTIVE ? S.ACTIVE.completed : 1,
          total: S.ACTIVE ? S.ACTIVE.total : 1,
        });
      } catch (e) {}
    }
  }

  var stepCard = null;
  var stepCardLines = [];

  function renderStepCard(text, status) {
    var messages = document.getElementById("messages");
    if (!messages) return;
    if (!stepCard) {
      stepCard = document.createElement("div");
      stepCard.className = "message bot ui-step-card";
      stepCard.style.cssText =
        "font-size:12px;color:var(--muted,#888);margin:2px 0;" +
        "padding:6px 10px;background:var(--surface,#1b1b1f);" +
        "border:1px solid var(--border,#2a2a2e);border-radius:8px;";
      messages.appendChild(stepCard);
    }
    stepCardLines.push(
      (status === "done" ? "\u2705 " : status === "warn" ? "\u26A0\uFE0F " : "\u2699\uFE0F ") +
        escapeForHtml(text)
    );
    stepCard.innerHTML = stepCardLines.join("<br>");
    if (!ChatState || !ChatState.isUserScrolling) {
      var scroller = document.querySelector("#messages, .chat-content-wrapper");
      if (scroller) scroller.scrollTop = scroller.scrollHeight;
    }
  }

  function resetStepCard() {
    stepCard = null;
    stepCardLines = [];
  }

  function escapeForHtml(text) {
    var div = document.createElement("div");
    div.textContent = text || "";
    return div.innerHTML;
  }

  function waitFor(predicate, timeoutMs) {
    return new Promise(function (resolve) {
      var elapsed = 0;
      var interval = setInterval(function () {
        elapsed += 100;
        var ok = false;
        try { ok = predicate(); } catch (e) {}
        if (ok || elapsed >= timeoutMs) {
          clearInterval(interval);
          resolve(ok);
        }
      }, 100);
    });
  }


  N.STEP_TIMEOUT_MS = STEP_TIMEOUT_MS;
  N.TYPE_DELAY_MS = TYPE_DELAY_MS;
  N.ensureOverlay = ensureOverlay;
  N.moveCursorTo = moveCursorTo;
  N.pulseClick = pulseClick;
  N.highlightFlash = highlightFlash;
  N.normalize = normalize;
  N.labelMatches = labelMatches;
  N.labelOf = labelOf;
  N.fieldScope = fieldScope;
  N.findField = findField;
  N.CLICK_ALIASES = CLICK_ALIASES;
  N.clickableText = clickableText;
  N.clickTargets = clickTargets;
  N.findClickable = findClickable;
  N.visibleText = visibleText;
  N.reportStep = reportStep;
  N.renderStepCard = renderStepCard;
  N.resetStepCard = resetStepCard;
  N.escapeForHtml = escapeForHtml;
  N.waitFor = waitFor;
})(window, document);
