function renderSuggestions(suggestions) {
  var suggestionsEl = document.getElementById("suggestions");
  if (!suggestionsEl) return;

  suggestionsEl.innerHTML = "";
  suggestionsEl.classList.add("has-bot-suggestions");

  suggestions.forEach(function (suggestion) {
    var chip = document.createElement("button");
    chip.className = "suggestion-chip";
    chip.textContent = suggestion.text || "Suggestion";

chip.onclick = (function (sugg) {
return function () {
console.log("[SUGGESTION] Clicked:", sugg.text, "Action:", sugg.action);
if (sugg.action) {
try {
var actionData = typeof sugg.action === "string"
? JSON.parse(sugg.action)
: sugg.action;

console.log("[SUGGESTION] Parsed action:", actionData);

if (actionData.type === "invoke_tool") {
console.log("[SUGGESTION] Invoking tool:", actionData.tool);
// Check if WebSocket is available
if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
var msg = {
bot_id: ChatState.currentBotId,
user_id: ChatState.currentUserId,
session_id: ChatState.currentSessionId,
channel: "web",
content: actionData.tool,
message_type: 6,
active_switchers: Array.from(ChatState.activeSwitchers),
timestamp: new Date().toISOString(),
};
console.log("[SUGGESTION] Sending via WS:", msg);
ChatState.ws.send(JSON.stringify(msg));
console.log("[SUGGESTION] Sent successfully");
} else {
console.log("[SUGGESTION] WS not available, fallback to sendMessage");
// Fallback: send as regular message if WS not available
window.sendMessage(sugg.text);
}
return;
} else if (actionData.type === "switch_context" && actionData.switcher) {
if (!ChatState.activeSwitchers.has(actionData.switcher)) {
ChatState.activeSwitchers.add(actionData.switcher);
renderSwitcherChips();
}
window.sendMessage(sugg.text);
} else if (actionData.type === "send_message") {
window.sendMessage(actionData.message || sugg.text);
} else if (actionData.type === "select_context") {
window.sendMessage(actionData.context);
            } else {
              window.sendMessage(sugg.text);
            }
          } catch (e) {
            window.sendMessage(sugg.text);
          }
        } else {
          window.sendMessage(sugg.text);
        }
      };
    })(suggestion);

    suggestionsEl.appendChild(chip);
  });
}
