/**
 * Feature Manager for General Bots Desktop
 * Manages dynamic feature toggling with Alpine.js
 * Syncs with backend feature flags and persists user preferences
 */

const FeatureManager = (function () {
    "use strict";

    // Feature definitions matching Cargo.toml features
    const FEATURES = {
        // UI Features
        "web-server": {
            name: "Web Server",
            category: "ui",
            description: "Web interface and static file serving",
            icon: "🌐",
            required: true,
            dependencies: [],
        },
        desktop: {
            name: "Desktop UI",
            category: "ui",
            description: "Native desktop application with Tauri",
            icon: "🖥️",
            required: false,
            dependencies: ["web-server"],
        },

        // Core Integrations
        vectordb: {
            name: "Vector Database",
            category: "core",
            description: "Semantic search and AI-powered indexing",
            icon: "🔍",
            required: false,
            dependencies: [],
        },
        llm: {
            name: "LLM/AI",
            category: "core",
            description: "Large Language Model integration",
            icon: "🤖",
            required: false,
            dependencies: [],
        },
        nvidia: {
            name: "NVIDIA GPU",
            category: "core",
            description: "GPU acceleration for AI workloads",
            icon: "⚡",
            required: false,
            dependencies: ["llm"],
        },

        // Communication Channels
        email: {
            name: "Email",
            category: "communication",
            description: "IMAP/SMTP email integration",
            icon: "📧",
            required: false,
            dependencies: [],
        },
        whatsapp: {
            name: "WhatsApp",
            category: "communication",
            description: "WhatsApp messaging integration",
            icon: "💬",
            required: false,
            dependencies: [],
        },
        instagram: {
            name: "Instagram",
            category: "communication",
            description: "Instagram DM integration",
            icon: "📸",
            required: false,
            dependencies: [],
        },
        msteams: {
            name: "Microsoft Teams",
            category: "communication",
            description: "Teams messaging integration",
            icon: "👥",
            required: false,
            dependencies: [],
        },

        // Productivity Features
        chat: {
            name: "Chat",
            category: "productivity",
            description: "Core chat messaging interface",
            icon: "💬",
            required: true,
            dependencies: [],
        },
        drive: {
            name: "Drive",
            category: "productivity",
            description: "File storage and management",
            icon: "📁",
            required: false,
            dependencies: [],
        },
        tasks: {
            name: "Tasks",
            category: "productivity",
            description: "Task management system",
            icon: "✓",
            required: false,
            dependencies: [],
        },
        calendar: {
            name: "Calendar",
            category: "productivity",
            description: "Calendar and scheduling",
            icon: "📅",
            required: false,
            dependencies: [],
        },
        meet: {
            name: "Meet",
            category: "productivity",
            description: "Video conferencing with LiveKit",
            icon: "📹",
            required: false,
            dependencies: [],
        },
        mail: {
            name: "Mail",
            category: "productivity",
            description: "Email client interface",
            icon: "✉️",
            required: false,
            dependencies: ["email"],
        },

        // Enterprise Features
        compliance: {
            name: "Compliance",
            category: "enterprise",
            description: "Audit logging and compliance tracking",
            icon: "📋",
            required: false,
            dependencies: [],
        },
        attendance: {
            name: "Attendance",
            category: "enterprise",
            description: "Employee attendance tracking",
            icon: "👤",
            required: false,
            dependencies: [],
        },
        directory: {
            name: "Directory",
            category: "enterprise",
            description: "LDAP/Active Directory integration",
            icon: "📖",
            required: false,
            dependencies: [],
        },
        weba: {
            name: "Web Automation",
            category: "enterprise",
            description: "Browser automation capabilities",
            icon: "🔧",
            required: false,
            dependencies: [],
        },
    };

    // Category display names
    const CATEGORIES = {
        ui: { name: "User Interface", icon: "🖥️" },
        core: { name: "Core Integrations", icon: "⚙️" },
        communication: { name: "Communication Channels", icon: "💬" },
        productivity: { name: "Productivity Apps", icon: "📊" },
        enterprise: { name: "Enterprise Features", icon: "🏢" },
    };

    // State management
    let enabledFeatures = new Set();
    let availableFeatures = new Set();
    let subscribers = [];

    /**
     * Initialize feature manager
     */
    async function init() {
        console.log("🚀 Initializing Feature Manager...");

        // Load enabled features from localStorage
        loadFromStorage();

        // Fetch available features from backend
        await fetchServerFeatures();

        // Notify subscribers
        notifySubscribers();

        console.log("✓ Feature Manager initialized");
        console.log(`  Enabled: ${Array.from(enabledFeatures).join(", ")}`);
    }

    /**
     * Load features from localStorage
     */
    function loadFromStorage() {
        try {
            const stored = localStorage.getItem("enabledFeatures");
            if (stored) {
                const parsed = JSON.parse(stored);
                enabledFeatures = new Set(parsed);
            } else {
                // Default features if nothing stored
                enabledFeatures = new Set(["web-server", "chat"]);
            }
        } catch (e) {
            console.error("Failed to load features from storage:", e);
            enabledFeatures = new Set(["web-server", "chat"]);
        }
    }

    /**
     * Save features to localStorage
     */
    function saveToStorage() {
        try {
            const array = Array.from(enabledFeatures);
            localStorage.setItem("enabledFeatures", JSON.stringify(array));
        } catch (e) {
            console.error("Failed to save features to storage:", e);
        }
    }

    /**
     * Fetch available features from server
     */
    async function fetchServerFeatures() {
        try {
            const response = await fetch("/api/features/available");
            if (response.ok) {
                const data = await response.json();
                availableFeatures = new Set(data.features || []);
                console.log(
                    "✓ Server features loaded:",
                    Array.from(availableFeatures).join(", ")
                );
            } else {
                // Fallback: assume all features available
                availableFeatures = new Set(Object.keys(FEATURES));
                console.warn("⚠ Could not fetch server features, using all");
            }
        } catch (e) {
            console.warn("⚠ Could not connect to server:", e.message);
            // Fallback: assume all features available
            availableFeatures = new Set(Object.keys(FEATURES));
        }
    }

    /**
     * Check if a feature is enabled
     */
    function isEnabled(featureId) {
        return enabledFeatures.has(featureId);
    }

    /**
     * Check if a feature is available (compiled in)
     */
    function isAvailable(featureId) {
        return availableFeatures.has(featureId);
    }

    /**
     * Enable a feature
     */
    async function enable(featureId) {
        const feature = FEATURES[featureId];
        if (!feature) {
            console.error(`Unknown feature: ${featureId}`);
            return false;
        }

        if (!isAvailable(featureId)) {
            console.error(
                `Feature not available (not compiled): ${featureId}`
            );
            return false;
        }

        // Check dependencies
        for (const dep of feature.dependencies) {
            if (!isEnabled(dep)) {
                console.log(
                    `Enabling dependency: ${dep} for ${featureId}`
                );
                await enable(dep);
            }
        }

        // Enable the feature
        enabledFeatures.add(featureId);
        saveToStorage();

        // Notify server
        await notifyServer(featureId, true);

        notifySubscribers();
        console.log(`✓ Feature enabled: ${featureId}`);
        return true;
    }

    /**
     * Disable a feature
     */
    async function disable(featureId) {
        const feature = FEATURES[featureId];
        if (!feature) {
            console.error(`Unknown feature: ${featureId}`);
            return false;
        }

        if (feature.required) {
            console.error(`Cannot disable required feature: ${featureId}`);
            return false;
        }

        // Check if any enabled feature depends on this
        for (const [id, f] of Object.entries(FEATURES)) {
            if (
                isEnabled(id) &&
                f.dependencies.includes(featureId)
            ) {
                console.log(
                    `Disabling dependent feature: ${id}`
                );
                await disable(id);
            }
        }

        // Disable the feature
        enabledFeatures.delete(featureId);
        saveToStorage();

        // Notify server
        await notifyServer(featureId, false);

        notifySubscribers();
        console.log(`✓ Feature disabled: ${featureId}`);
        return true;
    }

    /**
     * Toggle a feature on/off
     */
    async function toggle(featureId) {
        if (isEnabled(featureId)) {
            return await disable(featureId);
        } else {
            return await enable(featureId);
        }
    }

    /**
     * Notify server about feature change
     */
    async function notifyServer(featureId, enabled) {
        try {
            await fetch("/api/features/toggle", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    feature: featureId,
                    enabled: enabled,
                }),
            });
        } catch (e) {
            console.warn("Could not notify server:", e.message);
        }
    }

    /**
     * Subscribe to feature changes
     */
    function subscribe(callback) {
        subscribers.push(callback);
        return () => {
            subscribers = subscribers.filter((cb) => cb !== callback);
        };
    }

    /**
     * Notify all subscribers
     */
    function notifySubscribers() {
        const data = {
            enabled: Array.from(enabledFeatures),
            available: Array.from(availableFeatures),
        };
        subscribers.forEach((callback) => callback(data));
    }

    /**
     * Get feature info
     */
    function getFeature(featureId) {
        return FEATURES[featureId] || null;
    }

    /**
     * Get all features by category
     */
    function getFeaturesByCategory() {
        const byCategory = {};
        for (const [id, feature] of Object.entries(FEATURES)) {
            if (!byCategory[feature.category]) {
                byCategory[feature.category] = [];
            }
            byCategory[feature.category].push({
                id,
                ...feature,
                enabled: isEnabled(id),
                available: isAvailable(id),
            });
        }
        return byCategory;
    }

    /**
     * Get category info
     */
    function getCategories() {
        return CATEGORIES;
    }

    /**
     * Get enabled feature IDs
     */
    function getEnabled() {
        return Array.from(enabledFeatures);
    }

    /**
     * Get available feature IDs
     */
    function getAvailable() {
        return Array.from(availableFeatures);
    }

    /**
     * Update UI visibility based on enabled features
     */
    function updateUI() {
        // Hide/show app menu items based on features
        const appItems = document.querySelectorAll(".app-item");
        appItems.forEach((item) => {
            const section = item.dataset.section;
            const featureId = section; // Assuming section names match feature IDs

            if (FEATURES[featureId]) {
                if (isEnabled(featureId)) {
                    item.style.display = "";
                    item.removeAttribute("disabled");
                } else {
                    item.style.display = "none";
                }
            }
        });

        // Update main content sections
        const mainContent = document.getElementById("main-content");
        if (mainContent) {
            // Mark sections as available/unavailable
            const sections = mainContent.querySelectorAll("[data-feature]");
            sections.forEach((section) => {
                const featureId = section.dataset.feature;
                if (!isEnabled(featureId)) {
                    section.classList.add("feature-disabled");
                } else {
                    section.classList.remove("feature-disabled");
                }
            });
        }
    }

    // Auto-update UI when features change
    subscribe(() => {
        updateUI();
    });

    // Public API
    return {
        init,
        isEnabled,
        isAvailable,
        enable,
        disable,
        toggle,
        subscribe,
        getFeature,
        getFeaturesByCategory,
        getCategories,
        getEnabled,
        getAvailable,
        updateUI,
    };
})();

// Initialize on DOM ready
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
        FeatureManager.init();
    });
} else {
    FeatureManager.init();
}

// Make available globally
window.FeatureManager = FeatureManager;
