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
    '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener noreferrer"></a>' +
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

// The `**Files**` heading and its bullets only existed to carry the attachment
// cards (render_result in botserver). Once vmGroupAttachments has moved those
// cards into the folder grid the items are empty and the heading is noise
// (#2008). This must run AFTER the move: before it every `li` still holds a
// card whose name/badge/description/button text makes textContent non-empty,
// so a pre-move cleanup never matches.
function vmDropOrphanFileList(content) {
  if (!content || !content.querySelectorAll) return;
  var lists = content.querySelectorAll("ul, ol");
  for (var i = 0; i < lists.length; i++) {
    var list = lists[i];
    var items = list.querySelectorAll("li");
    for (var j = 0; j < items.length; j++) {
      var item = items[j];
      if (item.querySelector(".gb-attachment")) continue;
      if (item.textContent.trim() === "") item.remove();
    }
    if (list.querySelector("li")) continue;
    var heading = list.previousElementSibling;
    list.remove();
    if (heading && /^(files|arquivos)\s*:?$/i.test((heading.textContent || "").trim())) {
      heading.remove();
    }
  }
}

// The same generated file reaches several bot messages (the WORKSPACE RUN
// progress card and the final answer both get one), so the conversation showed
// the same "output" listing twice. One card per file: walking newest -> oldest,
// the first card seen owns the file and every older duplicate is dropped
// (together with a folder left empty). Conversation-wide on purpose: hydration
// order is not guaranteed, so fixing only the current reply leaves the
// duplicate painted by an earlier pass.
function vmDedupeConversationCards(root, anchor) {
  var scope = root || document;
  var bots = Array.prototype.slice.call(scope.querySelectorAll(".message.bot"));
  var self = anchor && anchor.closest
    ? (anchor.classList && anchor.classList.contains("message") ? anchor : anchor.closest(".message.bot"))
    : null;
  var myIndex = bots.indexOf(self);
  // Files already carried by an EARLIER reply are not new: the user asked for
  // only what this turn produced, so those stay behind "Show all".
  // Only replies from PREVIOUS turns count. The progress card of the same turn
  // ("WORKSPACE RUN", rendered before the answer) carries the same files and
  // must not make the answer's own files look old.
  var turnStart = 0;
  for (var u = myIndex - 1; u >= 0; u--) {
    if (bots[u].classList && bots[u].classList.contains("user")) {
      turnStart = u + 1;
      break;
    }
  }
  var earlierPaths = {};
  for (var e = 0; e < turnStart; e++) {
    var early = bots[e].querySelectorAll(".gb-attachment");
    for (var q = 0; q < early.length; q++) {
      if (early[q].dataset.path) earlierPaths[early[q].dataset.path] = true;
    }
  }
  // One card per file: newest mention owns it, older duplicates are dropped.
  var seen = {};
  for (var i = bots.length - 1; i >= 0; i--) {
    var cards = bots[i].querySelectorAll(".gb-attachment");
    for (var j = 0; j < cards.length; j++) {
      var card = cards[j];
      var path = card.dataset.path;
      if (!path) continue;
      if (seen[path]) {
        card.remove();
        continue;
      }
      seen[path] = true;
      if (i === myIndex && earlierPaths[path]) card.dataset.gbOlder = "1";
    }
    var folders = bots[i].querySelectorAll(".gb-attachment-folder");
    for (var f = 0; f < folders.length; f++) {
      if (!folders[f].querySelector(".gb-attachment")) folders[f].remove();
    }
  }
}

