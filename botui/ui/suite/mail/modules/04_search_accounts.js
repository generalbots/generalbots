/* Mail app — split from mail/mail.js for #1731 */

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

  function saveAccount(evt) {
    var form = document.getElementById("account-form");
    if (!form) return;
    if (evt) evt.preventDefault();
    if (typeof form.requestSubmit === "function") {
      form.requestSubmit();
    } else {
      form.dispatchEvent(new Event("submit", { cancelable: true, bubbles: true }));
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

  function mailEscapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
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
