/* Mail app — split from mail/mail.js for #1731 */

  function initMail() {
    initFolderHandlers();

    document.body.removeEventListener("change", handleCheckboxChange);
    document.body.addEventListener("change", handleCheckboxChange);

    var unifiedItem = document.querySelector('.nav-item[data-folder="unified"]');
    if (unifiedItem && typeof htmx !== "undefined") {
      htmx.trigger(unifiedItem, "click");
    }
  }

  window.openCompose = openCompose;
  window.closeCompose = closeCompose;
  window.minimizeCompose = minimizeCompose;
  window.toggleCcBcc = toggleCcBcc;
  window.toggleScheduleMenu = toggleScheduleMenu;
  window.scheduleSend = scheduleSend;
  window.openCustomSchedule = openCustomSchedule;
  window.closeScheduleModal = closeScheduleModal;
  window.confirmSchedule = confirmSchedule;
  window.prepareSubmit = prepareSubmit;
  window.formatText = formatText;
  window.openTemplates = openTemplates;
  window.closeTemplates = closeTemplates;
  window.openSignatures = openSignatures;
  window.closeSignatures = closeSignatures;
  window.openRules = openRules;
  window.closeRules = closeRules;
  window.openAutoResponder = openAutoResponder;
  window.closeAutoResponder = closeAutoResponder;
  window.saveAutoResponder = saveAutoResponder;
  window.openLabelManager = openLabelManager;
  window.toggleSelectAll = toggleSelectAll;
  window.updateBulkActions = updateBulkActions;
  window.refreshMailList = refreshMailList;
  window.insertSignature = insertSignature;
  window.showTemplateSelector = showTemplateSelector;
  window.attachFile = attachFile;
  window.insertLink = insertLink;
  window.insertImage = insertImage;
  window.saveDraft = saveDraft;
  window.createNewTemplate = createNewTemplate;
  window.createNewSignature = createNewSignature;
  window.createNewRule = createNewRule;
  window.archiveSelected = archiveSelected;
  window.markAsRead = markAsRead;
  window.addLabelToSelected = addLabelToSelected;
  window.deleteSelected = deleteSelected;
  window.openAddAccount = openAddAccount;
  window.closeAddAccount = closeAddAccount;
  window.saveAccount = saveAccount;
  window.toggleSearchFilters = toggleSearchFilters;
  window.applySearchFilters = applySearchFilters;
  window.loadUnifiedInbox = loadUnifiedInbox;
  window.searchAllAccounts = searchAllAccounts;
  window.handleSearchInput = handleSearchInput;
  window.saveLabel = saveLabel;
  window.saveSignature = saveSignature;
  window.saveRule = saveRule;
  window.saveTemplate = saveTemplate;

  function snoozeEmail(emailId, preset) {
    fetch("/api/email/snooze", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email_ids: [emailId], preset: preset })
    })
      .then(function (r) { return r.json(); })
      .then(function (res) {
        if (res && res.snoozed_count !== undefined) {
          if (typeof window.showNotification === "function") {
            window.showNotification("Snoozed until " + gbFormatDateTime(new Date(res.snooze_until)), "success");
          }
        } else {
          throw new Error("Snooze failed");
        }
      })
      .catch(function (err) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Snooze error: " + err.message, "error");
        }
      });
  }

  function loadSmartReplies(emailId) {
    var container = document.getElementById("smart-reply-" + emailId);
    if (!container) return;
    fetch("/api/ai/generate-reply", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email_id: emailId })
    })
      .then(function (r) { return r.json(); })
      .then(function (res) {
        if (res && Array.isArray(res.suggestions) && res.suggestions.length) {
          var chips = res.suggestions.map(function (s) {
            return '<button class="smart-reply-chip" onclick="useSmartReply(\'' +
              s.replace(/'/g, "\\'") + '\')">' + s + '</button>';
          }).join("");
          container.innerHTML =
            '<div class="smart-reply-label">Smart replies</div>' + chips;
        }
      })
      .catch(function (err) {
        /* Best-effort smart-reply suggestions; compose remains fully usable. */
        console.debug("[gb-mail] Smart reply fetch failed:", err);
      });
  }

  function useSmartReply(text) {
    var body = document.getElementById("compose-body");
    if (body) {
      body.textContent = text;
      body.focus();
    }
    if (typeof openCompose === "function") {
      openCompose();
    } else {
      var modal = document.getElementById("compose-modal");
      if (modal && modal.showModal) modal.showModal();
    }
  }

  function loadNudgesForEmail(emailId, userId) {
    var banner = document.getElementById("nudges-banner-" + emailId);
    if (!banner) return;
    fetch("/api/email/nudges", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user_id: userId || "00000000-0000-0000-0000-000000000000" })
    })
      .then(function (r) { return r.json(); })
      .then(function (res) {
        if (res && Array.isArray(res.nudges)) {
          var mine = res.nudges.filter(function (n) { return n.email_id === emailId; });
          if (mine.length) {
            banner.innerHTML = mine.map(function (n) {
              return '<div class="nudge-item">No reply sent for ' + n.days_ago +
                ' day(s) — <button onclick="dismissNudge(\'' + n.email_id + '\')">Dismiss</button></div>';
            }).join("");
          }
        }
      })
      .catch(function (err) {
        /* Best-effort nudge suggestions; inbox remains usable without them. */
        console.debug("[gb-mail] Nudge fetch failed:", err);
      });
  }

  function dismissNudge(emailId) {
    fetch("/api/email/nudge/dismiss", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(emailId)
    })
      .then(function () {
        var banner = document.getElementById("nudges-banner-" + emailId);
        if (banner) banner.innerHTML = "";
      })
      .catch(function (err) {
        /* Best-effort nudge dismiss; UI is already updated optimistically. */
        console.debug("[gb-mail] Nudge dismiss failed:", err);
      });
  }

  function initEmailExtras() {
    var banners = document.querySelectorAll(".nudges-banner[data-email-id]");
    banners.forEach(function (el) {
      loadNudgesForEmail(el.dataset.emailId);
    });
    var chips = document.querySelectorAll(".smart-reply-chips[data-email-id]");
    chips.forEach(function (el) {
      loadSmartReplies(el.dataset.emailId);
    });
  }

  document.body.addEventListener("htmx:afterSwap", function (evt) {
    if (evt.detail.target && evt.detail.target.id === "mail-content") {
      setTimeout(initEmailExtras, 0);
    }
  });

  window.snoozeEmail = snoozeEmail;
  window.loadSmartReplies = loadSmartReplies;
  window.useSmartReply = useSmartReply;
  window.loadNudgesForEmail = loadNudgesForEmail;
  window.dismissNudge = dismissNudge;
  window.initEmailExtras = initEmailExtras;

  if (document.readyState === "loading") {
    (function(){ var __cb = initMail; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    initMail();
  }

  document.body.addEventListener("htmx:afterSwap", function (evt) {
    if (evt.detail.target && evt.detail.target.id === "main-content") {
      if (document.querySelector(".mail-layout")) {
        initMail();
      }
    }
  });
