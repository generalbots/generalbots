(function (window, document) {
  "use strict";
  var N = document.__gb_mod_collab_comments;
  if (!N) return;
  var S = N.s;
  var state = N.state;
  var req = N.req;
  var node = N.node;
  var esc = N.esc;
  var HEARTBEAT_MS = N.HEARTBEAT_MS;
  function load() {
    if (!S.listEl || !state.resourceId) return;
    S.listEl.innerHTML = '<div class="gbc-loading">Loading comments\u2026</div>';
    var qs = "/comments?resource_type=" + encodeURIComponent(state.resourceType) + "&resource_id=" + encodeURIComponent(state.resourceId);
    if (state.includeChildren) qs += "&include_children=true";
    req(qs)
      .then(function (items) {
        S.listEl.innerHTML = "";
        S.commentAuthors = [];
        var all = items || [];
        var seen = {};
        all.forEach(function (c) {
          if (c.author_name && !seen[c.author_name]) {
            seen[c.author_name] = true;
            S.commentAuthors.push({ author_id: c.author_id, author_name: c.author_name });
          }
        });
        var visible = all.filter(function (c) {
          if (state.filter === "open") return !c.resolved;
          if (state.filter === "resolved") return !!c.resolved;
          return true;
        });
        if (!visible.length) {
          S.listEl.innerHTML = '<div class="gbc-empty">No ' + (state.filter === "resolved" ? "resolved" : "open") + ' comments. Use @name to mention someone.</div>';
        } else {
          visible.forEach(function (c) { S.listEl.appendChild(node(c, false)); });
        }
        refreshMentionCandidates();
        loadPresence();
      })
      .catch(function (e) {
        S.listEl.innerHTML = '<div class="gbc-error">Failed to load comments: ' + esc(e.message) + "</div>";
      });
  }

  function loadPresence() {
    if (!S.presenceEl || !state.resourceId) return;
    req("/presence?resource_type=" + encodeURIComponent(state.resourceType) + "&resource_id=" + encodeURIComponent(state.resourceId))
      .then(function (items) {
        S.presenceUsers = items || [];
        refreshMentionCandidates();
        if (!items || !items.length) { S.presenceEl.style.display = "none"; S.presenceEl.textContent = ""; return; }
        var typing = items.filter(function (p) { return p.typing; });
        var names = items.map(function (p) { return p.user_name; });
        S.presenceEl.textContent = typing.length ? "typing: " + names.join(", ") : "viewing: " + names.join(", ");
        S.presenceEl.style.display = "inline-block";
      })
      .catch(function () {});
  }

  function sendPresence(typing) {
    if (!state.resourceId) return;
    req("/presence", {
      method: "POST",
      body: JSON.stringify({ resource_type: state.resourceType, resource_id: state.resourceId, typing: !!typing })
    }).catch(function () { /* presence is best-effort */ });
  }

  function heartbeat() {
    stopHeartbeat();
    if (!state.resourceId) return;
    sendPresence(false);
    S.heartbeatTimer = setInterval(function () { sendPresence(S.typingSent); }, HEARTBEAT_MS);
  }

  function stopHeartbeat() {
    if (S.heartbeatTimer) { clearInterval(S.heartbeatTimer); S.heartbeatTimer = null; }
    S.typingSent = false;
  }

  function addComment(body, parentId) {
    var payload = { resource_type: state.resourceType, resource_id: state.resourceId, body: body };
    if (parentId) payload.parent_id = parentId;
    return req("/comments", { method: "POST", body: JSON.stringify(payload) });
  }

  function post() {
    if (!S.inputEl) return;
    var body = S.inputEl.value.trim();
    if (!body) return;
    S.inputEl.value = "";
    S.typingSent = false;
    addComment(body, null)
      .then(function () {
        load();
        if (window.GBCollabA11y) window.GBCollabA11y.announce("Comment added");
      })
      .catch(function (e) { notify("Comment failed: " + e.message, "error"); });
  }

  function react(commentId, emoji) {
    req("/comments/" + commentId + "/reactions", { method: "POST", body: JSON.stringify({ emoji: emoji }) })
      .then(load)
      .catch(function (e) { notify("Reaction failed: " + e.message, "error"); });
  }

  function del(commentId) {
    var remove = function () {
      req("/comments/" + commentId, { method: "DELETE" })
        .then(load)
        .catch(function (e) { notify("Delete failed: " + e.message, "error"); });
    };
    if (window.WindowManager && window.WindowManager.confirmFloating) {
      window.WindowManager.confirmFloating("Delete comment", "Delete this comment?", remove, null, "Delete");
    } else if (window.confirm("Delete this comment?")) {
      remove();
    }
  }

  function reply(commentId) {
    var done = function (body) {
      if (!body) return;
      addComment(body.trim(), commentId).then(load).catch(function (e) { notify("Reply failed: " + e.message, "error"); });
    };
    if (window.WindowManager && window.WindowManager.promptFloating) {
      window.WindowManager.promptFloating("Reply", "Reply to comment:", "", done);
    } else {
      done(window.prompt("Reply to comment:"));
    }
  }

  function resolve(commentId, resolved) {
    req("/comments/" + commentId + "/resolve", { method: "POST", body: JSON.stringify({ resolved: resolved }) })
      .then(function () {
        load();
        if (window.GBCollabA11y) window.GBCollabA11y.announce(resolved ? "Comment resolved" : "Comment reopened");
      })
      .catch(function (e) { notify("Resolve failed: " + e.message, "error"); });
  }

  function markRead() {
    if (!state.resourceId) return;
    req("/comments/read", {
      method: "POST",
      body: JSON.stringify({ resource_type: state.resourceType, resource_id: state.resourceId })
    }).then(function () { if (S.badgeRefreshFn) S.badgeRefreshFn(); }).catch(function () {});
  }

  function notify(msg, type) {
    if (typeof state.notify === "function") { state.notify(msg, type); return; }
    // Minimal fallback toast.
    var t = document.createElement("div");
    t.style.cssText = "position:fixed;bottom:24px;right:24px;z-index:100001;padding:10px 16px;border-radius:8px;color:#fff;font-size:13px;box-shadow:0 4px 12px rgba(0,0,0,.3);background:" + (type === "error" ? "#ef4444" : "#3b82f6");
    t.textContent = msg;
    document.body.appendChild(t);
    setTimeout(function () { t.remove(); }, 4000);
  }

  // Merge active presence users with the thread's authors into the
  // autocomplete candidate list (deduplicated by display name).
  function refreshMentionCandidates() {
    var seen = {};
    var out = [];
    function add(id, name) {
      if (!name || seen[name]) return;
      seen[name] = true;
      out.push({ id: id || name, name: name });
    }
    S.presenceUsers.forEach(function (p) { add(p.user_id, p.user_name); });
    S.commentAuthors.forEach(function (a) { add(a.author_id, a.author_name); });
    S.mentionCandidates = out;
  }

  // The partial @token being typed at the caret, or null when not mentioning.
  function currentMentionToken() {
    if (!S.inputEl) return null;
    var value = S.inputEl.value;
    var pos = S.inputEl.selectionStart == null ? value.length : S.inputEl.selectionStart;
    var before = value.slice(0, pos);
    var at = before.lastIndexOf("@");
    if (at === -1) return null;
    var partial = before.slice(at + 1);
    if (!/^[\w.@-]*$/.test(partial)) return null;
    return { at: at, partial: partial };
  }

  function hideMentions() {
    if (S.mentionBox) { S.mentionBox.style.display = "none"; S.mentionBox.innerHTML = ""; }
    S.mentionItems = [];
    S.mentionIndex = -1;
    S.activeMention = null;
  }

  function highlightMention(idx) {
    if (!S.mentionBox) return;
    var children = S.mentionBox.children;
    for (var i = 0; i < children.length; i++) {
      children[i].style.background = i === idx ? "#334155" : "none";
    }
  }

  function renderMentions(token) {
    if (!S.mentionBox) return;
    var lower = token.partial.toLowerCase();
    var matches = S.mentionCandidates.filter(function (c) {
      return c.name.toLowerCase().indexOf(lower) === 0;
    }).slice(0, 8);
    if (!matches.length) { hideMentions(); return; }
    S.mentionBox.innerHTML = "";
    S.mentionItems = [];
    matches.forEach(function (c) {
      var b = document.createElement("button");
      b.type = "button";
      b.className = "gbc-mention-item";
      b.textContent = c.name;
      b.style.cssText = "display:block;width:100%;text-align:left;background:none;border:none;color:#f8fafc;font-size:13px;padding:8px 12px;cursor:pointer;";
      b.addEventListener("mouseover", function () {
        S.mentionIndex = S.mentionItems.indexOf(b);
        highlightMention(S.mentionIndex);
      });
      b.addEventListener("mousedown", function (e) { e.preventDefault(); applyMention(token.at, c.name); });
      S.mentionBox.appendChild(b);
      S.mentionItems.push(b);
    });
    S.activeMention = token;
    S.mentionIndex = 0;
    highlightMention(0);
    S.mentionBox.style.display = "block";
  }

  function selectMentionIndex() {
    if (S.mentionIndex < 0 || S.mentionIndex >= S.mentionItems.length || !S.activeMention) return false;
    applyMention(S.activeMention.at, S.mentionItems[S.mentionIndex].textContent);
    return true;
  }

  function applyMention(at, name) {
    if (!S.inputEl) return;
    var value = S.inputEl.value;
    var pos = S.inputEl.selectionStart == null ? value.length : S.inputEl.selectionStart;
    var newValue = value.slice(0, at) + "@" + name + " " + value.slice(pos);
    S.inputEl.value = newValue;
    var caret = at + name.length + 2; // after "@name "
    S.inputEl.focus();
    try { S.inputEl.setSelectionRange(caret, caret); } catch (_) {}
    hideMentions();
  }


  N.load = load;
  N.loadPresence = loadPresence;
  N.sendPresence = sendPresence;
  N.heartbeat = heartbeat;
  N.stopHeartbeat = stopHeartbeat;
  N.addComment = addComment;
  N.post = post;
  N.react = react;
  N.del = del;
  N.reply = reply;
  N.resolve = resolve;
  N.markRead = markRead;
  N.notify = notify;
  N.refreshMentionCandidates = refreshMentionCandidates;
  N.currentMentionToken = currentMentionToken;
  N.hideMentions = hideMentions;
  N.highlightMention = highlightMention;
  N.renderMentions = renderMentions;
  N.selectMentionIndex = selectMentionIndex;
  N.applyMention = applyMention;
})(window, document);
