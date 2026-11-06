// Tab switching
document.querySelectorAll(".tab").forEach((tab) => {
    tab.addEventListener("click", function () {
        document
            .querySelectorAll(".tab")
            .forEach((t) => t.classList.remove("active"));
        document
            .querySelectorAll(".tab-content")
            .forEach((c) => c.classList.remove("active"));

        this.classList.add("active");
        document
            .getElementById(this.dataset.tab)
            .classList.add("active");
    });
});

// Load settings
async function loadSettings() {
    const defaults = {
        serverUrl: "https://api.generalbots.com",
        gbServerUrl: "https://api.pragmatismo.com.br",
        instanceId: "",
        apiTimeout: 10000,
        whatsappNumber: "",
        grammarCorrection: true,
        showPreview: true,
        correctionLanguage: "auto",
        autoMode: false,
        autoApproval: true,
        responseStyle: "professional",
        customInstructions: "",
        hideContacts: false,
        hideIndicators: false,
        localOnly: false,
        storeOriginals: true,
        analytics: false,
        debugMode: false,
        showPanel: true,
        shortcut: "Ctrl+Shift+G",
    };

    chrome.storage.sync.get(defaults, (items) => {
        document.getElementById("server-url").value =
            items.serverUrl;
        document.getElementById("gb-server-url").value =
            items.gbServerUrl;
        document.getElementById("instance-id").value =
            items.instanceId;
        document.getElementById("api-timeout").value =
            items.apiTimeout;
        document.getElementById("whatsapp-number").value =
            items.whatsappNumber;
        document.getElementById("grammar-correction").checked =
            items.grammarCorrection;
        document.getElementById("show-preview").checked =
            items.showPreview;
        document.getElementById("correction-language").value =
            items.correctionLanguage;
        document.getElementById("auto-mode").checked =
            items.autoMode;
        document.getElementById("auto-approval").checked =
            items.autoApproval;
        document.getElementById("response-style").value =
            items.responseStyle;
        document.getElementById("custom-instructions").value =
            items.customInstructions;
        document.getElementById("hide-contacts").checked =
            items.hideContacts;
        document.getElementById("hide-indicators").checked =
            items.hideIndicators;
        document.getElementById("local-only").checked =
            items.localOnly;
        document.getElementById("store-originals").checked =
            items.storeOriginals;
        document.getElementById("analytics").checked =
            items.analytics;
        document.getElementById("debug-mode").checked =
            items.debugMode;
        document.getElementById("show-panel").checked =
            items.showPanel;
        document.getElementById("shortcut").value = items.shortcut;
    });

    // Check auth status
    chrome.runtime.sendMessage(
        { action: "getAuthStatus" },
        (response) => {
            const statusEl =
                document.getElementById("auth-status");
            const badgeEl = document.getElementById("auth-badge");

            if (response && response.authenticated) {
                statusEl.textContent = `Connected as ${response.whatsappNumber || "User"}`;
                badgeEl.textContent = "Connected";
                badgeEl.className = "badge badge-success";
            } else {
                statusEl.textContent = "Not connected";
                badgeEl.textContent = "Disconnected";
                badgeEl.className = "badge badge-error";
            }
        },
    );
}
