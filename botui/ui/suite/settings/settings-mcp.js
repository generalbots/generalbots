"use strict";

/* #1675 — MCP management embedded in Settings (desktop parity with Claude
   Desktop's Extensions panel). Every call targets the SAME endpoints the
   Integrations app uses — no parallel store, no new routes:
     GET  /api/sources/mcp                  JSON  MCP server catalog
     POST /api/sources/mcp/:name/enable     JSON  enable a server
     POST /api/sources/mcp/:name/disable    JSON  disable a server
     GET  /api/sources/mcp/:name/ui/:index  JSON  preview payload (base64 HTML)
   The list also reports hash-drifted ("pin_changed") UI resources so the user
   can be asked for re-consent from Settings, not only from chat. */

(function (ns) {
  var endpoints = {
    list: "/api/sources/mcp",
    apps: "/api/mcp/apps",
    action: function (name, action) {
      return "/api/sources/mcp/" + encodeURIComponent(name) + "/" + action;
    },
    preview: function (name, index) {
      return "/api/sources/mcp/" + encodeURIComponent(name) + "/ui/" + encodeURIComponent(index);
    }
  };

  function token() {
    return (
      localStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      ""
    );
  }

  function headers(extra) {
    var base = { Authorization: "Bearer " + token() };
    if (extra) { Object.keys(extra).forEach(function (key) { base[key] = extra[key]; }); }
    return base;
  }

  function botId() {
    return window.__INITIAL_BOT_NAME__ || "default";
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function normalize(payload) {
    var items = [];
    if (payload && Array.isArray(payload.data)) { items = payload.data; }
    else if (payload && Array.isArray(payload.items)) { items = payload.items; }
    else if (Array.isArray(payload)) { items = payload; }
    return items.filter(function (item) { return item && typeof item === "object"; })
      .map(function (server) {
        var tools = Number(server.tools_count);
        var resources = Array.isArray(server.ui_resources) ? server.ui_resources : [];
        return {
          name: String(server.name || "Unnamed server"),
          description: String(server.description || ""),
          server_type: String(server.server_type || "stdio"),
          enabled: server.enabled === true,
          tools_count: Number.isFinite(tools) ? tools : 0,
          ui_resources: resources.filter(function (resource) {
            return resource && typeof resource === "object" && resource.uri;
          }).map(function (resource) {
            return {
              uri: String(resource.uri || ""),
              name: String(resource.name || resource.uri || ""),
              hash_changed: resource.pin_changed === true
            };
          })
        };
      });
  }

  function badge(label, tone) {
    return '<span class="settings-mcp-badge is-' + tone + '">' + escapeHtml(label) + "</span>";
  }

  function serverRow(server) {
    var state = server.enabled
      ? badge("Active", "healthy")
      : badge("Disabled", "error");
    var drift = server.ui_resources.some(function (resource) { return resource.hash_changed; })
      ? badge("Changed since approval", "error")
      : "";
    var toggles = server.ui_resources.map(function (resource, index) {
      return '<button class="btn-secondary btn-sm settings-mcp-preview" data-mcp-name="' +
        escapeHtml(server.name) + '" data-resource-index="' + index + '">Preview ' + escapeHtml(resource.name) + "</button>";
    }).join("");
    return '<div class="connection-item settings-mcp-row">' +
      '<div class="connection-info">' +
      '<span class="connection-name">' + escapeHtml(server.name) +
      ' <span class="connection-status">' + escapeHtml(server.server_type) + " · " + server.tools_count + " tool" +
      (server.tools_count === 1 ? "" : "s") + "</span></span>" +
      '<span class="connection-status settings-mcp-desc">' + escapeHtml(server.description || "No description provided.") + "</span>" +
      "</div>" +
      '<div class="settings-mcp-actions">' + state + drift + toggles +
      '<button class="btn-secondary btn-sm settings-mcp-toggle" data-mcp-name="' + escapeHtml(server.name) +
      '" data-mcp-enabled="' + (server.enabled ? "true" : "false") + '">' +
      (server.enabled ? "Disable" : "Enable") + "</button>" +
      "</div></div>";
  }

  function decodeBase64(value) {
    try {
      var binary = window.atob(String(value || ""));
      var bytes = new Uint8Array(binary.length);
      for (var i = 0; i < binary.length; i += 1) { bytes[i] = binary.charCodeAt(i); }
      return bytes;
    } catch (error) {
      return null;
    }
  }

  function showPreview(serverName, index) {
    var url = endpoints.preview(serverName, index) + "?bot_id=" + encodeURIComponent(botId());
    fetch(url, { headers: headers({ Accept: "application/json" }) })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
      })
      .then(function (payload) {
        var data = payload && payload.data ? payload.data : payload;
        var bytes = decodeBase64(data && data.html_base64);
        if (!bytes) { throw new Error("Preview payload is not valid base64"); }
        var html = new TextDecoder("utf-8").decode(bytes);
        openHtmlPreview(html, (data && data.name) || "UI resource preview");
      })
      .catch(function (error) {
        status("MCP UI preview unavailable: " + error.message, true);
      });
  }

  /// Sandboxed HTML preview shared by external UI resources and the built-in
  /// `ui://generated/*` server: opaque iframe, strict CSP, no host access.
  function openHtmlPreview(html, label) {
    var blob = new Blob(["<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'\">" + html], { type: "text/html" });
    var objectUrl = URL.createObjectURL(blob);
    var overlay = document.createElement("div");
    overlay.className = "settings-mcp-preview-overlay";
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    overlay.setAttribute("aria-label", label || "UI resource preview");
    var panel = document.createElement("div");
    panel.className = "settings-mcp-preview-panel";
    var close = document.createElement("button");
    close.type = "button";
    close.textContent = "Close";
    close.className = "btn-secondary btn-sm";
    var frame = document.createElement("iframe");
    frame.className = "settings-mcp-preview-frame";
    frame.setAttribute("sandbox", "allow-scripts");
    frame.src = objectUrl;
    panel.appendChild(close);
    panel.appendChild(frame);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);
    var dismiss = function () {
      overlay.remove();
      URL.revokeObjectURL(objectUrl);
    };
    close.addEventListener("click", dismiss);
    overlay.addEventListener("click", function (event) {
      if (event.target === overlay) { dismiss(); }
    });
    close.focus();
  }

  /// The built-in MCP server (#1676) publishes generated apps as
  /// `ui://generated/{hash}` resources. Discovery lists them next to external
  /// servers so the same signed, sandboxed frame renders any MCP app.
  function loadGeneratedApps() {
    var list = document.getElementById("settings-mcp-list");
    if (!list) { return; }
    fetch(endpoints.apps, {
      method: "POST",
      headers: headers({ "Content-Type": "application/json", Accept: "application/json" }),
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "resources/list" }),
    })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
      })
      .then(function (payload) {
        var resources = (payload && payload.result && payload.result.resources) || [];
        var block = document.createElement("div");
        block.className = "settings-mcp-generated";
        var rows = resources.map(function (resource, index) {
          return '<div class="connection-item settings-mcp-row">' +
            '<div class="connection-info">' +
            '<span class="connection-name">' + escapeHtml(resource.name || resource.uri) +
            ' <span class="connection-status">generated app · ui://</span></span>' +
            '<span class="connection-status settings-mcp-desc">' + escapeHtml(resource.description || resource.uri) + "</span>" +
            "</div>" +
            '<div class="settings-mcp-actions">' +
            '<button class="btn-secondary btn-sm settings-mcp-app-preview" data-uri="' + escapeHtml(resource.uri) +
            '" data-name="' + escapeHtml(resource.name || resource.uri) + '" data-index="' + index + '">Preview</button>' +
            "</div></div>";
        }).join("");
        block.innerHTML =
          '<div class="settings-mcp-group-title">Generated apps (ui://)</div>' +
          (rows || '<div class="empty-state"><p>No generated apps yet — ask the assistant to build one.</p></div>');
        list.appendChild(block);
        block.querySelectorAll(".settings-mcp-app-preview").forEach(function (button) {
          button.addEventListener("click", function () {
            var uri = button.getAttribute("data-uri");
            var name = button.getAttribute("data-name");
            fetch(endpoints.apps, {
              method: "POST",
              headers: headers({ "Content-Type": "application/json", Accept: "application/json" }),
              body: JSON.stringify({ jsonrpc: "2.0", id: 2, method: "resources/read", params: { uri: uri } }),
            })
              .then(function (response) {
                if (!response.ok) { throw new Error("HTTP " + response.status); }
                return response.json();
              })
              .then(function (payload) {
                var contents = (payload && payload.result && payload.result.contents) || [];
                var html = contents.length ? contents[0].text : "";
                if (!html) { throw new Error("empty resource"); }
                openHtmlPreview(html, name);
              })
              .catch(function (error) {
                status("Generated app preview unavailable: " + error.message, true);
              });
          });
        });
      })
      .catch(function () {
        // A build without the built-in MCP server simply lists no apps.
      });
  }

  function status(message, isError) {
    var node = document.getElementById("settings-mcp-status");
    if (!node) { return; }
    node.textContent = message;
    node.className = "settings-mcp-status" + (isError ? " is-error" : "");
  }

  function render(servers) {
    var list = document.getElementById("settings-mcp-list");
    if (!list) { return; }
    list.setAttribute("aria-busy", "false");
    if (!Array.isArray(servers)) { return; }
    if (!servers.length) {
      list.innerHTML = '<div class="empty-state"><p>No MCP servers registered</p>' +
        '<p>MCP servers expose tools and UI resources to the agent over the Model Context Protocol.</p></div>';
      return;
    }
    list.innerHTML = servers.map(serverRow).join("");
  }

  function load() {
    var list = document.getElementById("settings-mcp-list");
    if (!list) { return; }
    list.setAttribute("aria-busy", "true");
    fetch(endpoints.list, { headers: headers({ Accept: "application/json" }) })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
      })
      .then(function (payload) {
        var servers = normalize(payload);
        status(servers.length + " MCP server" + (servers.length === 1 ? "" : "s"));
        render(servers);
        loadGeneratedApps();
      })
      .catch(function (error) {
        list.setAttribute("aria-busy", "false");
        list.innerHTML = '<div class="empty-state"><p>MCP servers unavailable</p>' +
          "<p>The sources service did not respond.</p></div>";
        status("Could not load MCP servers: " + error.message, true);
      });
  }

  function bind() {
    var list = document.getElementById("settings-mcp-list");
    if (!list || list.dataset.mcpBound === "1") { return; }
    list.dataset.mcpBound = "1";
    list.addEventListener("click", function (event) {
      var toggle = event.target.closest(".settings-mcp-toggle");
      if (toggle) {
        var name = toggle.getAttribute("data-mcp-name");
        var enabled = toggle.getAttribute("data-mcp-enabled") === "true";
        toggle.disabled = true;
        fetch(endpoints.action(name, enabled ? "disable" : "enable") + "?bot_id=" + encodeURIComponent(botId()), {
          method: "POST",
          headers: headers({ "Content-Type": "application/json" }),
          credentials: "same-origin",
          body: "{}"
        })
          .then(function (response) {
            if (!response.ok) { throw new Error("HTTP " + response.status); }
            status("MCP server " + (enabled ? "disabled" : "enabled"));
          })
          .catch(function (error) {
            status("MCP server update failed: " + error.message, true);
          })
          .then(load);
        return;
      }
      var preview = event.target.closest(".settings-mcp-preview");
      if (preview) {
        showPreview(preview.getAttribute("data-mcp-name"), preview.getAttribute("data-resource-index"));
      }
    });
  }

  function init() {
    var list = document.getElementById("settings-mcp-list");
    if (!list) { return; }
    bind();
    load();
  }

  document.addEventListener("DOMContentLoaded", init);
  if (document.readyState !== "loading") { init(); }

  ns.init = init;
  ns.load = load;
  ns.showPreview = showPreview;
}(window.GBSettingsMcp = window.GBSettingsMcp || {}));
