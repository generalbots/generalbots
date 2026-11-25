"use strict";

/**
 * import-security-notices.js — issue #2522.
 *
 * Renders the machine-readable `security_notices` array returned by the Office
 * import endpoints (Docs, Sheets, Slides) into a single accessible status
 * region. Only catalogue keys that already ship in every locale are rendered;
 * unknown keys are dropped rather than shown as raw identifiers. Notice data
 * is never trusted as markup — every line is written with textContent.
 */
(function (global) {
  var doc = global.document;
  if (!doc) return;

  var REGION_ID = "gb-import-security-notices";
  var IMPORT_PATH = /\/api\/(?:docs|sheet|slides)\/import(?:[?#]|$)/;
  var MAX_NOTICES = 8;
  var INSTALLED_FLAG = "__gbImportSecurityNoticesInstalled";

  // Keys already present in botlib/locales/{en,pt-BR,es,fr,de,ja,ko,zh-CN}/ui-common.ftl.
  // The authored English fallback is only used when the catalogue is missing.
  var MESSAGES = {
    "file-macros-removed": {
      key: "file-macros-removed",
      fallback: "Macros were removed before import ({ $count } executable part(s))."
    }
  };

  function messageFor(notice) {
    if (!notice || typeof notice !== "object") return "";
    var spec = MESSAGES[notice.message_key] || MESSAGES[notice.code];
    if (!spec) return "";
    var count = typeof notice.count === "number" && notice.count >= 0
      ? Math.floor(notice.count)
      : 0;
    var text = spec.fallback;
    if (typeof global.gbT === "function") {
      try {
        text = global.gbT(spec.key, spec.fallback, { count: count });
      } catch (e) {
        text = spec.fallback;
      }
    }
    // gbT substitutes Fluent parameters, but keep a manual fallback for hosts
    // where only the authored English string is available.
    return String(text)
      .replace("{ $count }", String(count))
      .replace("{$count}", String(count));
  }

  function statusRegion() {
    var el = doc.getElementById(REGION_ID);
    if (!el) {
      el = doc.createElement("div");
      el.id = REGION_ID;
      el.setAttribute("role", "status");
      el.setAttribute("aria-live", "polite");
      el.setAttribute("aria-atomic", "true");
      el.style.position = "fixed";
      el.style.right = "16px";
      el.style.bottom = "16px";
      el.style.zIndex = "var(--z-toast)";
      el.style.maxWidth = "360px";
      el.style.pointerEvents = "none";
      (doc.body || doc.documentElement).appendChild(el);
    }
    return el;
  }

  /**
   * Writes notices into `target` (or the shared status region). Returns the
   * number of lines actually written. Safe to call with any value.
   */
  function render(notices, target) {
    if (!Array.isArray(notices) || notices.length === 0) return 0;
    var host = target || statusRegion();
    host.setAttribute("role", "status");
    host.setAttribute("aria-live", "polite");
    var written = 0;
    for (var i = 0; i < notices.length && written < MAX_NOTICES; i++) {
      var message = messageFor(notices[i]);
      if (!message) continue;
      var line = doc.createElement("div");
      line.className = "gb-import-security-notice";
      line.textContent = message;
      host.appendChild(line);
      written++;
    }
    return written;
  }

  var installed = false;

  /** Wraps fetch once so every Office import response is inspected. */
  function install() {
    if (installed || global[INSTALLED_FLAG] || typeof global.fetch !== "function") return;
    var previous = global.fetch;
    global.fetch = function (input) {
      var url = typeof input === "string" ? input : (input && input.url) || "";
      var promise = previous.apply(global, arguments);
      if (!IMPORT_PATH.test(url)) return promise;
      return promise.then(function (response) {
        try {
          if (response && response.ok && typeof response.clone === "function") {
            response.clone().json().then(function (data) {
              if (data && Array.isArray(data.security_notices)) {
                render(data.security_notices);
              }
            }).catch(function () {
              /* non-JSON body: nothing to announce */
            });
          }
        } catch (e) {
          /* never alter the caller's response */
        }
        return response;
      });
    };
    installed = true;
    global[INSTALLED_FLAG] = true;
    global.GBImportSecurityNotices.installed = true;
  }

  global.GBImportSecurityNotices = {
    render: render,
    install: install,
    installed: global[INSTALLED_FLAG] === true
  };
  install();
})(window);
