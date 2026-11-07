"use strict";

// VM file attachments — inline reference scan, empty-list cleanup and hydration entry point.
// Split out of the former 70_vm_attachments module for #1731; globals unchanged.

// Markdown the model emits for generated files — `[name](vmfile:CONV/output/name)`
// — does not survive the markdown/sanitizer pipeline on every path, so it can
// reach the DOM as plain text ("[a.pdf](vmfile:...)") with no anchor to click.
// Scan the rendered text nodes and replace each reference with a real
// attachment card (Preview / Download / Save).
function vmRenderInlineReferences(root) {
  if (!root || !root.querySelectorAll) return;
  var nodes = root.querySelectorAll(".message-content, .bot-message, .message-text");
  var pattern = /\[([^\]\n]{1,200})\]\(vmfile:([^)\s]+)\)/g;
  for (var i = 0; i < nodes.length; i++) {
    var node = nodes[i];
    if (node.querySelector(".gb-attachment")) continue;
    var walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT, null);
    var targets = [];
    var textNode;
    while ((textNode = walker.nextNode())) {
      if (textNode.nodeValue && textNode.nodeValue.indexOf("vmfile:") !== -1) {
        targets.push(textNode);
      }
    }
    if (!targets.length) continue;
    for (var t = 0; t < targets.length; t++) {
      var current = targets[t];
      var value = current.nodeValue;
      var replaced = document.createDocumentFragment();
      var last = 0;
      var hit;
      pattern.lastIndex = 0;
      while ((hit = pattern.exec(value)) !== null) {
        if (hit.index > last) {
          replaced.appendChild(document.createTextNode(value.slice(last, hit.index)));
        }
        var parts = vmSplitTarget(hit[2]);
        var card = document.createElement("span");
        card.className = "gb-attachment";
        card.dataset.path = parts.path;
        card.dataset.conversationId = parts.conversationId || "";
        card.innerHTML =
          '<span class="gb-attachment-icon">' + vmFileIcon(parts.path) + "</span>" +
          '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener"></a>' +
          vmAttachmentButtons(parts.path);
        card.querySelector(".gb-attachment-name").textContent =
          (hit[1] || "").trim() || parts.path.split("/").pop();
        replaced.appendChild(card);
        last = hit.index + hit[0].length;
      }
      if (!replaced.childNodes.length) continue;
      if (last < value.length) {
        replaced.appendChild(document.createTextNode(value.slice(last)));
      }
      if (current.parentNode) current.parentNode.replaceChild(replaced, current);
    }
  }
}

// A card that replaced the only text of a markdown list item leaves
// `<ul><li></li></ul>` behind — the empty bullet the user sees above the folder
// card. Lift the card out of the list and drop the now empty list.
function vmDropEmptyListItems(root) {
  if (!root || !root.querySelectorAll) return;
  var items = root.querySelectorAll("li");
  for (var i = 0; i < items.length; i++) {
    var li = items[i];
    if (li.textContent.trim() !== "") continue;
    if (!li.querySelector(".gb-attachment")) continue;
    var list = li.parentElement;
    if (!list || !list.parentElement) continue;
    while (li.firstChild) {
      list.parentElement.insertBefore(li.firstChild, list);
    }
    li.remove();
    if (!list.querySelector("li")) list.remove();
  }
}

// Bind inline vmfile cards and append output/ files the model did not link.
function hydrateVmAttachments(container) {
  if (!container) return;
  // A live reply can add files; the cached newest-first ranking is stale.
  if (typeof ChatState === "undefined" || !ChatState.suppressPersist) {
    vmFolderOrder = {};
  }
  var ownerConversation = vmConversationIdForElement(container);
  var conversationPromise = ownerConversation
    ? Promise.resolve(ownerConversation)
    : vmCurrentConversationId();
  vmRenderInlineReferences(container);
  vmDropEmptyListItems(container);
  var cards = container.querySelectorAll(".gb-attachment");
  for (var i = 0; i < cards.length; i++) vmBindAttachment(cards[i]);

  var anchor = vmAttachmentAnchor(container);
  if (!anchor) return;
  vmGroupAttachments(anchor);

  // History replay already carries the files the model linked. The
  // conversation-wide fallback must not staple today's output/ folder onto
  // old turns, so it only runs for a live reply. Record the conversation's
  // existing files as seen first, otherwise the next live answer would
  // re-attach every PDF produced before the reload.
  if (typeof ChatState !== "undefined" && ChatState.suppressPersist) {
    conversationPromise.then(function (conversationId) {
      if (!conversationId) return;
      var seen = VmAttachments.renderedPaths[conversationId];
      if (!seen) {
        seen = {};
        VmAttachments.renderedPaths[conversationId] = seen;
      }
      var cards = anchor.querySelectorAll(".gb-attachment");
      for (var index = 0; index < cards.length; index++) {
        if (cards[index].dataset.path) seen[cards[index].dataset.path] = true;
      }
      return vmAttachmentsRequest(
        "/api/agent/vm/files?conversation_id=" + encodeURIComponent(conversationId)
      ).then(function (data) {
        ((data && data.files) || []).forEach(function (name) {
          seen["output/" + name] = true;
        });
      });
    }).catch(function () { /* replay stays best-effort */ });
    return;
  }
  conversationPromise
    .then(function (conversationId) {
      if (!conversationId) return;
      // Claim the conversation synchronously: replay hydrates every message
      // at once, and each callback used to create its own empty set, so the
      // same output/ files were appended to all 34 replies.
      var seen = VmAttachments.renderedPaths[conversationId];
      if (!seen) {
        seen = {};
        VmAttachments.renderedPaths[conversationId] = seen;
      }
      // The fallback folder belongs to the newest reply, never to an older
      // turn that happened to hydrate first. Stay inside the reply's own
      // pane so a background tab never paints the visible conversation.
      var fallbackPane = anchor.closest ? anchor.closest(".chat-pane-messages") : null;
      var fallback = null;
      if (fallbackPane) {
        var paneBots = fallbackPane.querySelectorAll(".message.bot");
        fallback = paneBots.length ? paneBots[paneBots.length - 1] : null;
      }
      fallback = fallback || vmLatestBotMessage() || anchor;
      var present = {};
      var existing = fallback.querySelectorAll(".gb-attachment");
      for (var j = 0; j < existing.length; j++) {
        present[existing[j].dataset.path] = true;
        seen[existing[j].dataset.path] = true;
      }
      return vmAttachmentsRequest(
        "/api/agent/vm/files?conversation_id=" + encodeURIComponent(conversationId)
      ).then(function (data) {
        var files = (data && data.files) || [];
        var missing = files.filter(function (name) {
          var path = "output/" + name;
          return !seen[path] && !present[path];
        });
        if (!missing.length) {
          vmGroupAttachments(fallback);
          return;
        }
        if (!fallback.isConnected) return;
        var wrapper = document.createElement("div");
        wrapper.className = "gb-attachments";
        missing.forEach(function (name) {
          var path = "output/" + name;
          vmAppendCard(wrapper, conversationId, path, name);
          seen[path] = true;
        });
        var target = fallback.querySelector(".message-content") || fallback;
        target.appendChild(wrapper);
        vmGroupAttachments(fallback);
        if (
          typeof ChatState !== "undefined" &&
          !ChatState.isUserScrolling &&
          typeof scrollToBottom === "function"
        ) {
          scrollToBottom(true);
        }
      });
    })
    .catch(vmNotifyError);
}
