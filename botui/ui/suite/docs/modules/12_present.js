"use strict";
/* docs advanced module: 12_present — full-screen presenting mode (#1695).
 *
 * "Collaborative documents and presentations" lists presenting as part of the
 * parity claim. The editor had no presentation surface: this module freezes
 * the document read-only, hides the editing chrome, enters full screen and
 * pages through the content with PageUp/PageDown (or the arrow keys / space),
 * restoring everything on Escape or the floating exit button.
 *
 * Public API (window.DocsPresent):
 *   enter() / exit() / toggle() / isActive() / nextPage() / prevPage()
 */
(function () {
  var active = false;
  var previousEditable = null;
  var exitBar = null;

  function article() {
    return document.querySelector("article[contenteditable]") ||
      document.querySelector("article");
  }

  function pageHeight() {
    return Math.max(200, window.innerHeight * 0.85);
  }

  function nextPage() {
    window.scrollBy({ top: pageHeight(), behavior: "smooth" });
  }

  function prevPage() {
    window.scrollBy({ top: -pageHeight(), behavior: "smooth" });
  }

  function buildExitBar() {
    if (exitBar) return exitBar;
    exitBar = document.createElement("div");
    exitBar.className = "docs-present-exitbar";
    exitBar.innerHTML =
      '<span class="docs-present-legend">Presenting</span>' +
      '<button type="button" class="docs-present-page" data-act="prev" aria-label="Previous page">\u25B2</button>' +
      '<button type="button" class="docs-present-page" data-act="next" aria-label="Next page">\u25BC</button>' +
      '<button type="button" class="docs-present-stop">Exit</button>';
    exitBar.addEventListener("click", function (event) {
      var button = event.target.closest("button");
      if (!button) return;
      if (button.dataset.act === "next") nextPage();
      else if (button.dataset.act === "prev") prevPage();
      else exit();
    });
    document.body.appendChild(exitBar);
    return exitBar;
  }

  function onKey(event) {
    if (!active) return;
    if (event.key === "Escape") {
      event.preventDefault();
      exit();
      return;
    }
    if (event.key === "PageDown" || event.key === "ArrowDown" || event.key === " ") {
      event.preventDefault();
      nextPage();
      return;
    }
    if (event.key === "PageUp" || event.key === "ArrowUp") {
      event.preventDefault();
      prevPage();
    }
  }

  function enter() {
    if (active) return;
    var body = document.body;
    var doc = article();
    previousEditable = doc ? doc.getAttribute("contenteditable") : null;
    if (doc) doc.setAttribute("contenteditable", "false");
    body.classList.add("docs-presenting");
    active = true;
    buildExitBar();
    document.addEventListener("keydown", onKey, true);
    if (document.documentElement.requestFullscreen && !document.fullscreenElement) {
      // Full screen is a nicety: a denied request must not break presenting.
      document.documentElement.requestFullscreen().catch(function () {});
    }
    window.scrollTo({ top: 0, behavior: "auto" });
  }

  function exit() {
    if (!active) return;
    active = false;
    document.body.classList.remove("docs-presenting");
    document.removeEventListener("keydown", onKey, true);
    var doc = article();
    if (doc) {
      if (previousEditable === null) doc.removeAttribute("contenteditable");
      else doc.setAttribute("contenteditable", previousEditable);
    }
    if (exitBar) {
      exitBar.remove();
      exitBar = null;
    }
    if (document.fullscreenElement && document.exitFullscreen) {
      document.exitFullscreen().catch(function () {});
    }
  }

  function toggle() {
    if (active) exit();
    else enter();
  }

  document.addEventListener("fullscreenchange", function () {
    // Leaving full screen by the browser UI must also leave present mode,
    // otherwise the editor stays read-only with no visible way back.
    if (!document.fullscreenElement && active) exit();
  });

  window.DocsPresent = {
    enter: enter,
    exit: exit,
    toggle: toggle,
    isActive: function () { return active; },
    nextPage: nextPage,
    prevPage: prevPage,
  };
})();
