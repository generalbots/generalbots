/**
 * GBUniversalSearch — one search bar for all ~80 suite apps.
 *
 * Debounced GET /api/ui/search returns grouped results; clicking a result
 * opens the target app window and focuses the record via GBUiOrchestrator.
 */
(function () {
  "use strict";

  if (window.GBUniversalSearch) return;

  var SEARCH_DEBOUNCE_MS = 300;
  var MIN_QUERY_LEN = 2;
  var panel = null;
  var input = null;
  var timer = null;
  var lastQuery = "";

  var APP_LABELS = {
    people: "People",
    crm: "CRM",
    products: "Products",
    tickets: "Tickets",
    research: "Research",
    drive: "Drive",
    admin: "Admin",
  };

  var TYPE_ICONS = {
    person: "\u{1F464}",
    contact: "\u{1F4C7}",
    product: "\u{1F4E6}",
    ticket: "\u{1F4CB}",
    document: "\u{1F4C4}",
    file: "\u{1F4C1}",
    bot: "\u{1F916}",
  };

  function normalize(s) {
    return (s || "").toLowerCase().replace(/\s+/g, " ").trim();
  }

  function buildPanel() {
    if (panel) return panel;
    panel = document.createElement("div");
    panel.id = "gb-search-panel";
    panel.style.cssText =
      "position:fixed;top:48px;left:50%;transform:translateX(-50%);width:520px;max-width:92vw;" +
      "max-height:60vh;overflow-y:auto;background:#1b1b1f;border:1px solid #3a3a3f;" +
      "border-radius:10px;box-shadow:0 12px 40px rgba(0,0,0,.5);z-index:2147483002;" +
      "display:none;font-family:'Fira Code',monospace;font-size:13px;color:#e8e8e8;";
    document.body.appendChild(panel);
    panel.addEventListener("mousedown", function (e) { e.stopPropagation(); });
    return panel;
  }

  function closePanel() {
    if (panel) panel.style.display = "none";
  }

  function groupByApp(results) {
    var groups = {};
    results.forEach(function (r) {
      if (!groups[r.app]) groups[r.app] = [];
      groups[r.app].push(r);
    });
    return groups;
  }

  function renderResults(results) {
    var p = buildPanel();
    p.innerHTML = "";
    if (!results.length) {
      p.innerHTML =
        '<div style="padding:16px;text-align:center;color:#888">' +
        "No results for '" + escapeHtml(lastQuery) + "'</div>";
      p.style.display = "block";
      return;
    }
    var groups = groupByApp(results);
    Object.keys(groups).forEach(function (app) {
      var header = document.createElement("div");
      header.textContent = APP_LABELS[app] || app;
      header.style.cssText =
        "padding:8px 14px 4px;font-size:11px;text-transform:uppercase;" +
        "letter-spacing:.08em;color:#22c55e;";
      p.appendChild(header);
      groups[app].forEach(function (r) {
        var item = document.createElement("div");
        item.style.cssText =
          "padding:8px 14px;display:flex;align-items:center;gap:10px;cursor:pointer;" +
          "border-top:1px solid #2a2a2e;";
        item.onmouseover = function () { item.style.background = "#26262b"; };
        item.onmouseout = function () { item.style.background = "transparent"; };
        var icon = document.createElement("span");
        icon.textContent = TYPE_ICONS[r.type] || "\u{1F50D}";
        icon.style.fontSize = "16px";
        var body = document.createElement("div");
        body.style.flex = "1";
        body.style.minWidth = "0";
        var title = document.createElement("div");
        title.textContent = r.title;
        title.style.fontWeight = "600";
        title.style.whiteSpace = "nowrap";
        title.style.overflow = "hidden";
        title.style.textOverflow = "ellipsis";
        var sub = document.createElement("div");
        sub.textContent = r.subtitle || r.type;
        sub.style.fontSize = "11px";
        sub.style.color = "#888";
        sub.style.whiteSpace = "nowrap";
        sub.style.overflow = "hidden";
        sub.style.textOverflow = "ellipsis";
        body.appendChild(title);
        body.appendChild(sub);
        var arrow = document.createElement("span");
        arrow.textContent = "\u2197";
        arrow.style.color = "#22c55e";
        item.appendChild(icon);
        item.appendChild(body);
        item.appendChild(arrow);
        item.onclick = function () {
          closePanel();
          if (input) input.value = "";
          if (window.GBUiOrchestrator) {
            window.GBUiOrchestrator.focusEntity(r);
          } else {
            window.open(r.url, "_blank");
          }
        };
        p.appendChild(item);
      });
    });
    p.style.display = "block";
  }

  function escapeHtml(s) {
    var div = document.createElement("div");
    div.textContent = s || "";
    return div.innerHTML;
  }

  function doSearch(query) {
    lastQuery = query;
    var headers = {};
    var token = window.GBAuthGuard && window.GBAuthGuard.getToken
      ? window.GBAuthGuard.getToken()
      : (window.localStorage ? localStorage.getItem("gb-access-token") : null);
    if (token) headers["Authorization"] = "Bearer " + token;
    fetch("/api/ui/search?q=" + encodeURIComponent(query), { headers: headers })
      .then(function (r) {
        if (r.status === 401) {
          buildPanel().innerHTML =
            '<div style="padding:16px;text-align:center;color:#f59e0b">' +
            "Sign in to search across apps</div>";
          buildPanel().style.display = "block";
          throw new Error("auth");
        }
        return r.ok ? r.json() : { results: [] };
      })
      .then(function (data) {
        if (lastQuery !== query) return;
        renderResults((data && data.results) || []);
      })
      .catch(function (e) {
        if (e && e.message === "auth") return;
        buildPanel().innerHTML =
          '<div style="padding:16px;text-align:center;color:#ef4444">' +
          "Search service unavailable</div>";
        buildPanel().style.display = "block";
      });
  }

  function onInput() {
    var q = (input.value || "").trim();
    if (q.length < MIN_QUERY_LEN) {
      closePanel();
      return;
    }
    clearTimeout(timer);
    timer = setTimeout(function () { doSearch(q); }, SEARCH_DEBOUNCE_MS);
  }

  function onKeyDown(e) {
    if (e.key === "Escape") closePanel();
  }

  function mount(inputEl) {
    input = inputEl;
    input.addEventListener("input", onInput);
    input.addEventListener("keydown", onKeyDown);
    input.addEventListener("focus", function () {
      if ((input.value || "").trim().length >= MIN_QUERY_LEN) doSearch(input.value.trim());
    });
    document.addEventListener("mousedown", function (e) {
      if (panel && panel.style.display === "block") {
        if (!panel.contains(e.target) && e.target !== input) closePanel();
      }
    });
  }

  /* Expose for any host page to bind its search box. */
  window.GBUniversalSearch = {
    mount: mount,
    close: closePanel,
  };
})();
