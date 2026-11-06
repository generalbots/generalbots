/* General Bots content script — module 5/5: runtime messages and boot
   Split from botplugin/content.js for #1731. Loaded last by manifest.json. */
(function () {
  "use strict";

  const GB = globalThis.__gbContent;
  const settings = GB.settings;

  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    switch (message.action) {
      case "tabReady":
        GB.init();
        break;

      case "settingsUpdated":
        Object.assign(settings, message.settings);
        GB.applyUIModifications();
        break;

      case "enableAutoMode":
        settings.autoMode = true;
        settings.whatsappNumber = message.whatsappNumber;
        GB.applyAutoModeIndicator();
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
          GB.injectControlPanel();
        }
        break;
    }

    sendResponse({ success: true });
    return true;
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", GB.init);
  } else {
    GB.init();
  }

  setTimeout(GB.init, 2000);

  // The parts registry is only needed while the modules above are loading.
  delete globalThis.__gbContent;
})();
