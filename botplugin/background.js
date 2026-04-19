const DEFAULT_CONFIG = {
  serverUrl: "https://api.generalbots.com",
  gbServerUrl: "https://api.pragmatismo.com.br",
  enableProcessing: true,
  hideContacts: false,
  autoMode: false,
  grammarCorrection: true,
  whatsappNumber: "",
  authToken: "",
  instanceId: "",
};

chrome.runtime.onInstalled.addListener(async (details) => {
  console.log("General Bots: Extension installed/updated", details.reason);

  const existing = await chrome.storage.sync.get(DEFAULT_CONFIG);
  await chrome.storage.sync.set({ ...DEFAULT_CONFIG, ...existing });

  chrome.contextMenus?.create({
    id: "gb-correct-grammar",
    title: "Correct Grammar with AI",
    contexts: ["selection"],
  });

  chrome.contextMenus?.create({
    id: "gb-translate",
    title: "Translate with AI",
    contexts: ["selection"],
  });
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (
    changeInfo.status === "complete" &&
    tab.url?.includes("web.whatsapp.com")
  ) {
    console.log("General Bots: WhatsApp Web detected, initializing...");

    chrome.tabs.sendMessage(tabId, { action: "tabReady" }).catch(() => {});

    checkAutoAuth(tabId);
  }
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log("General Bots: Received message", message.action);

  switch (message.action) {
    case "processText":
      handleProcessText(message.text, message.options)
        .then(sendResponse)
        .catch((err) => sendResponse({ error: err.message }));
      return true;

    case "correctGrammar":
      handleGrammarCorrection(message.text)
        .then(sendResponse)
        .catch((err) => sendResponse({ error: err.message }));
      return true;

    case "authenticate":
      handleAuthentication(message.whatsappNumber)
        .then(sendResponse)
        .catch((err) => sendResponse({ error: err.message }));
      return true;

    case "getAuthStatus":
      getAuthStatus()
        .then(sendResponse)
        .catch((err) => sendResponse({ error: err.message }));
      return true;

    case "generateAutoReply":
      handleAutoReply(message.context, message.lastMessages)
        .then(sendResponse)
        .catch((err) => sendResponse({ error: err.message }));
      return true;

    case "getSettings":
      chrome.storage.sync.get(DEFAULT_CONFIG).then(sendResponse);
      return true;

    case "saveSettings":
      chrome.storage.sync.set(message.settings).then(() => {
        broadcastSettingsUpdate(message.settings);
        sendResponse({ success: true });
      });
      return true;

    case "showNotification":
      showNotification(message.title, message.message, message.type);
      sendResponse({ success: true });
      return false;
  }

  return false;
});

chrome.contextMenus?.onClicked.addListener(async (info, tab) => {
  if (!info.selectionText) return;

  switch (info.menuItemId) {
    case "gb-correct-grammar":
      const corrected = await handleGrammarCorrection(info.selectionText);
      if (corrected.processedText && tab?.id) {
        chrome.tabs.sendMessage(tab.id, {
          action: "replaceSelection",
          text: corrected.processedText,
        });
      }
      break;

    case "gb-translate":
      break;
  }
});

async function handleProcessText(text, options = {}) {
  const settings = await chrome.storage.sync.get(DEFAULT_CONFIG);

  if (!settings.enableProcessing) {
    return { processedText: text, changed: false };
  }

  try {
    const response = await fetch(`${settings.serverUrl}/api/v1/llm/process`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${settings.authToken}`,
      },
      body: JSON.stringify({
        text,
        instanceId: settings.instanceId,
        options: {
          grammarCorrection: settings.grammarCorrection,
          ...options,
        },
      }),
    });

    if (!response.ok) {
      throw new Error(`Server error: ${response.status}`);
    }

    const data = await response.json();
    return {
      processedText: data.processedText || text,
      changed: data.processedText !== text,
      corrections: data.corrections || [],
    };
  } catch (error) {
    console.error("General Bots: Process text error", error);
    return { processedText: text, changed: false, error: error.message };
  }
}

async function handleGrammarCorrection(text) {
  const settings = await chrome.storage.sync.get(DEFAULT_CONFIG);

  try {
    const response = await fetch(`${settings.serverUrl}/api/v1/llm/grammar`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${settings.authToken}`,
      },
      body: JSON.stringify({
        text,
        instanceId: settings.instanceId,
        language: "auto",
      }),
    });

    if (!response.ok) {
      throw new Error(`Server error: ${response.status}`);
    }

    const data = await response.json();
    return {
      processedText: data.correctedText || text,
      original: text,
      corrections: data.corrections || [],
      language: data.detectedLanguage,
    };
  } catch (error) {
    console.error("General Bots: Grammar correction error", error);
    return { processedText: text, error: error.message };
  }
}

