"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.media = {

  currentChannel() {
    var ch = document.getElementById("campaign-channel");
    return ch ? ch.value : "email";
  },

  imagesFor(channel) {
    var st = window.CampStudio.state;
    if (!st.channelImages) st.channelImages = {};
    if (!st.channelImages[channel]) st.channelImages[channel] = [];
    return st.channelImages[channel];
  },

  setImages(channel, urls) {
    var st = window.CampStudio.state;
    if (!st.channelImages) st.channelImages = {};
    st.channelImages[channel] = urls || [];
  },

  firstImage(channel) {
    var imgs = this.imagesFor(channel);
    return imgs.length > 0 ? imgs[0] : null;
  },

  openDialog() {
    var dialog = document.getElementById("studio-img-dialog");
    if (!dialog) return;
    var hint = document.getElementById("studio-img-hint");
    if (hint) hint.textContent = "Will generate: " + this.currentChannel() + " visual";
    dialog.style.display = "flex";
  },

  closeDialog() {
    var dialog = document.getElementById("studio-img-dialog");
    if (dialog) dialog.style.display = "none";
  },

  async generate() {
    var prompt = document.getElementById("studio-img-prompt").value.trim();
    if (!prompt) {
      alert("Describe the image first.");
      return;
    }
    var status = document.getElementById("studio-img-status");
    var dialog = document.getElementById("studio-img-dialog");
    if (status) { status.style.display = "block"; status.textContent = "Generating image via BotModels…"; }

    var channel = this.currentChannel();
    var size = document.getElementById("studio-img-size").value;
    var style = document.getElementById("studio-img-style").value;
    var fullPrompt = prompt + (size !== "auto" ? " (" + size + ")" : "") + (style !== "none" ? ", " + style + " style" : "");

    try {
      var resp = await fetch("/api/crm/ai/image", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ prompt: fullPrompt }),
      });
      if (resp.status === 400) {
        var hint = document.getElementById("studio-img-hint");
        if (hint) {
          hint.textContent = "Image AI not configured — enable BotModels in the bot config (botmodels-enabled=true, botmodels-host, botmodels-port, botmodels-api-key).";
          hint.style.display = "block";
        }
        if (status) status.style.display = "none";
        return;
      }
      if (!resp.ok) throw new Error("Image generation failed: " + resp.status);
      var data = await resp.json();

      var imgs = this.imagesFor(channel);
      imgs.push(data.url);
      this.setImages(channel, imgs);
      this.renderStrip();
      window.CampStudio.events.emit("channel-changed", channel);
      dialog.style.display = "none";
      document.getElementById("studio-img-prompt").value = "";
      if (status) status.style.display = "none";
      alert("Image generated — it is applied to the " + channel + " preview automatically.");
    } catch (err) {
      console.error("Image generate error:", err);
      if (status) { status.style.display = "block"; status.textContent = "Error: " + err.message; }
    }
  },

  useInEmail(url) {
    var editor = window.CampStudio.editor;
    var html = editor.editEl.innerHTML;
    editor.setContent(html + '<div style="margin:12px 0;"><img src="' + url + '" alt="Campaign image" style="max-width:100%;border-radius:8px;"></div>');
  },

  regenerate(url) {
    var prompt = prompt("New prompt for this image (regenerate)?", "");
    if (!prompt) return;
    var channel = this.currentChannel();
    var status = document.getElementById("studio-img-status");
    var dialog = document.getElementById("studio-img-dialog");
    if (status) status.style.display = "block";

    fetch("/api/crm/ai/image", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ prompt: prompt }),
    }).then(function (resp) {
      if (!resp.ok) throw new Error("Image regeneration failed: " + resp.status);
      return resp.json();
    }).then(function (data) {
      var imgs = window.CampStudio.media.imagesFor(channel);
      var idx = imgs.indexOf(url);
      if (idx >= 0) imgs[idx] = data.url;
      window.CampStudio.media.setImages(channel, imgs);
      window.CampStudio.media.renderStrip();
      window.CampStudio.events.emit("channel-changed", channel);
      if (status) status.style.display = "none";
    }).catch(function (err) {
      console.error("Regenerate error:", err);
      if (status) { status.style.display = "block"; status.textContent = "Error: " + err.message; }
    });
  },

  remove(url) {
    var channel = this.currentChannel();
    var imgs = this.imagesFor(channel).filter(function (u) { return u !== url; });
    this.setImages(channel, imgs);
    this.renderStrip();
    window.CampStudio.events.emit("channel-changed", channel);
  },

  renderStrip() {
    var strip = document.getElementById("studio-media-strip");
    if (!strip) return;
    var channel = this.currentChannel();
    var imgs = this.imagesFor(channel);

    if (imgs.length === 0) {
      strip.innerHTML = '<span class="studio-media-empty">No images yet — click 🖼 AI Image.</span>';
      return;
    }
    strip.innerHTML = imgs.map(function (u, i) {
      return '<div class="studio-media-item">' +
        '<img src="' + u + '" alt="generated" loading="lazy">' +
        '<div class="studio-media-actions">' +
        '<button type="button" class="studio-var-btn" title="Use in email body" onclick="window.CampStudio.media.useInEmail(\'' + u + '\')">📧</button>' +
        '<button type="button" class="studio-var-btn" title="Regenerate" onclick="window.CampStudio.media.regenerate(\'' + u + '\')">🔄</button>' +
        '<button type="button" class="studio-var-btn" title="Remove" onclick="window.CampStudio.media.remove(\'' + u + '\')">✕</button>' +
        "</div></div>";
    }).join("");
  },

  bind() {
    var openBtn = document.getElementById("studio-img-btn");
    if (openBtn) openBtn.addEventListener("click", function () { window.CampStudio.media.openDialog(); });
    var closeBtn = document.getElementById("studio-img-close");
    if (closeBtn) closeBtn.addEventListener("click", function () { window.CampStudio.media.closeDialog(); });
    var genBtn = document.getElementById("studio-img-generate");
    if (genBtn) genBtn.addEventListener("click", function () { window.CampStudio.media.generate(); });
    var overlay = document.getElementById("studio-img-dialog");
    var ov = overlay && overlay.querySelector(".crm-modal-overlay");
    if (ov) ov.addEventListener("click", function () { window.CampStudio.media.closeDialog(); });

    window.CampStudio.events.on("channel-changed", function () { window.CampStudio.media.renderStrip(); });
  },
};

window.CampStudio.media.bind();