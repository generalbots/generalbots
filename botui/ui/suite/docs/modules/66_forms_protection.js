"use strict";
/* Document protection preview (#2427): a visible banner and local edit gating
 * that allows only form controls and comment regions while enabled. The
 * password is never persisted — only a "set" marker is kept locally. No
 * server-enforced permission is implied. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var P = (NS.protection = NS.protection || {});
  var BANNER_ID = "docs-forms-protect-banner";
  var COMMENT_SELECTOR = "[data-gb-comment],.gb-comment,.gb-review-balloon";

  P.enabled = function () { return NS.state.protect.enabled === true; };

  P.enable = function (password) {
    NS.state.protect = { enabled: true, passwordPreview: password ? "set" : "" };
    NS.persist();
    P.applyBanner();
  };

  P.disable = function () {
    NS.state.protect = { enabled: false, passwordPreview: "" };
    NS.persist();
    P.applyBanner();
  };

  P.applyBanner = function () {
    var existing = document.getElementById ? document.getElementById(BANNER_ID) : null;
    if (existing && existing.parentNode) existing.parentNode.removeChild(existing);
    var host = NS.getArticle();
    if (!P.enabled() || !host || !host.parentNode || !host.parentNode.insertBefore) return false;
    var banner = document.createElement("div");
    banner.id = BANNER_ID;
    banner.className = "gb-forms-protect-banner";
    banner.setAttribute("role", "status");
    banner.setAttribute("aria-live", "polite");
    banner.textContent = NS.tr("docs-forms-protect-banner",
      "Protected (preview) — editing limited to forms and comments");
    host.parentNode.insertBefore(banner, host);
    return true;
  };

  /* Pure gating contract for the offline harness. */
  P.shouldBlock = function (enabled, context) {
    var info = context || {};
    return enabled === true && !(info.isForm === true || info.isComment === true);
  };

  function contextOf(target) {
    if (!target || typeof target.closest !== "function") return { isForm: false, isComment: false };
    return {
      isForm: !!target.closest("[data-gb-form-control],[data-gb-forms-signature]"),
      isComment: !!target.closest(COMMENT_SELECTOR)
    };
  }
  P.contextOf = contextOf;

  function guard(event) {
    if (!P.enabled()) return;
    var info = contextOf(event && event.target);
    if (!P.shouldBlock(true, info)) return;
    if (event.preventDefault) event.preventDefault();
    NS.statusKey("docs-forms-protect-blocked",
      "Document is protected — only forms and comments can be edited");
  }
  P.guard = guard;

  P.install = function () {
    if (!document.addEventListener) return;
    document.addEventListener("beforeinput", guard, true);
    document.addEventListener("paste", guard, true);
  };
})(window);
