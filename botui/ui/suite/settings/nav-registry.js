/* #1889 — declarative settings navigation registry.
 *
 * One table describes every settings node. The tree is then shaped in a
 * SINGLE pass from three orthogonal inputs:
 *
 *   - edition: cloud | desktop | os  (from the explicit `edition` key in the
 *     product file, mirrored by /api/product — never inferred from
 *     no_security);
 *   - depth:   essential | advanced  (advanced requires Preview mode ON);
 *   - scope:   personal | admin      (RBAC decides before preview does).
 *
 * Adding, hiding or renaming a section is a change to this table. The legacy
 * patchers in main/02_edition.js and main/08_preview.js are thin adapters over
 * `applyNavVisibility()` so the same rules drive the CSS classes the rest of
 * the app already relies on.
 */
(function (window) {
  "use strict";

  var ALL = ["cloud", "desktop", "os"];

  var NODES = [
    { id: "profile", group: "personal", depth: "essential", editions: ALL, scope: "personal" },
    { id: "security", group: "personal", depth: "essential", editions: ALL, scope: "personal" },
    { id: "appearance", group: "personal", depth: "essential", editions: ALL, scope: "personal" },
    { id: "language", group: "personal", depth: "essential", editions: ALL, scope: "personal" },
    { id: "notifications", group: "personal", depth: "advanced", editions: ALL, scope: "personal" },

    { id: "storage", group: "workspace", depth: "advanced", editions: ALL, scope: "personal" },
    { id: "network", group: "workspace", depth: "advanced", editions: ["os"], scope: "personal", control: "shell" },
    { id: "shared-links", group: "workspace", depth: "essential", editions: ["os"], scope: "personal", control: "shell", requires: "p2p" },
    { id: "integrations", group: "workspace", depth: "essential", editions: ALL, scope: "personal" },
    { id: "ai-accounts", group: "workspace", depth: "advanced", editions: ALL, scope: "personal" },
    { id: "on-device-ai", group: "workspace", depth: "essential", editions: ALL, scope: "admin" },
    { id: "vibe", group: "workspace", depth: "advanced", editions: ["cloud", "desktop", "os"], scope: "personal" },
    { id: "privacy", group: "workspace", depth: "advanced", editions: ALL, scope: "personal" },
    { id: "memory", group: "workspace", depth: "advanced", editions: ALL, scope: "personal" },
    { id: "billing", group: "workspace", depth: "advanced", editions: ["cloud"], scope: "personal" },

    { id: "ledger", group: "admin", depth: "advanced", editions: ALL, scope: "admin" },
    { id: "users", group: "admin", depth: "advanced", editions: ALL, scope: "admin" },
    { id: "groups", group: "admin", depth: "advanced", editions: ALL, scope: "admin" },
    { id: "bots", group: "admin", depth: "advanced", editions: ALL, scope: "admin" },
    { id: "dns", group: "admin", depth: "advanced", editions: ["cloud", "os"], scope: "admin" },
    { id: "audit", group: "admin", depth: "advanced", editions: ALL, scope: "admin" },
    { id: "organizations", group: "admin", depth: "advanced", editions: ALL, scope: "admin" },

    { id: "os", group: "system", depth: "advanced", editions: ["desktop", "os"], scope: "admin", control: "shell" },
    { id: "updates", group: "system", depth: "essential", editions: ["desktop", "os"], scope: "personal", control: "shell" },
    { id: "system", group: "system", depth: "essential", editions: ALL, scope: "personal" },
    { id: "about", group: "system", depth: "essential", editions: ALL, scope: "personal" },
    { id: "versions", group: "system", depth: "essential", editions: ALL, scope: "personal" }
  ];

  var BY_ID = {};
  NODES.forEach(function (node) { BY_ID[node.id] = node; });

  function config() {
    return window.productConfig || {};
  }

  /** Explicit edition from the product file; the fallback keeps older builds
   *  working until their `.product` file gains `edition=`. */
  function edition() {
    var declared = String(config().edition || "").toLowerCase();
    if (declared === "cloud" || declared === "desktop" || declared === "os") {
      return declared;
    }
    if (config().shell_mode === true || config().no_security === true) {
      return "os";
    }
    if (window.__TAURI__ || window.__TAURI_INTERNALS__) {
      return "desktop";
    }
    return "cloud";
  }

  /** #1865 — Offline is a hard gate, so settings that only exist when the
   *  desktop can be reached must disappear with it. The master switch writes
   *  the same key from Settings → Network. */
  function peerMode() {
    try {
      return (window.localStorage && window.localStorage.getItem("gb_peer_network_mode")) || "offline";
    } catch (e) {
      return "offline";
    }
  }

  function previewOn() {
    return !!(window.GBPreviewMode && window.GBPreviewMode.isOn());
  }

  function isAdmin() {
    try {
      var user = gbStoreGet("gb-user", {}) || {};
      if (Array.isArray(user.roles) && user.roles.length) {
        return user.roles.some(function (role) {
          return String(role).toLowerCase().indexOf("admin") !== -1;
        });
      }
      if (user.is_admin === true) { return true; }
      if (localStorage.getItem("management_is_admin") === "true") { return true; }
    } catch (e) { /* unauthenticated: treated as non-admin */ }
    return false;
  }

  /** Whether a node is reachable in the current deployment. */
  function isVisible(id, options) {
    var node = BY_ID[id];
    if (!node) { return true; }
    var opts = options || {};
    var current = opts.edition || edition();
    if (node.editions.indexOf(current) === -1) { return false; }
    if (node.control === "shell" && current !== "os" && current !== "desktop") {
      return false;
    }
    if (node.requires === "p2p" && peerMode() !== "p2p") { return false; }
    if (node.scope === "admin" && !(opts.isAdmin === undefined ? isAdmin() : opts.isAdmin)) {
      return false;
    }
    // RBAC decides administration before preview does (#1882): an admin keeps
    // the admin nodes with Preview OFF.
    if (node.scope === "admin") { return true; }
    if (node.depth === "advanced" && !(opts.preview === undefined ? previewOn() : opts.preview)) {
      return false;
    }
    return true;
  }

  /** Single pass over the tree. Idempotent: HTMX swaps just call it again. */
  function applyNavVisibility() {
    var current = edition();
    var admin = isAdmin();
    var nav = document.querySelector(".settings-nav");
    if (!nav) { return; }
    if (document.body) {
      document.body.setAttribute("data-gb-edition", current);
    }
    var anyVisibleInGroup = { admin: false };
    nav.querySelectorAll("a.nav-item").forEach(function (item) {
      var id = (item.getAttribute("href") || "").replace("#", "");
      if (!BY_ID[id]) { return; }
      var visible = isVisible(id, { edition: current, isAdmin: admin });
      item.hidden = !visible;
      item.setAttribute("data-gb-visible", visible ? "true" : "false");
      if (visible && BY_ID[id].scope === "admin") { anyVisibleInGroup.admin = true; }
    });
    // Dividers follow their group: the Administration divider is shown only
    // when at least one admin node survived RBAC (#1882).
    nav.querySelectorAll(".nav-divider").forEach(function (divider) {
      var label = (divider.textContent || "").trim().toLowerCase();
      if (label.indexOf("administration") !== -1) {
        divider.hidden = !anyVisibleInGroup.admin;
      }
    });
  }

  window.GBNavRegistry = {
    nodes: NODES,
    get: function (id) { return BY_ID[id] || null; },
    edition: edition,
    isAdmin: isAdmin,
    isVisible: isVisible,
    peerMode: peerMode,
    applyNavVisibility: applyNavVisibility
  };
})(window);
