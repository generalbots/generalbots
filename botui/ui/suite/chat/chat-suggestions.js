function renderSuggestions(suggestions, targetEl) {
  // Parallel chat tabs each own a suggestions container; the caller passes
  // the tab's element so a background frame never paints the active pane.
  var suggestionsEl = targetEl || document.getElementById("suggestions");
  if (!suggestionsEl) return;

  suggestionsEl.innerHTML = "";
  suggestionsEl.classList.add("has-bot-suggestions");

  suggestions.forEach(function (suggestion) {
    var chip = document.createElement("button");
    chip.className = "suggestion-chip";
    chip.textContent = suggestion.text || "Suggestion";

chip.onclick = (function (sugg) {
return function () {
if (sugg.action) {
try {
var actionData = typeof sugg.action === "string"
? JSON.parse(sugg.action)
: sugg.action;

if (actionData.type === "invoke_tool") {
// TOOL_EXEC (message_type 6) only carries the tool name; a tool that
// declares parameters needs a normal turn so the model can ask for them.
if (actionData.prompt_for_params ||
    (Array.isArray(actionData.params) && actionData.params.length)) {
  window.sendMessage(sugg.text);
  return;
}
// Check if WebSocket is available
if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
var msg = {
bot_id: ChatState.currentBotId,
user_id: ChatState.currentUserId,
session_id: ChatState.currentSessionId,
channel: "web",
content: actionData.tool,
message_type: 6,
active_switchers: typeof activeSwitcherIds === "function"
  ? activeSwitcherIds()
  : Array.from(ChatState.activeSwitchers || []),
timestamp: new Date().toISOString(),
};
ChatState.ws.send(JSON.stringify(msg));
} else {
// Fallback: send as regular message if WS not available
window.sendMessage(sugg.text);
}
return;
} else if (actionData.type === "switch_context" && actionData.switcher) {
// #1753 — mutate THIS tab's switcher set (stock tab keeps ChatState's).
var switcherSet = typeof activeSwitcherSet === "function"
  ? activeSwitcherSet()
  : ChatState.activeSwitchers;
if (!switcherSet.has(actionData.switcher)) {
switcherSet.add(actionData.switcher);
if (typeof renderSwitcherChips === "function") renderSwitcherChips();
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
