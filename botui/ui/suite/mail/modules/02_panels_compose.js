/* Mail app — split from mail/mail.js for #1731 */

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
            mailEscapeHtml(file.name) +
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
