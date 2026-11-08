/* #1881 — "This workspace" key-information card.
 *
 * Read-only status only. Every chip deep-links into the surface that owns the
 * value (Storage, Versions, Cloud billing, Preview); nothing is edited here.
 * When an endpoint fails the chip says "unavailable" — never a fake value.
 */
(function (window) {
  "use strict";

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

  function setText(id, value) {
    var node = byId(id);
    if (node) { node.textContent = value; }
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
    fetch("/api/auth/me", { headers: headers(), cache: "no-store" })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
      })
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
    fetch("/api/system/versions", { headers: headers(), cache: "no-store" })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
      })
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
    fetch("/api/user/billing/plan", { headers: headers(), cache: "no-store" })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.text();
      })
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
  document.addEventListener("gb-product-config", render);
  document.addEventListener("htmx:load", render);
  document.addEventListener("htmx:afterSwap", render);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }
})(window);
