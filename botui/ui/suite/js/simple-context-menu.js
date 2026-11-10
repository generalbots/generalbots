/* ============================================ */
/* GB FLUENT — Simple context menu everywhere   */
/* Copy / paste / cut / select all, plus        */
/* DuckDuckGo search and Wikipedia definition.  */
/* Desktop (wallpaper) menu stays separate.      */
/* ============================================ */
(function () {
  "use strict";
  // #2095 — user-visible strings resolve through the shared catalogue
  // (window.gbT) with the authored English text as fallback.
  function T(key, fallback, params) {
    return window.gbT ? window.gbT(key, fallback, params) : fallback;
  }


  var menu = null;

  function closeMenu() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function getSelectionText() {
    var text = "";
    var sel = window.getSelection();
    if (sel) text = String(sel).trim();
    if (!text) {
      var active = document.activeElement;
      if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA")) {
        var s = active.selectionStart;
        var e = active.selectionEnd;
        if (typeof s === "number" && typeof e === "number" && e > s) {
          text = active.value.substring(s, e).trim();
        }
      }
    }
    return text;
  }

  function execCommand(cmd) {
    try {
      document.execCommand(cmd, false, null);
    } catch (err) {
      /* ignore */
    }
  }

  function doCopy() {
    var text = getSelectionText();
    if (text && navigator.clipboard && typeof navigator.clipboard.writeText === "function") {
      navigator.clipboard.writeText(text).catch(function () {
        execCommand("copy");
      });
    } else {
      execCommand("copy");
    }
  }

  function doCut() {
    execCommand("cut");
  }

  function doPaste() {
    if (navigator.clipboard && typeof navigator.clipboard.readText === "function") {
      navigator.clipboard.readText().then(function (text) {
        var active = document.activeElement;
        if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA")) {
          var s = active.selectionStart;
          var e = active.selectionEnd;
          var v = active.value;
          active.value = v.substring(0, s) + text + v.substring(e);
          active.selectionStart = active.selectionEnd = s + text.length;
        } else {
          execCommand("paste");
        }
      }).catch(function () {
        execCommand("paste");
      });
    } else {
      execCommand("paste");
    }
  }

  function doSelectAll() {
    var active = document.activeElement;
    if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA")) {
      active.select();
      return;
    }
    var sel = window.getSelection();
    if (sel && typeof sel.removeAllRanges === "function") {
      var range = document.createRange();
      try {
        range.selectNodeContents(document.body);
        sel.removeAllRanges();
        sel.addRange(range);
      } catch (err) {
        execCommand("selectAll");
      }
    } else {
      execCommand("selectAll");
    }
  }

  function openUrl(url) {
    try {
      window.open(url, "_blank", "noopener,noreferrer");
    } catch (err) {
      window.location.href = url;
    }
  }

  function searchDuckDuckGo() {
    var q = getSelectionText();
    if (!q) {
      try {
        q = (window.GBModal || { prompt: prompt }).prompt(
          T("ctxmenu-search-prompt", "Search DuckDuckGo:"));
      } catch (err) {
        q = "";
      }
    }
    if (!q) return;
    openUrl("https://duckduckgo.com/?q=" + encodeURIComponent(q));
  }

  function wikipediaDefinition() {
    var q = getSelectionText();
    if (!q) {
      try {
        q = (window.GBModal || { prompt: prompt }).prompt(
          T("ctxmenu-wikipedia-prompt", "Wikipedia definition:"));
      } catch (err) {
        q = "";
      }
    }
    if (!q) return;
    openUrl("https://en.wikipedia.org/w/index.php?search=" + encodeURIComponent(q));
  }

  function buildMenu(x, y) {
    closeMenu();
    menu = document.createElement("div");
    menu.className = "desktop-context-menu simple-context-menu";
    menu.style.left = Math.min(x, window.innerWidth - 200) + "px";
    menu.style.top = Math.min(y, window.innerHeight - 280) + "px";

    var items = [
      { label: "Cut", action: doCut },
      { label: T("action-copy", "Copy"), action: doCopy },
      { label: T("action-paste", "Paste"), action: doPaste },
      { label: T("action-select-all", "Select all"), action: doSelectAll },
      { separator: true },
      { label: T("ctxmenu-search", "Search DuckDuckGo"), action: searchDuckDuckGo },
      { label: T("ctxmenu-wikipedia", "Wikipedia definition"), action: wikipediaDefinition },
    ];

    items.forEach(function (item) {
      if (item.separator) {
        var sep = document.createElement("div");
        sep.className = "desktop-context-separator";
        menu.appendChild(sep);
        return;
      }
      var row = document.createElement("div");
      row.className = "desktop-context-item";
      row.textContent = item.label;
      row.addEventListener("click", function (ev) {
        ev.stopPropagation();
        closeMenu();
        try {
          item.action();
        } catch (err) {
          /* ignore */
        }
      });
      menu.appendChild(row);
    });

    document.body.appendChild(menu);
    setTimeout(function () {
      document.addEventListener("click", closeMenu, { once: true });
    }, 0);
  }

  function onContextMenu(e) {
    if (e.defaultPrevented) return;
    // Drive files and folders (HTMX app) own their context menu with
    // actions like Download, Rename or Duplicate. That handler registers its
    // document listener later than this one (the drive app is injected after
    // the shell), so this menu must step aside for those targets — otherwise
    // two menus appear at once. Not preventDefault: the drive handler then
    // runs normally.
    if (typeof e.target.closest === "function" &&
        e.target.closest(".file-card, .drive-file-item")) {
      return;
    }
    e.preventDefault();
    buildMenu(e.clientX, e.clientY);
  }

  function init() {
    document.addEventListener("contextmenu", onContextMenu, false);
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") closeMenu();
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();