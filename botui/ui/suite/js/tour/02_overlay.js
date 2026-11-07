/* Guided tour engine — prompt, spotlight, callout and keyboard (2/3). */
(function (window, document) {
  "use strict";

  var T = window.__GBTour;
  if (!T) return;

  var R = T.runtime;

  function makeElement(tag, className) {
    var element = document.createElement(tag);
    if (className) element.className = className;
    return element;
  }

  function translatedText(element, key, fallback) {
    if (!key) return element;
    element.setAttribute("data-i18n", key);
    element.textContent = T.translate(key, fallback);
    return element;
  }

  function makeButton(action, key, fallback, variant) {
    var button = makeElement("button", "gb-tour-btn" + (variant ? " gb-tour-btn--" + variant : ""));
    button.type = "button";
    button.setAttribute("data-tour-action", action);
    translatedText(button, key, fallback);
    return button;
  }

  function actionFrom(event) {
    var node = event.target;
    while (node && node !== document) {
      if (node.getAttribute && node.getAttribute("data-tour-action")) return node;
      node = node.parentNode;
    }
    return null;
  }

  function reducedMotion() {
    return !!(window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches);
  }

  function clearLayer() {
    if (R.layer && R.layer.root && R.layer.root.parentNode) {
      R.layer.root.parentNode.removeChild(R.layer.root);
    }
    R.layer = null;
  }

  function focusFirst(container, selector) {
    if (!container) return;
    var target = selector ? container.querySelector(selector) : null;
    if (!target) return;
    try {
      target.focus({ preventScroll: true });
    } catch (e) {
      target.focus();
    }
  }

  function showPrompt(appId, def, options) {
    if (!def || !def.steps || !def.steps.length) return;
    options = options || {};
    if (!options.force && T.isComplete(appId)) return;
    if (R.state || R.promptState) return;

    clearLayer();
    R.promptState = { appId: appId, def: def };
    if (!R.lastFocused && document.activeElement && document.activeElement !== document.body) {
      R.lastFocused = document.activeElement;
    }
    if (!T.isComplete(appId)) T.mark(appId, "seen");

    var overlay = makeElement("div", "gb-tour-prompt-layer");
    overlay.setAttribute("data-gb-tour", "prompt");

    var card = makeElement("div", "gb-tour-prompt");
    card.setAttribute("role", "dialog");
    card.setAttribute("aria-modal", "true");
    card.setAttribute("aria-labelledby", "gb-tour-prompt-title");
    card.setAttribute("aria-describedby", "gb-tour-prompt-body");

    var eyebrow = makeElement("p", "gb-tour-eyebrow");
    translatedText(eyebrow, "tour-welcome-eyebrow", "Guided tour");
    card.appendChild(eyebrow);

    var title = makeElement("h2", "gb-tour-prompt-title");
    title.id = "gb-tour-prompt-title";
    translatedText(title, "tour-welcome-title", "Take a quick tour");
    card.appendChild(title);

    var body = makeElement("p", "gb-tour-prompt-body");
    body.id = "gb-tour-prompt-body";
    translatedText(body, "tour-welcome-body", "A few short steps to show you around this app. You can skip at any time.");
    card.appendChild(body);

    var actions = makeElement("div", "gb-tour-prompt-actions");
    actions.appendChild(makeButton("skip", "tour-skip", "Skip", "ghost"));
    actions.appendChild(makeButton("start-tour", "tour-start", "Start tour", "primary"));
    card.appendChild(actions);

    overlay.appendChild(card);
    overlay.addEventListener("click", function (event) {
      var button = actionFrom(event);
      if (!button || !overlay.contains(button)) return;
      event.preventDefault();
      var action = button.getAttribute("data-tour-action");
      if (action === "start-tour" && R.promptState) {
        start(R.promptState.appId, { force: true });
      } else if (action === "skip" && R.promptState) {
        skipPrompt();
      }
    });
    document.body.appendChild(overlay);
    document.body.classList.add("gb-tour-active");
    R.layer = { root: overlay, callout: null, spotlight: null };
    bindGlobalListeners();
    focusFirst(actions, '[data-tour-action="start-tour"]');
  }

  function skipPrompt() {
    if (!R.promptState) return;
    T.mark(R.promptState.appId, "skipped");
    close();
  }

  function start(appId, options) {
    options = options || {};
    var def = T.definitions[appId];
    if (!def || !def.steps || !def.steps.length) return false;
    var root = options.root || document.querySelector(def.root || "[data-gb-app]");
    if (!root) return false;

    clearLayer();
    R.promptState = null;
    if (!R.lastFocused && document.activeElement && document.activeElement !== document.body) {
      R.lastFocused = document.activeElement;
    }
    R.state = { appId: appId, def: def, root: root, index: 0, currentTarget: null };
    if (!T.isComplete(appId)) T.mark(appId, "seen");
    document.body.classList.add("gb-tour-active");
    bindGlobalListeners();
    renderStep(0, 1);
    return true;
  }

  function renderStep(index, direction) {
    if (!R.state) return;
    var steps = R.state.def.steps;
    var cursor = index;
    var found = -1;
    var target = null;

    while (cursor >= 0 && cursor < steps.length) {
      target = T.resolveTarget(steps[cursor], R.state.root);
      if (target) {
        found = cursor;
        break;
      }
      cursor += direction < 0 ? -1 : 1;
    }

    if (found < 0) {
      finish();
      return;
    }

    R.state.index = found;
    R.state.currentTarget = target;
    buildLayer(steps[found]);
    scrollToTarget(target);
    position();
  }

  function buildLayer(step) {
    clearLayer();
    var root = makeElement("div", "gb-tour-layer");
    root.setAttribute("data-gb-tour", "active");

    var spotlight = makeElement("div", "gb-tour-spotlight");
    spotlight.setAttribute("aria-hidden", "true");

    var callout = makeElement("div", "gb-tour-callout");
    callout.setAttribute("role", "dialog");
    callout.setAttribute("aria-modal", "true");
    callout.setAttribute("aria-labelledby", "gb-tour-callout-title");

    var progress = makeElement("p", "gb-tour-progress");
    progress.setAttribute("aria-live", "polite");
    progress.setAttribute("aria-atomic", "true");
    progress.setAttribute("data-i18n", "tour-step");
    progress.setAttribute("data-i18n-params", JSON.stringify({
      current: R.state.index + 1,
      total: R.state.def.steps.length,
    }));
    progress.textContent = T.translate("tour-step", "Step {current} of {total}", {
      current: R.state.index + 1,
      total: R.state.def.steps.length,
    });
    callout.appendChild(progress);

    var title = makeElement("h3", "gb-tour-title");
    title.id = "gb-tour-callout-title";
    translatedText(title, step.titleKey, step.title || (step.titleKey || ""));
    callout.appendChild(title);

    var body = makeElement("p", "gb-tour-body");
    translatedText(body, step.bodyKey, step.body || (step.bodyKey || ""));
    callout.appendChild(body);

    var footer = makeElement("div", "gb-tour-footer");
    footer.appendChild(makeButton("skip", "tour-skip", "Skip", "ghost"));

    var navigation = makeElement("div", "gb-tour-navigation");
    var back = makeButton("prev", "tour-back", "Back");
    back.disabled = R.state.index === 0;
    navigation.appendChild(back);

    var last = R.state.index === R.state.def.steps.length - 1;
    navigation.appendChild(makeButton("next", last ? "tour-done" : "tour-next", last ? "Done" : "Next", "primary"));
    footer.appendChild(navigation);
    callout.appendChild(footer);

    callout.addEventListener("click", function (event) {
      var button = actionFrom(event);
      if (!button || !callout.contains(button)) return;
      event.preventDefault();
      var action = button.getAttribute("data-tour-action");
      if (action === "next") next();
      else if (action === "prev") prev();
      else if (action === "skip") skip();
    });

    root.appendChild(spotlight);
    root.appendChild(callout);
    document.body.appendChild(root);
    R.layer = { root: root, callout: callout, spotlight: spotlight };
    focusFirst(navigation, '[data-tour-action="next"]');
  }

  function scrollToTarget(target) {
    try {
      target.scrollIntoView({
        block: "center",
        inline: "center",
        behavior: reducedMotion() ? "auto" : "smooth",
      });
    } catch (e) {
      try {
        target.scrollIntoView();
      } catch (ignore) {
        /* detached target; positioning keeps the last rect */
      }
    }
  }

  function schedulePosition() {
    if (R.positionScheduled) return;
    R.positionScheduled = true;
    window.requestAnimationFrame(function () {
      R.positionScheduled = false;
      position();
    });
  }

  function position() {
    if (!R.state || !R.layer || !R.state.currentTarget || !R.layer.spotlight || !R.layer.callout) return;
    var rect = R.state.currentTarget.getBoundingClientRect();
    var pad = 8;
    var width = Math.max(rect.width, 16);
    var height = Math.max(rect.height, 16);

    var spotlight = R.layer.spotlight;
    spotlight.style.top = Math.max(rect.top - pad, 0) + "px";
    spotlight.style.left = Math.max(rect.left - pad, 0) + "px";
    spotlight.style.width = Math.min(width + pad * 2, window.innerWidth) + "px";
    spotlight.style.height = Math.min(height + pad * 2, window.innerHeight) + "px";

    var callout = R.layer.callout;
    var step = R.state.def.steps[R.state.index] || {};
    var preference = step.placement || "below";
    var margin = 16;
    var gap = 14;
    callout.style.visibility = "hidden";
    var cardWidth = callout.offsetWidth;
    var cardHeight = callout.offsetHeight;
    var left = rect.left + rect.width / 2 - cardWidth / 2;
    var top = rect.bottom + gap;
    var placement = "below";

    if (preference === "right" && rect.right + gap + cardWidth <= window.innerWidth - margin) {
      left = rect.right + gap;
      top = rect.top + rect.height / 2 - cardHeight / 2;
      placement = "right";
    } else if (preference === "above" && rect.top - cardHeight - gap >= margin) {
      top = rect.top - cardHeight - gap;
      placement = "above";
    }

    if (top + cardHeight > window.innerHeight - margin && placement !== "above") {
      top = rect.top - cardHeight - gap;
      placement = "above";
    }
    left = Math.max(margin, Math.min(window.innerWidth - cardWidth - margin, left));
    if (top < margin) {
      top = margin;
      placement = "center";
    }

    callout.style.left = left + "px";
    callout.style.top = top + "px";
    callout.style.visibility = "visible";
    callout.classList.toggle("gb-tour-callout--above", placement === "above");
    callout.classList.toggle("gb-tour-callout--center", placement === "center");
  }

  function next() {
    if (!R.state) return;
    if (R.state.index >= R.state.def.steps.length - 1) {
      finish();
      return;
    }
    renderStep(R.state.index + 1, 1);
  }

  function prev() {
    if (!R.state || R.state.index <= 0) return;
    renderStep(R.state.index - 1, -1);
  }

  function skip() {
    if (!R.state) return;
    T.mark(R.state.appId, "skipped");
    close();
  }

  function finish() {
    if (!R.state) return;
    T.mark(R.state.appId, "completed");
    close();
  }

  function close() {
    clearLayer();
    document.body.classList.remove("gb-tour-active");
    R.state = null;
    R.promptState = null;
    if (R.lastFocused && typeof R.lastFocused.focus === "function") {
      try {
        R.lastFocused.focus({ preventScroll: true });
      } catch (e) {
        /* the previous element may no longer be focusable */
      }
    }
    R.lastFocused = null;
    window.setTimeout(T.drainWaiters, 120);
  }

  function trapFocus(event) {
    if (!R.layer) return;
    var container = R.layer.callout || (R.layer.root && R.layer.root.querySelector(".gb-tour-prompt"));
    if (!container) return;
    var nodes = container.querySelectorAll(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );
    if (!nodes.length) return;
    var first = nodes[0];
    var last = nodes[nodes.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function onKeyDown(event) {
    if (R.promptState) {
      if (event.key === "Escape") {
        event.preventDefault();
        skipPrompt();
      } else if (event.key === "Tab") {
        trapFocus(event);
      }
      return;
    }
    if (!R.state) return;
    if (event.key === "Escape") {
      event.preventDefault();
      skip();
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      next();
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      prev();
    } else if (event.key === "Tab") {
      trapFocus(event);
    }
  }

  function bindGlobalListeners() {
    if (R.listenersBound) return;
    R.listenersBound = true;
    document.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("resize", schedulePosition, true);
    window.addEventListener("scroll", schedulePosition, true);
    window.addEventListener("localeChanged", function () {
      if (!R.layer) return;
      if (window.i18n && typeof window.i18n.translatePage === "function") {
        window.i18n.translatePage(R.layer.root);
      }
      schedulePosition();
    });
  }

  T.showPrompt = showPrompt;
  T.start = start;
  T.next = next;
  T.prev = prev;
  T.skip = skip;
  T.finish = finish;
  T.close = close;
  T.current = function () {
    return R.state ? { appId: R.state.appId, index: R.state.index } : null;
  };
})(window, document);
