"use strict";

function initCampStudio() {
  var st = window.CampStudio.state;

  var channelSelect = document.getElementById("campaign-channel");
  if (channelSelect) {
    channelSelect.addEventListener("change", function () {
      var socialBtn = document.getElementById("studio-social-btn");
      st.channel = channelSelect.value;
      if (socialBtn) socialBtn.style.display = st.channel === "instagram" || st.channel === "facebook" ? "" : "none";
      window.CampStudio.events.emit("channel-changed", st.channel);
    });
  }

  var deviceToggle = document.getElementById("studio-device-toggle");
  if (deviceToggle) {
    deviceToggle.addEventListener("click", function () {
      st.device = st.device === "desktop" ? "mobile" : "desktop";
      deviceToggle.textContent = st.device === "desktop" ? "🖥 Desktop" : "📱 Mobile";
      window.CampStudio.events.emit("device-changed", st.device);
    });
  }

  var socialBtn = document.getElementById("studio-social-btn");
  if (socialBtn) {
    socialBtn.addEventListener("click", function () {
      window.CampStudio.api.createInstagram(
        window.CampStudio.preview.plainText(st.content),
        1,
        null
      ).then(function () {
        alert("Instagram campaign scheduled (AI content pipeline).");
        window.CampStudio.monitor.load();
      }).catch(function (err) {
        alert("Instagram campaign failed: " + err.message);
      });
    });
  }

  var form = document.getElementById("campaign-form");
  if (form) {
    form.addEventListener("submit", function (e) { window.CampStudio.actions.save(e); });
  }

  var newBtn = document.getElementById("campaign-new-btn");
  if (newBtn) {
    newBtn.addEventListener("click", function () { window.CampStudio.actions.showModal(); });
  }

  window.CampStudio.monitor.bindFilters();
  window.CampStudio.events.emit("content-changed", st.content);
  window.CampStudio.monitor.load();
}

(function(){ var __cb = function () {
  initCampStudio();
  var campaignsView = document.getElementById("campaigns-view");
  if (campaignsView) {
    var observer = new MutationObserver(function () {
      if (campaignsView.classList.contains("active")) {
        window.CampStudio.monitor.load();
      }
    });
    observer.observe(campaignsView, { attributes: true, attributeFilter: ["class"] });
  }
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();