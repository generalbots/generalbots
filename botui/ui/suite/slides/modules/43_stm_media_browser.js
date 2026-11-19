"use strict";
/* Slides table & media authoring (#2487) — media browser: Drive, upload and
 * URL insertion with thumbnail and duration read-out. */
(function () {
  var STM = window.SlidesTableMedia = window.SlidesTableMedia || {};
  var VIDEO_EXT = /\.(mp4|webm|mov|m4v|ogv)$/i;
  var AUDIO_EXT = /\.(mp3|wav|ogg|m4a|aac|flac)$/i;
  var IMAGE_EXT = /\.(png|jpe?g|gif|webp|svg|avif|bmp)$/i;

  STM.mediaKind = function (src, explicit) {
    if (explicit === "image" || explicit === "video" || explicit === "audio") return explicit;
    var clean = String(src || "").split("?")[0].split("#")[0];
    if (VIDEO_EXT.test(clean)) return "video";
    if (AUDIO_EXT.test(clean)) return "audio";
    if (IMAGE_EXT.test(clean)) return "image";
    return "image";
  };

  STM.formatTime = function (seconds) {
    var total = Math.max(0, Math.floor(Number(seconds) || 0));
    var minutes = Math.floor(total / 60);
    return minutes + ":" + String(total % 60).padStart(2, "0");
  };

  function probeTimeout(ms) {
    return new Promise(function (resolve) { window.setTimeout(function () { resolve({}); }, ms); });
  }

  STM.probeMedia = function (src, kind) {
    if (!src) return Promise.resolve({});
    if (kind === "image") {
      return new Promise(function (resolve) {
        var image = new Image();
        var done = false;
        var finish = function (data) { if (!done) { done = true; resolve(data || {}); } };
        image.onload = function () { finish({ width: image.naturalWidth || 0, height: image.naturalHeight || 0, thumb: src }); };
        image.onerror = function () { finish({ thumb: src }); };
        image.src = src;
        probeTimeout(6000).then(function () { finish({ thumb: src }); });
      });
    }
    return new Promise(function (resolve) {
      var media = document.createElement(kind === "audio" ? "audio" : "video");
      var done = false;
      var finish = function (data) {
        if (done) return;
        done = true;
        var result = data || {};
        result.duration = result.duration || (isFinite(media.duration) ? media.duration : 0);
        result.thumb = src;
        try { media.removeAttribute("src"); media.load(); } catch (e) { /* best effort */ }
        resolve(result);
      };
      media.preload = "metadata";
      media.muted = true;
      media.addEventListener("loadedmetadata", function () { finish({ duration: media.duration }); });
      media.addEventListener("error", function () { finish({}); });
      media.src = src;
      probeTimeout(6000).then(function () { finish({ duration: media.duration }); });
    });
  };

  function fileItems(payload) {
    if (Array.isArray(payload)) return payload;
    if (payload && Array.isArray(payload.files)) return payload.files;
    if (payload && Array.isArray(payload.items)) return payload.items;
    return [];
  }

  STM.listDriveMedia = function () {
    var query = "?scope=user";
    try {
      var bucket = window.sessionStorage.getItem("drive-bucket") || "";
      if (bucket) query += "&bucket=" + encodeURIComponent(bucket);
    } catch (e) { /* storage disabled */ }
    return STM.getJson("/api/files/list" + query).then(function (data) {
      return fileItems(data).filter(function (item) {
        var name = item.name || item.path || "";
        return VIDEO_EXT.test(name) || AUDIO_EXT.test(name) || IMAGE_EXT.test(name);
      }).map(function (item) {
        return {
          name: item.name || String(item.path || "").split("/").pop(),
          path: item.path || item.name || "",
          bucket: item.bucket || "",
          kind: STM.mediaKind(item.path || item.name || ""),
          raw: item
        };
      });
    });
  };

  STM.resolveDrive = function (path) {
    if (!path) return Promise.resolve("");
    if (/^https?:\/\//i.test(path)) return Promise.resolve(path);
    if (window.SlidesInsertPlace && typeof window.SlidesInsertPlace.resolveDrive === "function") {
      return window.SlidesInsertPlace.resolveDrive(path);
    }
    var scope = "user";
    try { scope = window.sessionStorage.getItem("drive-scope") || "user"; } catch (e) { /* storage disabled */ }
    return STM.postJson("/api/files/share-link", { bucket: "", path: path, scope: scope })
      .then(function (data) {
        var url = String((data && data.url) || "");
        return /^https?:\/\//i.test(url) ? url : (url ? window.location.origin + url : "");
      });
  };

  function readAsBase64(file) {
    return new Promise(function (resolve, reject) {
      var reader = new FileReader();
      reader.onload = function () { resolve(String(reader.result || "").split(",").pop()); };
      reader.onerror = function () { reject(new Error("read failed")); };
      reader.readAsDataURL(file);
    });
  }

  STM.uploadMedia = function (file) {
    if (!file) return Promise.reject(new Error("no file"));
    var scope = "user";
    try { scope = window.sessionStorage.getItem("drive-scope") || "user"; } catch (e) { /* storage disabled */ }
    var path = "slides-media/" + Date.now() + "-" + String(file.name || "media").replace(/[^A-Za-z0-9._-]/g, "_");
    return readAsBase64(file).then(function (content) {
      return STM.postJson("/api/files/write", { bucket: "", path: path, content: content, scope: scope });
    }).then(function () { return STM.resolveDrive(path); });
  };

  STM.insertMedia = function (spec) {
    var kind = STM.mediaKind(spec.src, spec.kind);
    var id = STM.mediaId(spec.src, spec.id);
    var media = {
      id: id,
      media_type: kind,
      src: spec.src,
      poster: spec.poster || null,
      autoplay: false,
      loop_playback: !!(spec.loop_playback),
      muted: true,
      controls: true,
      start_time: typeof spec.start_time === "number" ? spec.start_time : null,
      end_time: typeof spec.end_time === "number" ? spec.end_time : null,
      volume: typeof spec.volume === "number" ? spec.volume : 0.8,
      x: 120, y: 120, width: kind === "audio" ? 420 : 480, height: kind === "audio" ? 90 : 270
    };
    return STM.postJson("/api/slides/media", {
      presentation_id: STM.deck(), slide_index: STM.slide(), media: media
    }).then(function (response) {
      var saved = (response && response.media) || media;
      STM.saveMediaMeta(saved.id || id, {
        alt: spec.alt || "", captions: "", captionCss: {}, local: true
      });
      STM.status("slides-stm-media-inserted", "Media inserted");
      STM.refresh();
      return saved;
    });
  };

  function thumbnailFor(item) {
    if (item.kind === "image") {
      var img = document.createElement("img");
      img.className = "stm-thumb";
      img.alt = "";
      img.src = item.src;
      return img;
    }
    if (item.kind === "video") {
      var video = document.createElement("video");
      video.className = "stm-thumb";
      video.muted = true;
      video.preload = "metadata";
      video.setAttribute("aria-hidden", "true");
      video.src = item.src;
      return video;
    }
    var icon = document.createElement("span");
    icon.className = "stm-thumb stm-thumb-audio";
    icon.textContent = "\u266b";
    icon.setAttribute("aria-hidden", "true");
    return icon;
  }

  function addResultCard(container, item, onPick) {
    var card = document.createElement("div");
    card.className = "stm-media-card";
    card.setAttribute("role", "button");
    card.tabIndex = 0;
    var thumb = document.createElement("div");
    thumb.className = "stm-thumb-wrap";
    thumb.appendChild(thumbnailFor(item));
    card.appendChild(thumb);
    var name = document.createElement("span");
    name.className = "stm-card-name";
    name.textContent = item.name || "";
    card.appendChild(name);
    var meta = document.createElement("span");
    meta.className = "stm-card-meta";
    meta.textContent = item.duration ? STM.formatTime(item.duration) : "";
    card.appendChild(meta);
    function pick() { onPick(item, card); }
    card.addEventListener("click", pick);
    card.addEventListener("keydown", function (event) {
      if (event.key === "Enter" || event.key === " ") { event.preventDefault(); pick(); }
    });
    container.appendChild(card);
    return card;
  }

  STM.openMediaBrowser = function () {
    var selected = null;
    var dialog = STM.openDialog({
      titleKey: "slides-stm-media-title",
      titleFallback: "Media browser",
      onBuild: function (api) {
        var tabs = document.createElement("div");
        tabs.className = "stm-tabs";
        api.body.appendChild(tabs);
        var pages = {};
        [["drive", "slides-stm-media-tab-drive", "Drive"],
          ["upload", "slides-stm-media-tab-upload", "Upload"],
          ["url", "slides-stm-media-tab-url", "URL"]].forEach(function (entry, index) {
          var tab = STM.button(entry[1], entry[2], function () {
            Object.keys(pages).forEach(function (key) { pages[key].hidden = key !== entry[0]; });
            Array.prototype.slice.call(tabs.children).forEach(function (node) {
              node.setAttribute("aria-selected", node === tab ? "true" : "false");
            });
          });
          tab.setAttribute("role", "tab");
          tab.setAttribute("aria-selected", index === 0 ? "true" : "false");
          tabs.appendChild(tab);
          var page = document.createElement("div");
          page.className = "stm-tab-page";
          page.hidden = index !== 0;
          api.body.appendChild(page);
          pages[entry[0]] = page;
        });

        var list = document.createElement("div");
        list.className = "stm-media-grid";
        pages.drive.appendChild(list);
        var search = document.createElement("input");
        search.type = "search";
        STM.labelled(search, "data-i18n-placeholder", "slides-stm-media-search-placeholder", "Filter by name");
        STM.labelled(search, "data-i18n-aria-label", "slides-stm-media-search", "Search media");
        pages.drive.insertBefore(search, list);

        function choose(item, card) {
          selected = item;
          Array.prototype.slice.call(list.children).forEach(function (node) { node.classList.remove("stm-card-active"); });
          if (card) card.classList.add("stm-card-active");
          api.setStatus("slides-stm-media-selected", "Media selected");
          insertBtn.disabled = false;
        }
        search.addEventListener("input", function () {
          var term = search.value.toLowerCase();
          Array.prototype.slice.call(list.children).forEach(function (card) {
            card.hidden = card.textContent.toLowerCase().indexOf(term) === -1;
          });
        });
        api.setStatus("slides-stm-media-loading", "Loading media…");
        STM.listDriveMedia().then(function (items) {
          list.innerHTML = "";
          if (!items.length) {
            var empty = document.createElement("p");
            STM.labelled(empty, "data-i18n", "slides-stm-media-empty", "No media files found.");
            list.appendChild(empty);
            api.setStatus("slides-stm-media-empty", "No media files found.");
            return;
          }
          items.forEach(function (item) {
            STM.resolveDrive(item.path).then(function (url) {
              item.src = url;
              var card = addResultCard(list, item, choose);
              STM.probeMedia(url, item.kind).then(function (info) {
                item.duration = info.duration;
                card.querySelector(".stm-card-meta").textContent = info.duration ? STM.formatTime(info.duration) : "";
              });
            }).catch(function () { /* skip unshareable file */ });
          });
          api.setStatus("slides-stm-media-ready", "Choose a media item.");
        }).catch(function () {
          list.innerHTML = "";
          var empty = document.createElement("p");
          STM.labelled(empty, "data-i18n", "slides-stm-media-empty", "No media files found.");
          list.appendChild(empty);
          api.setStatus("slides-stm-media-empty", "No media files found.");
        });

        var fileLabel = document.createElement("label");
        fileLabel.className = "stm-field-label";
        STM.labelled(fileLabel, "data-i18n", "slides-stm-media-upload-label", "Choose a file");
        var fileInput = document.createElement("input");
        fileInput.type = "file";
        fileInput.accept = "image/*,video/*,audio/*";
        fileLabel.appendChild(fileInput);
        pages.upload.appendChild(fileLabel);
        var uploadPreview = document.createElement("div");
        uploadPreview.className = "stm-media-grid";
        pages.upload.appendChild(uploadPreview);
        fileInput.addEventListener("change", function () {
          var file = fileInput.files && fileInput.files[0];
          if (!file) return;
          api.setStatus("slides-stm-media-loading", "Loading media…");
          var localUrl = URL.createObjectURL(file);
          uploadPreview.innerHTML = "";
          var local = { name: file.name, src: localUrl, kind: STM.mediaKind(file.name, file.type) };
          var card = addResultCard(uploadPreview, local, function () {
            STM.uploadMedia(file).then(function (url) {
              selected = { name: file.name, src: url, kind: STM.mediaKind(file.name, file.type) };
              insertBtn.disabled = false;
              api.setStatus("slides-stm-media-selected", "Media selected");
            }).catch(function () {
              api.setStatus("slides-stm-status-failed", "Could not save; change kept locally.");
            });
          });
          STM.probeMedia(localUrl, local.kind).then(function (info) {
            local.duration = info.duration;
            card.querySelector(".stm-card-meta").textContent = info.duration ? STM.formatTime(info.duration) : "";
          });
        });

        var urlLabel = document.createElement("label");
        urlLabel.className = "stm-field-label";
        STM.labelled(urlLabel, "data-i18n", "slides-stm-media-url-label", "Media URL");
        var urlInput = document.createElement("input");
        urlInput.type = "url";
        STM.labelled(urlInput, "data-i18n-placeholder", "slides-stm-media-url-placeholder", "https://example.com/video.mp4");
        urlLabel.appendChild(urlInput);
        pages.url.appendChild(urlLabel);
        urlInput.addEventListener("change", function () {
          if (!urlInput.value.trim()) return;
          selected = { name: urlInput.value.split("/").pop(), src: urlInput.value.trim(), kind: STM.mediaKind(urlInput.value) };
          insertBtn.disabled = false;
          api.setStatus("slides-stm-media-selected", "Media selected");
          STM.probeMedia(selected.src, selected.kind).then(function (info) {
            selected.duration = info.duration;
          });
        });

        var insertBtn = api.addButton("slides-stm-insert", "Insert", function () {
          if (!selected) return;
          var item = selected;
          api.close();
          STM.insertMedia(item).catch(function () {
            STM.status("slides-stm-status-failed", "Could not save; change kept locally.");
          });
        }, "primary");
        insertBtn.disabled = true;
        api.addButton("slides-stm-cancel", "Cancel", function () { api.close(); });
      }
    });
    return dialog;
  };
})();
