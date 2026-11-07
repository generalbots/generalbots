// Save settings
document
    .getElementById("btn-save")
    .addEventListener("click", () => {
        const settings = {
            serverUrl: document.getElementById("server-url").value,
            gbServerUrl:
                document.getElementById("gb-server-url").value,
            instanceId: document.getElementById("instance-id").value,
            apiTimeout: parseInt(
                document.getElementById("api-timeout").value,
            ),
            whatsappNumber:
                document.getElementById("whatsapp-number").value,
            grammarCorrection:
                document.getElementById("grammar-correction")
                    .checked,
            showPreview:
                document.getElementById("show-preview").checked,
            correctionLanguage: document.getElementById(
                "correction-language",
            ).value,
            autoMode: document.getElementById("auto-mode").checked,
            autoApproval:
                document.getElementById("auto-approval").checked,
            responseStyle:
                document.getElementById("response-style").value,
            customInstructions: document.getElementById(
                "custom-instructions",
            ).value,
            hideContacts:
                document.getElementById("hide-contacts").checked,
            hideIndicators:
                document.getElementById("hide-indicators").checked,
            localOnly:
                document.getElementById("local-only").checked,
            storeOriginals:
                document.getElementById("store-originals").checked,
            debugMode:
                document.getElementById("debug-mode").checked,
            showPanel:
                document.getElementById("show-panel").checked,
            shortcut: document.getElementById("shortcut").value,
        };

        chrome.storage.sync.set(settings, () => {
            showNotification("Settings saved successfully!");
        });
    });

// Reset settings
document
    .getElementById("btn-reset")
    .addEventListener("click", () => {
        if (
            confirm(
                "Are you sure you want to reset all settings to defaults?",
            )
        ) {
            chrome.storage.sync.clear(() => {
                loadSettings();
                showNotification("Settings reset to defaults");
            });
        }
    });

// Clear data
document
    .getElementById("btn-clear")
    .addEventListener("click", () => {
        if (
            confirm(
                "Are you sure you want to clear all stored data?",
            )
        ) {
            chrome.storage.local.clear(() => {
                showNotification("All data cleared");
            });
        }
    });

// Show notification
function showNotification(message, type = "success") {
    const existing = document.querySelector(".notification");
    if (existing) existing.remove();

    const notification = document.createElement("div");
    notification.className = `notification ${type}`;
    notification.textContent = message;
    document.body.appendChild(notification);

    setTimeout(() => notification.remove(), 3000);
}

// Cancel button
document
    .getElementById("btn-cancel")
    .addEventListener("click", () => {
        window.close();
    });

// Initialize
loadSettings();
