/* General Bots content script — module 3/5: input handling, grammar and auto-reply
   Split from botplugin/content.js for #1731. Loaded third by manifest.json. */
(function () {
  "use strict";

  const GB = globalThis.__gbContent;
  const { settings, state, SELECTORS } = GB;

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
              GB.showProcessingIndicator(inputField);
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
                  GB.setInputText(inputField, result.processedText);
                  state.originalMessages.set(Date.now(), {
                    original: originalText,
                    corrected: result.processedText,
                  });
                }
              }

              GB.hideProcessingIndicator();
              GB.simulateEnterPress(inputField);
            } catch (error) {
              console.error("General Bots: Error processing message", error);
              GB.hideProcessingIndicator();
              GB.simulateEnterPress(inputField);
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
      if (GB.levenshteinDistance(original, corrected) < 3) {
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
                <p>${GB.escapeHtml(original)}</p>
              </div>
              <div class="gb-corrected">
                <label>Corrected:</label>
                <p>${GB.escapeHtml(corrected)}</p>
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

    const currentContact = GB.getCurrentContactName();
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

    GB.setInputText(inputField, text);

    await new Promise((r) => setTimeout(r, 500));

    GB.simulateEnterPress(inputField);
  }

  GB.setupInputListener = setupInputListener;
  GB.setupMessageObserver = setupMessageObserver;
})();
