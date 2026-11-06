/* General Bots content script — module 1/5: state, settings and helpers
   Split from botplugin/content.js for #1731. Loaded first by manifest.json. */
(function () {
  "use strict";

  const GB = (globalThis.__gbContent = globalThis.__gbContent || {});

  const settings = {
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

  GB.settings = settings;
  GB.state = state;
  GB.SELECTORS = SELECTORS;

  async function init() {
    if (state.initialized) return;

    console.log("General Bots: Initializing content script...");

    await loadSettings();
    GB.applyUIModifications();
    GB.setupInputListener();
    GB.setupMessageObserver();
    GB.setupContactObserver();
    GB.injectControlPanel();

    state.initialized = true;
    console.log("General Bots: Content script initialized");
  }

  async function loadSettings() {
    return new Promise((resolve) => {
      chrome.storage.sync.get(settings, (items) => {
        Object.assign(settings, items);
        console.log("General Bots: Settings loaded", settings);
        resolve(settings);
      });
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

  GB.init = init;
  GB.loadSettings = loadSettings;
  GB.saveSettings = saveSettings;
  GB.getCurrentContactName = getCurrentContactName;
  GB.setInputText = setInputText;
  GB.simulateEnterPress = simulateEnterPress;
  GB.escapeHtml = escapeHtml;
  GB.levenshteinDistance = levenshteinDistance;
})();
