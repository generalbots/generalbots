/* Suite app — user profile loading (7/7). */
// User Profile Loading
(function () {
  function updateUserUI(user) {
    if (!user) return;

    const userName = document.getElementById("userName");
    const userEmail = document.getElementById("userEmail");
    const userAvatar = document.getElementById("userAvatar");
    const userAvatarLarge = document.getElementById("userAvatarLarge");
    const authAction = document.getElementById("authAction");
    const authText = document.getElementById("authText");
    const authIcon = document.getElementById("authIcon");
    const settingsBtn = document.getElementById("settingsBtn");
    const appsButton = document.getElementById("appsButton");
    const notificationsBtn = document.getElementById("notificationsBtn");

    const displayName =
      user.display_name || user.first_name || user.username || "User";
    const email = user.email || "";
    const initial = (displayName.charAt(0) || "U").toUpperCase();

    console.log("Updating user UI:", displayName, email);

    if (userName) userName.textContent = displayName;
    if (userEmail) userEmail.textContent = email;
    if (userAvatar) {
      const avatarSpan = userAvatar.querySelector("span");
      if (avatarSpan) avatarSpan.textContent = initial;
    }
    if (userAvatarLarge) userAvatarLarge.textContent = initial;

    if (authAction) {
      authAction.href = "#";
      authAction.onclick = function (e) {
        e.preventDefault();
        fetch("/api/auth/logout", {
          method: "POST",
        }).finally(function () {
          localStorage.removeItem("gb-access-token");
          localStorage.removeItem("gb-refresh-token");
          localStorage.removeItem("gb-user-data");
          sessionStorage.removeItem("gb-access-token");
          var i = 0, keys = [];
          for (i = 0; i < localStorage.length; i++) { var k = localStorage.key(i); if (k && k.indexOf("gb_chat_") === 0) keys.push(k); }
          keys.forEach(function (k) { localStorage.removeItem(k); });
          if (window.GBSecurity && window.GBSecurity.broadcastLogout) window.GBSecurity.broadcastLogout();
          window.location.href = window.GB_LOGIN_URL || "/login";
        });
      };
      authAction.style.color = "var(--error)";
    }
    if (authText) authText.textContent = "Sign out";
    if (authIcon) {
      authIcon.innerHTML =
        '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path><polyline points="16 17 21 12 16 7"></polyline><line x1="21" y1="12" x2="9" y2="12"></line>';
    }

    if (settingsBtn) settingsBtn.style.display = "";
    if (appsButton) appsButton.style.display = "";
    if (notificationsBtn) notificationsBtn.style.display = "";

    // Show omnibox (search bar) when signed in
    const omnibox = document.getElementById("omnibox");
    if (omnibox) omnibox.style.display = "";

    // Show Drive, Tasks, CRM, and Calendar navigation when signed in (all instances)
    const driveTabs = document.querySelectorAll('[data-section="drive"]');
    const tasksTabs = document.querySelectorAll('[data-section="tasks"]');
    const crmTabs = document.querySelectorAll('[data-section="crm"]');
    const calendarTabs = document.querySelectorAll('[data-section="calendar"]');

    driveTabs.forEach(tab => tab.style.display = "");
    tasksTabs.forEach(tab => tab.style.display = "");
    crmTabs.forEach(tab => tab.style.display = "");
    calendarTabs.forEach(tab => tab.style.display = "");
  }

  function loadUserProfile() {
    var token = window.getGBAccessToken && window.getGBAccessToken();
    
    if (!token) {
      console.log("No auth token found - user is signed out");
      updateSignedOutUI();
      return;
    }

    console.log(
      "Loading user profile with token:",
      token.substring(0, 10) + "...",
    );

    fetch("/api/auth/me", {
      headers: { Authorization: "Bearer " + token },
    })
      .then(function (res) {
        if (!res.ok) {
          console.log("User not authenticated");
          updateSignedOutUI();
          throw new Error("Not authenticated");
        }
        return res.json();
      })
      .then(function (user) {
        console.log("User profile loaded:", user);
        updateUserUI(user);
        localStorage.setItem("gb-user-data", JSON.stringify(user));
      })
      .catch(function (err) {
        console.log("Failed to load user profile:", err);
        updateSignedOutUI();
      });
  }

  function updateSignedOutUI() {
    const userName = document.getElementById("userName");
    const userEmail = document.getElementById("userEmail");
    const userAvatar = document.getElementById("userAvatar");
    const userAvatarLarge = document.getElementById("userAvatarLarge");
    const authAction = document.getElementById("authAction");
    const authText = document.getElementById("authText");
    const authIcon = document.getElementById("authIcon");
    const settingsBtn = document.getElementById("settingsBtn");
    const appsButton = document.getElementById("appsButton");
    const notificationsBtn = document.getElementById("notificationsBtn");

    if (userName) userName.textContent = "User";
    if (userEmail) userEmail.textContent = "user@example.com";
    if (userAvatar) {
      const avatarSpan = userAvatar.querySelector("span");
      if (avatarSpan) avatarSpan.textContent = "U";
    }
    if (userAvatarLarge) userAvatarLarge.textContent = "U";

    if (authAction) {
      authAction.href = window.GB_LOGIN_URL || "/login";
      authAction.removeAttribute("onclick");
      authAction.style.color = "var(--primary)";
    }
    if (authText) authText.textContent = "Sign in";
    if (authIcon) {
      authIcon.innerHTML =
        '<path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path><polyline points="10 17 15 12 21 12"></polyline><line x1="15" y1="12" x2="3" y2="12"></line>';
    }

    if (settingsBtn) settingsBtn.style.display = "none";
    if (appsButton) appsButton.style.display = "none";
    if (notificationsBtn) notificationsBtn.style.display = "none";

    // Hide omnibox (search bar) when signed out
    const omnibox = document.getElementById("omnibox");
    if (omnibox) omnibox.style.display = "none";

    // Hide Drive, Tasks, CRM, and Calendar navigation when signed out (all instances)
    const driveTabs = document.querySelectorAll('[data-section="drive"]');
    const tasksTabs = document.querySelectorAll('[data-section="tasks"]');
    const crmTabs = document.querySelectorAll('[data-section="crm"]');
    const calendarTabs = document.querySelectorAll('[data-section="calendar"]');

    driveTabs.forEach(tab => tab.style.display = "none");
    tasksTabs.forEach(tab => tab.style.display = "none");
    crmTabs.forEach(tab => tab.style.display = "none");
    calendarTabs.forEach(tab => tab.style.display = "none");
  }

  // Try to load cached user first
  var cachedUser = localStorage.getItem("gb-user-data");
  if (cachedUser) {
    try {
      var user = JSON.parse(cachedUser);
      if (user && user.email) {
        updateUserUI(user);
      }
    } catch (e) {}
  }

  // Always fetch fresh user data
  if (document.readyState === "loading") {
    (function(){ var __cb = loadUserProfile; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    loadUserProfile();
  }
})();
