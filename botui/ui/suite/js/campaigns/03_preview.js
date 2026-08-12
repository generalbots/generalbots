"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.preview = {
  presets: {
    email: { desktop: { width: 600 }, mobile: { width: 320 } },
    whatsapp: { width: 375, minHeight: 300 },
    instagram: { width: 320, height: 400, ratio: "4:5" },
    facebook: { width: 340, height: 190, ratio: "16:9" },
    sms: { width: 360, minHeight: 120 },
    multi: { width: 600, minHeight: 300 },
  },

  plainText: function (html) {
    var div = document.createElement("div");
    div.innerHTML = html || "";
    return div.textContent || "";
  },

  render: function () {
    var frame = document.getElementById("studio-preview-frame");
    if (!frame) return;
    var st = window.CampStudio.state;
    var html = st.content || "";
    switch (st.channel) {
      case "whatsapp": this.renderWhatsApp(frame, html); break;
      case "instagram": this.renderInstagram(frame, html); break;
      case "facebook": this.renderFacebook(frame, html); break;
      case "sms": this.renderSms(frame, html); break;
      default: this.renderEmail(frame, html, st.device); break;
    }
  },

  container: function (width, minHeight, extraClass) {
    var el = document.createElement("div");
    el.className = "studio-device " + (extraClass || "");
    el.style.width = width + "px";
    if (minHeight) el.style.minHeight = minHeight + "px";
    el.style.margin = "0 auto";
    return el;
  },

  renderEmail: function (frame, html, device) {
    var preset = this.presets.email[device] || this.presets.email.desktop;
    var el = this.container(preset.width, 300, "studio-device-email");
    var header = '<div style="background:#e2e8f0;color:#475569;font-size:11px;padding:6px 10px;font-family:sans-serif;" class="studio-email-address">no-reply@pragmatismo.com.br</div>';
    var body = '<div style="padding:16px;font-family:sans-serif;">' + (html || '<p style="color:#94a3b8;">Your message will appear here</p>') + "</div>";
    el.innerHTML = header + body;
    frame.innerHTML = "";
    frame.appendChild(el);
  },

  renderWhatsApp: function (frame, html) {
    var el = this.container(375, 300, "studio-device-whatsapp");
    el.innerHTML =
      '<div class="wa-chat">' +
      '<div class="wa-header">📱 WhatsApp</div>' +
      '<div class="wa-bubble">' +
      (html || '<p style="color:#94a3b8;margin:0;">Your message will appear here</p>') +
      '<div class="wa-time">' + new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) + "</div>" +
      "</div></div>";
    frame.innerHTML = "";
    frame.appendChild(el);
  },

  renderInstagram: function (frame, html) {
    var el = this.container(320, 400, "studio-device-instagram");
    var caption = this.plainText(html) || "Your caption will appear here";
    var img = window.CampStudio.media ? window.CampStudio.media.firstImage("instagram") : null;
    var imageHtml = img
      ? '<img src="' + img + '" alt="generated" style="width:100%;height:100%;object-fit:cover;">'
      : '📸<br><small>1080×1350</small>';
    el.innerHTML =
      '<div class="ig-post">' +
      '<div class="ig-head"><span class="ig-avatar"></span><span class="ig-user">@your.brand</span></div>' +
      '<div class="ig-image">' + imageHtml + "</div>" +
      '<div class="ig-caption"><span class="ig-user">@your.brand</span> ' + escapeHtml(caption).replace(/\n/g, "<br>") + "</div>" +
      "</div>";
    frame.innerHTML = "";
    frame.appendChild(el);
  },

  renderFacebook: function (frame, html) {
    var el = this.container(340, 260, "studio-device-facebook");
    var text = this.plainText(html) || "Your post will appear here";
    var img = window.CampStudio.media ? window.CampStudio.media.firstImage("facebook") : null;
    var imageHtml = img
      ? '<img src="' + img + '" alt="generated" style="width:100%;height:100%;object-fit:cover;">'
      : '📘<br><small>1200×630</small>';
    el.innerHTML =
      '<div class="fb-post">' +
      '<div class="ig-head"><span class="ig-avatar"></span><span class="ig-user">Your Page</span></div>' +
      '<div class="ig-image">' + imageHtml + "</div>" +
      '<div class="ig-caption">' + escapeHtml(text).replace(/\n/g, "<br>") + "</div>" +
      "</div>";
    frame.innerHTML = "";
    frame.appendChild(el);
  },

  renderSms: function (frame, html) {
    var el = this.container(360, 140, "studio-device-sms");
    var text = this.plainText(html);
    var count = text.length;
    var cls = count > 160 ? "studio-sms-over" : count > 140 ? "studio-sms-warn" : "";
    el.innerHTML =
      '<div class="sms-preview">' +
      '<div class="sms-bubble">' + escapeHtml(text || "Your SMS message will appear here") + "</div>" +
      '<div class="sms-counter ' + cls + '">' + count + " / 160 chars" + (count > 160 ? " — will be split" : "") + "</div>" +
      "</div>";
    frame.innerHTML = "";
    frame.appendChild(el);
  },

  init: function () {
    var self = this;
    window.CampStudio.events.on("content-changed", function () { self.render(); });
    window.CampStudio.events.on("channel-changed", function () { self.render(); });
    window.CampStudio.events.on("device-changed", function () { self.render(); });
  },
};

function escapeHtml(str) {
  var div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

window.CampStudio.preview.init();