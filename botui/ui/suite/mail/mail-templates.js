"use strict";

/**
 * mail-templates.js
 *
 * Populates the three template slots declared in mail.html with real, functional
 * markup wired to the existing backend endpoints:
 *   - mailTplSlot1 : main mail layout (folder sidebar + message list + detail)
 *   - mailTplSlot2 : compose modal body (compose form posting to /api/email/send)
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

  function escapeHtml(text) {
    var div = document.createElement("div");
    div.textContent = text == null ? "" : String(text);
    return div.innerHTML;
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
    '      <input type="search" id="email-search" class="mail-search" placeholder="Search mail"',
    '             oninput="handleSearchInput(event)" autocomplete="off" />',
    '      <button class="btn-icon" type="button" onclick="toggleSearchFilters()" title="Filters">&#128269;</button>',
    '    </div>',
    '    <div id="bulk-actions" class="bulk-actions" style="display:none;">',
    '      <span class="selected-count">0 selected</span>',
    '      <button class="btn-secondary" type="button" onclick="archiveSelected()">Archive</button>',
    '      <button class="btn-secondary" type="button" onclick="markAsRead()">Mark read</button>',
    '      <button class="btn-secondary" type="button" onclick="addLabelToSelected()">Label</button>',
    '      <button class="btn-danger" type="button" onclick="deleteSelected()">Delete</button>',
    '    </div>',
    '    <div id="mail-list" class="mail-list">',
    '      <div class="loading-state"><div class="spinner"></div></div>',
    '    </div>',
    '    <div id="mail-content" class="mail-content"></div>',
    '  </section>',
    '</div>',
  ].join("\n");

  inject("mailTplSlot1", mainLayout);

  // ---- Slot 2: compose modal body ---------------------------------------------
  var composeForm = [
    '<form id="compose-form" class="compose-form"',
    '      hx-post="/api/email/send" hx-target="#mail-content" hx-swap="innerHTML">',
    '  <div class="compose-row">',
    '    <input type="email" name="to" id="compose-to" class="compose-input" placeholder="To" required />',
    '    <button type="button" class="btn-link" onclick="toggleCcBcc()">Cc/Bcc</button>',
    '  </div>',
    '  <div class="cc-bcc" style="display:none;">',
    '    <input type="email" name="cc" class="compose-input" placeholder="Cc" />',
    '    <input type="email" name="bcc" class="compose-input" placeholder="Bcc" />',
    '  </div>',
    '  <input type="text" name="subject" id="compose-subject" class="compose-input" placeholder="Subject" />',
    '  <div class="compose-toolbar">',
    '    <button type="button" class="btn-icon" onclick="formatText(\'bold\')" title="Bold"><b>B</b></button>',
    '    <button type="button" class="btn-icon" onclick="formatText(\'italic\')" title="Italic"><i>I</i></button>',
    '    <button type="button" class="btn-icon" onclick="formatText(\'underline\')" title="Underline"><u>U</u></button>',
    '    <button type="button" class="btn-icon" onclick="insertLink()" title="Link">&#128279;</button>',
    '    <button type="button" class="btn-icon" onclick="insertImage()" title="Image">&#128247;</button>',
    '    <button type="button" class="btn-icon" onclick="attachFile()" title="Attach">&#128206;</button>',
    '    <button type="button" class="btn-icon" onclick="insertSignature()" title="Signature">&#9993;</button>',
    '    <button type="button" class="btn-icon" onclick="openTemplates()" title="Templates">&#128196;</button>',
    '    <div class="schedule-wrap">',
    '      <button type="button" class="btn-icon" onclick="toggleScheduleMenu()" title="Schedule">&#128197;</button>',
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
    '    <input type="email" name="email" id="account-email" placeholder="you@example.com" required />',
    '  </div>',
    '  <div class="form-group">',
    '    <label>Password / app password</label>',
    '    <input type="password" name="password" id="account-password" placeholder="••••••••" required />',
    '  </div>',
    '  <div class="form-group">',
    '    <label>IMAP server</label>',
    '    <input type="text" name="imap_server" id="account-imap" placeholder="imap.example.com" required />',
    '  </div>',
    '  <div class="form-row">',
    '    <div class="form-group">',
    '      <label>IMAP port</label>',
    '      <input type="number" name="imap_port" id="account-imap-port" value="993" min="1" max="65535" required />',
    '    </div>',
    '    <div class="form-group">',
    '      <label>SMTP server</label>',
    '      <input type="text" name="smtp_server" id="account-smtp" placeholder="smtp.example.com" required />',
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

  if (window.GBAppLifecycle) GBAppLifecycle.end("mail");
})();
