/**
 * Analytics Module JavaScript
 * Dashboard functionality, AI chat, time range, and chart controls
 */

// Time range management
let currentTimeRange = "24h";

function updateTimeRange(range) {
    currentTimeRange = range;
    refreshDashboard();
}

function refreshDashboard() {
    // Trigger all HTMX elements to refresh
    document.querySelectorAll("[hx-get]").forEach((el) => {
        htmx.trigger(el, "load");
    });
}

// Analytics chat functionality
function askAnalytics(question) {
    document.getElementById("analyticsQuery").value = question;
    sendAnalyticsQuery();
}

async function sendAnalyticsQuery() {
    const input = document.getElementById("analyticsQuery");
    const query = input.value.trim();
    if (!query) return;

    const messagesContainer = document.getElementById("analyticsChatMessages");

    // Add user message
    const userMessage = document.createElement("div");
    userMessage.className = "chat-message user";
    userMessage.innerHTML = `
        <div class="message-avatar">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" stroke="currentColor" stroke-width="2"/>
                <circle cx="12" cy="7" r="4" stroke="currentColor" stroke-width="2"/>
            </svg>
        </div>
        <div class="message-content">${escapeHtml(query)}</div>
    `;
    messagesContainer.appendChild(userMessage);

    // Clear input
    input.value = "";

    // Scroll to bottom
    messagesContainer.scrollTop = messagesContainer.scrollHeight;

    // Add loading indicator
    const loadingMessage = document.createElement("div");
    loadingMessage.className = "chat-message assistant";
    loadingMessage.id = "loading-message";
    loadingMessage.innerHTML = `
        <div class="message-avatar">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <rect x="3" y="4" width="18" height="16" rx="2" stroke="currentColor" stroke-width="2"/>
                <circle cx="9" cy="10" r="2" fill="currentColor"/>
                <circle cx="15" cy="10" r="2" fill="currentColor"/>
                <path d="M9 15h6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
        </div>
        <div class="message-content">
            <div class="spinner" style="width: 16px; height: 16px;"></div>
            Analyzing your data...
        </div>
    `;
    messagesContainer.appendChild(loadingMessage);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;

    try {
        const response = await fetch("/api/ui/analytics/chat", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                query: query,
                timeRange: currentTimeRange,
            }),
        });

        const data = await response.json();

        // Remove loading message
        document.getElementById("loading-message")?.remove();

        // Add assistant response
        const assistantMessage = document.createElement("div");
        assistantMessage.className = "chat-message assistant";
        assistantMessage.innerHTML = `
            <div class="message-avatar">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                    <rect x="3" y="4" width="18" height="16" rx="2" stroke="currentColor" stroke-width="2"/>
                    <circle cx="9" cy="10" r="2" fill="currentColor"/>
                    <circle cx="15" cy="10" r="2" fill="currentColor"/>
                    <path d="M9 15h6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                </svg>
            </div>
            <div class="message-content">${formatAnalyticsResponse(data)}</div>
        `;
        messagesContainer.appendChild(assistantMessage);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    } catch (error) {
        document.getElementById("loading-message")?.remove();

        const errorMessage = document.createElement("div");
        errorMessage.className = "chat-message assistant";
        errorMessage.innerHTML = `
            <div class="message-avatar">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                    <rect x="3" y="4" width="18" height="16" rx="2" stroke="currentColor" stroke-width="2"/>
                    <circle cx="9" cy="10" r="2" fill="currentColor"/>
                    <circle cx="15" cy="10" r="2" fill="currentColor"/>
                    <path d="M9 15h6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                </svg>
            </div>
            <div class="message-content">Sorry, I encountered an error analyzing your data. Please try again.</div>
        `;
        messagesContainer.appendChild(errorMessage);
    }
}

function formatAnalyticsResponse(data) {
    if (data.error) {
        return `<p style="color: var(--text-secondary);">${escapeHtml(data.error)}</p>`;
    }

    let html = "";

    if (data.answer) {
        html += `<p>${escapeHtml(data.answer)}</p>`;
    }

    if (data.metrics && data.metrics.length > 0) {
        html += '<div class="analytics-results">';
        data.metrics.forEach((metric) => {
            html += `<div class="metric-result">
                <strong>${escapeHtml(metric.name)}:</strong> ${escapeHtml(metric.value)}
            </div>`;
        });
        html += "</div>";
    }

    if (data.insight) {
        html += `<p class="insight"><em>${escapeHtml(data.insight)}</em></p>`;
    }

    return html || "<p>No data available for that query.</p>";
}

function escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
}

// Chart type switching
document.querySelectorAll(".chart-btn").forEach((btn) => {
    btn.addEventListener("click", function () {
        const chart = this.dataset.chart;
        const type = this.dataset.type;

        // Update active state
        this.parentElement
            .querySelectorAll(".chart-btn")
            .forEach((b) => b.classList.remove("active"));
        this.classList.add("active");

        // Could trigger chart re-render here
        console.log(`Switching ${chart} chart to ${type}`);
    });
});

// Initialize
document.addEventListener("DOMContentLoaded", () => {
    console.log("Analytics Dashboard initialized");
});
