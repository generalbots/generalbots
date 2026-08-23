(function () {
  "use strict";
  if (window.GBAppLifecycle) GBAppLifecycle.begin("mail");

  var selectedEmails = new Set();
  var currentFolder = "inbox";

  function openCompose() {
    var modal = document.getElementById("compose-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeCompose() {
    var modal = document.getElementById("compose-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function minimizeCompose() {
    closeCompose();
  }

  function toggleCcBcc() {
    document.querySelectorAll(".cc-bcc").forEach(function (el) {
      el.style.display = el.style.display === "none" ? "flex" : "none";
    });
  }

  function toggleScheduleMenu() {
    var menu = document.getElementById("schedule-menu");
    if (menu) {
      menu.classList.toggle("show");
    }
  }

  function scheduleSend(option) {
    var date = new Date();
    switch (option) {
      case "tomorrow-morning":
        date.setDate(date.getDate() + 1);
        date.setHours(8, 0, 0, 0);
        break;
      case "tomorrow-afternoon":
        date.setDate(date.getDate() + 1);
        date.setHours(13, 0, 0, 0);
        break;
      case "monday":
        var daysUntilMonday = (8 - date.getDay()) % 7 || 7;
        date.setDate(date.getDate() + daysUntilMonday);
        date.setHours(8, 0, 0, 0);
        break;
    }
    confirmScheduleSend(date);
    toggleScheduleMenu();
  }

  function openCustomSchedule() {
    toggleScheduleMenu();
    var today = new Date().toISOString().split("T")[0];
    var dateInput = document.getElementById("schedule-date");
    if (dateInput) {
      dateInput.min = today;
      dateInput.value = today;
    }
    var modal = document.getElementById("schedule-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeScheduleModal() {
    var modal = document.getElementById("schedule-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function confirmSchedule() {
    var dateInput = document.getElementById("schedule-date");
    var timeInput = document.getElementById("schedule-time");
    if (dateInput && timeInput && dateInput.value && timeInput.value) {
      var scheduledDate = new Date(dateInput.value + "T" + timeInput.value);
      if (isNaN(scheduledDate.getTime())) {
        if (typeof window.showNotification === "function") {
          window.showNotification(
            "Please select a valid date and time",
            "error",
          );
        }
        return;
      }
      confirmScheduleSend(scheduledDate);
    } else {
      if (typeof window.showNotification === "function") {
        window.showNotification("Please select a date and time", "error");
      }
      return;
    }
    closeScheduleModal();
  }

  function confirmScheduleSend(date) {
    var form = document.getElementById("compose-form");
    if (form) {
      var input = document.createElement("input");
      input.type = "hidden";
      input.name = "scheduled_at";
      input.value = date.toISOString();
      form.appendChild(input);
      prepareSubmit();
      form.requestSubmit();
    }
  }

  function prepareSubmit() {
    var body = document.getElementById("compose-body");
    var hidden = document.getElementById("compose-body-hidden");
    if (body && hidden) {
      hidden.value = body.innerHTML;
    }
  }

  function formatText(command) {
    var selection = window.getSelection();
    if (selection.rangeCount) {
      var range = selection.getRangeAt(0);
      var tag = "";
      if (command === "bold") tag = "strong";
      else if (command === "italic") tag = "em";
      else if (command === "underline") tag = "u";

      if (tag) {
        var el = document.createElement(tag);
        try {
          range.surroundContents(el);
        } catch (e) {
          var contents = range.extractContents();
          el.appendChild(contents);
          range.insertNode(el);
        }
      }
    }
    var body = document.getElementById("compose-body");
    if (body) {
      body.focus();
    }
  }

  function openTemplates() {
    var modal = document.getElementById("templates-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeTemplates() {
    var modal = document.getElementById("templates-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function openSignatures() {
    var modal = document.getElementById("signatures-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeSignatures() {
    var modal = document.getElementById("signatures-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function openRules() {
    var modal = document.getElementById("rules-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeRules() {
    var modal = document.getElementById("rules-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function openAutoResponder() {
    var modal = document.getElementById("autoresponder-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeAutoResponder() {
    var modal = document.getElementById("autoresponder-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function saveAutoResponder() {
    var form = document.getElementById("autoresponder-form");
    if (form && typeof htmx !== "undefined") {
      htmx.trigger(form, "submit");
    }
    closeAutoResponder();
    if (typeof window.showNotification === "function") {
      window.showNotification("Auto-reply settings saved", "success");
    }
  }

  function openLabelManager() {
    var modal = document.getElementById("labels-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function toggleSelectAll(checkbox) {
    var items = document.querySelectorAll('.mail-item input[type="checkbox"]');
    items.forEach(function (item) {
      item.checked = checkbox.checked;
      if (checkbox.checked) {
        selectedEmails.add(item.dataset.id);
      } else {
        selectedEmails.delete(item.dataset.id);
      }
    });
    updateBulkActions();
  }

  function updateBulkActions() {
    var bulkBar = document.getElementById("bulk-actions");
    if (bulkBar) {
      if (selectedEmails.size > 0) {
        bulkBar.style.display = "flex";
        var countEl = bulkBar.querySelector(".selected-count");
        if (countEl) {
          countEl.textContent = selectedEmails.size + " selected";
        }
      } else {
        bulkBar.style.display = "none";
      }
    }
  }

  function refreshMailList() {
    var folderEl = document.querySelector(
      '[data-folder="' + currentFolder + '"]',
    );
    if (folderEl && typeof htmx !== "undefined") {
      htmx.trigger(folderEl, "click");
    }
  }

  function insertSignature() {
    fetch("/api/email/signatures/default")
      .then(function (r) {
        return r.json();
      })
      .then(function (sig) {
        if (sig.content_html) {
          var body = document.getElementById("compose-body");
          if (body) {
            body.innerHTML += "<br><br>" + sig.content_html;
          }
        }
      })
      .catch(function (e) {
        console.warn("Failed to load signature:", e);
      });
  }

  function showTemplateSelector() {
    openTemplates();
  }

  function attachFile() {
    var input = document.createElement("input");
    input.type = "file";
    input.multiple = true;
    input.onchange = function (e) {
      var files = e.target.files;
      var container = document.getElementById("compose-attachments");
      if (container) {
        Array.from(files).forEach(function (file) {
          var chip = document.createElement("div");
          chip.className = "attachment-chip";
          chip.innerHTML =
            "<span>" +
            escapeHtml(file.name) +
            "</span>" +
            '<button type="button" onclick="this.parentElement.remove()">×</button>';
          container.appendChild(chip);
        });
      }
    };
    input.click();
  }

  function insertLink() {
    var url = prompt("Enter URL:");
    if (url) {
      var selection = window.getSelection();
      if (selection.rangeCount) {
        var range = selection.getRangeAt(0);
        var a = document.createElement("a");
        a.href = url;
        a.target = "_blank";
        if (range.collapsed) {
          a.textContent = url;
          range.insertNode(a);
        } else {
          try {
            range.surroundContents(a);
          } catch (e) {
            var contents = range.extractContents();
            a.appendChild(contents);
            range.insertNode(a);
          }
        }
      }
    }
  }

  function insertImage() {
    var url = prompt("Enter image URL:");
    if (url) {
      var selection = window.getSelection();
      if (selection.rangeCount) {
        var range = selection.getRangeAt(0);
        var img = document.createElement("img");
        img.src = url;
        img.style.maxWidth = "100%";
        range.insertNode(img);
      }
    }
  }

  function saveDraft() {
    prepareSubmit();
    var form = document.getElementById("compose-form");
    if (form) {
      var formData = new FormData(form);
      fetch("/api/email/draft", {
        method: "POST",
        body: formData,
      })
        .then(function () {
          if (typeof window.showNotification === "function") {
            window.showNotification("Draft saved", "success");
          }
        })
        .catch(function (e) {
          console.warn("Failed to save draft:", e);
        });
    }
  }

  function createNewTemplate() {
    var modal = document.getElementById("new-template-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function createNewSignature() {
    var modal = document.getElementById("new-signature-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function createNewRule() {
    var modal = document.getElementById("new-rule-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function archiveSelected() {
    if (selectedEmails.size === 0) return;
    var ids = Array.from(selectedEmails);
    fetch("/api/email/bulk/archive", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ids: ids })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res && res.success) {
        if (typeof window.showNotification === "function") {
          window.showNotification(ids.length + " emails archived", "success");
        }
        selectedEmails.clear();
        updateBulkActions();
        refreshMailList();
      } else {
        throw new Error(res.message || "Failed to archive");
      }
    })
    .catch(function(err) {
      if (typeof window.showNotification === "function") {
        window.showNotification("Archive error: " + err.message, "error");
      }
    });
  }

  function markAsRead() {
    if (selectedEmails.size === 0) return;
    var ids = Array.from(selectedEmails);
    fetch("/api/email/bulk/mark-read", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ids: ids, is_read: true })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res && res.success) {
        if (typeof window.showNotification === "function") {
          window.showNotification(ids.length + " emails marked as read", "success");
        }
        selectedEmails.clear();
        updateBulkActions();
        refreshMailList();
      } else {
        throw new Error(res.message || "Failed to mark as read");
      }
    })
    .catch(function(err) {
      if (typeof window.showNotification === "function") {
        window.showNotification("Update error: " + err.message, "error");
      }
    });
  }

  function addLabelToSelected() {
    if (selectedEmails.size === 0) {
      if (typeof window.showNotification === "function") {
        window.showNotification("No emails selected", "warning");
      }
      return;
    }
    var labelName = prompt("Enter label name to add to selected emails:");
    if (!labelName || !labelName.trim()) return;

    var ids = Array.from(selectedEmails);
    fetch("/api/email/bulk/add-label", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ids: ids, label_name: labelName.trim() })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res && res.success) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Label applied to selected emails", "success");
        }
        selectedEmails.clear();
        updateBulkActions();
        refreshMailList();
        var labelsList = document.getElementById("labels-list");
        if (labelsList && typeof htmx !== "undefined") {
          htmx.trigger(labelsList, "load");
        }
      }
    })
    .catch(function(e) {
      console.error(e);
    });
  }

  function deleteSelected() {
    if (selectedEmails.size === 0) return;
    if (confirm("Delete " + selectedEmails.size + " emails?")) {
      var ids = Array.from(selectedEmails);
      fetch("/api/email/bulk/delete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ids: ids })
      })
      .then(function(r) { return r.json(); })
      .then(function(res) {
        if (res && res.success) {
          if (typeof window.showNotification === "function") {
            window.showNotification(ids.length + " emails deleted", "success");
          }
          selectedEmails.clear();
          updateBulkActions();
          refreshMailList();
        } else {
          throw new Error(res.message || "Failed to delete");
        }
      })
      .catch(function(err) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Delete error: " + err.message, "error");
        }
      });
    }
  }

  function saveLabel() {
    var nameInput = document.getElementById("new-label-name");
    var colorInput = document.getElementById("new-label-color");
    if (!nameInput || !nameInput.value.trim()) return;

    var name = nameInput.value.trim();
    var color = colorInput ? colorInput.value : "#3b82f6";

    fetch("/api/email/labels", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: name, color: color })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res && res.success) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Label created successfully", "success");
        }
        document.getElementById("labels-modal").close();
        nameInput.value = "";
        var labelsList = document.getElementById("labels-list");
        if (labelsList && typeof htmx !== "undefined") {
          htmx.trigger(labelsList, "load");
        }
      }
    })
    .catch(function(e) {
      console.error(e);
    });
  }

  function saveSignature() {
    var nameInput = document.getElementById("new-sig-name");
    var contentInput = document.getElementById("new-sig-content");
    if (!nameInput || !nameInput.value.trim() || !contentInput || !contentInput.value.trim()) return;

    var name = nameInput.value.trim();
    var content = contentInput.value.trim();

    fetch("/api/email/signatures", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: name, content_html: content, content_plain: content, is_default: false })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Signature saved successfully", "success");
        }
        document.getElementById("new-signature-modal").close();
        nameInput.value = "";
        contentInput.value = "";
        var sigList = document.getElementById("signatures-list");
        if (sigList && typeof htmx !== "undefined") {
          htmx.trigger(sigList, "load");
        }
      }
    })
    .catch(function(e) {
      console.error(e);
    });
  }

  function saveRule() {
    var nameInput = document.getElementById("new-rule-name");
    var condInput = document.getElementById("new-rule-condition");
    var actionInput = document.getElementById("new-rule-action");
    if (!nameInput || !nameInput.value.trim()) return;

    var name = nameInput.value.trim();
    var condition = condInput ? condInput.value.trim() : "";
    var action = actionInput ? actionInput.value : "archive";

    fetch("/api/email/rules", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: name, condition: condition, action: action })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res && res.success) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Rule '" + name + "' created successfully", "success");
        }
        document.getElementById("new-rule-modal").close();
        nameInput.value = "";
        if (condInput) condInput.value = "";
        var rulesList = document.getElementById("rules-list");
        if (rulesList && typeof htmx !== "undefined") {
          htmx.trigger(rulesList, "load");
        }
      }
    })
    .catch(function(e) {
      console.error(e);
    });
  }

  function saveTemplate() {
    var nameInput = document.getElementById("new-template-name");
    var subjInput = document.getElementById("new-template-subject");
    var bodyInput = document.getElementById("new-template-body");
    if (!nameInput || !nameInput.value.trim()) return;

    var name = nameInput.value.trim();
    var subject = subjInput ? subjInput.value.trim() : "";
    var body = bodyInput ? bodyInput.value.trim() : "";

    fetch("/api/email/templates", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: name, subject: subject, body: body })
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
      if (res && res.success) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Template '" + name + "' saved successfully", "success");
        }
        document.getElementById("new-template-modal").close();
        nameInput.value = "";
        if (subjInput) subjInput.value = "";
        if (bodyInput) bodyInput.value = "";
        var templatesList = document.getElementById("templates-list");
        if (templatesList && typeof htmx !== "undefined") {
          htmx.trigger(templatesList, "load");
        }
      }
    })
    .catch(function(e) {
      console.error(e);
    });
  }

  function handleCheckboxChange(e) {
    if (e.target.classList.contains("mail-item-checkbox")) {
      if (e.target.checked) {
        selectedEmails.add(e.target.dataset.id);
      } else {
        selectedEmails.delete(e.target.dataset.id);
      }
      updateBulkActions();
    }
  }

  function openAddAccount() {
    var modal = document.getElementById("add-account-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeAddAccount() {
    var modal = document.getElementById("add-account-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function saveAccount() {
    var form = document.getElementById("account-form");
    if (form && typeof htmx !== "undefined") {
      htmx.trigger(form, "submit");
    }
    closeAddAccount();
    if (typeof window.showNotification === "function") {
      window.showNotification("Email account added", "success");
    }
  }

  function toggleSearchFilters() {
    var filters = document.getElementById("search-filters");
    if (filters) {
      var isVisible = filters.style.display !== "none";
      filters.style.display = isVisible ? "none" : "block";
    }
  }

  function loadUnifiedInbox() {
    var mailList = document.getElementById("mail-list");
    if (mailList && typeof htmx !== "undefined") {
      htmx.ajax("GET", "/api/ui/email/unified-list", {
        target: "#mail-list",
        swap: "innerHTML",
      });
    }
  }

  function searchAllAccounts(query) {
    if (!query || query.trim().length === 0) {
      loadUnifiedInbox();
      return;
    }
    var mailList = document.getElementById("mail-list");
    if (mailList && typeof htmx !== "undefined") {
      htmx.ajax("GET", "/api/ui/email/search-all?q=" + encodeURIComponent(query), {
        target: "#mail-list",
        swap: "innerHTML",
        source: "#email-search",
      });
    }
  }

  var searchDebounceTimer = null;

  function handleSearchInput(e) {
    var query = e.target.value;
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer);
    }
    searchDebounceTimer = setTimeout(function () {
      searchAllAccounts(query);
    }, 300);
  }

  function escapeHtml(text) {
    var div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function initFolderHandlers() {
    document
      .querySelectorAll(".nav-item[data-folder]")
      .forEach(function (item) {
        item.addEventListener("click", function () {
          document.querySelectorAll(".nav-item").forEach(function (i) {
            i.classList.remove("active");
          });
          this.classList.add("active");
          currentFolder = this.dataset.folder;
        });
      });

    var searchInput = document.getElementById("email-search");
    if (searchInput) {
      searchInput.removeEventListener("input", handleSearchInput);
      searchInput.addEventListener("input", handleSearchInput);
    }
  }

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
            window.showNotification("Snoozed until " + new Date(res.snooze_until).toLocaleString(), "success");
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
      .catch(function () {});
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
      .catch(function () {});
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
      .catch(function () {});
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
})();
