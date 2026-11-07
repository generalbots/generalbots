/* "What's New" window — desktop only.
 *
 * Opens once per version change, on the first launch after an update. The notes
 * are compiled into the running build (`whatsnew.md` → `/api/whats-new`), so the
 * window works offline and always matches the version actually installed. The
 * GitHub release body is never used here: it describes the latest release, not
 * the one on this machine.
 *
 * `whatsnew.md` is written by `scripts/whats_new.py` when a release is tagged.
 */
"use strict";

(function () {
  var SEEN_KEY = "gb-whats-new-seen";
  var STARTUP_DELAY = 2500;

  function isDesktop() {
    return !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
  }

  function storageGet(key) {
    try {
      return window.localStorage.getItem(key) || "";
    } catch (error) {
      return "";
    }
  }

  function storageSet(key, value) {
    try {
      window.localStorage.setItem(key, String(value));
    } catch (error) {
      /* private mode: show again next launch */
    }
  }

  function sectionForVersion(markdown, version) {
    var wanted = String(version || "").replace(/^v/, "");
    if (!wanted) return "";
    var lines = String(markdown || "").split(/\r?\n/);
    var body = [];
    var inside = false;
    for (var index = 0; index < lines.length; index++) {
      var heading = /^##\s+v?(\d+\.\d+\.\d+)\b/.exec(lines[index]);
      if (heading) {
        if (inside) break;
        inside = heading[1] === wanted;
        continue;
      }
      if (inside) body.push(lines[index]);
    }
    return body.join("\n").trim();
  }

  // The release-notes modal renders plain text, so turn the small markdown
  // subset used by whatsnew.md into something readable without an HTML parser.
  function toPlainText(markdown) {
    return String(markdown || "")
      .split(/\r?\n/)
      .map(function (line) {
        if (/^###\s+/.test(line)) return line.replace(/^###\s+/, "").toUpperCase();
        if (/^-\s+/.test(line)) return "• " + line.replace(/^-\s+/, "");
        return line;
      })
      .join("\n")
      .replace(/\n{3,}/g, "\n\n")
      .trim();
  }

  function show() {
    if (!isDesktop()) return Promise.resolve();
    var update = window.GBUpdate;
    if (!update || typeof update.showReleaseNotes !== "function") {
      return Promise.resolve();
    }
    return Promise.all([
      fetch("/api/product", { cache: "no-store" })
        .then(function (response) {
          return response.ok ? response.json() : null;
        })
        .catch(function () {
          return null;
        }),
      fetch("/api/whats-new", { cache: "no-store" })
        .then(function (response) {
          return response.ok ? response.text() : "";
        })
        .catch(function () {
          return "";
        }),
    ])
      .then(function (results) {
        var product = results[0] || {};
        var version = String(product.version || update.version || "").replace(/^v/, "");
        if (!version || version === "0.0.0") return null;
        if (storageGet(SEEN_KEY) === version) return null;
        storageSet(SEEN_KEY, version);
        var section = sectionForVersion(results[1], version);
        if (!section) return null;
        return update.showReleaseNotes({
          release_notes: toPlainText(section),
          latest_version: version,
        });
      })
      .catch(function (error) {
        console.warn("[whats-new] could not open the notes:", error);
        return null;
      });
  }

  function schedule() {
    window.setTimeout(show, STARTUP_DELAY);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", schedule);
  } else {
    schedule();
  }

  window.GBWhatsNew = { show: show, sectionForVersion: sectionForVersion };
})();
