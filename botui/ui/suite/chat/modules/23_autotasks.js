"use strict";

/**
 * Conversation autotasks (#1613) — long-running work owned by a conversation.
 *
 * A conversation may own cron entries and persistent systemd services in the
 * user's agent workspace. The backend mirrors each one in
 * `conversation_autotasks`; this module surfaces that state:
 *
 *   - a green dot on the conversation tab while a job is active;
 *   - a distinct (deeper, pulsing) green for persistent systemd services;
 *   - a status line under the tab strip with the schedule and a cancel action.
 *
 * It is additive: when the endpoint is unavailable (older server, no agent
 * workspace) the module stays silent and never blocks the chat.
 */

(function () {
  if (window.GBAutotasks) return;

  var POLL_MS = 30000;
  var AT = (window.GBAutotasks = {
    state: { bySession: {}, timer: null, lastError: null },
  });

  function token() {
    return (
      (window.GBTabs && GBTabs.token && GBTabs.token()) ||
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  }

  function escapeHtml(value) {
    var host = document.createElement("div");
    host.textContent = value == null ? "" : String(value);
    return host.innerHTML;
  }

  function activeSessionId() {
    var tab = window.GBTabs && GBTabs.activeTab ? GBTabs.activeTab() : null;
    if (tab && tab.sessionId) return tab.sessionId;
    if (window.ChatState && ChatState.currentSessionId) return ChatState.currentSessionId;
    return null;
  }

  function statusHost() {
    var host = document.getElementById("gbAutotaskStatus");
    if (host) return host;
    var strip = document.getElementById("gbTabStrip");
    if (!strip || !strip.parentElement) return null;
    host = document.createElement("div");
    host.id = "gbAutotaskStatus";
    host.className = "gb-autotask-status";
    host.hidden = true;
    strip.parentElement.insertBefore(host, strip.nextSibling);
    return host;
  }

  function scheduleText(job) {
    if (job.schedule_human) return job.schedule_human;
    return job.kind === "systemd" ? "Persistent service" : job.schedule;
  }

  function decorateTabs() {
    var summaries = AT.state.bySession;
    document.querySelectorAll(".gb-tab").forEach(function (el) {
      el.classList.remove("has-autotask", "has-autotask-persistent");
      el.removeAttribute("data-autotask-title");
    });
    var tabs = window.GBTabs && GBTabs.state ? GBTabs.state.tabs || [] : [];
    tabs.forEach(function (tab) {
      if (!tab.sessionId) return;
      var summary = summaries[tab.sessionId];
      if (!summary || !summary.has_work) return;
      var el = document.querySelector('.gb-tab[data-tab-id="' + tab.id + '"]');
      if (!el) return;
      el.classList.add("has-autotask");
      if (summary.persistent > 0) el.classList.add("has-autotask-persistent");
      var label = summary.autotasks
        .filter(function (job) {
          return job.status === "running" || job.status === "scheduled";
        })
        .map(function (job) {
          return job.title + " — " + scheduleText(job);
        })
        .join("; ");
      el.setAttribute("data-autotask-title", label);
      el.title = label;
    });
  }

  function renderStatus(sessionId) {
    var host = statusHost();
    if (!host) return;
    var summary = AT.state.bySession[sessionId];
    if (!summary || !summary.has_work) {
      host.hidden = true;
      host.innerHTML = "";
      return;
    }
    var jobs = summary.autotasks.filter(function (job) {
      return job.status === "running" || job.status === "scheduled";
    });
    if (!jobs.length) {
      host.hidden = true;
      host.innerHTML = "";
      return;
    }
    var persistent = summary.persistent > 0;
    host.hidden = false;
    host.className = "gb-autotask-status" + (persistent ? " persistent" : "");
    host.innerHTML =
      '<span class="gb-autotask-dot" aria-hidden="true"></span>' +
      '<span class="gb-autotask-text">' +
      escapeHtml(
        jobs.length === 1
          ? jobs[0].title + " — " + scheduleText(jobs[0])
          : jobs.length + " background jobs running",
      ) +
      "</span>" +
      jobs
        .map(function (job) {
          return (
            '<button type="button" class="gb-autotask-cancel" data-autotask-id="' +
            escapeHtml(job.id) +
            '" title="Stop ' +
            escapeHtml(job.title) +
            '">Stop</button>'
          );
        })
        .join("");
  }

  function apply(sessionId, payload) {
    if (!sessionId) return;
    AT.state.bySession[sessionId] = payload;
    decorateTabs();
    if (sessionId === activeSessionId()) renderStatus(sessionId);
  }

  async function refresh(sessionId) {
    if (!sessionId) return;
    try {
      var response = await fetch("/api/conversations/" + encodeURIComponent(sessionId) + "/autotasks", {
        headers: { Authorization: "Bearer " + token() },
      });
      if (!response.ok) throw new Error("HTTP " + response.status);
      apply(sessionId, await response.json());
      AT.state.lastError = null;
    } catch (error) {
      // Silent by design: the indicator must never break the conversation.
      AT.state.lastError = error && error.message ? error.message : "unavailable";
    }
  }

  async function stop(id) {
    try {
      var response = await fetch("/api/conversation-autotasks/" + encodeURIComponent(id) + "/cancel", {
        method: "POST",
        headers: { Authorization: "Bearer " + token() },
      });
      if (!response.ok) throw new Error("HTTP " + response.status);
      var sessionId = activeSessionId();
      await refresh(sessionId);
      if (window.GBToasts) {
        window.GBToasts.show("Background job stopped", "The schedule was removed from the workspace.", "success");
      }
    } catch (error) {
      if (window.GBToasts) {
        window.GBToasts.show(
          "Could not stop the job",
          "The workspace did not confirm the cancellation. Try again.",
          "error",
        );
      }
    }
  }

  function poll() {
    refresh(activeSessionId());
  }

  function start() {
    if (AT.state.timer) return;
    AT.state.timer = setInterval(poll, POLL_MS);
  }

  document.addEventListener("click", function (event) {
    var button = event.target.closest ? event.target.closest("[data-autotask-id]") : null;
    if (!button) return;
    event.preventDefault();
    stop(button.getAttribute("data-autotask-id"));
  });

  // Tab switches and new sessions change which conversation is visible.
  document.addEventListener("gb-tab-activated", poll);
  document.addEventListener("gb-session-changed", poll);
  document.addEventListener("gb-chat-ready", function () {
    poll();
    start();
  });
  document.addEventListener("DOMContentLoaded", function () {
    setTimeout(poll, 1500);
    start();
  });

  AT.refresh = refresh;
  AT.stop = stop;
})();
