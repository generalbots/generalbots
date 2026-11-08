/* #1863 — Settings → Shared Links.
 *
 * The panel answers one question: who can actually reach a public file link?
 * It renders the three reachability tiers, the URL this installation currently
 * composes, and the configuration each tier needs. No request is made outside
 * the local host; the only server call is the existing /api/system/os endpoint
 * used to turn the machine hostname into a LAN address.
 */
(function (window) {
  "use strict";

  var PORT_FALLBACK = "8080";
  var LOCAL_URL_ID = "shared-links-local-url";
  var LAN_URL_ID = "shared-links-lan-url";
  var DOMAIN_URL_ID = "shared-links-domain-url";
  var FREE_URL_ID = "shared-links-free-url";

  var loading = false;

  function byId(id) {
    return document.getElementById(id);
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function token() {
    return localStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      "";
  }

  function authHeaders() {
    var value = token();
    return value ? { Authorization: "Bearer " + value } : {};
  }

  function normalizeBase(value) {
    var base = String(value == null ? "" : value).trim();
    if (!base || !/^https?:\/\//i.test(base)) {
      return "";
    }
    return base.replace(/\/+$/, "");
  }

  function isLoopbackHost(host) {
    var value = String(host || "").toLowerCase();
    return value === "localhost" ||
      value === "127.0.0.1" ||
      value === "::1" ||
      value === "[::1]" ||
      value.endsWith(".localhost");
  }

  function currentPort() {
    if (window.location.port) {
      return window.location.port;
    }
    return PORT_FALLBACK;
  }

  function apiOrigin() {
    var location = window.location;
    if ((location.protocol === "http:" || location.protocol === "https:") && location.host) {
      return location.protocol + "//" + location.host;
    }
    return "http://127.0.0.1:" + PORT_FALLBACK;
  }

  function loopbackBase() {
    var location = window.location;
    var host = location.hostname || "";
    if (isLoopbackHost(host)) {
      return apiOrigin();
    }
    // The panel was opened from another device; loopback still names the
    // machine that runs the server, which is what the app can hand out.
    return "http://127.0.0.1:" + currentPort();
  }

  function lanBaseFromLocation() {
    var location = window.location;
    var host = location.hostname || "";
    if ((location.protocol === "http:" || location.protocol === "https:") &&
        host && !isLoopbackHost(host) && host !== "0.0.0.0") {
      return location.protocol + "//" + location.host;
    }
    return "";
  }

  function lanBaseFromHostname(hostname) {
    var host = String(hostname || "").trim().toLowerCase();
    if (!host || host === "unknown" || host === "localhost") {
      return "";
    }
    host = host.replace(/[^a-z0-9.-]/g, "-");
    if (host.endsWith(".local") || host.indexOf(".") !== -1) {
      return "http://" + host + ":" + currentPort();
    }
    return "http://" + host + ".local:" + currentPort();
  }

  function configuredDomain() {
    var config = window.productConfig || {};
    return normalizeBase(
      window.GB_PUBLIC_BASE_URL ||
      config.public_base_url ||
      config.publicBaseUrl ||
      config.public_url ||
      ""
    );
  }

  function linkTemplate(base) {
    return String(base).replace(/\/+$/, "") + "/api/files/public/…";
  }

  function setText(id, value) {
    var node = byId(id);
    if (node) {
      node.textContent = value == null ? "" : String(value);
    }
  }

  function setStatus(id, label, tone) {
    var node = byId(id);
    if (!node) {
      return;
    }
    node.className = "shared-status shared-status-" + (tone || "off");
    node.innerHTML = '<span class="shared-status-dot"></span>' + escapeHtml(label);
  }

  function setDetail(id, text) {
    var node = byId(id);
    if (node) {
      node.textContent = text;
    }
  }

  function renderLocal() {
    var base = loopbackBase();
    setText(LOCAL_URL_ID, linkTemplate(base));
    setText("shared-links-port", currentPort());
  }

  function renderLan(base) {
    var lanBase = base || lanBaseFromLocation();
    var status = byId("shared-lan-status");
    if (!lanBase) {
      if (status) {
        status.title = "";
      }
      setStatus("shared-lan-status", "This machine only", "warn");
      setText(LAN_URL_ID, "Not detected — use this computer's IPv4 address");
      setDetail(
        "shared-lan-detail",
        "No network address was detected. The link above still works on this " +
        "computer; share it elsewhere only after configuring a domain."
      );
      return;
    }
    setStatus("shared-lan-status", "Ready", "ready");
    setText(LAN_URL_ID, linkTemplate(lanBase));
    setDetail(
      "shared-lan-detail",
      "Works for anyone on the same network. Nothing leaves this machine."
    );
    if (status) {
      status.title = "Reachable from devices on the same network";
    }
  }

  function renderDomain() {
    var base = configuredDomain();
    if (base) {
      setStatus("shared-domain-status", "Configured", "ready");
      setText(DOMAIN_URL_ID, linkTemplate(base));
      setDetail(
        "shared-domain-detail",
        "New links are composed against " + base + "."
      );
      return;
    }
    setStatus("shared-domain-status", "Not configured", "off");
    setText(DOMAIN_URL_ID, linkTemplate("https://files.example.com"));
    setDetail(
      "shared-domain-detail",
      "Not configured — new links fall back to the local address above."
    );
  }

  function renderFree() {
    setStatus("shared-free-status", "Unavailable", "off");
    setText(FREE_URL_ID, linkTemplate("https://your-name.generalbots.org"));
  }

  function renderReachNote(lanBase) {
    var note = byId("shared-links-reach-note");
    if (!note) {
      return;
    }
    var domain = configuredDomain();
    note.hidden = !!lanBase || !!domain;
  }

  function fetchHostname() {
    return fetch("/api/system/os", {
      headers: authHeaders(),
      cache: "no-store"
    }).then(function (response) {
      return response.ok ? response.json() : null;
    }).then(function (data) {
      return data && data.hostname ? String(data.hostname) : "";
    }).catch(function () {
      return "";
    });
  }

  function stampChecked() {
    var node = byId("shared-links-checked");
    if (!node) {
      return;
    }
    var now = new Date();
    node.textContent = "Checked " + now.toLocaleTimeString();
  }

  function refresh(force) {
    var section = byId("shared-links-section");
    if (!section || section.hidden) {
      return;
    }
    if (loading && !force) {
      return;
    }
    loading = true;

    renderLocal();
    renderDomain();
    renderFree();

    var fromLocation = lanBaseFromLocation();
    if (fromLocation) {
      renderLan(fromLocation);
      renderReachNote(fromLocation);
      stampChecked();
      loading = false;
      return;
    }

    fetchHostname().then(function (hostname) {
      var lanBase = lanBaseFromHostname(hostname);
      renderLan(lanBase);
      renderReachNote(lanBase);
      stampChecked();
      loading = false;
    });
  }

  function copyFallback(text, done) {
    var area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.left = "-9999px";
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand("copy");
      done();
    } catch (error) {
      /* Clipboard access can be denied; leaving the value selectable is safe. */
    }
    document.body.removeChild(area);
  }

  function copyValue(text, button) {
    function done() {
      if (!button) {
        return;
      }
      var original = button.textContent;
      button.textContent = "Copied";
      window.setTimeout(function () {
        button.textContent = original;
      }, 1400);
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done).catch(function () {
        copyFallback(text, done);
      });
      return;
    }
    copyFallback(text, done);
  }

  function bind(section) {
    section.addEventListener("click", function (event) {
      var target = event.target && event.target.closest
        ? event.target.closest("[data-copy-target]")
        : null;
      if (!target) {
        return;
      }
      var source = byId(target.getAttribute("data-copy-target"));
      if (source) {
        copyValue(source.textContent || "", target);
      }
    });
  }

  function init() {
    var section = byId("shared-links-section");
    if (!section) {
      return;
    }
    if (!section.__gbSharedLinksBound) {
      bind(section);
      section.__gbSharedLinksBound = true;
    }
    // A deep link can open the settings fragment before this partial arrives;
    // activate the section now that it exists.
    if (window.location.hash === "#shared-links" &&
        typeof window.showSection === "function") {
      var nav = document.querySelector('.settings-nav a[href="#shared-links"]');
      if (nav) {
        window.showSection("shared-links", nav);
      }
    }
    wireMasterSwitch();
    refresh();
  }

  function wireMasterSwitch() {
    var select = byId("peer-network-mode");
    var button = byId("peer-master-switch");
    if (!select || !button || button.getAttribute("data-wired") === "1") {
      return;
    }
    button.setAttribute("data-wired", "1");
    try {
      var params = new URLSearchParams(window.location.search);
      var mode = (window.localStorage && window.localStorage.getItem("gb_peer_network_mode")) || "offline";
      select.value = mode === "p2p" ? "p2p" : "offline";
    } catch (e) {}
    button.addEventListener("click", function () {
      var chosen = select.value === "p2p" ? "p2p" : "offline";
      try {
        if (window.localStorage) {
          window.localStorage.setItem("gb_peer_network_mode", chosen);
        }
      } catch (e) {}
      var node = byId("shared-links-checked");
      if (node) {
        node.textContent = "Network mode set to " + chosen +
          ". Restart the app for the change to take effect.";
      }
    });
  }

  window.loadSharedLinks = init;
  window.refreshSharedLinks = function () {
    refresh(true);
  };

  document.addEventListener("gb-product-config", function () {
    refresh(true);
  });

  document.addEventListener("htmx:load", function () {
    if (byId("shared-links-section")) {
      init();
    }
  });

  window.addEventListener("hashchange", function () {
    if (window.location.hash === "#shared-links") {
      refresh(true);
    }
  });

  // #1877 — the section stays hidden until the edition pass confirms shell
  // mode. Refresh on that transition so the panel opens with real values
  // instead of the static placeholder URLs.
  window.addEventListener("gb-shell-mode-changed", function (e) {
    if (e && e.detail && e.detail.shell) {
      refresh(true);
    }
  });
  document.addEventListener("gb-shell-mode-changed", function (e) {
    if (e && e.detail && e.detail.shell) {
      refresh(true);
    }
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})(window);