async function handleAutoReply(context, lastMessages = []) {
  const settings = await chrome.storage.sync.get(DEFAULT_CONFIG);

  if (!settings.autoMode) {
    return { reply: null, autoModeDisabled: true };
  }

  try {
    const response = await fetch(
      `${settings.serverUrl}/api/v1/llm/auto-reply`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${settings.authToken}`,
        },
        body: JSON.stringify({
          context,
          lastMessages,
          instanceId: settings.instanceId,
          whatsappNumber: settings.whatsappNumber,
        }),
      },
    );

    if (!response.ok) {
      throw new Error(`Server error: ${response.status}`);
    }

    const data = await response.json();
    return {
      reply: data.suggestedReply,
      confidence: data.confidence,
      autoSend: data.autoSend && settings.autoMode,
    };
  } catch (error) {
    console.error("General Bots: Auto-reply error", error);
    return { reply: null, error: error.message };
  }
}

async function handleAuthentication(whatsappNumber) {
  const settings = await chrome.storage.sync.get(DEFAULT_CONFIG);

  try {
    const response = await fetch(
      `${settings.gbServerUrl}/api/v1/auth/whatsapp/request`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          whatsappNumber,
          extensionId: chrome.runtime.id,
          timestamp: Date.now(),
        }),
      },
    );

    if (!response.ok) {
      throw new Error(`Authentication request failed: ${response.status}`);
    }

    const data = await response.json();

    await chrome.storage.sync.set({
      whatsappNumber,
      authPending: true,
      authRequestId: data.requestId,
    });

    showNotification(
      "Authentication Requested",
      "Check your WhatsApp for a message from General Bots to complete authentication.",
      "info",
    );

    pollAuthCompletion(data.requestId);

    return { success: true, requestId: data.requestId };
  } catch (error) {
    console.error("General Bots: Authentication error", error);
    return { success: false, error: error.message };
  }
}

async function pollAuthCompletion(requestId, attempts = 0) {
  if (attempts > 60) {
    await chrome.storage.sync.set({ authPending: false });
    showNotification("Authentication Timeout", "Please try again.", "error");
    return;
  }

  const settings = await chrome.storage.sync.get(DEFAULT_CONFIG);

  try {
    const response = await fetch(
      `${settings.gbServerUrl}/api/v1/auth/whatsapp/status/${requestId}`,
    );

    if (response.ok) {
      const data = await response.json();

      if (data.status === "completed") {
        await chrome.storage.sync.set({
          authToken: data.token,
          instanceId: data.instanceId,
          authPending: false,
          authenticated: true,
        });

        showNotification(
          "Authentication Complete",
          "You are now connected to General Bots!",
          "success",
        );

        broadcastSettingsUpdate({ authenticated: true });
        return;
      } else if (data.status === "failed") {
        await chrome.storage.sync.set({ authPending: false });
        showNotification(
          "Authentication Failed",
          data.message || "Please try again.",
          "error",
        );
        return;
      }
    }
  } catch (error) {
    console.error("General Bots: Poll auth error", error);
  }

  setTimeout(() => pollAuthCompletion(requestId, attempts + 1), 5000);
}

async function getAuthStatus() {
  const settings = await chrome.storage.sync.get([
    "authToken",
    "authenticated",
    "whatsappNumber",
    "instanceId",
  ]);

  if (!settings.authToken) {
    return { authenticated: false };
  }

  try {
    const response = await fetch(
      `${DEFAULT_CONFIG.gbServerUrl}/api/v1/auth/verify`,
      {
        headers: {
          Authorization: `Bearer ${settings.authToken}`,
        },
      },
    );

    if (response.ok) {
      return {
        authenticated: true,
        whatsappNumber: settings.whatsappNumber,
        instanceId: settings.instanceId,
      };
    }
  } catch (error) {
    console.error("General Bots: Verify auth error", error);
  }

  await chrome.storage.sync.set({
    authToken: "",
    authenticated: false,
  });

  return { authenticated: false };
}

async function checkAutoAuth(tabId) {
  const settings = await chrome.storage.sync.get([
    "authenticated",
    "autoMode",
    "whatsappNumber",
  ]);

  if (settings.authenticated && settings.autoMode) {
    setTimeout(() => {
      chrome.tabs
        .sendMessage(tabId, {
          action: "enableAutoMode",
          whatsappNumber: settings.whatsappNumber,
        })
        .catch(() => {});
    }, 2000);
  }
}

async function broadcastSettingsUpdate(settings) {
  const tabs = await chrome.tabs.query({ url: "https://web.whatsapp.com/*" });

  for (const tab of tabs) {
    chrome.tabs
      .sendMessage(tab.id, {
        action: "settingsUpdated",
        settings,
      })
      .catch(() => {});
  }
}

function showNotification(title, message, type = "info") {
  const iconPath = type === "error" ? "icons/icon48.png" : "icons/icon48.png";

  chrome.notifications?.create({
    type: "basic",
    iconUrl: iconPath,
    title: `General Bots - ${title}`,
    message: message,
    priority: type === "error" ? 2 : 1,
  });
}

chrome.alarms?.create("checkAuth", { periodInMinutes: 30 });

chrome.alarms?.onAlarm.addListener(async (alarm) => {
  if (alarm.name === "checkAuth") {
    const status = await getAuthStatus();
    if (!status.authenticated) {
      console.log("General Bots: Auth token expired or invalid");
    }
  }
});

console.log("General Bots: Background service worker initialized");
