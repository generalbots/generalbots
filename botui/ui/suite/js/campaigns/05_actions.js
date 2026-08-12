"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.actions = {

  showModal(id) {
    var st = window.CampStudio.state;
    var modal = document.getElementById("campaign-modal");
    if (!modal) return;
    var title = document.getElementById("campaign-modal-title");
    st.editingId = id || null;
    if (title) title.textContent = id ? "Edit Campaign" : "Create Campaign";
    modal.style.display = "flex";
    document.body.style.overflow = "hidden";

    if (id) {
      window.CampStudio.api.get(decodeURIComponent(id)).then(function (c) {
        var nameInput = document.getElementById("campaign-name");
        var channelSelect = document.getElementById("campaign-channel");
        var budgetInput = document.getElementById("campaign-budget");
        var scheduleInput = document.getElementById("campaign-schedule");
        var subjectInput = document.getElementById("campaign-subject");
        var metrics = c.metrics || {};
        if (nameInput) nameInput.value = c.name || "";
        if (channelSelect) { channelSelect.value = c.campaign_type || c.channel || "email"; st.channel = channelSelect.value; }
        if (budgetInput) budgetInput.value = c.budget || "";
        if (scheduleInput) scheduleInput.value = c.scheduled_at && c.scheduled_at.length > 16 ? c.scheduled_at.slice(0, 16) : (c.starts_at && c.starts_at.length > 16 ? c.starts_at.slice(0, 16) : "");
        if (subjectInput) subjectInput.value = metrics.subject || "";
        window.CampStudio.editor.setContent(metrics.body || "");
        window.CampStudio.events.emit("channel-changed", st.channel);
      }).catch(function (err) {
        console.error("Failed to load campaign for edit:", err);
      });
    } else {
      var form = document.getElementById("campaign-form");
      if (form) form.reset();
      window.CampStudio.editor.setContent("");
      st.channel = "email";
      var ch = document.getElementById("campaign-channel");
      if (ch) ch.value = "email";
      window.CampStudio.events.emit("channel-changed", "email");
    }
  },

  hideModal() {
    var modal = document.getElementById("campaign-modal");
    if (!modal) return;
    modal.style.display = "none";
    document.body.style.overflow = "";
    var form = document.getElementById("campaign-form");
    if (form) form.reset();
    window.CampStudio.state.editingId = null;
  },

  async save(e) {
    if (e) e.preventDefault();
    var st = window.CampStudio.state;
    var name = document.getElementById("campaign-name").value.trim();
    var channel = document.getElementById("campaign-channel").value;
    var budget = document.getElementById("campaign-budget").value;
    var scheduled = document.getElementById("campaign-schedule").value;
    var subject = document.getElementById("campaign-subject").value.trim();
    var body = window.CampStudio.editor.getContent();

    var payload = {
      name: name,
      campaign_type: channel,
      metrics: { subject: subject, body: body },
    };
    if (budget) payload.budget = parseFloat(budget);
    if (scheduled) {
      var d = new Date(scheduled);
      payload.scheduled_at = d.toISOString();
    }

    try {
      if (st.editingId) {
        await window.CampStudio.api.update(decodeURIComponent(st.editingId), payload);
      } else {
        await window.CampStudio.api.create(payload);
      }
      this.hideModal();
      window.CampStudio.monitor.load();
    } catch (err) {
      alert("Failed to save campaign: " + err.message);
    }
  },

  async run(id) {
    if (!confirm("Run this campaign now?")) return;
    try {
      await window.CampStudio.api.update(decodeURIComponent(id), { status: "running" });
      alert("Campaign is now running!");
      window.CampStudio.monitor.load();
    } catch (err) {
      alert("Failed to run campaign: " + err.message);
    }
  },

  async remove(id) {
    if (!confirm("Delete this campaign permanently?")) return;
    try {
      await window.CampStudio.api.remove(decodeURIComponent(id));
      window.CampStudio.monitor.load();
    } catch (err) {
      alert("Failed to delete campaign: " + err.message);
    }
  },
};

// Backwards-compatible globals used by older inline handlers and desktop.html.
window.CampaignsAPI = window.CampStudio.api;
window.loadCampaigns = function () { window.CampStudio.monitor.load(); };
window.showCampaignModal = function (id) { window.CampStudio.actions.showModal(id); };
window.hideCampaignModal = function () { window.CampStudio.actions.hideModal(); };
window.editCampaign = function (id) { window.CampStudio.actions.showModal(id); };
window.runCampaign = function (id) { window.CampStudio.actions.run(id); };
window.deleteCampaign = function (id) { window.CampStudio.actions.remove(id); };