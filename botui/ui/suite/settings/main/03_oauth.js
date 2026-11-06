/* Settings module 03: OAuth account linking. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var getAuthToken = S.getAuthToken;
  var showToast = S.showToast;

/**
 * OAuth account linking (#899). The Connect buttons call the backend
 * with the bearer token, receive the provider authorization URL, and
 * navigate the top window there. On return the callback re-renders the
 * linked-accounts list.
 */
function initOAuthLinking() {
  document.querySelectorAll("[data-oauth-connect]").forEach(function (btn) {
    btn.addEventListener("click", function (e) {
      e.preventDefault();
      var provider = btn.getAttribute("data-oauth-connect");
      var token = getAuthToken();
      if (!token) {
        showToast("Please sign in to link an account", "error");
        return;
      }
      btn.disabled = true;
      fetch("/api/oauth/" + provider + "/connect", {
        method: "POST",
        headers: { Authorization: "Bearer " + token },
      })
        .then(function (r) {
          return r.json().catch(function () {
            return { error: "OAuth is not configured" };
          });
        })
        .then(function (data) {
          if (data.redirect_url) {
            window.location.href = data.redirect_url;
          } else {
            showToast(data.error || "Failed to start OAuth flow", "error");
            btn.disabled = false;
          }
        })
        .catch(function (err) {
          showToast("OAuth request failed: " + err.message, "error");
          btn.disabled = false;
        });
    });
  });

  // Unlink buttons are rendered server-side with HTMX; after removal the
  // list refreshes itself via hx-swap. Nothing else to bind here.
  var accountsEl = document.getElementById("oauth-accounts-fragment");
  if (accountsEl && !accountsEl.getAttribute("data-loaded")) {
    accountsEl.setAttribute("data-loaded", "true");
    loadOAuthAccounts(accountsEl);
  }
}

function loadOAuthAccounts(container) {
  var token = getAuthToken();
  if (!token) return;
  fetch("/api/oauth/accounts", {
    headers: { Authorization: "Bearer " + token },
  })
    .then(function (r) {
      return r.text();
    })
    .then(function (html) {
      if (container) container.innerHTML = html;
    })
    .catch(function (err) {
      console.warn("Failed to load OAuth accounts:", err);
    });
}

  S.initOAuthLinking = initOAuthLinking;
  S.loadOAuthAccounts = loadOAuthAccounts;
})(window);
