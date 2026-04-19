(function () {
  "use strict";

  let settings = {
    serverUrl: "https://api.generalbots.com",
    enableProcessing: true,
    hideContacts: false,
    autoMode: false,
    grammarCorrection: true,
    whatsappNumber: "",
    authenticated: false,
  };

  const state = {
    initialized: false,
    currentContact: null,
    autoModeContacts: new Set(),
    originalMessages: new Map(),
    processingQueue: [],
    isProcessing: false,
  };

  const SELECTORS = {
    contactList: "#pane-side",
    chatContainer: ".copyable-area",
    messageInput: 'div[contenteditable="true"][data-tab="10"]',
    sendButton: 'button[data-tab="11"]',
    messageOut: ".message-out",
    messageIn: ".message-in",
    messageText: ".selectable-text",
    contactName: "header ._amig span",
    chatHeader: "header._amid",
    conversationPanel: "#main",
    searchBox: 'div[data-tab="3"]',
  };

  async function init() {
    if (state.initialized) return;

    console.log("General Bots: Initializing content script...");

    await loadSettings();
    applyUIModifications();
    setupInputListener();
    setupMessageObserver();
    setupContactObserver();
    injectControlPanel();

    state.initialized = true;
    console.log("General Bots: Content script initialized");
  }

  async function loadSettings() {
    return new Promise((resolve) => {
      chrome.storage.sync.get(settings, (items) => {
        settings = { ...settings, ...items };
        console.log("General Bots: Settings loaded", settings);
        resolve(settings);
      });
    });
  }

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

  function setupInputListener() {
    const observer = new MutationObserver(() => {
      const inputField = document.querySelector(SELECTORS.messageInput);
      if (inputField && !inputField.getAttribute("gb-monitored")) {
        setupFieldMonitoring(inputField);
        inputField.setAttribute("gb-monitored", "true");
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });

    const inputField = document.querySelector(SELECTORS.messageInput);
    if (inputField && !inputField.getAttribute("gb-monitored")) {
      setupFieldMonitoring(inputField);
      inputField.setAttribute("gb-monitored", "true");
    }
  }

  function setupFieldMonitoring(inputField) {
    console.log("General Bots: Setting up input field monitoring");

    inputField.addEventListener("keydown", async (event) => {
      if (event.key === "Enter" && !event.shiftKey) {
        const originalText = inputField.textContent.trim();

        if (originalText.length > 0 && settings.enableProcessing) {
          if (settings.grammarCorrection) {
            event.preventDefault();
            event.stopPropagation();

            try {
              showProcessingIndicator(inputField);
              const result = await processMessageWithLLM(originalText);

              if (
                result.processedText &&
                result.processedText !== originalText
              ) {
                const shouldSend = await showCorrectionPreview(
                  originalText,
                  result.processedText,
                  inputField,
                );

                if (shouldSend) {
                  setInputText(inputField, result.processedText);
                  state.originalMessages.set(Date.now(), {
                    original: originalText,
                    corrected: result.processedText,
                  });
                }
              }

              hideProcessingIndicator();
              simulateEnterPress(inputField);
            } catch (error) {
              console.error("General Bots: Error processing message", error);
              hideProcessingIndicator();
              simulateEnterPress(inputField);
            }
          }
        }
      }
    });

    inputField.classList.add("gb-monitored-input");
  }

  async function processMessageWithLLM(text) {
    return new Promise((resolve) => {
      chrome.runtime.sendMessage(
        {
          action: "correctGrammar",
          text: text,
        },
        (response) => {
          if (chrome.runtime.lastError) {
            console.error(
              "General Bots: Runtime error",
              chrome.runtime.lastError,
            );
            resolve({ processedText: text });
            return;
          }
          resolve(response || { processedText: text });
        },
      );
    });
  }

  async function showCorrectionPreview(original, corrected, inputField) {
    return new Promise((resolve) => {
      if (levenshteinDistance(original, corrected) < 3) {
        resolve(true);
        return;
      }

      const modal = document.createElement("div");
      modal.className = "gb-correction-modal";
      modal.innerHTML = `
        <div class="gb-correction-content">
          <div class="gb-correction-header">
            <span class="gb-correction-icon">✨</span>
            <span>Grammar Correction</span>
          </div>
          <div class="gb-correction-body">
            <div class="gb-text-compare">
              <div class="gb-original">
                <label>Original:</label>
                <p>${escapeHtml(original)}</p>
              </div>
              <div class="gb-corrected">
                <label>Corrected:</label>
                <p>${escapeHtml(corrected)}</p>
              </div>
            </div>
          </div>
          <div class="gb-correction-actions">
            <button class="gb-btn gb-btn-secondary" id="gb-reject">Keep Original</button>
            <button class="gb-btn gb-btn-primary" id="gb-accept">Use Corrected</button>
          </div>
        </div>
      `;

      document.body.appendChild(modal);

      const autoClose = setTimeout(() => {
        modal.remove();
        resolve(true);
      }, 5000);

      document.getElementById("gb-accept").addEventListener("click", () => {
        clearTimeout(autoClose);
        modal.remove();
        resolve(true);
      });

      document.getElementById("gb-reject").addEventListener("click", () => {
        clearTimeout(autoClose);
        modal.remove();
        resolve(false);
      });
    });
  }

  function setupMessageObserver() {
    const observer = new MutationObserver((mutations) => {
      if (!settings.autoMode) return;

      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (node.nodeType === Node.ELEMENT_NODE) {
            const incomingMsg = node.querySelector
              ? node.querySelector(SELECTORS.messageIn)
              : null;
            if (
              incomingMsg ||
              (node.classList && node.classList.contains("message-in"))
            ) {
              handleIncomingMessage(incomingMsg || node);
            }
          }
        }
      }
    });

    const waitForChat = setInterval(() => {
      const chatContainer = document.querySelector(SELECTORS.chatContainer);
      if (chatContainer) {
        clearInterval(waitForChat);
        observer.observe(chatContainer, {
          childList: true,
          subtree: true,
        });
        console.log("General Bots: Message observer started");
      }
    }, 1000);
  }

  async function handleIncomingMessage(messageElement) {
    if (!settings.autoMode || !settings.authenticated) return;

    const currentContact = getCurrentContactName();
    if (!currentContact) return;

    if (!state.autoModeContacts.has(currentContact)) return;

    const messageText = messageElement.querySelector(SELECTORS.messageText);
    if (!messageText) return;

    const text = messageText.textContent.trim();
    if (!text) return;

    console.log(
      "General Bots: Processing incoming message for auto-reply",
      text,
    );

    const context = getConversationContext();

    chrome.runtime.sendMessage(
      {
        action: "generateAutoReply",
        context: {
          contact: currentContact,
          lastMessage: text,
        },
        lastMessages: context,
      },
      async (response) => {
        if (response && response.reply && response.autoSend) {
          await sendAutoReply(response.reply);
        }
      },
    );
  }

  function getConversationContext() {
    const messages = [];
    const messageElements = document.querySelectorAll(
      `${SELECTORS.messageIn}, ${SELECTORS.messageOut}`,
    );

    const recentMessages = Array.from(messageElements).slice(-10);

    for (const msg of recentMessages) {
      const textEl = msg.querySelector(SELECTORS.messageText);
      if (textEl) {
        messages.push({
          type: msg.classList.contains("message-out") ? "sent" : "received",
          text: textEl.textContent.trim(),
        });
      }
    }

    return messages;
  }

  async function sendAutoReply(text) {
    const inputField = document.querySelector(SELECTORS.messageInput);
    if (!inputField) return;

    setInputText(inputField, text);

    await new Promise((r) => setTimeout(r, 500));

    simulateEnterPress(inputField);
  }

  function setupContactObserver() {
    const observer = new MutationObserver(() => {
      const header = document.querySelector(SELECTORS.chatHeader);
      if (header && !header.querySelector(".gb-contact-controls")) {
        injectContactControls(header);
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  }

  function injectContactControls(header) {
    const contactName = getCurrentContactName();
    if (!contactName) return;

    const controls = document.createElement("div");
    controls.className = "gb-contact-controls";

    const isAutoEnabled = state.autoModeContacts.has(contactName);

    controls.innerHTML = `
      <button class="gb-contact-btn ${isAutoEnabled ? "active" : ""}"
              id="gb-toggle-auto"
              title="Toggle Auto Mode for this contact">
        <span class="gb-icon">🤖</span>
        <span class="gb-label">${isAutoEnabled ? "Auto ON" : "Auto OFF"}</span>
      </button>
    `;

    header.appendChild(controls);

    document
      .getElementById("gb-toggle-auto")
      .addEventListener("click", function () {
        if (state.autoModeContacts.has(contactName)) {
          state.autoModeContacts.delete(contactName);
          this.classList.remove("active");
          this.querySelector(".gb-label").textContent = "Auto OFF";
        } else {
          state.autoModeContacts.add(contactName);
          this.classList.add("active");
          this.querySelector(".gb-label").textContent = "Auto ON";
        }
      });
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
        saveSettings();
      });

    document
      .getElementById("gb-contacts-toggle")
      ?.addEventListener("change", function () {
        settings.hideContacts = this.checked;
        applyContactVisibility();
        saveSettings();
      });

    document
      .getElementById("gb-auto-toggle")
      ?.addEventListener("change", function () {
        settings.autoMode = this.checked;
        applyAutoModeIndicator();
        saveSettings();
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

  function saveSettings() {
    chrome.storage.sync.set(settings);
    chrome.runtime.sendMessage({
      action: "saveSettings",
      settings: settings,
    });
  }

  function getCurrentContactName() {
    const nameEl = document.querySelector(SELECTORS.contactName);
    return nameEl ? nameEl.textContent.trim() : null;
  }

  function setInputText(inputField, text) {
    inputField.textContent = text;
    inputField.dispatchEvent(new InputEvent("input", { bubbles: true }));
  }

  function simulateEnterPress(element) {
    const enterEvent = new KeyboardEvent("keydown", {
      key: "Enter",
      code: "Enter",
      keyCode: 13,
      which: 13,
      bubbles: true,
      cancelable: true,
    });
    element.dispatchEvent(enterEvent);
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

  function escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function levenshteinDistance(str1, str2) {
    const m = str1.length;
    const n = str2.length;
    const dp = Array(m + 1)
      .fill(null)
      .map(() => Array(n + 1).fill(0));

    for (let i = 0; i <= m; i++) dp[i][0] = i;
    for (let j = 0; j <= n; j++) dp[0][j] = j;

    for (let i = 1; i <= m; i++) {
      for (let j = 1; j <= n; j++) {
        if (str1[i - 1] === str2[j - 1]) {
          dp[i][j] = dp[i - 1][j - 1];
        } else {
          dp[i][j] = Math.min(dp[i - 1][j - 1], dp[i - 1][j], dp[i][j - 1]) + 1;
        }
      }
    }

    return dp[m][n];
  }

  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    switch (message.action) {
      case "tabReady":
        init();
        break;

      case "settingsUpdated":
        settings = { ...settings, ...message.settings };
        applyUIModifications();
        break;

      case "enableAutoMode":
        settings.autoMode = true;
        settings.whatsappNumber = message.whatsappNumber;
        applyAutoModeIndicator();
        break;

      case "replaceSelection":
        const selection = window.getSelection();
        if (selection.rangeCount > 0) {
          const range = selection.getRangeAt(0);
          range.deleteContents();
          range.insertNode(document.createTextNode(message.text));
        }
        break;

      case "authCompleted":
        settings.authenticated = true;
        const panel = document.getElementById("gb-control-panel");
        if (panel) {
          panel.remove();
          injectControlPanel();
        }
        break;
    }

    sendResponse({ success: true });
    return true;
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  setTimeout(init, 2000);
})();
