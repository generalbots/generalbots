"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.monitor = {

  async load() {
    var st = window.CampStudio.state;
    try {
      st.campaigns = await window.CampStudio.api.list();
    } catch (e) {
      st.campaigns = [];
      console.error("Monitor: failed to list campaigns", e);
    }
    var ig = await window.CampStudio.api.listInstagram();
    st.igCampaigns = Array.isArray(ig.campaigns) ? ig.campaigns : [];
    this.render();
  },

  totals: function (rows) {
    var t = { sent: 0, delivered: 0, opened: 0, clicked: 0, replied: 0, failed: 0 };
    rows.forEach(function (r) {
      var m = r.metrics || {};
      t.sent += m.sent || 0;
      t.delivered += m.delivered || 0;
      t.opened += m.opened || 0;
      t.clicked += m.clicked || 0;
      t.replied += m.replied || 0;
      t.failed += m.failed || 0;
    });
    return t;
  },

  render() {
    var st = window.CampStudio.state;
    var tbody = document.getElementById("studio-monitor-body");
    var totalsEl = document.getElementById("studio-monitor-totals");
    if (!tbody) return;

    var rows = st.campaigns.map(function (c) {
      return { id: c.id, name: c.name, channel: c.campaign_type || c.channel || "email", status: c.status || "draft", metrics: c.metrics || {}, created_at: c.created_at };
    });
    st.igCampaigns.forEach(function (c) {
      rows.push({ id: c.id, name: c.title || c.name || "Instagram post", channel: "instagram", status: c.status || "draft", metrics: c.metrics || { sent: 1 }, created_at: c.created_at });
    });
    rows.sort(function (a, b) { return String(b.created_at || "").localeCompare(String(a.created_at || "")); });

    var filter = st.monitorFilter;
    var visible = filter === "all" ? rows : rows.filter(function (r) { return r.channel === filter; });

    if (visible.length === 0) {
      tbody.innerHTML = '<tr><td colspan="8" style="padding:40px;text-align:center;color:var(--text-secondary,#888);">No campaigns yet — click <strong>New Campaign</strong> to create your first one.</td></tr>';
      if (totalsEl) totalsEl.textContent = "";
      return;
    }

    tbody.innerHTML = visible.map(function (r) {
      var m = r.metrics;
      return '<tr data-id="' + encodeURIComponent(r.id) + '">' +
        '<td class="studio-m-name">' + escapeHtml(r.name || "Untitled") + "</td>" +
        '<td><span class="campaign-channel-tag">' + escapeHtml(r.channel) + "</span></td>" +
        '<td><span class="campaign-status ' + escapeHtml(String(r.status).toLowerCase()) + '">' + escapeHtml(r.status) + "</span></td>" +
        '<td>' + (m.sent || 0) + "</td>" +
        '<td>' + (m.delivered || 0) + "</td>" +
        '<td>' + (m.opened || 0) + "</td>" +
        '<td>' + (m.clicked || 0) + "</td>" +
        '<td>' + (m.replied || 0) + "</td>" +
        '<td class="studio-m-actions">' +
        '<button class="campaign-action-btn" onclick="window.CampStudio.actions.edit(\'' + encodeURIComponent(r.id) + '\')">Edit</button> ' +
        '<button class="campaign-action-btn primary" onclick="window.CampStudio.actions.run(\'' + encodeURIComponent(r.id) + '\')">Run</button> ' +
        '<button class="campaign-action-btn" onclick="window.CampStudio.actions.remove(\'' + encodeURIComponent(r.id) + '\')" style="color:var(--danger,#ef4444);">Delete</button>' +
        "</td></tr>";
    }).join("");

    var t = this.totals(visible);
    if (totalsEl) {
      totalsEl.textContent = visible.length + " campaigns · " +
        t.sent + " sent · " + t.delivered + " delivered · " +
        t.opened + " opened · " + t.clicked + " clicked · " + t.failed + " failed";
    }
  },

  bindFilters() {
    var self = this;
    document.querySelectorAll("[data-monitor-filter]").forEach(function (chip) {
      chip.addEventListener("click", function () {
        document.querySelectorAll("[data-monitor-filter]").forEach(function (x) { x.classList.remove("active"); });
        chip.classList.add("active");
        window.CampStudio.state.monitorFilter = chip.dataset.monitorFilter;
        self.render();
      });
    });
    var refresh = document.getElementById("studio-monitor-refresh");
    if (refresh) refresh.addEventListener("click", function () { self.load(); });
  },
};