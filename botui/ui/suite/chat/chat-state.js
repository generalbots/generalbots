var ChatState = {
  ws: null,
  currentSessionId: null,
  // #2229 — when set, addMessage appends to this container (e.g. a
  // DocumentFragment during history replay) instead of the live #messages node.
  appendTarget: null,
  clientId: (function() { try { return (localStorage.getItem("gb.client.id") || ""); } catch(e) { return ""; } })() || (function() { var id = "c" + Math.random().toString(36).substring(2) + Date.now().toString(36); try { localStorage.setItem("gb.client.id", id); } catch (e) { window.gbLogError ? gbLogError(e, "chat-state") : console.error("chat-state", e); } return id; })(),
  currentUserId: null,
  currentBotId: "default",
  currentBotName: "default",
  isStreaming: false,
  streamingMessageId: null,
  currentStreamingContent: "",
  currentReasoning: "",
  streamingBuffer: "",
  lastRenderTime: 0,
  renderInterval: 200,
  reconnectAttempts: 0,
  maxReconnectAttempts: 5,
  disconnectNotified: false,
  isUserScrolling: false,
  isUserAtBottom: true,
  activeSwitchers: new Set(),
  switcherDefinitions: [],
  mentionState: {
    active: false,
    query: "",
    startPos: -1,
    selectedIndex: 0,
    results: [],
  },
  selectedMentions: [],
  projectCatalog: [],
};

// #2347 — bearer tokens must not be read from or persisted to localStorage.
// Chat keeps the live token only in GBSecurity memory/sessionStorage.
function getChatAccessToken() {
  try {
    if (window.GBSecurity && typeof window.GBSecurity.getToken === "function") {
      var t = window.GBSecurity.getToken();
      if (t) return t;
    }
  } catch (e) { window.gbLogError ? gbLogError(e, "getChatAccessToken") : console.error("getChatAccessToken", e); }
  try {
    return sessionStorage.getItem("gb-access-token") || "";
  } catch (e) {
    return "";
  }
}
window.getChatAccessToken = getChatAccessToken;

var WS_BASE_URL =
  window.location.protocol === "https:" ? "wss://" : "ws://";
var WS_URL = WS_BASE_URL + window.location.host + "/ws";

var MessageType = {
  EXTERNAL: 0,
  USER: 1,
  BOT_RESPONSE: 2,
  CONTINUE: 3,
  SUGGESTION: 4,
  CONTEXT_CHANGE: 5,
  TOOL_EXEC: 6,
  SWITCHER_TOGGLE: 8,
  UI_ACTION: 9,
};

var EntityTypes = {
  lead: { icon: "\u{1F464}", color: "var(--palette-4caf50)", label: "Lead", route: "crm" },
  opportunity: {
    icon: "\u{1F4B0}",
    color: "var(--palette-ff9800)",
    label: "Opportunity",
    route: "crm",
  },
  account: {
    icon: "\u{1F3E2}",
    color: "var(--palette-0ea5e9)",
    label: "Account",
    route: "crm",
  },
  contact: {
    icon: "\u{1F4C7}",
    color: "var(--palette-9c27b0)",
    label: "Contact",
    route: "crm",
  },
  invoice: {
    icon: "\u{1F4C4}",
    color: "var(--palette-f44336)",
    label: "Invoice",
    route: "billing",
  },
  quote: {
    icon: "\u{1F4CB}",
    color: "var(--palette-64748b)",
    label: "Quote",
    route: "billing",
  },
  case: {
    icon: "\u{1F3AB}",
    color: "var(--palette-db2777)",
    label: "Case",
    route: "tickets",
  },
  product: {
    icon: "\u{1F4E6}",
    color: "#795548",
    label: "Product",
    route: "products",
  },
  service: {
    icon: "\u2699\uFE0F",
    color: "var(--palette-06b6d4)",
    label: "Service",
    route: "products",
  },
  integration: {
    icon: "\u{1F50C}",
    color: "#009688",
    label: "Integrations",
    route: "apps",
  },
  project: {
    icon: "\u{1F4BB}",
    color: "var(--palette-84d669)",
    label: "Project",
    route: "vibe",
  },
};

var SWITCHER_ICONS = {
  tables: "\u{1F4CA}",
  infographic: "\u{1F4CD}",
  cards: "\u{1F0CF}",
  list: "\u{1F4CB}",
  comparison: "\u2696",
  timeline: "\u23F0",
  markdown: "\u{1F4DD}",
  chart: "\u{1F4C8}",
};

function escapeHtml(value) {
  if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
  return String(value == null ? "" : value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
