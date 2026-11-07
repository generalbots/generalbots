"use strict";

// VM file attachments — card append, conversation resolution and output/ folder grouping.
// Split out of the former 70_vm_attachments module for #1731; globals unchanged.

function vmAppendCard(wrapper, conversationId, path, name) {
  var card = document.createElement("span");
  card.className = "gb-attachment";
  card.dataset.path = path;
  card.dataset.conversationId = conversationId || "";
  card.dataset.contentType = vmFileMimeType(path);
  card.dataset.order = "0";
  card.innerHTML =
    '<span class="gb-attachment-icon"></span>' +
    '<span class="gb-attachment-order">0</span>' +
    '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener"></a>' +
    '<span class="gb-content-type-badge gb-type-badge"></span>' +
    '<span class="gb-attachment-desc gb-attachment-minor"></span>' +
    '<span class="gb-latest-chip gb-attachment-latest">Latest</span>' +
    vmAttachmentButtons(path);
  card.querySelector(".gb-attachment-icon").textContent = vmFileIcon(path);
  card.querySelector(".gb-attachment-name").textContent = name;
  card.querySelector(".gb-content-type-badge").textContent = vmContentTypeBadge(path);
  card.querySelector(".gb-attachment-desc").textContent = vmAttachmentDescription(path);
  wrapper.appendChild(card);
  vmBindAttachment(card);
  return card;
}

// Resolve the conversation a rendered reply belongs to. Background chat
// tabs hydrate detached panes; falling back to the active tab linked their
// files to the wrong conversation.
function vmConversationIdForElement(el) {
  try {
    var pane = el && el.closest ? el.closest(".chat-pane-messages") : null;
    if (!pane) return null;
    var holder = pane.parentElement;
    var tabId = (holder && holder.__mcTabId) || pane.__mcTabId;
    if (tabId && window.GBMultiChat && GBMultiChat.tabs && GBMultiChat.tabs[tabId]) {
      var tab = GBMultiChat.tabs[tabId];
      if (tab.sessionId) return tab.sessionId;
    }
  } catch (e) { /* stock surface */ }
  return null;
}

function vmConversationIdFromCards(root) {
  var card = root && root.querySelector
    ? root.querySelector(".gb-attachment[data-conversation-id]")
    : null;
  return card ? card.dataset.conversationId : "";
}

// The conversation the user is looking at, not the VM's last active folder.
function vmCurrentConversationId() {
  try {
    if (window.GBTabs && GBTabs.activeTab) {
      var tab = GBTabs.activeTab();
      if (tab && tab.sessionId) return Promise.resolve(tab.sessionId);
    }
    if (window.ChatState && ChatState.currentSessionId) {
      return Promise.resolve(ChatState.currentSessionId);
    }
  } catch (e) { /* fall through */ }
  return vmActiveConversationId();
}

function vmVisiblePane() {
  var panes = document.querySelectorAll(".chat-pane-messages, #messages");
  for (var i = 0; i < panes.length; i++) {
    var rect = panes[i].getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) return panes[i];
  }
  return null;
}

function vmLatestBotMessage() {
  var pane = vmVisiblePane();
  if (!pane) return null;
  var bots = pane.querySelectorAll(".message.bot");
  return bots.length ? bots[bots.length - 1] : null;
}

function vmAttachmentAnchor(container) {
  if (container && container.isConnected) {
    var rect = container.getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) return container;
  }
  // A background tab's pane is detached while it waits; keep its files in
  // that pane instead of stapling them onto the visible conversation.
  if (container && container.closest && container.closest(".chat-pane-messages")) {
    return container;
  }
  // The reply may render into a hidden chat tab; attach the manifest to the
  // last visible bot message so the active pane always shows the files.
  var pane = vmVisiblePane();
  if (!pane) return null;
  var bots = pane.querySelectorAll(".message.bot");
  return bots.length ? bots[bots.length - 1] : null;
}

// Folder rows follow the server's newest-first listing (#1693). Reply links
// arrive in tool order, which can put an older file first; the VM listing is
// ordered by mtime, so it is the source of truth. Cached per conversation and
// invalidated on every live reply that may add files.
var vmFolderOrder = {};
function vmApplyFolderOrder(grid, rank) {
  var items = Array.prototype.slice.call(grid.querySelectorAll(".gb-attachment"));
  items.sort(function (a, b) {
    var an = String(a.dataset.path || "").split("/").pop();
    var bn = String(b.dataset.path || "").split("/").pop();
    var ar = Object.prototype.hasOwnProperty.call(rank, an) ? rank[an] : 1e9;
    var br = Object.prototype.hasOwnProperty.call(rank, bn) ? rank[bn] : 1e9;
    return ar - br;
  });
  items.forEach(function (el) { grid.appendChild(el); });
  vmLabelAttachments(grid);
}

