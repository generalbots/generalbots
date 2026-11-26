"use strict";
/* Slides table & media authoring (#2487) — WebVTT captions, transcript panel,
 * accessible-export check and module bootstrap. */
(function () {
  var STM = window.SlidesTableMedia = window.SlidesTableMedia || {};

  function timestamp(seconds) {
    var ms = Math.max(0, Math.round((Number(seconds) || 0) * 1000));
    var hours = Math.floor(ms / 3600000);
    var minutes = Math.floor((ms % 3600000) / 60000);
    var secs = Math.floor((ms % 60000) / 1000);
    var millis = ms % 1000;
    return String(hours).padStart(2, "0") + ":" + String(minutes).padStart(2, "0") + ":" +
      String(secs).padStart(2, "0") + "." + String(millis).padStart(3, "0");
  }
  function parseStamp(value) {
    var parts = String(value || "").split(":");
    if (parts.length !== 3) return 0;
    var seconds = parseFloat(parts[2].replace(",", "."));
    return (parseInt(parts[0], 10) || 0) * 3600 + (parseInt(parts[1], 10) || 0) * 60 +
      (isFinite(seconds) ? seconds : 0);
  }

  STM.parseVtt = function (text) {
    var lines = String(text || "").replace(/\r\n?/g, "\n").split("\n");
    var cues = [];
    var skip = false;
    for (var i = 0; i < lines.length; i++) {
      var line = lines[i].trim();
      if (!line) continue;
      if (/^WEBVTT\b/.test(line)) { skip = false; continue; }
      if (/^(NOTE|STYLE|REGION)\b/.test(line)) { skip = true; continue; }
      if (line.indexOf("-->") === -1) {
        if (skip) continue;
        continue;
      }
      skip = false;
      var parts = line.split("-->");
      if (parts.length < 2) continue;
      var start = parseStamp(parts[0].trim());
      var endPart = parts[1].trim().split(/\s+/)[0];
      var end = parseStamp(endPart);
      var textLines = [];
      i += 1;
      while (i < lines.length && lines[i].trim()) {
        textLines.push(lines[i].trim());
        i += 1;
      }
      cues.push({ start: start, end: end, text: textLines.join(" ") });
    }
    return cues;
  };

  STM.captionCssText = function (css) {
    var safe = css || {};
    function color(value, fallback) { return /^#[0-9a-f]{3,8}$/i.test(value || "") ? value : fallback; }
    var size = parseInt(safe.size, 10) || 18;
    return "video::cue{color:" + color(safe.color, "#ffffff") +
      ";background-color:" + color(safe.background, "rgba(0,0,0,0.75)") +
      ";font-size:" + Math.max(10, Math.min(48, size)) + "px;}" +
      ".stm-transcript-cue{color:" + color(safe.color, "#f8fafc") +
      ";background:" + color(safe.background, "#1e293b") +
      ";font-size:" + Math.max(10, Math.min(48, size)) + "px;}";
  };

  STM.applyCaptionStyle = function (css) {
    var style = document.getElementById("stm-caption-css");
    if (!style) {
      style = document.createElement("style");
      style.id = "stm-caption-css";
      document.head.appendChild(style);
    }
    style.textContent = STM.captionCssText(css);
  };

  STM.openCaptions = function (id) {
    var item = STM.findMedia(id);
    if (!item) { STM.status("slides-stm-controls-select-media", "Select a media item on the slide first."); return; }
    var existing = STM.load();
    var vtt = existing.captions[id] || "";
    var css = Object.assign({ color: "#ffffff", background: "#1e293b", size: 18 },
      (existing.media[id] || {}).captionCss || {});
    var dialog = STM.openDialog({
      titleKey: "slides-stm-captions-title",
      titleFallback: "Captions and transcript",
      onBuild: function (api) {
        api.body.appendChild(api.localNote());
        var fileLabel = document.createElement("label");
        fileLabel.className = "stm-field-label";
        STM.labelled(fileLabel, "data-i18n", "slides-stm-captions-upload", "Upload WebVTT");
        var file = document.createElement("input");
        file.type = "file";
        file.accept=".vtt,text/vtt";
        fileLabel.appendChild(file);
        api.body.appendChild(fileLabel);
        file.addEventListener("change", function () {
          var chosen = file.files && file.files[0];
          if (!chosen) return;
          var reader = new FileReader();
          reader.onload = function () {
            var text = String(reader.result || "");
            if (!STM.parseVtt(text).length) {
              api.setStatus("slides-stm-captions-invalid", "Not a valid WebVTT file.");
              return;
            }
            area.value = text;
            render();
            api.setStatus("slides-stm-captions-loaded", "Loaded { $n } captions", { n: STM.parseVtt(text).length });
          };
          reader.readAsText(chosen);
        });

        var area = document.createElement("textarea");
        area.rows = 5;
        area.value = vtt;
        STM.labelled(area, "data-i18n-aria-label", "slides-stm-captions-text", "Caption text");
        STM.labelled(area, "data-i18n-placeholder", "slides-stm-captions-placeholder", "WEBVTT then timed cues");
        api.body.appendChild(area);

        var transcript = document.createElement("div");
        transcript.className = "stm-transcript";
        transcript.setAttribute("role", "log");
        transcript.setAttribute("aria-live", "polite");
        api.body.appendChild(transcript);
        function render() {
          transcript.innerHTML = "";
          var cues = STM.parseVtt(area.value);
          if (!cues.length) {
            var empty = document.createElement("p");
            STM.labelled(empty, "data-i18n", "slides-stm-captions-empty", "No captions loaded.");
            transcript.appendChild(empty);
            return;
          }
          cues.forEach(function (cue) {
            var row = document.createElement("div");
            row.className = "stm-transcript-cue";
            var time = document.createElement("span");
            time.className = "stm-cue-time";
            time.textContent = STM.formatTime(cue.start) + " – " + STM.formatTime(cue.end);
            var text = document.createElement("span");
            text.textContent = cue.text;
            row.appendChild(time);
            row.appendChild(text);
            transcript.appendChild(row);
          });
        }
        area.addEventListener("input", render);
        render();

        var styles = document.createElement("div");
        styles.className = "stm-caption-styles";
        api.body.appendChild(styles);
        [["color", "slides-stm-captions-style-color", "Text colour", "color"],
          ["background", "slides-stm-captions-style-bg", "Background", "color"],
          ["size", "slides-stm-captions-style-size", "Size", "number"]].forEach(function (entry) {
          var label = document.createElement("label");
          label.className = "stm-field-label";
          STM.labelled(label, "data-i18n", entry[1], entry[2]);
          var input = document.createElement("input");
          input.type = entry[3];
          if (entry[3] === "number") { input.min = "10"; input.max = "48"; }
          input.value = String(css[entry[0]]);
          input.addEventListener("change", function () {
            css[entry[0]] = entry[3] === "number" ? Number(input.value) : input.value;
            STM.applyCaptionStyle(css);
          });
          label.appendChild(input);
          styles.appendChild(label);
        });
        STM.applyCaptionStyle(css);

        api.addButton("slides-stm-cancel", "Cancel", function () { api.close(); });
        api.addButton("slides-stm-apply", "Apply", function () {
          var text = area.value;
          if (text.trim() && !STM.parseVtt(text).length) {
            api.setStatus("slides-stm-captions-invalid", "Not a valid WebVTT file.");
            return;
          }
          STM.patch(function (data) {
            data.captions[id] = text;
            data.media[id] = Object.assign({}, data.media[id], { captionCss: css, local: true });
          });
          api.close();
          STM.status("slides-stm-captions-saved", "Captions saved locally");
        }, "primary");
      }
    });
    return dialog;
  };

  STM.accessibilityGaps = function () {
    var data = STM.load();
    return STM.slideMedia.filter(function (item) {
      return item.media_type === "video" || item.media_type === "audio";
    }).filter(function (item) {
      var hasCaptions = !!(data.captions[item.id] || "").trim();
      var hasAlt = !!((data.media[item.id] || {}).alt || "").trim();
      return !hasCaptions || !hasAlt;
    });
  };

  STM.openAccessibleExportCheck = function () {
    var gaps = STM.accessibilityGaps();
    if (!gaps.length) {
      STM.status("slides-stm-export-ok", "All media has captions and descriptions.");
      return null;
    }
    return STM.openDialog({
      titleKey: "slides-stm-export-title",
      titleFallback: "Accessible export check",
      onBuild: function (api) {
        var summary = document.createElement("p");
        STM.labelled(summary, "data-i18n", "slides-stm-export-missing", "{ $n } media items are missing captions or alt text.");
        summary.setAttribute("data-i18n-params", JSON.stringify({ n: gaps.length }));
        summary.textContent = STM.tr("slides-stm-export-missing", "{ $n } media items are missing captions or alt text.", { n: gaps.length });
        api.body.appendChild(summary);
        var list = document.createElement("ul");
        gaps.forEach(function (item) {
          var row = document.createElement("li");
          row.textContent = (item.src || "").split("/").pop();
          list.appendChild(row);
        });
        api.body.appendChild(list);
        api.addButton("slides-stm-export-cancel", "Back to editing", function () { api.close(); });
        api.addButton("slides-stm-export-action", "Export anyway", function () {
          api.close();
          var button = document.getElementById("exportBtn");
          if (button) button.click();
        }, "primary");
      }
    });
  };

  STM.openHub = function () {
    return STM.openDialog({
      titleKey: "slides-stm-title",
      titleFallback: "Table and media",
      onBuild: function (api) {
        api.body.appendChild(api.localNote());
        api.addButton("slides-stm-table-title", "Table editing", function () {
          api.close(); STM.openTableEditor();
        });
        api.addButton("slides-stm-media-title", "Media browser", function () {
          api.close(); STM.openMediaBrowser();
        });
        api.addButton("slides-stm-captions-title", "Captions and transcript", function () {
          api.close();
          if (!STM.slideMedia.length) {
            STM.loadSlideMedia().then(function () {
              STM.openCaptions(STM.slideMedia[0] && STM.slideMedia[0].id);
            });
          } else {
            STM.openCaptions(STM.slideMedia[0] && STM.slideMedia[0].id);
          }
        });
        api.addButton("slides-stm-export-title", "Accessible export check", function () {
          api.close(); STM.openAccessibleExportCheck();
        });
        api.addButton("slides-stm-close", "Close", function () { api.close(); });
      }
    });
  };

  function isSlidesNode(node) {
    return !!(node && node.closest &&
      node.closest('#slides-app, [data-gb-app="slides"]'));
  }

  function slidesMounted() {
    return isSlidesNode(document.getElementById("slides-app"));
  }

  // #2947 — the body-level htmx:afterSwap listener outlives the Slides
  // window. Never issue GET /api/slides/media/list unless the Slides app
  // is actually mounted, so other apps do not log a 404 on open.
  function refreshAll() {
    if (!slidesMounted()) return;
    STM.tableElements().forEach(function (table) { STM.restoreTable(table); });
    STM.loadSlideMedia().then(function () { STM.renderMediaOverlay(); });
  }

  function bind() {
    var button = document.getElementById("slTableMediaBtn");
    if (button) button.addEventListener("click", function (event) {
      event.preventDefault();
      STM.openHub();
    });
    // #2947 — also gate on the swapped element: a lingering Slides window
    // must not fetch media for swaps that belong to another app, and the
    // global body listener must be installed only once across reopens.
    if (!STM.captionsSwapBound) {
      STM.captionsSwapBound = true;
      document.body.addEventListener("htmx:afterSwap", function (event) {
        var target = (event && event.detail && event.detail.target) ||
          (event && event.target);
        if (!isSlidesNode(target)) return;
        window.setTimeout(refreshAll, 0);
      });
    }
    refreshAll();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind);
  else bind();
})();
