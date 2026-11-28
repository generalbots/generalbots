"use strict";

/**
 * mail-templates.js
 *
 * Populates the three template slots declared in mail.html with real, functional
 * markup wired to the existing backend endpoints:
 *   - mailTplSlot1 : main mail layout (folder sidebar + message list + detail)
 *   - mailTplSlot2 : compose modal body (compose form posting JSON to /api/email/send)
 *   - mailTplSlot3 : add-account modal body (form posting to /api/email/accounts/add)
 *
 * No dummy / placeholder markup. Everything is bound to real endpoints.
 */
(function () {
  function inject(slotId, html) {
    var slot = document.getElementById(slotId);
    if (slot) {
      slot.innerHTML = html;
    } else {
      console.warn("mail-templates: missing slot " + slotId);
    }
  }

  function escapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  // ---- Slot 1: main mail layout ------------------------------------------------
  var mainLayout = [
    '<div class="mail-layout">',
    '  <aside class="mail-sidebar">',
    '    <div class="mail-sidebar-header">',
    '      <button class="btn-primary" type="button" onclick="openCompose()">Compose</button>',
    '      <button class="btn-secondary" type="button" onclick="openAddAccount()">Add Account</button>',
    '    </div>',
    '    <nav class="mail-folders">',
    '      <button class="nav-item active" data-folder="unified"',
    '              hx-get="/api/ui/email/unified-list" hx-target="#mail-list"',
    '              hx-trigger="click" hx-swap="innerHTML">Unified</button>',
    '      <button class="nav-item" data-folder="inbox"',
    '              hx-get="/api/ui/email/list?folder=inbox" hx-target="#mail-list"',
    '              hx-trigger="click" hx-swap="innerHTML">Inbox</button>',
    '      <button class="nav-item" data-folder="sent"',
    '              hx-get="/api/ui/email/list?folder=sent" hx-target="#mail-list"',
    '              hx-trigger="click" hx-swap="innerHTML">Sent</button>',
    '      <button class="nav-item" data-folder="drafts"',
    '              hx-get="/api/ui/email/list?folder=drafts" hx-target="#mail-list"',
    '              hx-trigger="click" hx-swap="innerHTML">Drafts</button>',
    '      <button class="nav-item" data-folder="starred"',
    '              hx-get="/api/ui/email/list?folder=starred" hx-target="#mail-list"',
    '              hx-trigger="click" hx-swap="innerHTML">Starred</button>',
    '      <button class="nav-item" data-folder="archive"',
    '              hx-get="/api/ui/email/list?folder=archive" hx-target="#mail-list"',
    '              hx-trigger="click" hx-swap="innerHTML">Archive</button>',
    '    </nav>',
    '    <div class="mail-sidebar-footer">',
    '      <button class="nav-item" type="button" onclick="openLabelManager()">Labels</button>',
    '      <button class="nav-item" type="button" onclick="openSignatures()">Signatures</button>',
    '      <button class="nav-item" type="button" onclick="openRules()">Rules</button>',
    '      <button class="nav-item" type="button" onclick="openAutoResponder()">Auto-Reply</button>',
    '      <button class="nav-item" type="button" onclick="openTemplates()">Templates</button>',
    '    </div>',
    '  </aside>',
    '  <section class="mail-main">',
    '    <div class="mail-toolbar">',
    '      <div class="search-box">',
    '        <input type="search" id="email-search" class="mail-search" placeholder="Search mail"',
    '               oninput="handleSearchInput(event)" autocomplete="off" />',
    '        <button class="btn-icon search-filter-toggle" type="button" onclick="toggleSearchFilters()" title="Filters" aria-label="Filters" data-i18n-title="search-filters" data-i18n-aria-label="search-filters" aria-expanded="false" aria-controls="search-filters">&#128269;</button>',
    '        <div id="search-filters" class="search-filters" style="display:none;">',
    '          <div class="filter-row">',
    '            <label><input type="checkbox" id="filter-unread" /> Unread</label>',
    '          </div>',
    '          <div class="filter-row">',
    '            <label><input type="checkbox" id="filter-has-attachment" /> Has attachment</label>',
    '          </div>',
    '          <div class="filter-row">',
    '            <label for="filter-from">From</label>',
    '            <input type="text" id="filter-from" class="filter-date" placeholder="sender@example.com" />',
    '          </div>',
    '          <div class="filter-row">',
    '            <button type="button" class="btn-secondary" onclick="applySearchFilters()">Apply</button>',
    '          </div>',
    '        </div>',
    '      </div>',
    '    </div>',
    '    <div id="bulk-actions" class="bulk-actions" style="display:none;">',
    '      <span class="selected-count">0 selected</span>',
    '      <button class="btn-secondary" type="button" onclick="archiveSelected()">Archive</button>',
    '      <button class="btn-secondary" type="button" onclick="markAsRead()">Mark read</button>',
    '      <button class="btn-secondary" type="button" onclick="addLabelToSelected()">Label</button>',
    '      <button class="btn-danger" type="button" onclick="deleteSelected()">Delete</button>',
    '    </div>',
    '    <div class="mail-panes gb-panes" data-gb-panes data-gb-panes-primary="#mail-list" data-gb-panes-detail="#mail-content" data-gb-panes-rows=".mail-item">',
    '      <div id="mail-list" class="mail-list gb-pane gb-pane-primary">',
    '        <div class="loading-state"><div class="spinner"></div></div>',
    '      </div>',
    '      <div id="mail-content" class="mail-content gb-pane gb-pane-detail"></div>',
    '    </div>',
    '  </section>',
    '</div>',
  ].join("\n");

  inject("mailTplSlot1", mainLayout);

  // ---- Slot 2: compose modal body ---------------------------------------------
  var composeForm = [
    // /api/email/send requires JSON (account_id, to, subject, body, is_html).
    // An htmx hx-post would send form-encoded data and the API answers 415, so
    // modules/01_compose.js owns submission with fetch + application/json.
    '<form id="compose-form" class="compose-form">',
    '  <select id="compose-account" name="account_id" class="compose-input"',
    '          aria-label="Account" data-i18n-aria-label="mail-dialog-account-name"></select>',
    '  <div class="compose-row">',
    '    <input type="email" name="to" id="compose-to" class="compose-input" placeholder="To" data-i18n-placeholder="label-to" required />',
    '    <button type="button" class="btn-link" onclick="toggleCcBcc()">Cc/Bcc</button>',
    '  </div>',
    '  <div class="cc-bcc" style="display:none;">',
    '    <input type="email" name="cc" class="compose-input" placeholder="Cc" data-i18n-placeholder="mail-cc" />',
    '    <input type="email" name="bcc" class="compose-input" placeholder="Bcc" data-i18n-placeholder="mail-bcc" />',
    '  </div>',
    '  <input type="text" name="subject" id="compose-subject" class="compose-input" placeholder="Subject" data-i18n-placeholder="mail-placeholder-subject" />',
    '  <div class="compose-toolbar">',
    '    <button type="button" class="btn-icon" onclick="formatText(\'bold\')" title="Bold" aria-label="Bold" data-i18n-title="paper-bold" data-i18n-aria-label="paper-bold"><b>B</b></button>',
    '    <button type="button" class="btn-icon" onclick="formatText(\'italic\')" title="Italic" aria-label="Italic" data-i18n-title="paper-italic" data-i18n-aria-label="paper-italic"><i>I</i></button>',
    '    <button type="button" class="btn-icon" onclick="formatText(\'underline\')" title="Underline" aria-label="Underline" data-i18n-title="paper-underline" data-i18n-aria-label="paper-underline"><u>U</u></button>',
    '    <button type="button" class="btn-icon" onclick="insertLink()" title="Link" aria-label="Insert link" data-i18n-aria-label="paper-insert-link">&#128279;</button>',
    '    <button type="button" class="btn-icon" onclick="insertImage()" title="Image" aria-label="Insert image" data-i18n-title="paper-image" data-i18n-aria-label="paper-insert-image">&#128247;</button>',
    '    <button type="button" class="btn-icon" onclick="attachFile()" title="Attach" aria-label="Attach file" data-i18n-aria-label="attendant-attach">&#128206;</button>',
    '    <button type="button" class="btn-icon" onclick="insertSignature()" title="Signature" aria-label="Insert signature" data-i18n-title="minutes-action-signature">&#9993;</button>',
    '    <button type="button" class="btn-icon" onclick="openTemplates()" title="Templates" aria-label="Insert template">&#128196;</button>',
    '    <div class="schedule-wrap">',
    '      <button type="button" class="btn-icon" onclick="toggleScheduleMenu()" title="Schedule" aria-label="Schedule" data-i18n-title="mail-dialog-schedule-action" data-i18n-aria-label="mail-dialog-schedule-action">&#128197;</button>',
    '      <div id="schedule-menu" class="schedule-menu">',
    '        <button type="button" onclick="scheduleSend(\'tomorrow-morning\')">Tomorrow 8am</button>',
    '        <button type="button" onclick="scheduleSend(\'tomorrow-afternoon\')">Tomorrow 1pm</button>',
    '        <button type="button" onclick="scheduleSend(\'monday\')">Next Monday</button>',
    '        <button type="button" onclick="openCustomSchedule()">Custom date…</button>',
    '      </div>',
    '    </div>',
    '  </div>',
    '  <div id="compose-body" class="compose-body" contenteditable="true"',
    '       data-placeholder="Write your message…"></div>',
    '  <input type="hidden" name="body" id="compose-body-hidden" />',
    '  <div id="compose-attachments" class="compose-attachments"></div>',
    '  <div id="compose-status" class="compose-status" role="status" aria-live="polite"></div>',
    '  <div class="compose-actions">',
    '    <button type="submit" class="btn-primary" onclick="prepareSubmit()">Send</button>',
    '    <button type="button" class="btn-secondary" onclick="saveDraft()">Save Draft</button>',
    '    <button type="button" class="btn-secondary" onclick="minimizeCompose()">Minimize</button>',
    '    <button type="button" class="btn-secondary" onclick="closeCompose()">Discard</button>',
    '  </div>',
    '</form>',
  ].join("\n");

  inject("mailTplSlot2", composeForm);

  // ---- Slot 3: add-account modal body ------------------------------------------
  var addAccountForm = [
    '<form id="account-form" class="account-form">',
    '  <div class="form-group">',
    '    <label>Display name</label>',
    '    <input type="text" name="display_name" id="account-display-name" placeholder="Jane Doe" required />',
    '  </div>',
    '  <div class="form-group">',
    '    <label>Email address</label>',
    '    <input type="email" name="email" id="account-email" placeholder="you@example.com" data-i18n-placeholder="settings-notification-recipient-placeholder" required />',
    '  </div>',
    '  <div class="form-group">',
    '    <label>Password / app password</label>',
    '    <input type="password" name="password" id="account-password" placeholder="••••••••" data-i18n-placeholder="mail-placeholder-password" required />',
    '  </div>',
    '  <div class="form-group">',
    '    <label>IMAP server</label>',
    '    <input type="text" name="imap_server" id="account-imap" placeholder="imap.example.com" data-i18n-placeholder="mail-placeholder-imap-server" required />',
    '  </div>',
    '  <div class="form-row">',
    '    <div class="form-group">',
    '      <label>IMAP port</label>',
    '      <input type="number" name="imap_port" id="account-imap-port" value="993" min="1" max="65535" required />',
    '    </div>',
    '    <div class="form-group">',
    '      <label>SMTP server</label>',
    '      <input type="text" name="smtp_server" id="account-smtp" placeholder="smtp.example.com" data-i18n-placeholder="mail-placeholder-smtp-server" required />',
    '    </div>',
    '    <div class="form-group">',
    '      <label>SMTP port</label>',
    '      <input type="number" name="smtp_port" id="account-smtp-port" value="587" min="1" max="65535" required />',
    '    </div>',
    '  </div>',
    '  <div class="form-group">',
    '    <label>Username (if different from email)</label>',
    '    <input type="text" name="username" id="account-username" placeholder="Same as email by default" />',
    '  </div>',
    '  <label class="checkbox-label">',
    '    <input type="checkbox" name="is_primary" id="account-primary" />',
    '    <span>Set as primary account</span>',
    '  </label>',
    // Microsoft 365, Outlook.com and Gmail refuse a static password over
    // IMAP/SMTP, so those mailboxes are connected through the provider's own
    // consent screen instead of this form.
    '  <div class="oauth-connect">',
    '    <p class="oauth-hint">Microsoft 365, Outlook.com and Gmail sign in through the provider. Fill in the email address above, then choose the provider.</p>',
    '    <div class="oauth-buttons">',
    '      <button type="button" class="btn-secondary" data-provider="microsoft" onclick="connectMailProvider(\'microsoft\')">Connect Microsoft 365</button>',
    '      <button type="button" class="btn-secondary" data-provider="google" onclick="connectMailProvider(\'google\')">Connect Google</button>',
    '    </div>',
    '  </div>',
    '  <div class="modal-footer">',
    '    <button type="button" class="btn-secondary" onclick="closeAddAccount()">Cancel</button>',
    '    <button type="submit" class="btn-primary" onclick="saveAccount(event)">Add Account</button>',
    '  </div>',
    '</form>',
  ].join("\n");

  inject("mailTplSlot3", addAccountForm);

  // The account form posts JSON to the real backend endpoint.
  var accountFormEl = document.getElementById("account-form");
  if (accountFormEl) {
    accountFormEl.addEventListener("submit", function (e) {
      e.preventDefault();
      var payload = {
        display_name: (document.getElementById("account-display-name") || {}).value || "",
        email: (document.getElementById("account-email") || {}).value || "",
        password: (document.getElementById("account-password") || {}).value || "",
        imap_server: (document.getElementById("account-imap") || {}).value || "",
        imap_port: parseInt((document.getElementById("account-imap-port") || {}).value || "993", 10),
        smtp_server: (document.getElementById("account-smtp") || {}).value || "",
        smtp_port: parseInt((document.getElementById("account-smtp-port") || {}).value || "587", 10),
        username:
          (document.getElementById("account-username") || {}).value ||
          (document.getElementById("account-email") || {}).value ||
          "",
        is_primary: !!(document.getElementById("account-primary") || {}).checked,
      };

      fetch("/api/email/accounts/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      })
        .then(function (r) {
          return r.json().then(function (data) {
            return { ok: r.ok, data: data };
          });
        })
        .then(function (res) {
          if (res.ok && res.data && (res.data.success || res.data.data)) {
            if (typeof window.showNotification === "function") {
              window.showNotification("Email account added", "success");
            }
            closeAddAccount();
            var list = document.getElementById("mail-list");
            if (list && typeof htmx !== "undefined") {
              htmx.trigger(document.querySelector('.nav-item[data-folder="unified"]'), "click");
            }
          } else {
            throw new Error((res.data && res.data.message) || "Failed to add account");
          }
        })
        .catch(function (err) {
          if (typeof window.showNotification === "function") {
            window.showNotification("Add account error: " + err.message, "error");
          }
        });
    });
  }

  // Starts the provider consent flow for an OAuth2 mailbox. The address is
  // required because the provider account it is connected to has to match the
  // mailbox being stored; the response carries the consent URL to visit.
  window.connectMailProvider = function (provider) {
    var email =
      ((document.getElementById("account-email") || {}).value || "").trim();
    if (!email) {
      if (typeof window.showNotification === "function") {
        window.showNotification(
          "Enter the mailbox address before connecting a provider",
          "error"
        );
      }
      return;
    }

    fetch("/api/email/oauth/start", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ provider: provider, email: email }),
    })
      .then(function (r) {
        return r.json().then(function (data) {
          return { ok: r.ok, data: data };
        });
      })
      .then(function (res) {
        var url = res.data && res.data.data && res.data.data.url;
        if (res.ok && url) {
          window.location.href = url;
          return;
        }
        throw new Error((res.data && res.data.message) || "Failed to start the provider sign-in");
      })
      .catch(function (err) {
        if (typeof window.showNotification === "function") {
          window.showNotification("Sign-in error: " + err.message, "error");
        }
      });
  };

  if (window.GBAppLifecycle) GBAppLifecycle.end("mail");
})();
