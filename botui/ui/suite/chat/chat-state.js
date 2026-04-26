var ChatState = {
  ws: null,
  currentSessionId: null,
  currentUserId: null,
  currentBotId: "default",
  currentBotName: "default",
  isStreaming: false,
  streamingMessageId: null,
  currentStreamingContent: "",
  streamingBuffer: "",
  lastRenderTime: 0,
  renderInterval: 200,
  reconnectAttempts: 0,
  maxReconnectAttempts: 5,
  disconnectNotified: false,
  isUserScrolling: false,
  activeSwitchers: new Set(),
  switcherDefinitions: [],
  mentionState: {
    active: false,
    query: "",
    startPos: -1,
    selectedIndex: 0,
    results: [],
  },
};

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
};

var EntityTypes = {
  lead: { icon: "\u{1F464}", color: "#4CAF50", label: "Lead", route: "crm" },
  opportunity: {
    icon: "\u{1F4B0}",
    color: "#FF9800",
    label: "Opportunity",
    route: "crm",
  },
  account: {
    icon: "\u{1F3E2}",
    color: "#2196F3",
    label: "Account",
    route: "crm",
  },
  contact: {
    icon: "\u{1F4C7}",
    color: "#9C27B0",
    label: "Contact",
    route: "crm",
  },
  invoice: {
    icon: "\u{1F4C4}",
    color: "#F44336",
    label: "Invoice",
    route: "billing",
  },
  quote: {
    icon: "\u{1F4CB}",
    color: "#607D8B",
    label: "Quote",
    route: "billing",
  },
  case: {
    icon: "\u{1F3AB}",
    color: "#E91E63",
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
    color: "#00BCD4",
    label: "Service",
    route: "products",
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

function escapeHtml(text) {
  var div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}
