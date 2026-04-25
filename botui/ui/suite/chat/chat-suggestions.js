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
        if (sugg.action) {
          try {
            var action = typeof sugg.action === "string"
              ? JSON.parse(sugg.action)
              : sugg.action;

            if (action.type === "invoke_tool") {
              ChatState.ws.send(JSON.stringify({
                bot_id: ChatState.currentBotId,
                user_id: ChatState.currentUserId,
                session_id: ChatState.currentSessionId,
                channel: "web",
                content: action.tool,
                message_type: 6,
                active_switchers: Array.from(ChatState.activeSwitchers),
                timestamp: new Date().toISOString(),
              }));
              return;
            } else if (action.type === "switch_context" && action.switcher) {
              if (!ChatState.activeSwitchers.has(action.switcher)) {
                ChatState.activeSwitchers.add(action.switcher);
                renderSwitcherChips();
              }
              window.sendMessage(sugg.text);
            } else if (action.type === "send_message") {
              window.sendMessage(action.message || sugg.text);
            } else if (action.type === "select_context") {
              window.sendMessage(action.context);
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
