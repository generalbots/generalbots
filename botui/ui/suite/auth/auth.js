/* Auth Module JavaScript - Login, Register, Forgot Password, Reset Password */

/**
 * Toggle password visibility
 * @param {string} inputId - ID of the password input
 * @param {string} eyeIconId - ID of the eye icon
 * @param {string} eyeOffIconId - ID of the eye-off icon
 */
function togglePassword(
  inputId = "password",
  eyeIconId = "eye-icon",
  eyeOffIconId = "eye-off-icon",
) {
  const passwordInput = document.getElementById(inputId);
  const eyeIcon = document.getElementById(eyeIconId);
  const eyeOffIcon = document.getElementById(eyeOffIconId);

  if (passwordInput.type === "password") {
    passwordInput.type = "text";
    if (eyeIcon) eyeIcon.style.display = "none";
    if (eyeOffIcon) eyeOffIcon.style.display = "block";
  } else {
    passwordInput.type = "password";
    if (eyeIcon) eyeIcon.style.display = "block";
    if (eyeOffIcon) eyeOffIcon.style.display = "none";
  }
}

/**
 * Initiate OAuth login flow
 * @param {string} provider - OAuth provider name (google, microsoft, github, apple)
 */
function oauthLogin(provider) {
  window.location.href = `/api/auth/oauth/${provider}`;
}

/**
 * Show 2FA challenge section
 * @param {string} sessionToken - Session token for 2FA verification
 */
function showTwoFAChallenge(sessionToken) {
  document.getElementById("login-section").style.display = "none";
  document.getElementById("twofa-section").classList.add("visible");
  document.getElementById("session-token").value = sessionToken;

  // Focus first code input
  const firstInput = document.querySelector('.code-input[data-index="0"]');
  if (firstInput) firstInput.focus();
}

/**
 * Return to login section from 2FA
 */
function backToLogin() {
  document.getElementById("login-section").style.display = "block";
  document.getElementById("twofa-section").classList.remove("visible");

  // Clear code inputs
  document.querySelectorAll(".code-input").forEach((input) => {
    input.value = "";
    input.classList.remove("filled");
  });
}

/**
 * Update the hidden full code field from individual inputs
 */
function updateFullCode() {
  const codeInputs = document.querySelectorAll(".code-input");
  const code = Array.from(codeInputs)
    .map((input) => input.value)
    .join("");
  const fullCodeInput = document.getElementById("full-code");
  if (fullCodeInput) fullCodeInput.value = code;
}

/**
 * Initialize 2FA code input handling
 */
function initCodeInputs() {
  const codeInputs = document.querySelectorAll(".code-input");

  codeInputs.forEach((input, index) => {
    input.addEventListener("input", (e) => {
      const value = e.target.value;

      // Only allow numbers
      e.target.value = value.replace(/[^0-9]/g, "");

      if (e.target.value) {
        e.target.classList.add("filled");
        // Move to next input
        if (index < codeInputs.length - 1) {
          codeInputs[index + 1].focus();
        }
      } else {
        e.target.classList.remove("filled");
      }

      updateFullCode();
    });

    input.addEventListener("keydown", (e) => {
      // Handle backspace
      if (e.key === "Backspace" && !e.target.value && index > 0) {
        codeInputs[index - 1].focus();
      }

      // Handle paste
      if (e.key === "v" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        navigator.clipboard.readText().then((text) => {
          const code = text.replace(/[^0-9]/g, "").slice(0, 6);
          code.split("").forEach((char, i) => {
            if (codeInputs[i]) {
              codeInputs[i].value = char;
              codeInputs[i].classList.add("filled");
            }
          });
          updateFullCode();
          if (code.length === 6) {
            codeInputs[5].focus();
          }
        });
      }
    });

    // Handle paste directly on input
    input.addEventListener("paste", (e) => {
      e.preventDefault();
      const text = e.clipboardData.getData("text");
      const code = text.replace(/[^0-9]/g, "").slice(0, 6);
      code.split("").forEach((char, i) => {
        if (codeInputs[i]) {
          codeInputs[i].value = char;
          codeInputs[i].classList.add("filled");
        }
      });
      updateFullCode();
      if (code.length === 6) {
        codeInputs[5].focus();
      }
    });
  });
}

/**
 * Resend 2FA code with cooldown
 */
let resendCooldown = 0;
function resendCode() {
  if (resendCooldown > 0) return;

  const sessionToken = document.getElementById("session-token").value;
  fetch("/api/auth/2fa/resend", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ session_token: sessionToken }),
  });

  // Start cooldown
  resendCooldown = 60;
  const resendBtn = document.getElementById("resend-btn");
  resendBtn.disabled = true;

  const interval = setInterval(() => {
    resendCooldown--;
    resendBtn.textContent = `Resend code (${resendCooldown}s)`;

    if (resendCooldown <= 0) {
      clearInterval(interval);
      resendBtn.textContent = "Resend code";
      resendBtn.disabled = false;
    }
  }, 1000);
}

/**
 * Show error message
 * @param {string} message - Error message to display
 */
function showError(message) {
  const errorBox = document.getElementById("error-message");
  const errorText = document.getElementById("error-text");
  if (errorText) errorText.textContent = message;
  if (errorBox) errorBox.classList.add("visible");
}

/**
 * Hide error message
 */
function hideError() {
  const errorBox = document.getElementById("error-message");
  if (errorBox) errorBox.classList.remove("visible");
}

/**
 * Show success message
 * @param {string} message - Success message to display
 */
