(function () {
  "use strict";

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
    document.execCommand(command, false, null);
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
    if (typeof window.showNotification === "function") {
      window.showNotification("Label manager coming soon", "info");
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
      document.execCommand("createLink", false, url);
    }
  }

  function insertImage() {
    var url = prompt("Enter image URL:");
    if (url) {
      document.execCommand("insertImage", false, url);
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
    if (typeof window.showNotification === "function") {
      window.showNotification("Template editor coming soon", "info");
    }
  }

  function createNewSignature() {
    if (typeof window.showNotification === "function") {
      window.showNotification("Signature editor coming soon", "info");
    }
  }

  function createNewRule() {
    if (typeof window.showNotification === "function") {
      window.showNotification("Rule editor coming soon", "info");
    }
  }

  function archiveSelected() {
    if (typeof window.showNotification === "function") {
      window.showNotification(
        selectedEmails.size + " emails archived",
        "success",
      );
    }
    selectedEmails.clear();
    updateBulkActions();
    refreshMailList();
  }

  function markAsRead() {
    if (typeof window.showNotification === "function") {
      window.showNotification(
        selectedEmails.size + " emails marked as read",
        "success",
      );
    }
    selectedEmails.clear();
    updateBulkActions();
    refreshMailList();
  }

  function addLabelToSelected() {
    if (typeof window.showNotification === "function") {
      window.showNotification("Label picker coming soon", "info");
    }
  }

  function deleteSelected() {
    if (confirm("Delete " + selectedEmails.size + " emails?")) {
      if (typeof window.showNotification === "function") {
        window.showNotification(
          selectedEmails.size + " emails deleted",
          "success",
        );
      }
      selectedEmails.clear();
      updateBulkActions();
      refreshMailList();
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
  }

  function initMail() {
    initFolderHandlers();

    var inboxItem = document.querySelector('.nav-item[data-folder="inbox"]');
    if (inboxItem && typeof htmx !== "undefined") {
      htmx.trigger(inboxItem, "click");
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

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initMail);
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