// One folder card for every attachment of the reply (#1584)// One folder card for every attachment of the reply (#1584)// One folder card for every attachment of the reply (#1584): the user sees
// "output" as a folder with all generated files, not a flat list of links.
function vmGroupAttachments(anchor) {
  if (!anchor) return;
  var dedupeScope = (anchor.closest && anchor.closest(".chat-pane-messages")) || document;
  vmDedupeConversationCards(dedupeScope, anchor);
  var cards = [];
  var all = anchor.querySelectorAll(".gb-attachment");
  for (var i = 0; i < all.length; i++) {
    if (!all[i].closest(".gb-attachment-folder")) cards.push(all[i]);
  }
  var target = anchor.querySelector(".message-content") || anchor;
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
      '<button type="button" class="gb-folder-toggle gb-attachment-btn">Show older</button>' +
      (vmAgentDriveEnabled()
        ? '<button type="button" class="gb-folder-save gb-attachment-btn">Save to Drive</button>'
        : "") +
      "</div>" +
      '<div class="gb-folder-grid"></div>';
    target.appendChild(folder);
  }
  var grid = folder.querySelector(".gb-folder-grid");
  // New cards lead the folder (#1693): the file the user just asked for is
  // the first row, older outputs follow as history. Inserting in reverse
  // keeps the source order (newest first) while placing it on top.
  for (var j = cards.length - 1; j >= 0; j--) {
    grid.insertBefore(cards[j], grid.firstChild);
  }
  var allCards = grid.querySelectorAll(".gb-attachment");
  var olderCards = grid.querySelectorAll('.gb-attachment[data-gb-older="1"]');
  var freshCount = allCards.length - olderCards.length;
  var countEl = folder.querySelector(".gb-folder-count");
  if (olderCards.length) {
    countEl.textContent =
      freshCount + " new file" + (freshCount === 1 ? "" : "s") +
      " \u00b7 " + allCards.length + " total";
  } else {
    countEl.textContent = allCards.length + " file" + (allCards.length === 1 ? "" : "s");
  }
  vmLabelAttachments(grid);
  vmOrderFolder(grid);
  var saveAll = folder.querySelector(".gb-folder-save");
  if (saveAll && !saveAll.dataset.bound) {
    saveAll.dataset.bound = "1";
    saveAll.title = "Save the folder to General Bots Drive or an external drive";
    saveAll.addEventListener("click", function () {
      var cardConversation = vmConversationIdFromCards(folder);
      (cardConversation ? Promise.resolve(cardConversation) : vmCurrentConversationId())
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          // #2047 — the menu needs the explicit list (the internal path still
          // treats an empty list as "everything in output/").
          return vmSaveWithMenu(saveAll, convId, vmFolderFileNames(folder));
        })
        .catch(vmNotifyError);
    });
  }

  // #1902 — only the most recent run's output is shown inline; older runs
  // collapse behind an expand control so empty/older bullets don't bury the
  // latest result.
  var toggle = folder.querySelector(".gb-folder-toggle");
  if (toggle && !toggle.dataset.bound) {
    toggle.dataset.bound = "1";
    var pane = anchor && anchor.closest ? anchor.closest(".chat-pane-messages") : null;
    var isLatest = true;
    try {
      var bots = (pane || document).querySelectorAll(".message.bot");
      if (bots.length && anchor !== bots[bots.length - 1]) isLatest = false;
    } catch (e) { /* default expanded */ }
    // Older files of the conversation stay hidden: the reply only advertises
    // what it just produced. "Show all" reveals the whole conversation.
    function applyOlderState(showAll) {
      var older = grid.querySelectorAll('.gb-attachment[data-gb-older="1"]');
      for (var o = 0; o < older.length; o++) {
        older[o].style.display = showAll ? "" : "none";
      }
      toggle.dataset.collapsed = showAll ? "0" : "1";
    }
    if (!isLatest) {
      // An older reply keeps its whole listing behind a control; expanding it
      // also reveals that turn's historical files.
      grid.style.display = "none";
      toggle.textContent = "Show older output";
      toggle.dataset.collapsed = "1";
      toggle.addEventListener("click", function () {
        var open = toggle.dataset.collapsed === "1";
        grid.style.display = open ? "" : "none";
        applyOlderState(open);
        toggle.dataset.collapsed = open ? "0" : "1";
        toggle.textContent = open ? "Hide" : "Show older output";
      });
    } else if (olderCards.length) {
      applyOlderState(false);
      toggle.textContent = "Show all (" + allCards.length + ")";
      toggle.dataset.collapsed = "1";
      toggle.addEventListener("click", function () {
        var showAll = toggle.dataset.collapsed === "1";
        applyOlderState(showAll);
        toggle.textContent = showAll ? "Show only new" : "Show all (" + allCards.length + ")";
      });
    } else {
      toggle.textContent = "Hide";
      toggle.dataset.collapsed = "0";
      toggle.addEventListener("click", function () {
        var collapsed = toggle.dataset.collapsed === "1";
        grid.style.display = collapsed ? "" : "none";
        toggle.dataset.collapsed = collapsed ? "0" : "1";
        toggle.textContent = collapsed ? "Hide" : "Show older output";
      });
    }
  }
  // The cards no longer live in the `**Files**` list, so drop the emptied
  // bullets and their heading after the move (#2008).
  vmDropOrphanFileList(target);
}
