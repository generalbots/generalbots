"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.ai = {

  openDialog() {
    var dialog = document.getElementById("studio-ai-dialog");
    if (!dialog) return;
    var channel = document.getElementById("campaign-channel");
    var channelHint = document.getElementById("studio-ai-channel-hint");
    if (channelHint) {
      var ch = channel ? channel.value : "email";
      channelHint.textContent = "Channel: " + ch;
    }
    dialog.style.display = "flex";
  },

  closeDialog() {
    var dialog = document.getElementById("studio-ai-dialog");
    if (dialog) dialog.style.display = "none";
  },

  async generate() {
    var goal = document.getElementById("studio-ai-goal").value.trim();
    var audience = document.getElementById("studio-ai-audience").value.trim();
    if (!goal || !audience) {
      alert("Describe your goal and audience first.");
      return;
    }
    var tone = document.getElementById("studio-ai-tone").value;
    var length = document.getElementById("studio-ai-length").value;
    var channel = document.getElementById("campaign-channel").value;
    var status = document.getElementById("studio-ai-status");
    if (status) status.style.display = "block";

    try {
      var resp = await fetch("/api/crm/ai/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          channel: channel,
          goal: goal,
          audience_description: audience,
          tone: tone || null,
          length: length || null,
        }),
      });
      if (!resp.ok) throw new Error("AI generate failed: " + resp.status);
      var result = await resp.json();

      var subjectInput = document.getElementById("campaign-subject");
      if (subjectInput && result.subject) subjectInput.value = result.subject;

      window.CampStudio.editor.setContent(result.body || "");
      window.CampStudio.state.generatedVariations = result.variations || [];

      var varRow = document.getElementById("studio-ai-variations");
      if (varRow) {
        if (window.CampStudio.state.generatedVariations.length > 0) {
          varRow.style.display = "";
          varRow.innerHTML = "Variations: " + window.CampStudio.state.generatedVariations
            .map(function (v, i) {
              return '<button type="button" class="studio-var-btn" onclick="window.CampStudio.ai.applyVariation(' + i + ')">' + escapeHtml(v.name || ("V" + (i + 1))) + "</button>";
            }).join(" ");
        } else {
          varRow.style.display = "none";
        }
      }
      if (status) status.style.display = "none";
      this.closeDialog();
      alert("AI draft ready — check subject and body, then save.");
    } catch (err) {
      console.error("AI generate error:", err);
      if (status) { status.textContent = "Error: " + err.message; }
    }
  },

  applyVariation(index) {
    var variations = window.CampStudio.state.generatedVariations || [];
    var v = variations[index];
    if (!v) return;
    window.CampStudio.editor.setContent(v.body || "");
    var subjectInput = document.getElementById("campaign-subject");
    if (subjectInput && v.subject) subjectInput.value = v.subject;
    var varRow = document.getElementById("studio-ai-variations");
    if (varRow) varRow.style.display = "none";
  },

  bind() {
    var openBtn = document.getElementById("studio-ai-btn");
    if (openBtn) openBtn.addEventListener("click", function () { window.CampStudio.ai.openDialog(); });
    var closeBtn = document.getElementById("studio-ai-close");
    if (closeBtn) closeBtn.addEventListener("click", function () { window.CampStudio.ai.closeDialog(); });
    var genBtn = document.getElementById("studio-ai-generate");
    if (genBtn) genBtn.addEventListener("click", function () { window.CampStudio.ai.generate(); });
    var overlay = document.getElementById("studio-ai-dialog");
    if (overlay && overlay.querySelector(".crm-modal-overlay")) {
      overlay.querySelector(".crm-modal-overlay").addEventListener("click", function () { window.CampStudio.ai.closeDialog(); });
    }
  },
};

window.CampStudio.ai.bind();