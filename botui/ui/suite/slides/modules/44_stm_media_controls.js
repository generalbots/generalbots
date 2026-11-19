"use strict";
/* Slides table & media authoring (#2487) — media overlay, trim/playback
 * controls, poster capture and present-mode preview. Playback flags persist via
 * the existing media/update API; poster, alt and caption metadata are local. */
(function () {
  var STM = window.SlidesTableMedia = window.SlidesTableMedia || {};
  STM.slideMedia = [];

  STM.loadSlideMedia = function () {
    return STM.getJson("/api/slides/media/list?presentation_id=" +
      encodeURIComponent(STM.deck()) + "&slide_index=" + encodeURIComponent(STM.slide()))
      .then(function (data) {
        STM.slideMedia = (data && data.media) || [];
        return STM.slideMedia;
      })
      .catch(function () { STM.slideMedia = []; return STM.slideMedia; });
  };

  STM.findMedia = function (id) {
    for (var i = 0; i < STM.slideMedia.length; i++) {
      if (STM.slideMedia[i].id === id) return STM.slideMedia[i];
    }
    return null;
  };

  function mediaNode(item) {
    var meta = STM.mediaMeta(item.id);
    var node;
    if (item.media_type === "video") {
      node = document.createElement("video");
      node.preload = "metadata";
      node.muted = true;
      if (meta.poster || item.poster) node.poster = meta.poster || item.poster;
    } else if (item.media_type === "audio") {
      node = document.createElement("audio");
      node.preload = "metadata";
      node.muted = true;
    } else {
      node = document.createElement("img");
      node.alt = meta.alt || "";
    }
    node.className = "stm-media-node";
    node.setAttribute("data-stm-media-id", item.id);
    node.src = item.src;
    return node;
  }

  STM.renderMediaOverlay = function () {
    var canvas = STM.canvas();
    if (!canvas) return;
    var existing = canvas.querySelector(".stm-media-layer");
    if (existing) existing.remove();
    if (!STM.slideMedia.length) return;
    var layer = document.createElement("div");
    layer.className = "stm-media-layer";
    STM.slideMedia.forEach(function (item) {
      var frame = document.createElement("div");
      frame.className = "stm-media-frame";
      frame.setAttribute("data-stm-media-id", item.id);
      frame.style.left = (item.x || 0) + "px";
      frame.style.top = (item.y || 0) + "px";
      frame.style.width = (item.width || 320) + "px";
      frame.style.height = (item.height || 220) + "px";
      frame.tabIndex = 0;
      frame.setAttribute("role", "button");
      STM.labelled(frame, "data-i18n-aria-label", "slides-stm-controls-title", "Playback and accessibility");
      frame.appendChild(mediaNode(item));
      var badge = document.createElement("span");
      badge.className = "stm-media-duration";
      badge.textContent = item.end_time && item.start_time ?
        STM.formatTime(item.end_time - item.start_time) : "";
      frame.appendChild(badge);
      frame.addEventListener("click", function (event) {
        event.stopPropagation();
        STM.openMediaControls(item.id);
      });
      frame.addEventListener("keydown", function (event) {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          STM.openMediaControls(item.id);
        }
      });
      layer.appendChild(frame);
    });
    canvas.appendChild(layer);
  };

  function field(labelKey, fallback, input) {
    var label = document.createElement("label");
    label.className = "stm-field-label";
    STM.labelled(label, "data-i18n", labelKey, fallback);
    label.appendChild(input);
    return label;
  }

  function checkbox(parent, key, fallback, checked, onChange) {
    var label = document.createElement("label");
    label.className = "stm-check";
    var input = document.createElement("input");
    input.type = "checkbox";
    input.checked = !!checked;
    input.addEventListener("change", function () { onChange(input.checked); });
    label.appendChild(input);
    label.appendChild(STM.labelled(document.createElement("span"), "data-i18n", key, fallback));
    parent.appendChild(label);
    return input;
  }

  function numberInput(value, key, fallback) {
    var input = document.createElement("input");
    input.type = "number";
    input.min = "0";
    input.step = "0.1";
    input.value = String(value === null || value === undefined ? 0 : value);
    STM.labelled(input, "data-i18n-aria-label", key, fallback);
    return input;
  }

  STM.capturePoster = function (video) {
    if (!video || !video.videoWidth) return "";
    try {
      var canvas = document.createElement("canvas");
      canvas.width = Math.min(640, video.videoWidth);
      canvas.height = Math.round(canvas.width * video.videoHeight / video.videoWidth);
      var context = canvas.getContext && canvas.getContext("2d");
      if (!context) return "";
      context.drawImage(video, 0, 0, canvas.width, canvas.height);
      return canvas.toDataURL("image/jpeg", 0.8);
    } catch (e) {
      return "";
    }
  };

  STM.openMediaControls = function (id) {
    var item = STM.findMedia(id);
    if (!item) { STM.status("slides-stm-controls-select-media", "Select a media item on the slide first."); return; }
    var meta = STM.mediaMeta(id);
    var draft = {
      start: item.start_time || 0,
      end: item.end_time || 0,
      autoplay: !!item.autoplay,
      loop: !!item.loop_playback,
      muted: item.muted !== false,
      volume: item.volume === null || item.volume === undefined ? 0.8 : item.volume,
      playOnClick: meta.playOnClick !== false,
      poster: meta.poster || item.poster || "",
      alt: meta.alt || ""
    };
    var dialog = STM.openDialog({
      titleKey: "slides-stm-controls-title",
      titleFallback: "Playback and accessibility",
      onBuild: function (api) {
        api.body.appendChild(api.localNote());
        var trimStart = numberInput(draft.start, "slides-stm-controls-trim-start", "Trim start (s)");
        var trimEnd = numberInput(draft.end, "slides-stm-controls-trim-end", "Trim end (s)");
        api.body.appendChild(field("slides-stm-controls-trim-start", "Trim start (s)", trimStart));
        api.body.appendChild(field("slides-stm-controls-trim-end", "Trim end (s)", trimEnd));

        if (item.media_type === "video") {
          var posterInput = document.createElement("input");
          posterInput.type = "url";
          posterInput.value = draft.poster || "";
          STM.labelled(posterInput, "data-i18n-placeholder", "slides-stm-controls-poster-placeholder", "https://example.com/poster.jpg");
          api.body.appendChild(field("slides-stm-controls-poster", "Poster frame URL", posterInput));
          api.addButton("slides-stm-controls-poster-pick", "Capture from video", function () {
            var node = document.createElement("video");
            node.src = item.src;
            node.muted = true;
            node.preload = "metadata";
            node.addEventListener("loadeddata", function () {
              try { node.currentTime = Math.max(0, Number(trimStart.value) || 0); } catch (e) { /* seek unsupported */ }
            });
            node.addEventListener("seeked", function () {
              var data = STM.capturePoster(node);
              if (data) {
                posterInput.value = data;
                draft.poster = data;
                api.setStatus("slides-stm-controls-poster-saved", "Poster frame captured (local only)");
              }
            });
          });
        }

        var flags = document.createElement("div");
        flags.className = "stm-check-row";
        api.body.appendChild(flags);
        var autoplay = checkbox(flags, "slides-stm-controls-autoplay", "Autoplay", draft.autoplay, function (value) {
          draft.autoplay = value;
          if (value && !draft.muted) {
            draft.muted = true;
            mute.checked = true;
            api.setStatus("slides-stm-controls-no-autoplay-sound", "Autoplay is off until the clip is muted.");
          }
        });
        var mute = checkbox(flags, "slides-stm-controls-mute", "Mute", draft.muted, function (value) { draft.muted = value; });
        checkbox(flags, "slides-stm-controls-loop", "Loop", draft.loop, function (value) { draft.loop = value; });
        checkbox(flags, "slides-stm-controls-play-on-click", "Play on click", draft.playOnClick, function (value) { draft.playOnClick = value; });

        var volume = document.createElement("input");
        volume.type = "range";
        volume.min = "0";
        volume.max = "1";
        volume.step = "0.05";
        volume.value = String(draft.volume);
        STM.labelled(volume, "data-i18n-aria-label", "slides-stm-controls-volume", "Volume");
        api.body.appendChild(field("slides-stm-controls-volume", "Volume", volume));

        var alt = document.createElement("textarea");
        alt.rows = 2;
        alt.value = draft.alt;
        STM.labelled(alt, "data-i18n-placeholder", "slides-stm-controls-alt-placeholder", "Describe the media for screen readers");
        STM.labelled(alt, "data-i18n-aria-label", "slides-stm-controls-alt", "Alt text / description");
        api.body.appendChild(field("slides-stm-controls-alt", "Alt text / description", alt));

        api.addButton("slides-stm-captions-title", "Captions and transcript", function () {
          STM.openCaptions(id);
        });
        api.addButton("slides-stm-preview-title", "Media preview", function () {
          STM.previewMedia(id);
        });
        api.addButton("slides-stm-cancel", "Cancel", function () { api.close(); });
        api.addButton("slides-stm-apply", "Apply", function () {
          var start = Number(trimStart.value) || 0;
          var end = Number(trimEnd.value) || 0;
          if (end > 0 && end <= start) {
            api.setStatus("slides-stm-controls-invalid-trim", "Trim end must be after trim start.");
            return;
          }
          var patch = {
            autoplay: !!draft.autoplay, loop_playback: !!draft.loop, muted: !!draft.muted,
            volume: Number(volume.value) || 0
          };
          if (start > 0) patch.start_time = start;
          if (end > 0) patch.end_time = end;
          api.close();
          STM.postJson("/api/slides/media/update", {
            presentation_id: STM.deck(), slide_index: STM.slide(), media_id: id,
            autoplay: patch.autoplay, loop_playback: patch.loop_playback,
            muted: patch.muted, volume: patch.volume,
            start_time: patch.start_time || null, end_time: patch.end_time || null
          }).then(function () {
            Object.assign(item, patch);
            STM.saveMediaMeta(id, {
              poster: draft.poster, alt: alt.value,
              playOnClick: draft.playOnClick, local: true
            });
            STM.status("slides-stm-controls-saved", "Playback settings saved");
            STM.refresh();
          }).catch(function () {
            STM.saveMediaMeta(id, {
              poster: draft.poster, alt: alt.value,
              playOnClick: draft.playOnClick, local: true
            });
            STM.status("slides-stm-status-failed", "Could not save; change kept locally.");
            STM.refresh();
          });
        }, "primary");
      }
    });
    return dialog;
  };

  STM.captionBlobUrl = function (text) {
    if (!text) return "";
    try {
      return URL.createObjectURL(new Blob([text], { type: "text/vtt" }));
    } catch (e) {
      return "";
    }
  };

  STM.previewMedia = function (id) {
    var item = STM.findMedia(id);
    if (!item || item.media_type === "image") {
      STM.status("slides-stm-preview-no-media", "No media to preview on this slide.");
      return;
    }
    var captions = (STM.load().captions[id] || "");
    var url = STM.captionBlobUrl(captions);
    var dialog = STM.openDialog({
      titleKey: "slides-stm-preview-title",
      titleFallback: "Media preview",
      onBuild: function (api) {
        var node = document.createElement(item.media_type === "audio" ? "audio" : "video");
        node.className = "stm-preview-player";
        node.controls = true;
        node.muted = true;
        node.preload = "metadata";
        var src = item.src;
        if (item.start_time || item.end_time) {
          src += (src.indexOf("?") === -1 ? "?" : "&") +
            "#t=" + (item.start_time || 0) + (item.end_time ? "," + item.end_time : "");
        }
        node.src = src;
        if (url) {
          var track = document.createElement("track");
          track.kind = "captions";
          track.srclang = "en";
          track.label = STM.tr("slides-stm-captions-title", "Captions and transcript");
          track.src = url;
          track.default = true;
          node.appendChild(track);
        }
        api.body.appendChild(node);
        var flags = STM.mediaMeta(id);
        var note = document.createElement("p");
        note.className = "stm-preview-note";
        note.textContent = STM.tr("slides-stm-preview-range", "Preview plays the trimmed range only.") +
          (item.start_time ? " " + STM.tr("slides-stm-preview-start", "Start") + ": " + STM.formatTime(item.start_time) : "") +
          (item.end_time ? " " + STM.tr("slides-stm-preview-end", "End") + ": " + STM.formatTime(item.end_time) : "");
        api.body.appendChild(note);
        if (flags.playOnClick !== false) {
          node.addEventListener("click", function () {
            if (node.paused) node.play().catch(function () { /* autoplay blocked */ });
          });
        }
        api.addButton("slides-stm-captions-toggle", "Show captions", function () {
          var tracks = node.textTracks || [];
          for (var i = 0; i < tracks.length; i++) tracks[i].mode = "showing";
          api.setStatus("slides-stm-captions-on", "Captions on");
        });
        api.addButton("slides-stm-close", "Close", function () { api.close(); });
      }
    });
    return dialog;
  };
})();
