/* Mail app — split from mail/mail.js for #1731 */

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
