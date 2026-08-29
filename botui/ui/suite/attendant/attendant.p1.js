/**
 * Attendant Module JavaScript
 * Human agent interface for live chat support
 *
 * NOTE: The original init()/setupWebSocket() block that wired selectors such as
 * `.queue-list` / `.conversation-messages` (which do not exist in index.html) and
 * opened a second, misconfigured WebSocket to `/api/attendance/ws` has been removed.
 * The real initialisation and the single canonical WebSocket connection are owned by
 * attendant.p4.js (variable `ws`, connected with `?attendant_id=...&token=...`).
 */

if (window.GBAppLifecycle) GBAppLifecycle.begin("attendant");

            // =====================================================================
            // Configuration
            // =====================================================================
            const API_BASE = window.location.origin;
            let currentSessionId = null;
            let currentAttendantId = null;
            let currentAttendantStatus = "online";
            let conversations = [];
            let attendants = [];
            let ws = null;
            let reconnectAttempts = 0;
            const MAX_RECONNECT_ATTEMPTS = 5;

            // LLM Assist configuration
            let llmAssistConfig = {
                tips_enabled: false,
                polish_enabled: false,
                smart_replies_enabled: false,
                auto_summary_enabled: false,
                sentiment_enabled: false
            };
            let conversationHistory = [];
