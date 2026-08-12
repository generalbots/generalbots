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
        if (metrics.images && typeof metrics.images === "object") {
          window.CampStudio.state.channelImages = {};
          Object.keys(metrics.images).forEach(function (ch) {
            if (Array.isArray(metrics.images[ch])) {
              window.CampStudio.state.channelImages[ch] = metrics.images[ch].slice();
            }
          });
        }
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
    var st = window.CampStudio.state;
    if (st.channelImages) payload.metrics.images = st.channelImages;
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

  async openPublish(id) {
    var st = window.CampStudio.state;
    st.publishingId = id || null;
    var dialog = document.getElementById("studio-publish-dialog");
    var listSel = document.getElementById("studio-publish-list");
    var desc = document.getElementById("studio-publish-desc");
    if (!dialog || !listSel) return;

    try {
      var resp = await fetch("/api/crm/lists");
      if (!resp.ok) throw new Error("Failed to load lists: " + resp.status);
      var lists = await resp.json();
      listSel.innerHTML = lists.map(function (l) {
        return '<option value="' + l.id + '">' + escapeHtml(l.name) + " (" + (l.member_count || l.contact_count || 0) + ")</option>";
      }).join("") || '<option value="">No lists</option>';
    } catch (err) {
      listSel.innerHTML = '<option value="">Error loading lists</option>';
      console.error("openPublish lists error:", err);
    }

    var campaign = st.campaigns.find(function (c) { return c.id === decodeURIComponent(id); });
    var channel = campaign ? (campaign.campaign_type || campaign.channel || "email") : "email";
    if (desc) desc.textContent = "Fan-out of " + (campaign ? campaign.name : "campaign") + " (" + channel + ")";

    var chk = document.querySelectorAll('[data-publish-channel]');
    if (channel === "multi") {
      chk.forEach(function (x) { x.checked = true; });
    } else {
      chk.forEach(function (x) { x.checked = x.dataset.publishChannel === channel; });
    }

    dialog.style.display = "flex";
  },

  closePublish() {
    var dialog = document.getElementById("studio-publish-dialog");
    if (dialog) dialog.style.display = "none";
  },

  async doPublish() {
    var st = window.CampStudio.state;
    var listId = document.getElementById("studio-publish-list").value;
    if (!listId) { alert("Pick a recipient list."); return; }
    var channels = [];
    document.querySelectorAll('[data-publish-channel]:checked').forEach(function (x) {
      channels.push(x.dataset.publishChannel);
    });
    if (channels.length === 0) { alert("Pick at least one channel."); return; }

    var status = document.getElementById("studio-publish-status");
    if (status) { status.style.display = "block"; status.textContent = "Publishing…"; }
    try {
      var resp = await fetch("/api/crm/campaigns/" + encodeURIComponent(st.publishingId) + "/publish", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ list_id: listId, channels: channels }),
      });
      var data = await resp.json();
      if (!resp.ok) throw new Error(data.error || ("Publish failed: " + resp.status));
      if (status) status.style.display = "none";
      this.closePublish();
      alert("Published: " + data.sent + " sent, " + data.failed + " failed across " + channels.join(", ") + ".");
      window.CampStudio.monitor.load();
    } catch (err) {
      console.error("Publish error:", err);
      if (status) { status.style.display = "block"; status.textContent = "Error: " + err.message; }
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