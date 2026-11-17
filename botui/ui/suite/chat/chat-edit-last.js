"use strict";
/**
 * Chat — ↑ on an empty composer recalls the last message you sent so it can be
 * edited and sent again (assistant shell behaviour, ported to the windowed
 * chat). Never fires while a mention/suggestion dropdown is open.
 */

(function () {
  function lastUserText() {
    var bubbles = document.querySelectorAll("#messages .message.user");
    if (!bubbles.length) return "";
    var bubble = bubbles[bubbles.length - 1];
    var raw = bubble.dataset ? bubble.dataset.originalContent : "";
    var text = (typeof raw === "string" && raw.trim())
      ? raw
      : (bubble.querySelector(".gb-message-text") || bubble).innerText || "";
    return String(text)
      .replace(/(?:^|\n)📎\s+[^\n]+$/gm, "")
      .trim();
  }

  function dropdownOpen() {
    var dropdown = document.getElementById("mentionDropdown");
    if (dropdown && dropdown.style && dropdown.style.display &&
        dropdown.style.display !== "none") return true;
    return !!document.querySelector(".suggestion-list.open, .mention-dropdown.active");
  }

  function recallLast(input) {
    var text = lastUserText();
    if (!text) return false;
    input.value = text;
    try { input.dispatchEvent(new Event("input", { bubbles: true })); } catch (error) { /* ignore */ }
    try {
      input.focus();
      input.setSelectionRange(text.length, text.length);
    } catch (error) { /* selection unsupported */ }
    return true;
  }

  function onKeydown(event) {
    if (event.key !== "ArrowUp" || event.shiftKey || event.ctrlKey || event.metaKey || event.altKey) return;
    var input = event.target;
    if (!input || input.id !== "messageInput") return;
    if (document.getElementById("messageInput") !== input) return;
    if ((input.value || "").trim() !== "") return;
    if (dropdownOpen()) return;
    if (recallLast(input)) event.preventDefault();
  }

  document.addEventListener("keydown", onKeydown);

  window.GBChatEditLast = { recallLast: recallLast, lastUserText: lastUserText };
})();
