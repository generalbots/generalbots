/**
 * #2960 — offline regression guard for the Integrations app's two reported
 * console errors on an authenticated open:
 *   - GET /api/bots/{id}/integration-connections must be advertised by a
 *     roleless authenticated RBAC entry (tenant binding stays in the handler);
 *   - the catalog-health renderer must not dereference a removed DOM node's
 *     classList after the window closes.
 *
 * No server, browser, cargo or i18n change is involved; the driver owns live
 * UAT, issue closure and deployment.
 */
"use strict";

var h = require("./00_harness");

var RBAC = "botserver/crates/botsecurity-auth/src/rbac_middleware/routes_user/rest_routes.rs";
var RENDER = "botui/ui/suite/integrations/modules/02_render.js";

function count(text, needle) { return text.split(needle).length - 1; }

function loadRenderer() {
  var ns = {
    text: function (value, fallback) { return fallback || ""; },
    escapeHtml: function (value) { return String(value); },
    titleCase: function (value) { return String(value); },
    getState: function () { return { catalogStatus: "ready", providers: [] }; }
  };
  var sandbox = {
    window: { GBIntegrationsCatalog: ns },
    console: console,
    Intl: Intl, Date: Date, Number: Number, Array: Array, String: String
  };
  h.vm.createContext(sandbox);
  h.vm.runInContext(h.readRepo(RENDER), sandbox, { filename: RENDER });
  return ns;
}

h.staticTest("#2960 integration connections GET is a roleless authenticated route", function () {
  var text = h.readRepo(RBAC);
  h.assert(
    text.indexOf('RoutePermission::new("/api/bots/{id}/integration-connections", "GET", "")') !== -1,
    "the base GET catalogue entry exists"
  );
  h.assert(
    text.indexOf('RoutePermission::new("/api/bots/{id}/integration-connections/**", "GET", "")') !== -1,
    "the nested GET catalogue entry exists"
  );
  var base = text.indexOf('RoutePermission::new("/api/bots/{id}/integration-connections", "GET", "")');
  var catchAll = text.indexOf('RoutePermission::new("/api/bots/{id}/**", "GET", "")');
  h.assert(base < catchAll, "explicit GET rules precede the broad bots catch-all");
});

h.staticTest("#2960 every integration classList access is behind a null guard", function () {
  var text = h.readRepo(RENDER);
  h.assert(count(text, "classList") === 5, "the five known classList accesses are unchanged");
  h.assert(count(text, "query(root, \"[data-catalog-health]\").classList") === 0,
    "no direct query result is dereferenced");
  var health = text.indexOf("namespace.renderCatalogHealth = function (root) {");
  var guard = text.indexOf("if (!health || !healthText) return;", health);
  var firstUse = text.indexOf("health.classList.remove", health);
  h.assert(guard !== -1 && guard < firstUse, "the renderer guards before classList use");
  var unavailable = text.indexOf("namespace.renderCatalogUnavailable = function (root) {");
  h.assert(text.indexOf("if (health && healthText) {", unavailable) <
    text.indexOf("health.classList.remove(\"is-ready\");", unavailable),
    "the failure renderer guards before classList use");
});

h.test("#2960 catalog health render after window close throws no classList TypeError", function () {
  var ns = loadRenderer();
  var detached = { querySelector: function () { return null; } };
  ns.renderCatalogHealth(detached);
  ns.renderCatalogUnavailable(detached, { message: "gone" });

  var calls = 0;
  var health = {
    classList: {
      remove: function () { calls++; },
      add: function () { calls++; }
    }
  };
  var healthText = { textContent: "" };
  var attached = {
    querySelector: function (selector) {
      if (selector === "[data-catalog-health]") return health;
      if (selector === "[data-catalog-health-text]") return healthText;
      return null;
    }
  };
  ns.renderCatalogHealth(attached);
  h.assert(calls >= 2, "the normal render path still updates classList when the node exists");
});
