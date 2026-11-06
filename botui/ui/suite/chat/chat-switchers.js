/* Chat response-format switchers.
 * #1753 — switcher state is PER TAB. Parallel conversations own their active
 * set and definitions, so a switch_context chip in one tab can never change
 * the `active_switchers` another tab sends. The stock tab keeps using
 * ChatState.activeSwitchers / ChatState.switcherDefinitions, so the
 * single-conversation behavior is unchanged. */
"use strict";

// Resolve the parallel tab a switcher action belongs to. `tabId` may name a
// specific tab (background frame rendering); omitted means the active tab.
// The stock tab (usesStockDom) resolves to null → ChatState globals.
function mcSwitcherTab(tabId) {
  var MC = window.GBMultiChat;
  if (!MC || !MC.tabs) return null;
  var id = tabId || MC.activeId;
  var tab = id ? MC.tabs[id] : null;
  return tab && !tab.usesStockDom ? tab : null;
}

function activeSwitcherSet() {
  var tab = mcSwitcherTab(null);
  if (tab) {
    if (!(tab.switchers instanceof Set)) tab.switchers = new Set();
    return tab.switchers;
  }
  if (!(ChatState.activeSwitchers instanceof Set)) {
    ChatState.activeSwitchers = new Set();
  }
  return ChatState.activeSwitchers;
}

function activeSwitcherIds() {
  return Array.from(activeSwitcherSet());
}

function switcherDefinitionsFor(tab) {
  if (tab) {
    if (!Array.isArray(tab.switcherDefinitions)) tab.switcherDefinitions = [];
    return tab.switcherDefinitions;
  }
  if (!Array.isArray(ChatState.switcherDefinitions)) {
    ChatState.switcherDefinitions = [];
  }
  return ChatState.switcherDefinitions;
}

function switcherChipsContainer(tab) {
  if (tab) {
    return tab.pane ? tab.pane.querySelector(".switchers-chips") : null;
  }
  // The stock strip always lives in the footer; querying it there keeps a
  // parked/active parallel pane from being mistaken for the stock tab.
  return document.querySelector("footer .switchers-chips") ||
    document.getElementById("switcherChips");
}

function switcherBoxContainer(tab) {
  if (tab) {
    return tab.pane ? tab.pane.querySelector(".switchers-container") : null;
  }
  return document.querySelector("footer .switchers-container") ||
    document.getElementById("switchers");
}

function switcherScopeVisible(tab) {
  // A parallel tab's strip lives in its own pane: safe to reveal even while
  // parked (a detached pane is invisible). Only the STOCK strip must stay
  // hidden while a parallel pane owns the visible chat.
  if (tab) return true;
  var MC = window.GBMultiChat;
  if (!MC || !MC.tabs || !MC.activeId) return true;
  var active = MC.tabs[MC.activeId];
  return !active || active.usesStockDom;
}

function renderSwitcherChips(tabId) {
  var tab = mcSwitcherTab(tabId);
  var container = switcherChipsContainer(tab);
  if (!container) return;
  var definitions = switcherDefinitionsFor(tab);
  var active = tab ? tab.switchers : ChatState.activeSwitchers;
  if (!(active instanceof Set)) {
    active = new Set();
    if (tab) tab.switchers = active;
    else ChatState.activeSwitchers = active;
  }
  container.innerHTML = definitions.map(function (sw) {
    var isActive = active.has(sw.id);
    return '<div class="switcher-chip' + (isActive ? ' active' : '') + '" ' +
      'data-switch-id="' + sw.id + '" ' +
      'style="--switcher-color: ' + (sw.color || '#666') + '; ' +
      (isActive ? 'color: white; background: ' + (sw.color || '#666') + '; ' : '') + '">' +
      '<span class="switcher-chip-icon">' + (sw.icon || '\u{1F500}') + '</span>' +
      '<span>' + sw.label + '</span></div>';
  }).join('');
}

function toggleSwitcher(switcherId, tabId) {
  var tab = mcSwitcherTab(tabId);
  var active = tab ? tab.switchers : ChatState.activeSwitchers;
  if (!(active instanceof Set)) {
    active = tab ? (tab.switchers = new Set()) : (ChatState.activeSwitchers = new Set());
  }
  if (active.has(switcherId)) {
    active.delete(switcherId);
  } else {
    active.add(switcherId);
  }
  renderSwitcherChips(tab ? tab.id : (tabId || undefined));

  var input = document.getElementById("messageInput");
  var inputText = input ? input.value.trim() : "";

  if (inputText) {
    window.sendMessage(inputText);
    return;
  }
  var payload = {
    bot_id: (tab && tab.botId) || ChatState.currentBotId,
    user_id: (tab && tab.userId) || ChatState.currentUserId,
    session_id: tab ? tab.sessionId : ChatState.currentSessionId,
    channel: "web",
    content: "",
    message_type: MessageType.SWITCHER_TOGGLE,
    active_switchers: Array.from(active),
    timestamp: new Date().toISOString(),
  };
  // A parallel tab must toggle on its OWN socket/session — the stock socket
  // belongs to the default conversation.
  if (tab) {
    if (tab.ws && tab.ws.readyState === WebSocket.OPEN) {
      try { tab.ws.send(JSON.stringify(payload)); } catch (e) { /* reconnect flushes */ }
    }
    return;
  }
  if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    ChatState.ws.send(JSON.stringify(payload));
  }
}

function renderBotSwitchers(switchers, tabId) {
  if (!switchers || switchers.length === 0) return;
  var tab = mcSwitcherTab(tabId);
  var definitions = switcherDefinitionsFor(tab);
  var existingIds = {};
  definitions.forEach(function (sw) { existingIds[sw.id] = true; });
  switchers.forEach(function (sw) {
    if (!existingIds[sw.id]) {
      definitions.push({
        id: sw.id,
        label: sw.label || sw.id,
        icon: sw.icon || SWITCHER_ICONS[sw.id] || "\u{1F500}",
        color: sw.color || "#666",
      });
      existingIds[sw.id] = true;
    }
  });
  var scope = tab ? tab.id : "tab-default";
  renderSwitcherChips(scope);
  var container = switcherBoxContainer(tab);
  if (container && definitions.length > 0 && switcherScopeVisible(tab)) {
    container.style.display = '';
  }
  var chipsContainer = switcherChipsContainer(tab);
  if (chipsContainer && !chipsContainer.dataset.hasClickHandler) {
    chipsContainer.addEventListener("click", function (e) {
      var chip = e.target.closest(".switcher-chip");
      if (chip) {
        var switcherId = chip.getAttribute("data-switch-id");
        if (switcherId) toggleSwitcher(switcherId, scope);
      }
    });
    chipsContainer.dataset.hasClickHandler = "true";
  }
}
