/* Mail app — split from mail/mail.js for #1731 */

"use strict";

  if (window.GBAppLifecycle) GBAppLifecycle.begin("mail");

  var selectedEmails = new Set();
  var currentFolder = "inbox";

  function openCompose() {
    var modal = document.getElementById("compose-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
    var status = document.getElementById("compose-status");
    if (status) status.textContent = "";
    loadMailAccounts();
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

  // ---------------------------------------------------------------------------
  // Accounts, OAuth2 hand-off and JSON compose submission.
  // The compose form is injected by mail-templates.js, so submission is handled
  // with a delegated listener (capture phase) that also stops any htmx binding.
  // ---------------------------------------------------------------------------

  function mailT(key, fallback, params) {
    return typeof window.gbT === "function"
      ? window.gbT(key, fallback, params)
      : fallback;
  }

  function mailStoredToken(key) {
    try {
      return (
        window.localStorage.getItem(key) ||
        window.sessionStorage.getItem(key) ||
        ""
      );
    } catch (e) {
      return "";
    }
  }

  function mailSuiteToken() {
    if (typeof window.getGBAccessToken === "function") {
      var canonical = window.getGBAccessToken() || "";
      if (canonical) return canonical;
    }
    return mailStoredToken("gb-access-token");
  }

  // /api/auth/login returns an opaque botserver token (gb_a…) as the access
  // token and a Zitadel JWT as session_token; lock-screen.js keeps the latter
  // in gb-ws-token. Stalwart's Zitadel introspection needs the JWT, so the
  // OAuth hand-off prefers it. It is only sent to the same-origin oauth route.
  function mailAccessToken() {
    var jwt = mailStoredToken("gb-ws-token");
    if (jwt.indexOf("eyJ") === 0) return jwt;
    return mailSuiteToken();
  }

  function mailAuthHeaders(token) {
    var headers = { "Content-Type": "application/json" };
    if (token) headers["Authorization"] = "Bearer " + token;
    return headers;
  }

  function isOAuth2Account(account) {
    if (!account) return false;
    var mode = String(account.auth_mode || "").toLowerCase();
    var provider = String(account.oauth_provider || account.provider || "").toLowerCase();
    return mode === "oauth2" || provider === "zitadel";
  }

  var mailAccountsPromise = null;

  function renderComposeAccounts() {
    var select = document.getElementById("compose-account");
    if (!select) return;
    var accounts = window.MailAccounts || [];
    var previous = select.value;
    while (select.firstChild) select.removeChild(select.firstChild);
    accounts.forEach(function (account) {
      var option = document.createElement("option");
      option.value = String(account.id || "");
      option.textContent = account.display_name
        ? account.display_name + " <" + (account.email || "") + ">"
        : account.email || String(account.id || "");
      select.appendChild(option);
    });
    var chosen = accounts.filter(function (a) { return a.is_primary; })[0] || accounts[0];
    var stillThere = accounts.some(function (a) { return String(a.id || "") === previous; });
    var target = stillThere ? previous : chosen ? String(chosen.id || "") : "";
    if (target) select.value = target;
  }

  function loadMailAccounts(force) {
    if (!force && mailAccountsPromise) return mailAccountsPromise;
    var token = mailSuiteToken();
    if (!token) {
      window.MailAccounts = [];
      window.MailOAuthAccountIds = [];
      renderComposeAccounts();
      return Promise.resolve([]);
    }
    mailAccountsPromise = fetch("/api/email/accounts", {
      headers: mailAuthHeaders(token),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("accounts request failed");
        return response.json();
      })
      .then(function (result) {
        var list = result && Array.isArray(result.data)
          ? result.data
          : Array.isArray(result) ? result : [];
        window.MailAccounts = list;
        window.MailOAuthAccountIds = list
          .filter(isOAuth2Account)
          .map(function (a) { return String(a.id || ""); });
        renderComposeAccounts();
        return list;
      })
      .catch(function () {
        mailAccountsPromise = null;
        window.MailAccounts = [];
        window.MailOAuthAccountIds = [];
        renderComposeAccounts();
        return [];
      });
    return mailAccountsPromise;
  }

  function currentAccountId() {
    var select = document.getElementById("compose-account");
    if (select && select.value) return select.value;
    var active = document.querySelector(
      ".account-item.active[data-account-id], .account-item.selected[data-account-id]"
    );
    if (active && active.getAttribute("data-account-id")) {
      return active.getAttribute("data-account-id");
    }
    var accounts = window.MailAccounts || [];
    var chosen = accounts.filter(function (a) { return a.is_primary; })[0] || accounts[0];
    return chosen ? String(chosen.id || "") : "";
  }

  function connectAccountOAuth(accountId, token) {
    if (!accountId) return Promise.resolve(true);
    var account = (window.MailAccounts || []).filter(function (a) {
      return String(a.id || "") === String(accountId);
    })[0];
    if (!isOAuth2Account(account)) return Promise.resolve(true);
    if (!token) return Promise.resolve(false);
    // The suite token goes to the same-origin API only. It is never logged,
    // rendered in the UI or included in a notification.
    return fetch("/api/email/accounts/" + encodeURIComponent(accountId) + "/oauth", {
      method: "POST",
      headers: mailAuthHeaders(token),
      body: JSON.stringify({ access_token: token }),
    })
      .then(function (response) { return response.ok; })
      .catch(function () { return false; });
  }

  function mailStatus(message, kind) {
    var region = document.getElementById("compose-status");
    if (region) region.textContent = message || "";
    if (typeof window.showNotification === "function") {
      window.showNotification(message, kind === "error" ? "error" : "success");
    }
  }

  function notifySendFailure() {
    mailStatus(
      mailT("error-email-send-failed", "Failed to send email", {
        message: mailT(
          "gb-state-error-desc",
          "The request failed. Check your connection and try again."
        ),
      }),
      "error"
    );
  }

  function onComposeSubmit(event) {
    var form = event.target;
    if (!form || form.id !== "compose-form") return;
    event.preventDefault();
    if (event.stopPropagation) event.stopPropagation();
    prepareSubmit();

    var accountId = currentAccountId();
    if (!accountId) {
      notifySendFailure();
      return;
    }
    var hiddenBody = document.getElementById("compose-body-hidden");
    var scheduled = form.querySelector('input[name="scheduled_at"]');
    var payload = {
      account_id: accountId,
      to: (document.getElementById("compose-to") || {}).value || "",
      cc: (form.querySelector('input[name="cc"]') || {}).value || "",
      bcc: (form.querySelector('input[name="bcc"]') || {}).value || "",
      subject: (document.getElementById("compose-subject") || {}).value || "",
      body: hiddenBody ? hiddenBody.value || "" : "",
      is_html: true,
    };
    if (scheduled && scheduled.value) payload.scheduled_at = scheduled.value;
    // The JWT is reserved for the same-origin OAuth exchange; the opaque suite
    // token authenticates the account list and send calls.
    var oauthToken = mailAccessToken();
    var suiteToken = mailSuiteToken();

    connectAccountOAuth(accountId, oauthToken)
      .then(function (connected) {
        if (!connected) return false;
        return fetch("/api/email/send", {
          method: "POST",
          headers: mailAuthHeaders(suiteToken),
          body: JSON.stringify(payload),
        }).then(function (response) { return response.ok; });
      })
      .then(function (sent) {
        if (!sent) {
          notifySendFailure();
          return;
        }
        mailStatus(mailT("mail-sent", "Email sent"), "success");
        closeCompose();
        if (typeof refreshMailList === "function") refreshMailList();
      })
      .catch(function () { notifySendFailure(); });
  }

  document.addEventListener("submit", onComposeSubmit, true);

  window.loadMailAccounts = loadMailAccounts;
  window.connectAccountOAuth = connectAccountOAuth;
  window.getSelectedMailAccountId = currentAccountId;
  window.isMailOAuth2Account = isOAuth2Account;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { loadMailAccounts(); });
  } else {
    loadMailAccounts();
  }