function showSuccess(message) {
  const successBox = document.getElementById("success-message");
  const successText = document.getElementById("success-text");
  if (successText) successText.textContent = message;
  if (successBox) successBox.classList.add("visible");
}

/**
 * Set loading state on a button
 * @param {string} btnId - Button ID
 * @param {boolean} loading - Loading state
 */
function setLoading(btnId, loading) {
  const btn = document.getElementById(btnId);
  if (!btn) return;

  if (loading) {
    btn.classList.add("loading");
    btn.disabled = true;
  } else {
    btn.classList.remove("loading");
    btn.disabled = false;
  }
}

/**
 * Check password strength
 * @param {string} password - Password to check
 * @returns {object} - Strength level and requirements met
 */
function checkPasswordStrength(password) {
  const requirements = {
    length: password.length >= 8,
    lowercase: /[a-z]/.test(password),
    uppercase: /[A-Z]/.test(password),
    number: /[0-9]/.test(password),
    special: /[!@#$%^&*(),.?":{}|<>]/.test(password),
  };

  const metCount = Object.values(requirements).filter(Boolean).length;

  let strength = "weak";
  if (metCount >= 5) strength = "strong";
  else if (metCount >= 4) strength = "good";
  else if (metCount >= 3) strength = "fair";

  return { strength, requirements, metCount };
}

/**
 * Update password strength indicator
 * @param {string} password - Password to check
 */
function updatePasswordStrength(password) {
  const { strength, requirements } = checkPasswordStrength(password);

  const strengthFill = document.querySelector(".strength-fill");
  const strengthText = document.querySelector(".strength-text");

  if (strengthFill) {
    strengthFill.className = "strength-fill " + strength;
  }

  if (strengthText) {
    const labels = {
      weak: "Weak",
      fair: "Fair",
      good: "Good",
      strong: "Strong",
    };
    strengthText.textContent = labels[strength];
  }

  // Update requirement indicators
  Object.entries(requirements).forEach(([key, met]) => {
    const reqEl = document.querySelector(`.requirement[data-req="${key}"]`);
    if (reqEl) {
      reqEl.classList.toggle("met", met);
    }
  });
}

/**
 * Initialize password strength checker
 */
function initPasswordStrength() {
  const passwordInput = document.getElementById("password");
  if (passwordInput) {
    passwordInput.addEventListener("input", (e) => {
      updatePasswordStrength(e.target.value);
    });
  }
}

/**
 * Handle HTMX events for auth forms
 */
function initHtmxHandlers() {
  document.body.addEventListener("htmx:beforeRequest", function (event) {
    hideError();
    if (event.target.id === "login-form") {
      setLoading("login-btn", true);
    } else if (event.target.id === "twofa-form") {
      setLoading("verify-btn", true);
    } else if (event.target.id === "register-form") {
      setLoading("register-btn", true);
    } else if (event.target.id === "forgot-form") {
      setLoading("forgot-btn", true);
    } else if (event.target.id === "reset-form") {
      setLoading("reset-btn", true);
    }
  });

  document.body.addEventListener("htmx:afterRequest", function (event) {
    if (event.target.id === "login-form") {
      setLoading("login-btn", false);
    } else if (event.target.id === "twofa-form") {
      setLoading("verify-btn", false);
    } else if (event.target.id === "register-form") {
      setLoading("register-btn", false);
    } else if (event.target.id === "forgot-form") {
      setLoading("forgot-btn", false);
    } else if (event.target.id === "reset-form") {
      setLoading("reset-btn", false);
    }

    if (event.detail.successful) {
      try {
        const response = JSON.parse(event.detail.xhr.responseText);

        // Check if 2FA is required
        if (response.requires_2fa) {
          showTwoFAChallenge(response.session_token);
          return;
        }

        // Save token using GBAuth service if available
        if (response.access_token) {
          const rememberMe = document.getElementById("remember");
          const remember = rememberMe && rememberMe.checked;

          if (window.AuthService && window.AuthService.storeTokens) {
            window.AuthService.storeTokens(
              response.access_token,
              response.refresh_token,
              response.expires_in,
              remember,
            );
            if (response.user_id) {
              window.AuthService.currentUser = { id: response.user_id };
            }
          } else {
            // Fallback to direct storage with correct keys
            const storage = remember ? localStorage : sessionStorage;
            storage.setItem("gb-access-token", response.access_token);
            if (response.refresh_token) {
              storage.setItem("gb-refresh-token", response.refresh_token);
            }
            if (response.expires_in) {
              const expiresAt = Date.now() + response.expires_in * 1000;
              storage.setItem("gb-token-expires", expiresAt.toString());
            }
          }
        }

        // Successful login/register - redirect
        if (response.redirect || response.success) {
          window.location.href = response.redirect || "/";
        }

        // Show success message
        if (response.message) {
          showSuccess(response.message);
        }
      } catch (e) {
        // If response is not JSON, check for redirect header
        if (event.detail.xhr.status === 200) {
          window.location.href = "/";
        }
      }
    } else {
      // Show error
      try {
        const response = JSON.parse(event.detail.xhr.responseText);
        showError(response.error || "An error occurred. Please try again.");
      } catch (e) {
        showError("An error occurred. Please try again.");
      }
    }
  });
}

/**
 * Initialize auth module
 */
function initAuth() {
  initCodeInputs();
  initPasswordStrength();
  initHtmxHandlers();

  // Clear error when user starts typing
  document.querySelectorAll(".form-input").forEach((input) => {
    input.addEventListener("input", hideError);
  });
}

// Auto-initialize when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAuth);
} else {
  initAuth();
}