function vmLabelAttachments(grid) {
  if (!grid) return;
  var items = grid.querySelectorAll(".gb-attachment");
  for (var i = 0; i < items.length; i++) {
    var card = items[i];
    card.dataset.order = String(i + 1);
    var order = card.querySelector(".gb-attachment-order");
    if (order) order.textContent = String(i + 1);
    if (i === 0 && items.length > 1) {
      card.classList.add("is-latest");
    } else {
      card.classList.remove("is-latest");
    }
    var badge = card.querySelector(".gb-content-type-badge");
    if (badge && !badge.textContent) {
      var path = card.dataset.path || "";
      badge.textContent = vmContentTypeBadge(path);
    }
    if (badge) badge.dataset.contentType = card.dataset.contentType || vmFileMimeType(card.dataset.path || "");
  }
}
var vmFolderPending = {};
function vmOrderFolder(grid) {
  if (!grid || grid.querySelectorAll(".gb-attachment").length < 2) return;
  var explicitConversation = vmConversationIdFromCards(grid);
  (explicitConversation ? Promise.resolve(explicitConversation) : vmCurrentConversationId())
    .then(function (conversationId) {
      if (!conversationId) return;
      if (vmFolderOrder[conversationId]) {
        vmApplyFolderOrder(grid, vmFolderOrder[conversationId]);
        return;
      }
      // History replay groups every reply at once; one listing per
      // conversation is enough, so concurrent folders share the request.
      if (!vmFolderPending[conversationId]) {
        vmFolderPending[conversationId] = vmAttachmentsRequest(
          "/api/agent/vm/files?conversation_id=" + encodeURIComponent(conversationId)
        )
          .then(function (data) {
            var rank = {};
            ((data && data.files) || []).forEach(function (name, i) { rank[name] = i; });
            vmFolderOrder[conversationId] = rank;
            return rank;
          })
          .finally(function () { delete vmFolderPending[conversationId]; });
      }
      return vmFolderPending[conversationId].then(function (rank) {
        vmApplyFolderOrder(grid, rank);
      });
    })
    .catch(function () { /* keep the reply order */ });
}

// One folder card for every attachment of the reply (#1584): the user sees
// "output" as a folder with all generated files, not a flat list of links.
function vmGroupAttachments(anchor) {
  if (!anchor) return;
  var cards = [];
  var all = anchor.querySelectorAll(".gb-attachment");
  for (var i = 0; i < all.length; i++) {
    if (!all[i].closest(".gb-attachment-folder")) cards.push(all[i]);
  }
  var folder = anchor.querySelector(".gb-attachment-folder");
  if (!folder) {
    if (!cards.length) return;
    folder = document.createElement("div");
    folder.className = "gb-attachment-folder";
    folder.innerHTML =
      '<div class="gb-folder-head">' +
      '<span class="gb-folder-icon">\u{1F4C1}</span>' +
      '<span class="gb-folder-title">output</span>' +
      '<span class="gb-folder-count"></span>' +
      (vmAgentDriveEnabled()
        ? '<button type="button" class="gb-folder-save gb-attachment-btn">Save to Drive</button>'
        : "") +
      "</div>" +
      '<div class="gb-folder-grid"></div>';
    var target = anchor.querySelector(".message-content") || anchor;
    target.appendChild(folder);
  }
  var grid = folder.querySelector(".gb-folder-grid");
  // New cards lead the folder (#1693): the file the user just asked for is
  // the first row, older outputs follow as history. Inserting in reverse
  // keeps the source order (newest first) while placing it on top.
  for (var j = cards.length - 1; j >= 0; j--) {
    grid.insertBefore(cards[j], grid.firstChild);
  }
  var count = grid.querySelectorAll(".gb-attachment").length;
  folder.querySelector(".gb-folder-count").textContent =
    count + " file" + (count === 1 ? "" : "s");
  vmLabelAttachments(grid);
  vmOrderFolder(grid);
  var saveAll = folder.querySelector(".gb-folder-save");
  if (saveAll && !saveAll.dataset.bound) {
    saveAll.dataset.bound = "1";
    saveAll.addEventListener("click", function () {
      var cardConversation = vmConversationIdFromCards(folder);
      (cardConversation ? Promise.resolve(cardConversation) : vmCurrentConversationId())
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          return vmSaveToDrive(convId, []);
        })
        .then(vmReportSaved)
        .catch(vmNotifyError);
    });
  }
}
