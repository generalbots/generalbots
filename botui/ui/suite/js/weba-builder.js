"use strict";
/* #2538 — site builder UI over the WebA API: create/publish sites, custom
 * domains with verification state, campaign links and published previews. */
(function (window, document) {
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function id(x) { return document.getElementById(x); }
  function headers(extra) {
    var h = Object.assign({ Accept: "application/json" }, extra || {});
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function status(msg) { var el = id("webaStatus"); if (el) el.textContent = msg; }
  function json(url, opts) {
    return fetch(url, Object.assign({ headers: headers(opts && opts.body ? { "Content-Type": "application/json" } : {}), credentials: "same-origin" }, opts || {}))
      .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.json().catch(function () { return {}; }); });
  }
  function refresh() {
    json("/apps").then(function (apps) {
      var el = id("webaSites");
      if (!el) return;
      el.textContent = "";
      (apps || []).forEach(function (app) {
        var row = document.createElement("div");
        row.className = "weba-site-row";
        row.textContent = app.name + " (/render/" + app.slug + ") — " + app.status +
          (app.config && app.config.custom_domain ? " · " + app.config.custom_domain + " (" + app.config.domain_state + ")" : "") +
          (app.config && app.config.campaign_link ? " · campaign: " + app.config.campaign_link : "");
        var publish = document.createElement("button");
        publish.type = "button";
        publish.textContent = app.status === "published" ? t("weba-unpublish", "Unpublish") : t("weba-publish", "Publish");
        publish.onclick = function () { setPublished(app.id, app.status !== "published"); };
        row.appendChild(publish);
        var preview = document.createElement("button");
        preview.type = "button";
        preview.textContent = t("weba-preview", "Preview");
        preview.onclick = function () { window.open("/render/" + app.slug, "_blank", "noopener,noreferrer"); };
        row.appendChild(preview);
        el.appendChild(row);
      });
      return apps[0] || null;
    }).catch(function (e) { status(t("weba-error", "Could not load sites: ") + e.message); });
  }
  function create() {
    var name = id("webaSiteName"), slug = id("webaSlug");
    if (!name || !name.value.trim() || !slug || !slug.value.trim()) return;
    var payload = { name: name.value.trim(), slug: slug.value.trim(), description: "", template: {}, status: "draft", config: {} };
    json("/apps", { method: "POST", body: JSON.stringify(payload) })
      .then(function (app) { status(t("weba-created", "Site created: ") + (app.slug || slug.value)); refresh(); })
      .catch(function (e) { status(t("weba-error", "Could not create site: ") + e.message); });
  }
  function currentId() {
    var rows = document.querySelectorAll(".weba-site-row");
    return rows.length ? Number(rows[0].getAttribute("data-id")) : null;
  }
  function setPublished(siteId, publish) {
    json("/apps/" + siteId + (publish ? "/publish" : "/unpublish"), { method: "POST" })
      .then(function () { status(publish ? t("weba-published", "Site published.") : t("weba-unpublished", "Site unpublished.")); refresh(); })
      .catch(function (e) { status(t("weba-error", "Action failed: ") + e.message); });
  }
  function firstId() {
    return fetch("/apps", { headers: headers(), credentials: "same-origin" })
      .then(function (r) { return r.json(); }).then(function (apps) { return apps && apps[0] ? apps[0].id : null; });
  }
  function setDomain() {
    var domain = id("webaDomain");
    if (!domain || !domain.value.trim()) return;
    firstId().then(function (siteId) {
      if (!siteId) return;
      return json("/apps/" + siteId + "/domain", { method: "POST", body: JSON.stringify({ domain: domain.value.trim() }) })
        .then(function (r) { status(t("weba-domain-pending", "Domain verification pending: ") + r.domain); refresh(); });
    }).catch(function (e) { status(t("weba-error", "Action failed: ") + e.message); });
  }
  function setCampaign() {
    var campaign = id("webaCampaign");
    if (!campaign || !campaign.value.trim()) return;
    firstId().then(function (siteId) {
      if (!siteId) return;
      return json("/apps/" + siteId + "/campaign-link", { method: "POST", body: JSON.stringify({ campaign_id: campaign.value.trim(), page: "" }) })
        .then(function (r) { status(t("weba-campaign-created", "Campaign link: ") + r.link); refresh(); });
    }).catch(function (e) { status(t("weba-error", "Action failed: ") + e.message); });
  }
  function init() {
    // expose row ids for the simple first-site helpers
    var orig = refresh;
    refresh = function () {
      return orig().then(function () {
        json("/apps").then(function (apps) {
          Array.prototype.slice.call(document.querySelectorAll(".weba-site-row")).forEach(function (row, i) {
            if (apps && apps[i]) row.setAttribute("data-id", apps[i].id);
          });
        });
      });
    };
  }
  init();
  window.GBWebaBuilder = { refresh: refresh, create: create, setDomain: setDomain, setCampaign: setCampaign };
})(window, document);
