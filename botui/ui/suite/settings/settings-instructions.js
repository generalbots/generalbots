"use strict";

// Settings → Custom instructions (#2568). Per-user instructions injected into
// the system prompt; the same text the assistant shell edits, through
// GET/PUT /api/settings/custom-instructions. Not admin-gated: every user owns
// their own instructions, so the API scopes them by the authenticated user.
(function () {
  var MAX = 1500;
  var state = { data: { about: "", response: "", enabled: true }, busy: false };

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function token() {
    if (typeof window.getGBAccessToken === "function") {
      var value = window.getGBAccessToken();
      if (value) return value;
    }
    return (
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  }

  function json(path, options) {
    var headers = { "Content-Type": "application/json" };
    var value = token();
    if (value) headers.Authorization = "Bearer " + value;
    return fetch(path, Object.assign({ headers: headers }, options || {})).then(function (response) {
      return response.json().catch(function () {
        return {};
      }).then(function (body) {
        if (!response.ok) {
          var err = new Error(body.error || "HTTP " + response.status);
          err.status = response.status;
          throw err;
        }
        return body;
      });
    });
  }

  function say(message, isError) {
    var node = document.getElementById("instr-status");
    if (!node) return;
    node.setAttribute("role", isError ? "alert" : "status");
    node.setAttribute("aria-live", isError ? "assertive" : "polite");
    node.textContent = message || "";
  }

  function setCounter(id, value) {
    var node = document.getElementById(id);
    if (node) node.textContent = value.length + " / " + MAX;
  }

  function paint() {
    var about = document.getElementById("instr-about");
    var response = document.getElementById("instr-response");
    var enabled = document.getElementById("instr-enabled");
    var fields = document.getElementById("instr-fields");
    if (about) about.value = state.data.about || "";
    if (response) response.value = state.data.response || "";
    if (enabled) enabled.checked = state.data.enabled !== false;
    if (fields) {
      fields.style.opacity = state.data.enabled === false ? "0.5" : "";
      fields.style.pointerEvents = state.data.enabled === false ? "none" : "";
    }
    setCounter("instr-about-count", state.data.about || "");
    setCounter("instr-response-count", state.data.response || "");
  }

  function save() {
    if (state.busy) return;
    var about = document.getElementById("instr-about");
    var response = document.getElementById("instr-response");
    var enabled = document.getElementById("instr-enabled");
    var button = document.getElementById("instr-save");
    state.busy = true;
    if (button) button.disabled = true;
    say(t("instr-saving", "Saving…"));
    json("/api/settings/custom-instructions", {
      method: "PUT",
      body: JSON.stringify({
        about: about ? about.value : state.data.about,
        response: response ? response.value : state.data.response,
        enabled: enabled ? enabled.checked : state.data.enabled
      })
    })
      .then(function () {
        say(t("instr-saved", "Saved."));
      })
      .catch(function (error) {
        say(t("instr-save-failed", "Could not save: %e").replace("%e", error.message || String(error)), true);
      })
      .then(function () {
        state.busy = false;
        if (button) button.disabled = false;
      });
  }

  function bind() {
    var about = document.getElementById("instr-about");
    var response = document.getElementById("instr-response");
    var enabled = document.getElementById("instr-enabled");
    var button = document.getElementById("instr-save");
    if (about && about.dataset.bound) return;
    if (about) about.dataset.bound = "1";
    if (about) {
      about.addEventListener("input", function () {
        setCounter("instr-about-count", about.value);
      });
    }
    if (response) {
      response.addEventListener("input", function () {
        setCounter("instr-response-count", response.value);
      });
    }
    if (enabled) {
      enabled.addEventListener("change", function () {
        state.data.enabled = enabled.checked;
        paint();
        save();
      });
    }
    if (button) button.addEventListener("click", save);
  }

  function load() {
    if (!document.getElementById("instructions-section")) return Promise.resolve();
    say(t("instr-loading", "Loading…"));
    return json("/api/settings/custom-instructions")
      .then(function (data) {
        state.data = {
          about: typeof data.about === "string" ? data.about : "",
          response: typeof data.response === "string" ? data.response : "",
          enabled: data.enabled !== false
        };
        paint();
        bind();
        say("");
      })
      .catch(function (error) {
        paint();
        bind();
        if (error && error.status === 404) {
          say(t("instr-unavailable", "This server does not expose custom instructions."), true);
        } else {
          say(t("instr-load-failed", "Could not load: %e").replace("%e", error.message || String(error)), true);
        }
      });
  }

  window.loadInstructionsPanel = load;

  var previousHook = window.GBSettingsOnSection;
  window.GBSettingsOnSection = function (section) {
    if (typeof previousHook === "function") {
      try {
        previousHook(section);
      } catch (e) {
        /* an earlier hook is optional */
      }
    }
    if (section === "instructions") window.loadInstructionsPanel();
  };

  if (window.location.hash === "#instructions") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", load);
    } else {
      load();
    }
  }
})();
