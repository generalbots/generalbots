/* General Bots content script — module 2/5: WhatsApp UI and control panel
   Split from botplugin/content.js for #1731. Loaded second by manifest.json. */
(function () {
  "use strict";

  const GB = globalThis.__gbContent;
  const { settings, SELECTORS } = GB;

  function applyUIModifications() {
    applyContactVisibility();
    applyAutoModeIndicator();
  }

  function applyContactVisibility() {
    const contactList = document.querySelector(SELECTORS.contactList);
    if (contactList) {
      const parent = contactList.parentElement;
      if (settings.hideContacts) {
        parent.classList.add("gb-hide-contacts");
        parent.style.cssText =
          "width: 0 !important; min-width: 0 !important; overflow: hidden !important;";
      } else {
        parent.classList.remove("gb-hide-contacts");
        parent.style.cssText = "";
      }
    }
  }

  function applyAutoModeIndicator() {
    let indicator = document.getElementById("gb-auto-mode-indicator");

    if (settings.autoMode) {
      if (!indicator) {
        indicator = document.createElement("div");
        indicator.id = "gb-auto-mode-indicator";
        indicator.className = "gb-auto-indicator";
        indicator.innerHTML = `
          <span class="gb-auto-dot"></span>
          <span>Auto Mode Active</span>
        `;
        document.body.appendChild(indicator);
      }
      indicator.style.display = "flex";
    } else if (indicator) {
      indicator.style.display = "none";
    }
  }

  function injectControlPanel() {
    const panel = document.createElement("div");
    panel.id = "gb-control-panel";
    panel.className = "gb-panel";
    panel.innerHTML = `
      <div class="gb-panel-header">
        <img src="${chrome.runtime.getURL("icons/icon48.png")}" alt="GB" class="gb-panel-logo">
        <span>General Bots</span>
        <button class="gb-panel-toggle" id="gb-panel-toggle">−</button>
      </div>
      <div class="gb-panel-body" id="gb-panel-body">
        <div class="gb-status ${settings.authenticated ? "connected" : "disconnected"}">
          <span class="gb-status-dot"></span>
          <span>${settings.authenticated ? "Connected" : "Not Connected"}</span>
        </div>

        <div class="gb-controls">
          <label class="gb-switch-label">
            <span>Grammar Correction</span>
            <input type="checkbox" id="gb-grammar-toggle" ${settings.grammarCorrection ? "checked" : ""}>
            <span class="gb-switch"></span>
          </label>

          <label class="gb-switch-label">
            <span>Hide Contacts</span>
            <input type="checkbox" id="gb-contacts-toggle" ${settings.hideContacts ? "checked" : ""}>
            <span class="gb-switch"></span>
          </label>

          <label class="gb-switch-label">
            <span>Auto Mode</span>
            <input type="checkbox" id="gb-auto-toggle" ${settings.autoMode ? "checked" : ""}>
            <span class="gb-switch"></span>
          </label>
        </div>

        ${
          !settings.authenticated
            ? `
          <div class="gb-auth-section">
            <p>Connect with your General Bots account:</p>
            <input type="text" id="gb-whatsapp-number" placeholder="Your WhatsApp number" value="${settings.whatsappNumber}">
            <button class="gb-btn gb-btn-primary" id="gb-auth-btn">Authenticate</button>
          </div>
        `
            : ""
        }
      </div>
    `;

    document.body.appendChild(panel);
    setupPanelListeners();
  }

  function setupPanelListeners() {
    document
      .getElementById("gb-panel-toggle")
      ?.addEventListener("click", function () {
        const body = document.getElementById("gb-panel-body");
        if (body.style.display === "none") {
          body.style.display = "block";
          this.textContent = "−";
        } else {
          body.style.display = "none";
          this.textContent = "+";
        }
      });

    document
      .getElementById("gb-grammar-toggle")
      ?.addEventListener("change", function () {
        settings.grammarCorrection = this.checked;
        GB.saveSettings();
      });

    document
      .getElementById("gb-contacts-toggle")
      ?.addEventListener("change", function () {
        settings.hideContacts = this.checked;
        applyContactVisibility();
        GB.saveSettings();
      });

    document
      .getElementById("gb-auto-toggle")
      ?.addEventListener("change", function () {
        settings.autoMode = this.checked;
        applyAutoModeIndicator();
        GB.saveSettings();
      });

    document
      .getElementById("gb-auth-btn")
      ?.addEventListener("click", async function () {
        const numberInput = document.getElementById("gb-whatsapp-number");
        const number = numberInput?.value.trim();

        if (!number) {
          alert("Please enter your WhatsApp number");
          return;
        }

        this.disabled = true;
        this.textContent = "Authenticating...";

        chrome.runtime.sendMessage(
          {
            action: "authenticate",
            whatsappNumber: number,
          },
          (response) => {
            if (response.success) {
              this.textContent = "Check WhatsApp";
            } else {
              this.textContent = "Authenticate";
              this.disabled = false;
              alert(
                "Authentication failed: " + (response.error || "Unknown error"),
              );
            }
          },
        );
      });
  }

  function showProcessingIndicator(inputField) {
    let indicator = document.getElementById("gb-processing");
    if (!indicator) {
      indicator = document.createElement("div");
      indicator.id = "gb-processing";
      indicator.className = "gb-processing-indicator";
      indicator.innerHTML = `
        <div class="gb-spinner"></div>
        <span>Processing with AI...</span>
      `;
      inputField.parentElement.appendChild(indicator);
    }
    indicator.style.display = "flex";
  }

  function hideProcessingIndicator() {
    const indicator = document.getElementById("gb-processing");
    if (indicator) {
      indicator.style.display = "none";
    }
  }

  GB.applyUIModifications = applyUIModifications;
  GB.applyAutoModeIndicator = applyAutoModeIndicator;
  GB.injectControlPanel = injectControlPanel;
  GB.showProcessingIndicator = showProcessingIndicator;
  GB.hideProcessingIndicator = hideProcessingIndicator;
})();
