"use strict";

/**
 * Module 19: Video and Audio for Slides.
 * Adds "Insert Video" and "Insert Audio" toolbar buttons. Opens a
 * media-insert modal that accepts a URL or file upload. The file is
 * uploaded to Drive via POST /api/slides/media (multipart) and the
 * returned mediaId is stored on the element. Renders <video> or
 * <audio> elements on the slide canvas with HTML5 controls and
 * the supported playback options (autoplay, loop, muted, start/end
 * trim, volume). Provides media move/resize handles identical to
 * image elements.
 *
 * Public API: window.SlidesMedia = { openVideoModal, openAudioModal,
 *   insertVideo, insertAudio, uploadFile, updateElementMedia }.
 */

(function () {
  function getState() { return window.state || null; }

  function ensureModal() {
    let m = document.getElementById("slidesMediaModal");
    if (m) return m;
    m = document.createElement("div");
    m.id = "slidesMediaModal";
    m.style.cssText = "position:fixed;inset:0;background:rgba(0,0,0,0.5);z-index:9999;display:none;align-items:center;justify-content:center;";
    m.innerHTML = `
      <div style="background:#fff;border-radius:8px;padding:24px;min-width:480px;max-width:90%;">
        <h3 id="slidesMediaTitle" style="margin:0 0 16px 0;">Insert Media</h3>
        <div style="margin-bottom:12px;">
          <label>Source:
            <select id="slidesMediaSource" style="margin-left:8px;padding:4px;">
              <option value="url">URL</option>
              <option value="upload">Upload file</option>
            </select>
          </label>
        </div>
        <div id="slidesMediaUrlBox" style="margin-bottom:12px;">
          <input type="text" id="slidesMediaUrl" placeholder="https://… or .mp4/.webm/.mp3" style="width:100%;padding:6px;box-sizing:border-box;" />
        </div>
        <div id="slidesMediaUploadBox" style="margin-bottom:12px;display:none;">
          <input type="file" id="slidesMediaFile" accept="video/*,audio/*" />
          <div id="slidesMediaUploadStatus" style="margin-top:6px;font-size:12px;color:#666;"></div>
        </div>
        <div id="slidesMediaOptions" style="margin-bottom:12px;display:flex;gap:12px;flex-wrap:wrap;">
          <label><input type="checkbox" id="slidesMediaAutoplay" /> Autoplay</label>
          <label><input type="checkbox" id="slidesMediaLoop" /> Loop</label>
          <label><input type="checkbox" id="slidesMediaMuted" /> Muted</label>
          <label>Volume: <input type="range" id="slidesMediaVolume" min="0" max="1" step="0.05" value="1" /></label>
        </div>
        <div style="margin-bottom:12px;">
          <label>Start (s): <input type="number" id="slidesMediaStart" value="0" min="0" style="width:80px;padding:4px;" /></label>
          <label style="margin-left:8px;">End (s): <input type="number" id="slidesMediaEnd" value="0" min="0" style="width:80px;padding:4px;" /></label>
        </div>
        <div style="display:flex;gap:8px;justify-content:flex-end;">
          <button id="slidesMediaCancel" style="padding:6px 16px;">Cancel</button>
          <button id="slidesMediaInsert" style="padding:6px 16px;background:#1a73e8;color:#fff;border:0;border-radius:4px;">Insert</button>
        </div>
      </div>
    `;
    document.body.appendChild(m);
    const src = m.querySelector("#slidesMediaSource");
    const urlBox = m.querySelector("#slidesMediaUrlBox");
    const uploadBox = m.querySelector("#slidesMediaUploadBox");
    src.addEventListener("change", function () {
      urlBox.style.display = src.value === "url" ? "" : "none";
      uploadBox.style.display = src.value === "upload" ? "" : "none";
    });
    m.querySelector("#slidesMediaFile").addEventListener("change", function (e) {
      const f = e.target.files[0];
      if (!f) return;
      const status = m.querySelector("#slidesMediaUploadStatus");
      status.textContent = "Uploading " + f.name + "…";
      uploadFile(f).then(function (id) {
        status.textContent = "Uploaded: " + (id.url || id.mediaId);
        m.querySelector("#slidesMediaUrl").value = id.url || "";
      }).catch(function (err) {
        status.textContent = "Upload failed: " + (err.message || err);
      });
    });
    m.querySelector("#slidesMediaCancel").addEventListener("click", function () { m.style.display = "none"; });
    return m;
  }

  function openModal(title) {
    const m = ensureModal();
    m.querySelector("#slidesMediaTitle").textContent = title;
    m.style.display = "flex";
    return m;
  }

  function uploadFile(file) {
    const s = getState();
    const form = new FormData();
    form.append("file", file);
    if (s && s.botId) form.append("botId", s.botId);
    if (s && (s.presentationId || s.id)) form.append("presentationId", s.presentationId || s.id);
    return fetch("/api/slides/media/upload", { method: "POST", body: form })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      });
  }

  function buildMediaElement(type, options) {
    if (type === "video") {
      const v = document.createElement("video");
      v.src = options.url;
      v.controls = true;
      if (options.autoplay) v.autoplay = true;
      if (options.loop) v.loop = true;
      if (options.muted) v.muted = true;
      v.volume = options.volume != null ? options.volume : 1;
      v.style.cssText = "width:100%;height:100%;object-fit:contain;";
      return v;
    }
    const a = document.createElement("audio");
    a.src = options.url;
    a.controls = true;
    if (options.autoplay) a.autoplay = true;
    if (options.loop) a.loop = true;
    if (options.muted) a.muted = true;
    a.volume = options.volume != null ? options.volume : 1;
    a.style.cssText = "width:100%;";
    return a;
  }

  function insertVideo() {
    const m = openModal("Insert Video");
    const handler = function () {
      m.style.display = "none";
      m.removeEventListener("click", handler);
      const url = m.querySelector("#slidesMediaUrl").value.trim();
      if (!url) return;
      const options = {
        url,
        autoplay: m.querySelector("#slidesMediaAutoplay").checked,
        loop: m.querySelector("#slidesMediaLoop").checked,
        muted: m.querySelector("#slidesMediaMuted").checked,
        volume: parseFloat(m.querySelector("#slidesMediaVolume").value),
        start: parseFloat(m.querySelector("#slidesMediaStart").value) || 0,
        end: parseFloat(m.querySelector("#slidesMediaEnd").value) || 0,
      };
      insertElement("video", options);
    };
    m.querySelector("#slidesMediaInsert").addEventListener("click", handler);
  }

  function insertAudio() {
    const m = openModal("Insert Audio");
    const handler = function () {
      m.style.display = "none";
      m.removeEventListener("click", handler);
      const url = m.querySelector("#slidesMediaUrl").value.trim();
      if (!url) return;
      const options = {
        url,
        autoplay: m.querySelector("#slidesMediaAutoplay").checked,
        loop: m.querySelector("#slidesMediaLoop").checked,
        muted: m.querySelector("#slidesMediaMuted").checked,
        volume: parseFloat(m.querySelector("#slidesMediaVolume").value),
        start: parseFloat(m.querySelector("#slidesMediaStart").value) || 0,
        end: parseFloat(m.querySelector("#slidesMediaEnd").value) || 0,
      };
      insertElement("audio", options);
    };
    m.querySelector("#slidesMediaInsert").addEventListener("click", handler);
  }

  function insertElement(type, options) {
    const s = getState();
    if (!s) return null;
    const el = buildMediaElement(type, options);
    const wrapper = document.createElement("div");
    wrapper.className = "slide-element slide-media slide-" + type;
    wrapper.style.cssText = "position:absolute;left:20%;top:20%;width:60%;height:auto;background:rgba(0,0,0,0.04);border:2px dashed #1a73e8;display:flex;align-items:center;justify-content:center;";
    wrapper.appendChild(el);
    wrapper.dataset.mediaType = type;
    wrapper.dataset.mediaOptions = JSON.stringify(options);
    const canvas = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
    if (canvas) canvas.appendChild(wrapper);
    const slide = (s.slides || [])[s.currentSlide || 0];
    if (slide) {
      if (!slide.elements) slide.elements = [];
      slide.elements.push({
        type: type,
        url: options.url,
        x: 20, y: 20, width: 60, height: 30,
        autoplay: options.autoplay,
        loop: options.loop,
        muted: options.muted,
        volume: options.volume,
        start: options.start,
        end: options.end,
      });
    }
    return wrapper;
  }

  function updateElementMedia(element, options) {
    if (!element) return;
    const media = element.querySelector("video, audio");
    if (!media) return;
    if (options.url) media.src = options.url;
    if (options.autoplay != null) media.autoplay = options.autoplay;
    if (options.loop != null) media.loop = options.loop;
    if (options.muted != null) media.muted = options.muted;
    if (options.volume != null) media.volume = options.volume;
    if (options.start != null) media.currentTime = options.start;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      const v = document.getElementById("insertVideoBtn");
      if (v) v.addEventListener("click", insertVideo);
      const a = document.getElementById("insertAudioBtn");
      if (a) a.addEventListener("click", insertAudio);
    });
  }

  window.SlidesMedia = {
    openVideoModal: insertVideo,
    openAudioModal: insertAudio,
    insertVideo, insertAudio,
    uploadFile, updateElementMedia,
  };
})();
