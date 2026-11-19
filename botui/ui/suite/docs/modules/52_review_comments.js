"use strict";
/* Docs enterprise review workflow (#2425), threaded range comments.
 *
 * Comments are anchored to a stable element id plus an offset inside that
 * element (never a bare document character index), so edits made above the
 * anchor cannot detach them. Replies, resolve/reopen and @mention chips are
 * local; no notification is sent (deferred). The reply composer is a
 * focus-trapped dialog — never a native browser input dialog. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;

  var FILTERS = [
    ["open", "docs-review-filter-open", "Open"],
    ["mine", "docs-review-filter-mine", "Mine"],
    ["mentions", "docs-review-filter-mentions", "Mentions"]
  ];
  NS.COMMENT_FILTERS = FILTERS;

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }

  function i18n(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    return node;
  }

  /* Render text with @name tokens as non-editable chips. */
  function richText(text) {
    var wrap = el("span", "gb-review-rich");
    String(text == null ? "" : text).split(/(@[\w.-]+)/g).forEach(function (token) {
      if (/^@[\w.-]+$/.test(token)) {
        var chip = el("span", "gb-review-mention");
        chip.setAttribute("data-review-mention", token.slice(1));
        chip.textContent = token;
        wrap.appendChild(chip);
      } else if (token) {
        wrap.appendChild(document.createTextNode(token));
      }
    });
    return wrap;
  }
  NS.richText = richText;

  /* Capture a stable anchor for the current selection inside the article. */
  function captureSelection() {
    var article = NS.getArticle();
    if (!article || !window.getSelection || !article.querySelector) return null;
    var sel = window.getSelection();
    if (!sel || !sel.rangeCount || sel.isCollapsed) return null;
    var range = sel.getRangeAt(0);
    if (!article.contains(range.commonAncestorContainer)) return null;
    var node = range.startContainer;
    if (node.nodeType === 3) node = node.parentNode;
    if (!node || !article.contains(node)) return null;
    var block = node;
    while (block && block !== article && block.children && block.children.length === 1 &&
           block.firstChild && block.firstChild.nodeType === 1 &&
           ["SPAN", "STRONG", "EM", "B", "I"].indexOf(block.firstChild.tagName) >= 0) {
      block = block.firstChild;
    }
    if (!block.dataset) return null;
    if (!block.dataset.reviewAnchor) block.dataset.reviewAnchor = NS.uid("anchor");
    var pre = document.createRange ? document.createRange() : null;
    var offset = 0;
    if (pre) {
      try {
        pre.selectNodeContents(block);
        pre.setEnd(range.startContainer, range.startOffset);
        offset = pre.toString().length;
      } catch (error) { offset = 0; }
    }
    return {
      anchorId: block.dataset.reviewAnchor,
      offset: offset,
      quote: sel.toString().slice(0, 240)
    };
  }
  NS.captureSelection = captureSelection;

  function replyCard(reply) {
    var item = el("div", "gb-review-reply");
    var head = el("div", "gb-review-reply-head");
    var author = el("strong", "gb-review-reply-author");
    author.textContent = reply.author;
    head.appendChild(author);
    var time = el("time", "gb-review-time");
    time.textContent = reply.time;
    head.appendChild(time);
    item.appendChild(head);
    var body = el("div", "gb-review-reply-body");
    body.appendChild(richText(reply.text));
    item.appendChild(body);
    return item;
  }

  function threadCard(thread) {
    var card = el("article", "gb-review-thread");
    card.setAttribute("data-review-thread", thread.id);
    if (thread.status === "resolved") card.classList.add("gb-review-resolved");
    var head = el("div", "gb-review-thread-head");
    var author = el("strong", "gb-review-thread-author");
    author.textContent = thread.author;
    head.appendChild(author);
    var time = el("time", "gb-review-time");
    time.textContent = thread.time;
    head.appendChild(time);
    var anchor = el("span", "gb-review-thread-anchor");
    var anchorLabel = el("span");
    anchorLabel.setAttribute("data-i18n", "docs-review-thread-anchor");
    anchorLabel.textContent = NS.tr("docs-review-thread-anchor", "Anchored comment");
    anchor.appendChild(anchorLabel);
    anchor.appendChild(document.createTextNode(" " + thread.anchorId + ":" + thread.offset));
    head.appendChild(anchor);
    card.appendChild(head);
    if (thread.quote) {
      var quote = el("blockquote", "gb-review-quote");
      quote.textContent = thread.quote;
      card.appendChild(quote);
    }
    var body = el("div", "gb-review-thread-body");
    body.appendChild(richText(thread.firstText || ""));
    card.appendChild(body);
    thread.replies.forEach(function (reply) { card.appendChild(replyCard(reply)); });
    var actions = el("div", "gb-review-thread-actions");
    var replyButton = el("button", "gb-review-btn");
    replyButton.setAttribute("type", "button");
    replyButton.setAttribute("data-i18n", "docs-review-reply");
    replyButton.textContent = NS.tr("docs-review-reply", "Reply");
    replyButton.addEventListener("click", function () { openReply(thread.id); });
    actions.appendChild(replyButton);
    var toggle = el("button", "gb-review-btn");
    toggle.setAttribute("type", "button");
    var resolved = thread.status === "resolved";
    toggle.setAttribute("data-i18n", resolved ? "docs-review-reopen" : "docs-review-resolve");
    toggle.textContent = NS.tr(resolved ? "docs-review-reopen" : "docs-review-resolve",
      resolved ? "Reopen" : "Resolve");
    toggle.addEventListener("click", function () {
      if (resolved) { NS.reopenThread(thread.id); NS.statusKey("docs-review-status-reopened", "Thread reopened"); }
      else { NS.resolveThread(thread.id); NS.statusKey("docs-review-status-resolved", "Thread resolved"); }
      NS.renderCommentPanel();
    });
    actions.appendChild(toggle);
    card.appendChild(actions);
    return card;
  }

  function openReply(threadId) {
    if (!NS.dialog || !NS.dialog.open) return;
    NS.dialog.open({
      titleKey: "docs-review-reply",
      titleFallback: "Reply",
      placeholderKey: "docs-review-reply-placeholder",
      placeholderFallback: "Write a reply… use @name to mention",
      submitKey: "docs-review-reply",
      submitFallback: "Reply",
      onSubmit: function (text) {
        var entry = NS.reply(threadId, { text: text });
        NS.statusKey("docs-review-status-reply-added", "Reply added");
        NS.renderCommentPanel();
        return !!entry;
      }
    });
  }
  NS.openReply = openReply;

  function openNewThread() {
    var anchor = captureSelection();
    if (!anchor) {
      NS.statusKey("docs-review-status-no-selection", "Select text in the document first");
      return;
    }
    if (!NS.dialog || !NS.dialog.open) return;
    NS.dialog.open({
      titleKey: "docs-review-thread-new",
      titleFallback: "New comment",
      placeholderKey: "docs-review-comment-placeholder",
      placeholderFallback: "Write a comment… use @name to mention",
      submitKey: "docs-review-add-thread",
      submitFallback: "Add comment",
      onSubmit: function (text) {
        var thread = NS.addThread(Object.assign({}, anchor, { text: text, firstText: text }));
        NS.statusKey("docs-review-status-thread-added", "Comment added");
        NS.renderCommentPanel();
        return !!thread;
      }
    });
  }
  NS.openNewThread = openNewThread;

  function renderCommentPanel(host) {
    var panel = host || document.getElementById("docs-review-comments");
    if (!panel) return;
    panel.textContent = "";
    panel.className = "gb-review-comments";
    var bar = el("div", "gb-review-comments-bar");
    var select = el("select", "gb-review-filter");
    select.setAttribute("aria-label", NS.tr("docs-review-filter-label", "Filter"));
    select.setAttribute("data-i18n-aria-label", "docs-review-filter-label");
    FILTERS.forEach(function (spec) {
      var option = el("option");
      option.value = spec[0];
      option.setAttribute("data-i18n", spec[1]);
      option.textContent = NS.tr(spec[1], spec[2]);
      select.appendChild(option);
    });
    bar.appendChild(select);
    var add = el("button", "gb-review-btn gb-review-btn-primary");
    add.setAttribute("type", "button");
    add.setAttribute("data-i18n", "docs-review-thread-new");
    add.textContent = NS.tr("docs-review-thread-new", "New comment");
    add.addEventListener("click", openNewThread);
    bar.appendChild(add);
    panel.appendChild(bar);
    var list = el("div", "gb-review-thread-list");
    var filter = select.value || "open";
    var threads = NS.filterThreads(filter);
    if (!threads.length) {
      var empty = el("p", "gb-review-empty");
      empty.setAttribute("data-i18n", "docs-review-no-threads");
      empty.textContent = NS.tr("docs-review-no-threads", "No comment threads yet");
      list.appendChild(empty);
    }
    threads.forEach(function (thread) { list.appendChild(threadCard(thread)); });
    panel.appendChild(list);
    select.addEventListener("change", function () {
      filter = select.value;
      list.textContent = "";
      NS.filterThreads(filter).forEach(function (thread) { list.appendChild(threadCard(thread)); });
      if (!list.children.length) {
        var none = el("p", "gb-review-empty");
        none.setAttribute("data-i18n", "docs-review-no-threads");
        none.textContent = NS.tr("docs-review-no-threads", "No comment threads yet");
        list.appendChild(none);
      }
    });
  }
  NS.renderCommentPanel = renderCommentPanel;
})(window);
