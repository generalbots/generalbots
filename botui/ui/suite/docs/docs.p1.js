
(function () {
  const SIDEBAR_TAB_KEY = "docs_sidebar_tab";
  const SAVE_DEBOUNCE_MS = 1500;
  const TITLE_BG = "#0f172a";
  const TITLE_COLOR = "#f8fafc";
  const TITLE_BORDER = "#334155";
  const SAVE_OK_COLOR = "#94a3b8";
  const SAVE_ERR_COLOR = "#f87171";

  function getCaretCharacterOffsetWithin(element) {
    let caretOffset = 0;
    const doc = element.ownerDocument || element.document;
    const win = doc.defaultView || doc.parentWindow;
    if (win && win.getSelection) {
      const sel = win.getSelection();
      if (sel.rangeCount > 0) {
        const range = sel.getRangeAt(0);
        const preCaretRange = range.cloneRange();
        preCaretRange.selectNodeContents(element);
        preCaretRange.setEnd(range.startContainer, range.startOffset);
        caretOffset = preCaretRange.toString().length;
      }
    }
    return caretOffset;
  }

  function getPlainTextLength(root) {
    return (root.textContent || "").length;
  }

  function applyDeltaEdit(article, position, content, removeLength) {
    const doc = article.ownerDocument || article.document;
    if (!doc) return;
    const remove = Math.max(0, removeLength || 0);
    const insert = content == null ? "" : String(content);
    const walker = doc.createTreeWalker(article, NodeFilter.SHOW_TEXT, null);
    const range = doc.createRange();
    let remaining = position;
    let startNode = null;
    let startOffset = 0;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const len = node.textContent.length;
      if (remaining <= len) {
        startNode = node;
        startOffset = remaining;
        break;
      }
      remaining -= len;
    }
    if (!startNode) {
      startNode = article;
      startOffset = article.childNodes.length;
    }
    range.setStart(startNode, startOffset);
    let endNode = startNode;
    let endOffset = startOffset;
    let toRemove = remove;
    if (toRemove > 0) {
      const walker2 = doc.createTreeWalker(article, NodeFilter.SHOW_TEXT, null);
      let collected = 0;
      let lastNode = null;
      let lastOffset = 0;
      while (walker2.nextNode()) {
        const node = walker2.currentNode;
        const len = node.textContent.length;
        if (collected + len > position) {
          lastNode = node;
          lastOffset = position - collected;
          collected = position + (len - lastOffset);
          break;
        }
        collected += len;
        lastNode = node;
        lastOffset = len;
      }
      let remainingRemove = toRemove;
      let cursor = lastNode;
      let cursorOffset = lastOffset || 0;
      while (remainingRemove > 0 && cursor) {
        const len = (cursor.textContent || "").length;
        const take = Math.min(len - cursorOffset, remainingRemove);
        if (take < len - cursorOffset) {
          cursor = cursor.splitText(cursorOffset + take);
          cursorOffset = 0;
        } else {
          const next = walker2.nextNode();
          cursor.deleteData(cursorOffset, len - cursorOffset);
          cursor = next;
          cursorOffset = 0;
        }
        remainingRemove -= take;
      }
      endNode = cursor || lastNode;
      endOffset = cursorOffset;
    }
    range.setEnd(endNode, endOffset);
    const frag = doc.createDocumentFragment();
    if (insert.length > 0) {
      frag.appendChild(doc.createTextNode(insert));
    }
    range.deleteContents();
    if (insert.length > 0) range.insertNode(frag);
  }

  function setCaretPosition(element, offset) {
    const doc = element.ownerDocument || element.document;
    const win = doc.defaultView || doc.parentWindow;
    if (win && win.getSelection) {
      const sel = win.getSelection();
      const range = doc.createRange();
      let currentOffset = 0;
      let nodeToSelect = null;
      let offsetWithinNode = 0;
      function traverse(node) {
        if (nodeToSelect) return;
        if (node.nodeType === 3) {
          const length = node.textContent.length;
          if (currentOffset + length >= offset) {
            nodeToSelect = node;
            offsetWithinNode = offset - currentOffset;
          } else {
            currentOffset += length;
          }
        } else {
          for (let i = 0; i < node.childNodes.length; i++) {
            traverse(node.childNodes[i]);
          }
        }
      }
      traverse(element);
      if (nodeToSelect) {
        range.setStart(nodeToSelect, offsetWithinNode);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
      }
    }
  }

  function $(s, r) { return (r || document).querySelector(s); }
  function $$(s, r) { return Array.from((r || document).querySelectorAll(s)); }

  document.addEventListener("click", function (e) {
    const tab = e.target.closest("[data-sidebar-tab]");
    if (tab) {
      const which = tab.dataset.sidebarTab;
      $$(".sidebar-tab").forEach(function (b) {
        b.classList.toggle("active", b === tab);
        b.style.background = b === tab ? "#1e293b" : TITLE_BG;
        b.style.color = b === tab ? TITLE_COLOR : "#94a3b8";
      });
      $$(".sidebar-content").forEach(function (c) {
        c.style.display = c.dataset.sidebarContent === which ? "flex" : "none";
      });
      try { sessionStorage.setItem(SIDEBAR_TAB_KEY, which); } catch (_) {}
    }
  });

  function initSidebar() {
    let saved = null;
    try { saved = sessionStorage.getItem(SIDEBAR_TAB_KEY); } catch (_) {}
    if (saved) {
      const btn = document.querySelector('[data-sidebar-tab="' + saved + '"]');
      if (btn) btn.click();
    }
  }

  function setSaveStatus(text, isError) {
    const el = document.getElementById("saveStatus");
    if (el) {
      el.textContent = text;
      el.style.color = isError ? SAVE_ERR_COLOR : SAVE_OK_COLOR;
    }
  }

  let saveTimer = null;
  function scheduleSave(id, content) {
    clearTimeout(saveTimer);
    setSaveStatus("Saving...", false);
    saveTimer = setTimeout(function () {
      fetch("/api/docs/autosave", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: id || "current", content: content })
      })
        .then(function (r) { if (!r.ok) throw new Error("http " + r.status); return r.json(); })
        .then(function () { setSaveStatus("All changes saved", false); })
        .catch(function () { setSaveStatus("Save failed", true); });
    }, SAVE_DEBOUNCE_MS);
  }

  function attachEditorHandlers(host) {
    if (!host) return;
    const article = host.querySelector("article[contenteditable]");
    if (!article) return;
    let lastTextLength = getPlainTextLength(article);
    let pendingEdit = null;
    article.addEventListener("beforeinput", function (e) {
      pendingEdit = { inputType: e.inputType, data: e.data, targetRanges: e.getTargetRanges ? e.getTargetRanges() : [] };
    });
    article.addEventListener("input", function (e) {
      scheduleSave(article.dataset.docId, article.innerHTML);
      updatePageCount();
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        const sel = window.getSelection();
        const pos = sel && sel.rangeCount ? getCaretCharacterOffsetWithin(article) : 0;
        const newLength = getPlainTextLength(article);
        const ed = pendingEdit || {};
        var content = null;
        var removeLen = 0;
        if (ed.inputType === "insertText" && ed.data) {
          content = ed.data;
        } else if (ed.inputType === "insertFromPaste" || ed.inputType === "insertFromDrop") {
          content = (article.textContent || "").substring(pos, pos + (newLength - lastTextLength));
        } else if (ed.inputType === "deleteContentBackward") {
          removeLen = lastTextLength - newLength;
        } else if (ed.inputType === "deleteContentForward") {
          removeLen = lastTextLength - newLength;
        } else if (ed.inputType && ed.inputType.indexOf("delete") >= 0) {
          removeLen = lastTextLength - newLength;
        } else if (newLength !== lastTextLength) {
          content = (article.textContent || "").substring(pos, pos + Math.max(0, newLength - lastTextLength));
          removeLen = Math.max(0, lastTextLength - newLength);
        }
        if (content !== null || removeLen > 0) {
          window.GBCollab.debouncedTypingStart(pos);
          window.GBCollab.sendEdit({ position: pos, content: content, length: content ? content.length : 0, removeLength: removeLen });
        }
      }
      lastTextLength = getPlainTextLength(article);
      pendingEdit = null;
    });
    article.addEventListener("blur", function () {
      if (saveTimer) {
        clearTimeout(saveTimer);
        scheduleSave(article.dataset.docId, article.innerHTML);
      }
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        window.GBCollab.sendTypingStop();
      }
    });
    article.addEventListener("keyup", function () {
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        const sel = window.getSelection();
        if (sel && sel.rangeCount) {
          const range = sel.getRangeAt(0);
          const start = range.startOffset;
          const end = range.endOffset;
          if (start !== end) window.GBCollab.sendSelection(start, end);
        }
      }
    });
    article.addEventListener("click", function () {
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        const sel = window.getSelection();
        if (sel && sel.rangeCount) {
          window.GBCollab.sendCursor(sel.getRangeAt(0).startOffset);
        }
      }
    });
  }

  const DocumentAIDriver = {
    summarize: function (id) {
      return fetch("/api/docs/ai", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: id, action: "summarize" })
      }).then(function (r) { return r.json(); }).then(function (j) { return j.result || j.content || ""; }).catch(function () { return ""; });
    },
    expand: function (id, prompt) {
      return fetch("/api/docs/ai", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: id, action: "expand", prompt: prompt })
      }).then(function (r) { return r.json(); }).then(function (j) { return j.result || ""; }).catch(function () { return ""; });
    },
    improve: function (id) {
      return fetch("/api/docs/ai", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: id, action: "improve" })
      }).then(function (r) { return r.json(); }).then(function (j) { return j.result || ""; }).catch(function () { return ""; });
    },
    simplify: function (id) {
      return fetch("/api/docs/ai", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: id, action: "simplify" })
      }).then(function (r) { return r.json(); }).then(function (j) { return j.result || ""; }).catch(function () { return ""; });
    }
  };

  function updatePageCount() {
    const el = document.getElementById("pageCount");
    if (!el) return;
    const article = getActiveArticle();
    if (!article) { el.textContent = "1 page"; return; }
    const h = article.scrollHeight;
    const pageH = 1056;
    var count = Math.max(1, Math.ceil(h / pageH));
    el.textContent = count + " page" + (count !== 1 ? "s" : "");
  }

  function injectEditorStyles() {
    if (document.getElementById("docs-editor-styles")) return;
    const style = document.createElement("style");
    style.id = "docs-editor-styles";
    style.textContent = [
      "article.docs-doc-view{background:#ffffff;color:#1e293b;padding:96px;width:816px;margin:32px auto;border:1px solid #e2e8f0;border-radius:4px;font-family:Georgia,serif;font-size:16px;line-height:1.7;min-height:1056px;outline:none;box-shadow:0 10px 30px rgba(0,0,0,0.25);box-sizing:border-box;}",
      "article.docs-doc-view h1{font-size:32px;font-weight:700;margin:0.8em 0 0.4em;color:#0f172a;}",
      "article.docs-doc-view h2{font-size:24px;font-weight:600;margin:0.7em 0 0.35em;color:#0f172a;}",
      "article.docs-doc-view h3{font-size:20px;font-weight:600;margin:0.6em 0 0.3em;color:#334155;}",
      "article.docs-doc-view p{margin:0.5em 0;}",
      "article.docs-doc-view blockquote{border-left:3px solid #3b82f6;padding-left:16px;color:#475569;font-style:italic;margin:0.8em 0;}",
      "article.docs-doc-view ul,article.docs-doc-view ol{margin:0.5em 0 0.5em 1.5em;}",
      "article.docs-doc-view li{margin:0.25em 0;}",
      "article.docs-doc-view a{color:#2563eb;text-decoration:underline;}",
      "article.docs-doc-view code{background:#f1f5f9;padding:2px 6px;border-radius:3px;font-family:'Courier New',monospace;font-size:0.9em;color:#b45309;}",
      "article.docs-doc-view pre{background:#f1f5f9;padding:12px 16px;border-radius:6px;overflow-x:auto;color:#334155;}",
      "article.docs-doc-view:focus{outline:2px solid #3b82f6;outline-offset:-2px;}",
      "article.docs-doc-view .docs-page-break{break-after:page;page-break-after:always;display:block;height:0;border:0;border-top:1px dashed #cbd5e1;margin:24px 0;}",
      "article.docs-doc-view .docs-header-zone{min-height:32px;padding:6px 0;border-bottom:1px dotted #e2e8f0;margin-bottom:18px;color:#64748b;font-size:12px;font-style:italic;}",
      "article.docs-doc-view .docs-footer-zone{min-height:32px;padding:6px 0;border-top:1px dotted #e2e8f0;margin-top:18px;color:#64748b;font-size:12px;font-style:italic;}",
      "@media print{article.docs-doc-view{box-shadow:none;border:none;margin:0;}.docs-page-break{break-after:page;page-break-after:always;}}"
    ].join("");
    document.head.appendChild(style);
  }

  document.addEventListener("htmx:afterSwap", function (e) {
    if (e.target.id === "docs-content") {
      attachEditorHandlers(e.target);
    }
  });

  function initAuth() {
    if (window.GBAuthGuard) GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
  }

  function getActiveArticle() {
    return document.querySelector("article[contenteditable]");
  }

  function insertPageBreak() {
    const article = getActiveArticle();
    if (!article) return;
    article.focus();
    const br = document.createElement("div");
    br.className = "docs-page-break";
    br.contentEditable = "false";
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0) {
      const range = sel.getRangeAt(0);
      range.collapse(false);
      range.insertNode(br);
      range.setStartAfter(br);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    } else {
      article.appendChild(br);
    }
    scheduleSave(article.dataset.docId, article.innerHTML);
  }

  function insertHeaderFooterZone(kind) {
    const article = getActiveArticle();
    if (!article) return;
    article.focus();
    const zone = document.createElement("div");
    zone.className = kind === "header" ? "docs-header-zone" : "docs-footer-zone";
    zone.contentEditable = "true";
    zone.dataset.zoneKind = kind;
    zone.setAttribute("data-placeholder", kind === "header" ? "Cabeçalho — clique para editar" : "Rodapé — clique para editar");
    if (!zone.textContent) zone.textContent = zone.getAttribute("data-placeholder");
    if (kind === "header") article.insertBefore(zone, article.firstChild);
    else article.appendChild(zone);
    scheduleSave(article.dataset.docId, article.innerHTML);
  }

  function openHeaderFooterModal() {
    const host = document.getElementById("modal-container");
    if (!host) return;
    host.innerHTML = [
      "<div class=\"docs-modal\" id=\"docs-hf-modal\" style=\"position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;\">",
      "<div style=\"background:#1e293b;border:1px solid #334155;border-radius:12px;width:520px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;\">",
      "<h3 style=\"margin:0;font-size:16px;\">Cabeçalho e Rodapé</h3>",
      "<label style=\"font-size:12px;color:#94a3b8;\">Cabeçalho (aparece no topo de cada página)<input id=\"hf-header\" type=\"text\" placeholder=\"Cabeçalho...\" style=\"width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;\"/></label>",
      "<label style=\"font-size:12px;color:#94a3b8;\">Rodapé (aparece no final de cada página)<input id=\"hf-footer\" type=\"text\" placeholder=\"Rodapé...\" style=\"width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;\"/></label>",
      "<div style=\"display:flex;gap:8px;justify-content:flex-end;\">",
      "<button id=\"hf-cancel\" type=\"button\" style=\"background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;\">Cancelar</button>",
      "<button id=\"hf-apply\" type=\"button\" style=\"background:#3b82f6;color:white;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;\">Aplicar</button>",
      "</div></div></div>"
    ].join("");
    document.getElementById("hf-cancel").onclick = function () { host.innerHTML = ""; };
    document.getElementById("hf-apply").onclick = function () {
      const header = document.getElementById("hf-header").value || "";
      const footer = document.getElementById("hf-footer").value || "";
      const article = getActiveArticle();
      if (!article) { host.innerHTML = ""; return; }
      let h = article.querySelector(".docs-header-zone");
      if (!h) { h = document.createElement("div"); h.className = "docs-header-zone"; h.contentEditable = "true"; article.insertBefore(h, article.firstChild); }
      h.textContent = header;
      let f = article.querySelector(".docs-footer-zone");
      if (!f) { f = document.createElement("div"); f.className = "docs-footer-zone"; f.contentEditable = "true"; article.appendChild(f); }
      f.textContent = footer;
      scheduleSave(article.dataset.docId, article.innerHTML);
      host.innerHTML = "";
    };
  }

  function initCollab() {
    if (!window.GBCollab) return;
    const connStatus = document.getElementById("gb-conn-status");
    const docId = (document.getElementById("docTitle") && document.getElementById("docTitle").value) || "current";
    const typingEl = document.getElementById("typing-indicator");
    window.GBCollab.connect({
      app: "docs",
      docId: docId,
      collaboratorsEl: document.getElementById("collaborators"),
      typingEl: typingEl,
      onConnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status online"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "online"; }
      },
      onDisconnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status offline"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "offline"; }
      },
      onTyping: function (msg) {
        const map = (window.__gbTypingUsers = window.__gbTypingUsers || new Map());
        if (msg.msg_type === "typing_start") map.set(msg.user_id, msg);
        else map.delete(msg.user_id);
        const arr = Array.from(map.values()).filter(function (m) { return Date.now() - (m.timestamp || 0) < 5000; });
        if (window.GBCollab && window.GBCollab.helpers) {
          window.GBCollab.helpers.renderTypingIndicator(typingEl, arr);
        }
      },
      onEdit: function (msg) {
        if (!msg) return;
        const article = document.querySelector("article[contenteditable]");
        if (!article || article.dataset.suppressRemote) return;
        const hasDelta = typeof msg.position === "number" && (typeof msg.removeLength === "number" || typeof msg.length === "number");
        article.dataset.suppressRemote = "1";
        if (hasDelta) {
          const pos = Math.max(0, msg.position | 0);
          const removeLength = typeof msg.removeLength === "number" ? msg.removeLength : (msg.length && msg.length > 0 && msg.content === "" ? msg.length : 0);
          applyDeltaEdit(article, pos, msg.content, removeLength);
        } else if (typeof msg.content === "string") {
          const offset = getCaretCharacterOffsetWithin(article);
          article.innerHTML = msg.content;
          setCaretPosition(article, offset);
        }
        article.dataset.suppressRemote = "";
      }
    });
  }

  window.addEventListener("DOMContentLoaded", function () {
    injectEditorStyles();
    initSidebar();
    initAuth();
    initCollab();
    window.DocsEditor = {
      setSaveStatus: setSaveStatus,
      scheduleSave: scheduleSave,
      AI: DocumentAIDriver,
      insertPageBreak: insertPageBreak,
      openHeaderFooterModal: openHeaderFooterModal,
      insertHeaderFooterZone: insertHeaderFooterZone,
      updatePageCount: updatePageCount
    };
  });
})();
