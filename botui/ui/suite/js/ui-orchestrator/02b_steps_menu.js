/* UI orchestrator — toolbar menus and downloads (2b/3). Issue #2037.
 *
 * `menu` walks a label path (Download → PDF) because one click cannot open a
 * submenu; `download` presses an export control and proves an artefact
 * appeared instead of trusting the click. Kept in its own module so
 * 02_steps.js stays under the size limit.
 */
(function (window, document) {
  "use strict";
  var N = document.__gb_mod_ui_orchestrator;
  if (!N || !N.EXECUTORS) return;
  var S = N.s;
  var reportStep = N.reportStep;
  var waitFor = N.waitFor;
  var findClickable = N.findClickable;
  var moveCursorTo = N.moveCursorTo;
  var highlightFlash = N.highlightFlash;
  var pulseClick = N.pulseClick;

  function getElementRect(el) {
    var r = el.getBoundingClientRect();
    return { left: r.left, top: r.top, width: r.width, height: r.height };
  }

  function clickElement(el, label) {
    var rect = getElementRect(el);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        highlightFlash(el);
        pulseClick();
        try {
          el.click();
        } catch (e) {
          try { el.dispatchEvent(new MouseEvent("click", { bubbles: true })); } catch (e2) {}
        }
        reportStep("Menu → " + label, "done");
        setTimeout(function () {
          S.focusRing.style.display = "none";
          resolve(true);
        }, 400);
      });
    });
  }

  function stepMenu(step) {
    var path = Array.isArray(step.path) ? step.path : (step.labels || []);
    if (!path.length && step.label) { path = [step.label]; }
    if (!path.length) {
      reportStep("Menu step has no path", "warn");
      return Promise.resolve(false);
    }
    var chain = Promise.resolve(true);
    path.forEach(function (label) {
      chain = chain.then(function (ok) {
        if (!ok) { return false; }
        var el = findClickable(label);
        if (!el) {
          reportStep("Menu '" + label + "' — element not found", "warn");
          return false;
        }
        return clickElement(el, label);
      });
    });
    return chain;
  }

  function stepDownload(step) {
    var label = step.label || "Download";
    var before = document.querySelectorAll('a[download], a[href^="blob:"]').length;
    var el = findClickable(label);
    if (!el) {
      reportStep("Download '" + label + "' — control not found", "warn");
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
          try { el.dispatchEvent(new MouseEvent("click", { bubbles: true })); } catch (e2) {}
        }
        waitFor(function () {
          var anchors = document.querySelectorAll('a[download], a[href^="blob:"]').length;
          var toast = document.querySelector('.gb-toast, .toast, [role="status"].toast');
          return anchors > before || !!toast;
        }, 4000).then(function (ok) {
          reportStep(
            ok
              ? "Download started for '" + label + "'"
              : "No download artefact appeared for '" + label + "'",
            ok ? "done" : "warn",
          );
          S.focusRing.style.display = "none";
          resolve(ok);
        });
      });
    });
  }

  N.stepMenu = stepMenu;
  N.stepDownload = stepDownload;
  N.EXECUTORS.menu = stepMenu;
  N.EXECUTORS.download = stepDownload;
})(window, document);
