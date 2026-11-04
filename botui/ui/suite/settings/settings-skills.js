"use strict";

/* #1681 — Agent Skills packages in Settings. Imports a SKILL.md document into
   the bot, lists installed packages and toggles them. Same API surface used by
   the prompt builder: /api/sources/skills. Packages are data; the imported
   instructions are advertised as guidance, never executed. */

(function (ns) {
  var BASE = "/api/sources/skills";

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
    if (extra) {
      Object.keys(extra).forEach(function (key) {
        base[key] = extra[key];
      });
    }
    return base;
  }

  function botId() {
    return window.__INITIAL_BOT_NAME__ || "default";
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function status(message, isError) {
    var node = document.getElementById("settings-skills-status");
    if (!node) return;
    node.textContent = message;
    node.className = "settings-mcp-status" + (isError ? " is-error" : "");
  }

  function render(skills) {
    var list = document.getElementById("settings-skills-list");
    if (!list) return;
    list.setAttribute("aria-busy", "false");
    if (!skills || !skills.length) {
      list.innerHTML =
        '<div class="empty-state"><p>No skills imported</p>' +
        "<p>Paste a SKILL.md package above to teach this bot a reusable procedure.</p></div>";
      return;
    }
    list.innerHTML = skills
      .map(function (skill) {
        var state = skill.enabled
          ? '<span class="settings-mcp-badge is-healthy">Enabled</span>'
          : '<span class="settings-mcp-badge is-error">Disabled</span>';
        return (
          '<div class="settings-mcp-row">' +
          '<div class="connection-info">' +
          '<span class="connection-name">' +
          escapeHtml(skill.name) +
          ' <span class="connection-status">v' +
          escapeHtml(skill.version || "?") +
          " · " +
          escapeHtml((skill.hash || "").slice(0, 10)) +
          "</span></span>" +
          '<span class="connection-status settings-mcp-desc">' +
          escapeHtml(skill.description || "") +
          "</span></div>" +
          '<div class="settings-mcp-actions">' +
          state +
          '<button class="btn-secondary btn-sm settings-skill-toggle" data-skill="' +
          escapeHtml(skill.id) +
          '" data-enabled="' +
          (skill.enabled ? "true" : "false") +
          '">' +
          (skill.enabled ? "Disable" : "Enable") +
          "</button>" +
          '<button class="btn-secondary btn-sm settings-skill-remove" data-skill="' +
          escapeHtml(skill.id) +
          '">Remove</button>' +
          "</div></div>"
        );
      })
      .join("");
  }

  function load() {
    var list = document.getElementById("settings-skills-list");
    if (!list) return;
    list.setAttribute("aria-busy", "true");
    fetch(BASE + "?bot_id=" + encodeURIComponent(botId()), {
      headers: headers({ Accept: "application/json" }),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (payload) {
        var skills = (payload && payload.skills) || [];
        status(skills.length + " skill" + (skills.length === 1 ? "" : "s") + " imported");
        render(skills);
      })
      .catch(function (error) {
        list.setAttribute("aria-busy", "false");
        list.innerHTML =
          '<div class="empty-state"><p>Skills unavailable</p><p>The sources service did not respond.</p></div>';
        status("Could not load skills: " + error.message, true);
      });
  }

  function importSkill() {
    var input = document.getElementById("settings-skills-input");
    if (!input) return;
    var content = String(input.value || "").trim();
    if (!content) {
      status("Paste a SKILL.md document first", true);
      return;
    }
    status("Importing…");
    fetch(BASE + "?bot_id=" + encodeURIComponent(botId()), {
      method: "POST",
      headers: headers({ "Content-Type": "application/json" }),
      body: JSON.stringify({ content: content }),
    })
      .then(function (response) {
        return response.json().then(function (payload) {
          if (!response.ok || !payload.ok) {
            throw new Error(payload.error || "HTTP " + response.status);
          }
          return payload;
        });
      })
      .then(function (payload) {
        input.value = "";
        status("Imported " + (payload.skill && payload.skill.name ? payload.skill.name : "skill"));
        load();
      })
      .catch(function (error) {
        status("Import failed: " + error.message, true);
      });
  }

  function bind() {
    var list = document.getElementById("settings-skills-list");
    var button = document.getElementById("settings-skills-import");
    if (button && button.dataset.bound !== "1") {
      button.dataset.bound = "1";
      button.addEventListener("click", importSkill);
    }
    if (!list || list.dataset.bound === "1") return;
    list.dataset.bound = "1";
    list.addEventListener("click", function (event) {
      var toggle = event.target.closest(".settings-skill-toggle");
      if (toggle) {
        var skill = toggle.getAttribute("data-skill");
        var enabled = toggle.getAttribute("data-enabled") === "true";
        toggle.disabled = true;
        fetch(
          BASE +
            "/" +
            encodeURIComponent(skill) +
            (enabled ? "/disable" : "/enable") +
            "?bot_id=" +
            encodeURIComponent(botId()),
          { method: "POST", headers: headers({ "Content-Type": "application/json" }), body: "{}" }
        )
          .then(load)
          .catch(function (error) {
            status("Skill update failed: " + error.message, true);
          });
        return;
      }
      var remove = event.target.closest(".settings-skill-remove");
      if (remove) {
        var id = remove.getAttribute("data-skill");
        if (!window.confirm("Remove skill " + id + "?")) return;
        fetch(
          BASE + "/" + encodeURIComponent(id) + "?bot_id=" + encodeURIComponent(botId()),
          { method: "DELETE", headers: headers() }
        )
          .then(load)
          .catch(function (error) {
            status("Skill removal failed: " + error.message, true);
          });
      }
    });
  }

  function init() {
    var list = document.getElementById("settings-skills-list");
    if (!list) return;
    bind();
    load();
  }

  document.addEventListener("DOMContentLoaded", init);
  if (document.readyState !== "loading") init();

  ns.init = init;
  ns.load = load;
})((window.GBSettingsSkills = window.GBSettingsSkills || {}));
