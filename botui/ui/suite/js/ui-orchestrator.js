/**
 * GBUiOrchestrator — Agentic UI driver.
 *
 * Executes validated UI plans (message_type 9) with visible animation:
 * ghost cursor, focus rings, typing effects, click pulses, and a step
 * checklist in the chat window. Generic across all suite apps: fields are
 * resolved by data-gb-field attribute, label text, placeholder or name —
 * never by hardcoded IDs.
 */
(function () {
  "use strict";

  if (window.GBUiOrchestrator) return;

  var ACTIVE = null;
  var ghostCursor = null;
  var focusRing = null;
  var currentApp = null;

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
    focusRing = document.createElement("div");
    focusRing.id = "gb-focus-ring";
    focusRing.style.cssText =
      "position:fixed;border:2px solid #22c55e;border-radius:6px;display:none;" +
      "box-shadow:0 0 0 3px rgba(34,197,94,.3),0 0 18px rgba(34,197,94,.6);" +
      "transition:left .3s ease,top .3s ease,width .3s ease,height .3s ease;";
    host.appendChild(ghostCursor);
    host.appendChild(focusRing);
    document.body.appendChild(host);
  }

  function moveCursorTo(rect, onDone) {
    ensureOverlay();
    var x = rect.left + rect.width / 2;
    var y = rect.top + rect.height / 2;
    ghostCursor.style.left = x - 12 + "px";
    ghostCursor.style.top = y - 12 + "px";
    focusRing.style.display = "block";
    focusRing.style.left = rect.left - 4 + "px";
    focusRing.style.top = rect.top - 4 + "px";
    focusRing.style.width = rect.width + 8 + "px";
    focusRing.style.height = rect.height + 8 + "px";
    setTimeout(onDone, 400);
  }

  function pulseClick() {
    if (!focusRing) return;
    var ring = focusRing;
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
    if (currentApp) {
      var win = document.getElementById("window-body-" + currentApp) ||
        document.getElementById("window-" + currentApp);
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
      if (r === 0 && currentApp) continue;
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
    return normalize(
      el.textContent || el.value || el.getAttribute("aria-label") || el.title || ""
    );
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
      if (r === 0 && currentApp) continue;
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
          current: ACTIVE ? ACTIVE.completed : 1,
          total: ACTIVE ? ACTIVE.total : 1,
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

  function waitMs(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms); });
  }

  function getElementRect(el) {
    var r = el.getBoundingClientRect();
    return { left: r.left, top: r.top, width: r.width, height: r.height };
  }

  function openApp(appId) {
    return new Promise(function (resolve) {
      var reg = (window.APPS_REGISTRY || []);
      var app = null;
      for (var i = 0; i < reg.length; i++) {
        if (reg[i].id === appId) { app = reg[i]; break; }
      }
      if (!app) {
        var all = document.querySelectorAll(".app-item[data-app]");
        for (var j = 0; j < all.length; j++) {
          if (all[j].getAttribute("data-app") === appId) {
            all[j].click();
            resolve(true);
            return;
          }
        }
        resolve(false);
        return;
      }
      if (!window.WindowManager) { resolve(false); return; }
      window.WindowManager.open(appId, app.title || appId, "");
      if (window.WindowManager._injectBodyContent && app.hxGet) {
        fetch(app.hxGet)
          .then(function (r) { return r.ok ? r.text() : ""; })
          .then(function (html) {
            var body = document.getElementById("window-body-" + appId);
            if (body && html) {
              window.WindowManager._injectBodyContent(appId, html);
            }
            resolve(true);
          })
          .catch(function () { resolve(true); });
      } else {
        resolve(true);
      }
    });
  }

  function stepOpen(step, plan) {
    var appId = step.app || plan.app || "chat";
    return openApp(appId).then(function (ok) {
      currentApp = appId;
      reportStep("Opened " + appId, ok ? "done" : "warn");
      if (!ok) {
        reportStep("App '" + appId + "' not found — continuing", "warn");
      }
      return waitFor(function () {
        var appWin = document.getElementById("window-" + appId);
        return appWin && appWin.querySelector("input, select, textarea, button");
      }, STEP_TIMEOUT_MS).then(function () {
        return waitMs(600);
      });
    });
  }

  function stepClick(step) {
    var el = findClickable(step.label || "");
    if (!el) {
      reportStep("Click '" + (step.label || "?") + "' — element not found", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(el);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        highlightFlash(el);
        pulseClick();
        try {
          el.click();
        } catch (e) {
          try {
            el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
          } catch (e2) {}
        }
        reportStep("Clicked '" + (visibleText(el) || step.label || "") + "'", "done");
        setTimeout(function () {
          focusRing.style.display = "none";
          resolve(true);
        }, 500);
      });
    });
  }

  function stepFill(step) {
    var el = findField(step.field || "");
    if (!el) {
      reportStep("Field '" + (step.field || "?") + "' not found", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(el);
    var value = step.value || "";
    var isEditable = el.isContentEditable === true;
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        try { el.focus(); } catch (e) {}
        if (isEditable) {
          el.textContent = value;
          el.dispatchEvent(new Event("input", { bubbles: true }));
          el.dispatchEvent(new Event("change", { bubbles: true }));
          syncHiddenField(el, value);
          reportStep("Filled '" + (step.field || "?") + "' = " + value, "done");
          setTimeout(function () {
            focusRing.style.display = "none";
            resolve(true);
          }, 300);
          return;
        }
        var i = 0;
        el.value = "";
        function typeNext() {
          if (i >= value.length) {
            el.dispatchEvent(new Event("input", { bubbles: true }));
            el.dispatchEvent(new Event("change", { bubbles: true }));
            reportStep("Filled '" + (step.field || "?") + "' = " + value, "done");
            setTimeout(function () {
              focusRing.style.display = "none";
              resolve(true);
            }, 300);
            return;
          }
          el.value += value[i];
          el.dispatchEvent(new Event("input", { bubbles: true }));
          i++;
          setTimeout(typeNext, TYPE_DELAY_MS);
        }
        typeNext();
      });
    });
  }

  function syncHiddenField(el, value) {
    var container = el.closest("form, dialog, .modal-content") || document;
    var hidden = container.querySelector('input[type=hidden][name="' + (el.name || "body") + '"]');
    if (!hidden && container === document) {
      var allHidden = document.querySelectorAll('input[type=hidden]');
      for (var i = 0; i < allHidden.length; i++) {
        if (allHidden[i].name === (el.name || "body")) { hidden = allHidden[i]; break; }
      }
    }
    if (!hidden) {
      var id = el.id || "";
      if (id) {
        var byId = document.getElementById(id + "-hidden");
        if (byId && byId.type === "hidden") hidden = byId;
      }
    }
    if (hidden) hidden.value = value;
  }

  function stepSelect(step) {
    var el = findField(step.field || "");
    if (!el || el.tagName !== "SELECT") {
      reportStep("Select '" + (step.field || "?") + "' not found", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(el);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        var matched = false;
        for (var i = 0; i < el.options.length; i++) {
          var opt = el.options[i];
          var t = normalize(opt.textContent || opt.value);
          var target = normalize(step.value || "");
          if (t === target || t.indexOf(target) !== -1 || target.indexOf(t) !== -1) {
            el.selectedIndex = i;
            matched = true;
            break;
          }
        }
        if (!matched && step.value) {
          el.value = step.value;
          matched = el.value === step.value;
        }
        el.dispatchEvent(new Event("change", { bubbles: true }));
        reportStep(
          "Selected '" + (step.field || "?") + "' = " + (step.value || ""),
          matched ? "done" : "warn"
        );
        setTimeout(function () {
          focusRing.style.display = "none";
          resolve(matched);
        }, 300);
      });
    });
  }

  function stepSubmit(step) {
    var scope = fieldScope();
    var form = scope.querySelector("form");
    if (!form && scope !== document) form = document.querySelector("form");
    if (!form) {
      reportStep("No form found to submit", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(form);
    var submitBtn = null;
    var btns = form.querySelectorAll("button[type=submit], input[type=submit]");
    if (btns.length) submitBtn = btns[0];
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        reportStep("Submitting form…", "info");
        if (submitBtn) {
          highlightFlash(submitBtn);
          pulseClick();
          try { submitBtn.click(); } catch (e) {}
        }
        try {
          form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
        } catch (e) {}
        setTimeout(function () {
          focusRing.style.display = "none";
          resolve(true);
        }, 800);
      });
    });
  }

  function stepWait(step) {
    return waitMs(Math.min(step.ms || 500, 10000)).then(function () {
      reportStep("Waiting " + (step.ms || 500) + "ms", "info");
      return true;
    });
  }

  function parseCellRef(ref) {
    var m = /^([A-Z]+)([0-9]+)$/.exec((ref || "").toUpperCase());
    if (!m) return null;
    var col = 0;
    for (var i = 0; i < m[1].length; i++) {
      col = col * 26 + (m[1].charCodeAt(i) - 64);
    }
    return { row: parseInt(m[2], 10) - 1, col: col - 1, ref: ref.toUpperCase() };
  }

  function stepCell(step) {
    var parsed = parseCellRef(step.cell || "");
    if (!parsed) {
      reportStep("Invalid cell reference '" + (step.cell || "?") + "'", "warn");
      return Promise.resolve(false);
    }
    var scope = fieldScope();
    var cellEl = scope.querySelector(
      '[data-row="' + parsed.row + '"][data-col="' + parsed.col + '"], ' +
      '[data-cell="' + parsed.ref + '"], [data-ref="' + parsed.ref + '"]'
    );
    if (!cellEl && scope !== document) {
      cellEl = document.querySelector(
        '[data-row="' + parsed.row + '"][data-col="' + parsed.col + '"], ' +
        '[data-cell="' + parsed.ref + '"], [data-ref="' + parsed.ref + '"]'
      );
    }
    if (!cellEl) {
      reportStep("Cell '" + parsed.ref + "' not found in app", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(cellEl);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        highlightFlash(cellEl);
        pulseClick();
        try { cellEl.dispatchEvent(new MouseEvent("mousedown", { bubbles: true })); } catch (e) {}
        try { cellEl.dispatchEvent(new MouseEvent("mouseup", { bubbles: true })); } catch (e) {}
        try { cellEl.dispatchEvent(new MouseEvent("click", { bubbles: true })); } catch (e) {}
        setTimeout(function () {
          var formula = document.getElementById("formulaInput");
          if (!formula) {
            reportStep("No formula bar for cell '" + parsed.ref + "'", "warn");
            resolve(false);
            return;
          }
          formula.focus();
          formula.value = step.value || "";
          formula.dispatchEvent(new Event("input", { bubbles: true }));
          formula.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }));
          reportStep("Cell '" + parsed.ref + "' = " + (step.value || ""), "done");
          setTimeout(function () {
            focusRing.style.display = "none";
            resolve(true);
          }, 500);
        }, 300);
      });
    });
  }

  var EXECUTORS = {
    open: stepOpen,
    click: stepClick,
    fill: stepFill,
    select: stepSelect,
    submit: stepSubmit,
    wait: stepWait,
    cell: stepCell,
  };

  /* ------------------------------------------------------------
     Public API
     ------------------------------------------------------------ */

  function executePlan(plan) {
    if (!plan || !plan.steps || !plan.steps.length) {
      reportStep("Empty UI plan", "warn");
      return Promise.resolve(false);
    }
    if (ACTIVE) {
      reportStep("Another automation is already running", "warn");
      return Promise.resolve(false);
    }
    ACTIVE = { completed: 0, total: plan.steps.length };
    ensureOverlay();
    resetStepCard();
    reportStep("Starting automation for app '" + (plan.app || "?") + "'", "info");

    var chain = Promise.resolve(true);
    plan.steps.forEach(function (step) {
      chain = chain.then(function () {
        var fn = EXECUTORS[step.op] || null;
        if (!fn) {
          reportStep("Unknown step op '" + step.op + "'", "warn");
          return false;
        }
        return fn(step, plan).then(function (ok) {
          ACTIVE.completed++;
          var evt = new CustomEvent("gb:ui-step", {
            detail: { text: "Step " + ACTIVE.completed + "/" + ACTIVE.total, status: "progress" },
          });
          window.dispatchEvent(evt);
          return ok;
        });
      });
    });

    return chain
      .then(function () {
        reportStep("Automation complete", "done");
        ACTIVE = null;
        if (focusRing) focusRing.style.display = "none";
        return true;
      })
      .catch(function (e) {
        reportStep("Automation error: " + e.message, "warn");
        ACTIVE = null;
        return false;
      });
  }

  function focusEntity(result) {
    if (!result) return Promise.resolve(false);
    return openApp(result.app).then(function () {
      reportStep("Opening " + result.type + " '" + result.title + "'", "info");
      var query = result.title || "";
      return waitFor(function () {
        var rows = document.querySelectorAll("tr, [data-entity], [data-id]");
        for (var i = 0; i < rows.length; i++) {
          var txt = normalize(rows[i].textContent);
          if (query && txt.indexOf(normalize(query)) !== -1) return rows[i];
        }
        return null;
      }, STEP_TIMEOUT_MS).then(function (found) {
        if (!found) {
          reportStep("Could not locate record in window", "warn");
          return false;
        }
        var rect = getElementRect(found);
        found.scrollIntoView({ block: "center", behavior: "smooth" });
        setTimeout(function () {
          moveCursorTo(rect, function () {
            highlightFlash(found);
            pulseClick();
            try { found.click(); } catch (e) {}
            reportStep("Focused " + result.type + " '" + result.title + "'", "done");
            setTimeout(function () {
              if (focusRing) focusRing.style.display = "none";
            }, 600);
          });
        }, 400);
        return true;
      });
    });
  }

  window.GBUiOrchestrator = {
    executePlan: executePlan,
    focusEntity: focusEntity,
    isRunning: function () { return !!ACTIVE; },
  };
})();
