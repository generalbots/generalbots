/* General Bots content script — module 4/5: contact header controls
   Split from botplugin/content.js for #1731. Loaded fourth by manifest.json. */
(function () {
  "use strict";

  const GB = globalThis.__gbContent;
  const { state, SELECTORS } = GB;

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
    const contactName = GB.getCurrentContactName();
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

  GB.setupContactObserver = setupContactObserver;
  GB.injectContactControls = injectContactControls;
})();
