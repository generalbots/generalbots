/* #1881 — "This workspace" key-information card.
 *
 * Read-only status only. Every chip deep-links into the surface that owns the
 * value (Storage, Versions, Cloud billing, Preview); nothing is edited here.
 * When an endpoint fails the chip says "unavailable" — never a fake value.
 *
 * #2936: identity / version / plan reads are memoized per panel load and use
 * bounded backoff (honouring Retry-After). HTMX swaps reuse the cached promise
 * so /api/auth/me is never re-issued on every fragment swap.
 */
(function (window) {
  "use strict";

  var MAX_RETRIES = 2;
  var BACKOFF_CAP_MS = 8000;
  var RETRY_AFTER_CAP_S = 30;
  var fetchCache = Object.create(null);

  function byId(id) {
    return document.getElementById(id);
  }

  function token() {
    try {
      return localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") ||
        "";
    } catch (e) {
      return "";
    }
  }

  function headers() {
    var value = token();
    return value ? { Authorization: "Bearer " + value } : {};
  }

  function edition(current) {
    if (current) { return current; }
    if (window.GBNavRegistry && typeof window.GBNavRegistry.edition === "function") {
      return window.GBNavRegistry.edition();
    }
    var config = window.productConfig || {};
    if (config.edition === "cloud" || config.edition === "desktop" || config.edition === "os") {
      return config.edition;
    }
    return "cloud";
  }

  function editionLabel(value) {
    if (value === "os") { return "OS"; }
    if (value === "desktop") { return "Desktop"; }
    return "Cloud";
  }

  /* A panel load is identified by the auth token plus the active edition.
   * Changing either is the only reason to issue a fresh read; an HTMX swap
   * keeps the same identity, so it reuses the memoized promise below. */
  function cacheId(key) {
    return key + "|" + token() + "|" + edition();
  }

  function retryDelayMs(response, attempt) {
    var header = response && response.headers && response.headers.get
      ? response.headers.get("Retry-After") : null;
    if (header !== null && header !== undefined) {
      var seconds = Number(String(header).trim());
      if (isFinite(seconds) && seconds >= 0) {
        return Math.min(seconds, RETRY_AFTER_CAP_S) * 1000;
      }
      var at = Date.parse(String(header));
      if (isFinite(at)) {
        return Math.max(0, Math.min(at - Date.now(), RETRY_AFTER_CAP_S * 1000));
      }
    }
    // No usable Retry-After: bounded exponential backoff, never an open loop.
    return Math.min(1000 * Math.pow(2, attempt), BACKOFF_CAP_MS);
  }

  function isRetryable(response) {
    var status = response ? response.status : 0;
    return status === 429 || (status >= 500 && status < 600);
  }

  /* Single memoized fetch path. The first caller stores the in-flight promise;
   * every later render (including afterSwap) gets that exact promise back, so
   * a failed terminal result is also cached: after MAX_RETRIES the module
   * stops instead of restarting a 429 storm on the next fragment swap. */
  function memoFetch(key, url, options, parser) {
    var id = cacheId(key);
    var hit = fetchCache[id];
    if (hit) { return hit; }

    var attempt = 0;
    function run() {
      return fetch(url, options).then(function (response) {
        if (isRetryable(response) && attempt < MAX_RETRIES) {
          var wait = retryDelayMs(response, attempt);
          attempt += 1;
          return new Promise(function (resolve) { setTimeout(resolve, wait); }).then(run);
        }
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        // Cache the PARSED payload: a Response body can only be consumed once,
        // so re-renders after htmx:afterSwap must not call .json() again.
        return parser(response);
      });
    }
    var pending = Promise.resolve().then(run);
    fetchCache[id] = pending;
    return pending;
  }

  function clearFetchCache() {
    fetchCache = Object.create(null);
  }

  function setText(id, value) {
    var node = byId(id);
    if (!node) { return; }
    // The HTML ships a data-i18n loading placeholder; once real data lands,
    // drop it so a later locale re-apply cannot overwrite the rendered value.
    if (node.hasAttribute("data-i18n")) { node.removeAttribute("data-i18n"); }
    node.textContent = value;
  }

  function renderEdition() {
    var value = edition();
    setText("workspace-edition", editionLabel(value));
    var badge = byId("workspace-edition");
    if (!badge) { return; }
    badge.setAttribute("data-gb-edition", value);
    // OS/desktop editions have no Cloud plan; the chip says so instead of
    // leaving a Cloud-only surface looking broken.
    if (value !== "cloud") {
      var plan = byId("workspace-plan");
      if (plan) {
        plan.textContent = "Plan: n/a on this edition";
        plan.disabled = true;
      }
    }
  }

  function renderIdentity() {
    memoFetch("identity", "/api/auth/me", { headers: headers(), cache: "no-store" }, function (response) { return response.json(); })
      .then(function (payload) {
        var user = payload && (payload.user || payload);
        if (!user) { throw new Error("no user"); }
        var name = user.display_name || user.name || user.username || user.email || "Signed in";
        var org = user.org_name || user.organization || user.branch_name || "";
        var where = org || (edition() === "cloud" ? "Cloud workspace" : "This device");
        setText("workspace-identity", name + " · " + where);
      })
      .catch(function () {
        setText("workspace-identity", "Identity unavailable");
      });
  }

  function renderVersions() {
    var chip = document.querySelector('[data-workspace-goto="versions"]');
    if (!chip) { return; }
    memoFetch("versions", "/api/system/versions", { headers: headers(), cache: "no-store" }, function (response) { return response.json(); })
      .then(function (versions) {
        var value = (versions && (versions.botserver || versions.version)) || "";
        chip.textContent = value ? "Version: " + value : "Versions";
      })
      .catch(function () {
        chip.textContent = "Versions: unavailable";
      });
  }

  function renderPlan() {
    if (edition() !== "cloud") { return; }
    var chip = byId("workspace-plan");
    if (!chip) { return; }
    memoFetch("plan", "/api/user/billing/plan", { headers: headers(), cache: "no-store" }, function (response) { return response.text(); })
      .then(function (html) {
        var text = String(html || "").replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
        if (!text) {
          chip.textContent = "Plan: none managed here";
          return;
        }
        chip.textContent = "Plan: " + text.slice(0, 48);
      })
      .catch(function () {
        chip.textContent = "Plan: unavailable";
      });
  }

  function renderPreviewChip() {
    var chip = byId("workspace-preview");
    if (!chip) { return; }
    var on = !!(window.GBPreviewMode && GBPreviewMode.isOn());
    chip.textContent = "Preview: " + (on ? "on" : "off");
    chip.setAttribute("aria-pressed", on ? "true" : "false");
  }

  function gotoSection(id) {
    var nav = document.querySelector('.settings-nav a[href="#' + id + '"]');
    if (nav && typeof window.showSection === "function") {
      window.showSection(id, nav);
      if (id === "versions" && typeof window.loadVersions === "function") {
        window.loadVersions();
      }
    }
  }

  function bind() {
    var card = byId("workspace-card");
    if (!card || card.getAttribute("data-wired") === "1") { return; }
    card.setAttribute("data-wired", "1");
    card.addEventListener("click", function (event) {
      var target = event.target && event.target.closest ? event.target : null;
      if (!target) { return; }
      var goto = target.closest("[data-workspace-goto]");
      if (goto) {
        gotoSection(goto.getAttribute("data-workspace-goto"));
        return;
      }
      if (target.closest("#workspace-plan")) {
        if (edition() === "cloud") {
          if (typeof window.openCloudBilling === "function") { window.openCloudBilling(); }
          else { gotoSection("billing"); }
        }
        return;
      }
      if (target.closest("#workspace-preview") && window.GBPreviewMode) {
        window.GBPreviewMode.set(!GBPreviewMode.isOn());
      }
    });
  }

  function render() {
    bind();
    renderEdition();
    renderIdentity();
    renderVersions();
    renderPlan();
    renderPreviewChip();
  }

  window.addEventListener("gb-preview-changed", renderPreviewChip);
  // Only a genuine product-config change invalidates the panel-load cache.
  document.addEventListener("gb-product-config", function () {
    clearFetchCache();
    render();
  });
  document.addEventListener("htmx:load", render);
  // #2936 — the ONE afterSwap listener. It is guarded: fragment swaps reuse
  // the memoized fetches and must never clear the cache or start a new fetch.
  document.addEventListener("htmx:afterSwap", function () {
    if (!byId("workspace-card")) { return; }
    render();
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }
})(window);
