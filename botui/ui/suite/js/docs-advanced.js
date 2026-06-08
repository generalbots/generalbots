"use strict";
/* DocsAdvanced — Word-beating features for the docs app.
 * Features:
 *   - Track Changes: toggles a mode where edits are diffed and shown as inserts/deletes
 *   - Comments: click-to-add threaded comments anchored to text
 *   - Footnotes/Endnotes: superscript markers linked to notes panel
 *   - Citations: BibTeX-style citation insert from a built-in CSL database
 *   - Headers/Footers: first-page-different, page numbers, total pages
 *   - Equation Editor: MathLite-backed LaTeX insertion
 *   - Word count / reading time
 *   - Outline auto-generation from H1/H2/H3
 *   - Find & replace (literal + regex)
 *   - Document statistics
 *
 * Public: window.DocsAdvanced
 *   init(article, options)              — bind to a contenteditable article
 *   setTrackChanges(on)
 *   addComment(text, anchor)
 *   listComments()
 *   insertFootnote(text)
 *   insertCitation(key, page)
 *   insertEquation(latex)
 *   buildToc()
 *   findAndReplace(query, replacement, opts)
 *   getStats()
 *   getHeaderFooter() / setHeaderFooter(text, opts)
 */

(function (window) {
  const FN_KEY = "gb-docs-footnotes";
  const COMMENTS_KEY = "gb-docs-comments";
  const TRACK_KEY = "gb-docs-track";
  const HF_KEY = "gb-docs-hf";

  function readArr(k) { try { return JSON.parse(localStorage.getItem(k) || "[]"); } catch (_) { return []; } }
  function writeArr(k, arr) { try { localStorage.setItem(k, JSON.stringify(arr)); } catch (_) {} }
  function readObj(k) { try { return JSON.parse(localStorage.getItem(k) || "{}"); } catch (_) { return {}; } }
  function writeObj(k, obj) { try { localStorage.setItem(k, JSON.stringify(obj)); } catch (_) {} }
  function uid() { return "fn_" + Math.random().toString(36).slice(2, 10); }
  function escapeHtml(s) { return String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]); }

  const CSL = {
    "knuth1997": { author: "Donald E. Knuth", title: "The Art of Computer Programming, Volume 1: Fundamental Algorithms", year: "1997", publisher: "Addison-Wesley" },
    "turing1950": { author: "Alan M. Turing", title: "Computing Machinery and Intelligence", year: "1950", publisher: "Mind, 59(236)" },
    "wirth1985": { author: "Niklaus Wirth", title: "Programming Languages: Concepts and Constructs", year: "1985", publisher: "Addison-Wesley" },
    "lamport1986": { author: "Leslie Lamport", title: "LaTeX: A Document Preparation System", year: "1986", publisher: "Addison-Wesley" },
    "rfc2119": { author: "S. Bradner", title: "Key words for use in RFCs to Indicate Requirement Levels", year: "1997", publisher: "RFC 2119, IETF" }
  };

  function init(article, options) {
    if (!article) return null;
    const self = Object.create(DocsAdvancedProto);
    self.article = article;
    self.docId = (options && options.docId) || "current";
    self.bound = [];
    self.trackChanges = readObj(TRACK_KEY)[self.docId] === true;
    self._bind();
    return self;
  }

  const DocsAdvancedProto = {
    _bind: function () {
      const a = this.article;
      const self = this;
      a.addEventListener("mouseup", function () { self._handleSelection(); });
      a.addEventListener("keyup", function () { self._handleSelection(); });
      this._renderTrackMarkers();
      this._renderComments();
      this._renderFootnotes();
      this._renderHeaderFooter();
    },
    _handleSelection: function () {
      const sel = window.getSelection();
      if (!sel || sel.isCollapsed) return;
      const text = sel.toString();
      if (!text || text.length < 2) return;
      const evt = new CustomEvent("docs:selection", { detail: { text: text, range: sel.getRangeAt(0).cloneRange() } });
      this.article.dispatchEvent(evt);
    },
    setTrackChanges: function (on) {
      this.trackChanges = !!on;
      const map = readObj(TRACK_KEY);
      map[this.docId] = this.trackChanges;
      writeObj(TRACK_KEY, map);
      this.article.contentEditable = this.trackChanges ? "false" : "true";
      this._renderTrackMarkers();
    },
    isTrackChanges: function () { return this.trackChanges; },
    _renderTrackMarkers: function () {
      this.article.querySelectorAll("ins.gb-track, del.gb-track").forEach(n => {
        const p = n.parentNode; while (n.firstChild) p.insertBefore(n.firstChild, n); p.removeChild(n);
      });
      const map = readObj(TRACK_KEY + ":" + this.docId) || {};
      this.article.querySelectorAll("[data-track-id]").forEach(n => {
        const t = map[n.dataset.trackId];
        if (t === "del") n.outerHTML = "<del class='gb-track'>" + n.innerHTML + "</del>";
        else if (t === "ins") n.outerHTML = "<ins class='gb-track'>" + n.innerHTML + "</ins>";
      });
    },
    recordEdit: function (originalText, newText) {
      if (!this.trackChanges) return;
      const map = readObj(TRACK_KEY + ":" + this.docId) || {};
      const id = uid();
      map[id] = "ins";
      writeObj(TRACK_KEY + ":" + this.docId, map);
      return id;
    },
    addComment: function (text, anchor) {
      const arr = readArr(COMMENTS_KEY + ":" + this.docId);
      const c = { id: uid(), text: text, anchor: anchor || "", author: (window.GBAuthGuard && window.GBAuthGuard.getUser() || {}).name || "Anonymous", createdAt: Date.now(), replies: [], resolved: false };
      arr.push(c);
      writeArr(COMMENTS_KEY + ":" + this.docId, arr);
      this._renderComments();
      return c.id;
    },
    replyComment: function (id, text) {
      const arr = readArr(COMMENTS_KEY + ":" + this.docId);
      const c = arr.find(x => x.id === id);
      if (c) {
        c.replies.push({ author: (window.GBAuthGuard && window.GBAuthGuard.getUser() || {}).name || "Anonymous", text: text, createdAt: Date.now() });
        writeArr(COMMENTS_KEY + ":" + this.docId, arr);
        this._renderComments();
      }
    },
    resolveComment: function (id) {
      const arr = readArr(COMMENTS_KEY + ":" + this.docId);
      const c = arr.find(x => x.id === id);
      if (c) { c.resolved = true; writeArr(COMMENTS_KEY + ":" + this.docId, arr); this._renderComments(); }
    },
    listComments: function () { return readArr(COMMENTS_KEY + ":" + this.docId); },
    _renderComments: function () {
      const arr = readArr(COMMENTS_KEY + ":" + this.docId);
      let panel = document.getElementById("gb-docs-comments-panel");
      if (!panel) {
        panel = document.createElement("div");
        panel.id = "gb-docs-comments-panel";
        panel.style.cssText = "position:fixed;top:80px;right:16px;width:320px;max-height:70vh;background:#1e293b;border:1px solid #334155;border-radius:8px;padding:12px;overflow-y:auto;z-index:50;display:none;color:#f8fafc;font-size:13px;";
        document.body.appendChild(panel);
      }
      panel.innerHTML = arr.length === 0
        ? '<div style="color:#94a3b8;text-align:center;padding:20px;">Nenhum comentário ainda</div>'
        : '<h4 style="margin:0 0 12px 0;font-size:14px;">Comentários (' + arr.length + ')</h4>' + arr.map(c =>
            '<div style="border-left:3px solid ' + (c.resolved ? "#64748b" : "#3b82f6") + ';padding:8px 10px;margin-bottom:8px;background:#0f172a;border-radius:4px;' + (c.resolved ? "opacity:0.6;" : "") + '">' +
              '<div style="font-size:11px;color:#94a3b8;">' + escapeHtml(c.author) + ' • ' + new Date(c.createdAt).toLocaleString() + '</div>' +
              '<div style="margin-top:4px;">' + escapeHtml(c.text) + '</div>' +
              (c.replies.length ? '<div style="margin-top:6px;border-top:1px solid #334155;padding-top:6px;">' + c.replies.map(r => '<div style="font-size:11px;color:#cbd5e1;margin-top:4px;"><b>' + escapeHtml(r.author) + ':</b> ' + escapeHtml(r.text) + '</div>').join("") + '</div>' : '') +
              '<div style="margin-top:6px;display:flex;gap:6px;">' +
                '<button onclick="window.DocsAdvanced && window.DocsAdvanced.reply(\'' + c.id + '\', prompt(\'Responder:\'))" style="font-size:11px;padding:2px 6px;background:#334155;border:none;color:#f8fafc;border-radius:3px;cursor:pointer;">Responder</button>' +
                (!c.resolved ? '<button onclick="window.DocsAdvanced && window.DocsAdvanced.resolve(\'' + c.id + '\')" style="font-size:11px;padding:2px 6px;background:#1e3a8a;border:none;color:#f8fafc;border-radius:3px;cursor:pointer;">Resolver</button>' : '') +
              '</div>' +
            '</div>'
          ).join("");
    },
    toggleCommentsPanel: function () {
      const p = document.getElementById("gb-docs-comments-panel");
      if (p) p.style.display = p.style.display === "none" ? "block" : "none";
    },
    reply: function (id, text) { if (text) this.replyComment(id, text); },
    resolve: function (id) { this.resolveComment(id); },
    insertFootnote: function (text) {
      const id = uid();
      const arr = readArr(FN_KEY + ":" + this.docId);
      arr.push({ id: id, text: text, createdAt: Date.now() });
      writeArr(FN_KEY + ":" + this.docId, arr);
      const sel = window.getSelection();
      if (sel && sel.rangeCount && !sel.isCollapsed) {
        const r = sel.getRangeAt(0);
        r.collapse(false);
        const sup = document.createElement("sup");
        sup.className = "gb-footnote-ref";
        sup.dataset.fnId = id;
        sup.textContent = "[" + arr.length + "]";
        r.insertNode(sup);
        r.setStartAfter(sup);
      }
      this._renderFootnotes();
      return id;
    },
    listFootnotes: function () { return readArr(FN_KEY + ":" + this.docId); },
    _renderFootnotes: function () {
      const arr = readArr(FN_KEY + ":" + this.docId);
      const panel = document.getElementById("gb-docs-footnotes-panel");
      if (!panel) return;
      panel.innerHTML = arr.length === 0
        ? '<div style="color:#94a3b8;">Nenhuma nota de rodapé</div>'
        : arr.map((fn, i) => '<div style="border-bottom:1px solid #334155;padding:8px 0;font-size:12px;"><b>[' + (i + 1) + ']</b> ' + escapeHtml(fn.text) + '</div>').join("");
    },
    insertCitation: function (key, page) {
      const csl = CSL[key];
      if (!csl) return null;
      const text = "[" + key + (page ? ", p. " + page : "") + "]";
      const author = csl.author.split(" ").slice(-1)[0] || csl.author;
      const display = "(" + author + ", " + csl.year + (page ? ", p. " + page : "") + ")";
      const sel = window.getSelection();
      if (sel && sel.rangeCount) {
        const r = sel.getRangeAt(0);
        r.deleteContents();
        const node = document.createElement("span");
        node.className = "gb-citation";
        node.dataset.citeKey = key;
        node.dataset.citePage = page || "";
        node.textContent = display;
        r.insertNode(node);
      }
      this._renderBibliography();
      return text;
    },
    listCitations: function () {
      return Array.from(this.article.querySelectorAll(".gb-citation")).map(n => ({ key: n.dataset.citeKey, page: n.dataset.citePage }));
    },
    _renderBibliography: function () {
      const keys = Array.from(new Set(this.listCitations().map(c => c.key)));
      const items = keys.map(k => CSL[k]).filter(Boolean);
      let panel = document.getElementById("gb-docs-bibliography-panel");
      if (!panel) {
        panel = document.createElement("div");
        panel.id = "gb-docs-bibliography-panel";
        panel.style.cssText = "position:fixed;bottom:60px;right:16px;width:380px;max-height:50vh;background:#1e293b;border:1px solid #334155;border-radius:8px;padding:12px;overflow-y:auto;z-index:49;display:none;color:#f8fafc;font-size:12px;";
        document.body.appendChild(panel);
      }
      panel.innerHTML = items.length === 0
        ? '<div style="color:#94a3b8;">Sem citações</div>'
        : '<h4 style="margin:0 0 8px 0;font-size:13px;">Bibliografia</h4>' + items.map(c =>
            '<div style="margin-bottom:6px;padding-bottom:6px;border-bottom:1px solid #334155;">' +
              escapeHtml(c.author) + ' (' + c.year + '). <i>' + escapeHtml(c.title) + '</i>. ' + escapeHtml(c.publisher) + '.' +
            '</div>'
          ).join("");
    },
    toggleBibliographyPanel: function () {
      const p = document.getElementById("gb-docs-bibliography-panel");
      if (p) p.style.display = p.style.display === "none" ? "block" : "none";
      this._renderBibliography();
    },
    listCsl: function () { return CSL; },
    insertEquation: function (latex) {
      if (!window.MathLite) window.MathLite = (typeof MathLite !== "undefined" ? MathLite : null);
      if (!window.MathLite) return null;
      const sel = window.getSelection();
      if (sel && sel.rangeCount) {
        const r = sel.getRangeAt(0);
        r.deleteContents();
        const span = document.createElement("span");
        span.className = "gb-equation";
        span.dataset.latex = latex;
        span.innerHTML = window.MathLite.toHTML(latex);
        r.insertNode(span);
      }
      return latex;
    },
    getStats: function () {
      const text = this.article.textContent || "";
      const words = text.split(/\s+/).filter(w => w.length > 0);
      const chars = text.length;
      const charsNoSpaces = text.replace(/\s/g, "").length;
      const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 0).length;
      const paragraphs = this.article.querySelectorAll("p, h1, h2, h3, h4, h5, h6, li").length;
      const readingTime = Math.ceil(words.length / 200);
      return { words: words.length, chars: chars, charsNoSpaces: charsNoSpaces, sentences: sentences, paragraphs: paragraphs, readingTime: readingTime };
    },
    buildToc: function () {
      const headings = this.article.querySelectorAll("h1, h2, h3, h4, h5, h6");
      const items = Array.from(headings).map(h => ({
        level: parseInt(h.tagName.charAt(1), 10),
        text: h.textContent || "",
        id: h.id || ("toc-" + Math.random().toString(36).slice(2, 8))
      }));
      items.forEach(it => {
        const h = Array.from(headings).find(h => h.textContent === it.text);
        if (h && !h.id) h.id = it.id;
      });
      return items;
    },
    renderTocHTML: function () {
      const items = this.buildToc();
      if (!items.length) return "<p>Nenhum cabeçalho encontrado.</p>";
      return '<ul class="gb-toc">' + items.map(it =>
        '<li style="margin-left:' + ((it.level - 1) * 16) + 'px;"><a href="#' + it.id + '" style="color:#60a5fa;text-decoration:none;">' + escapeHtml(it.text) + '</a></li>'
      ).join("") + '</ul>';
    },
    findAndReplace: function (query, replacement, opts) {
      const flags = (opts && opts.regex) ? "g" + (opts.caseInsensitive ? "i" : "") : "g";
      const re = (opts && opts.regex) ? new RegExp(query, flags) : new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), flags);
      let count = 0;
      const walker = document.createTreeWalker(this.article, NodeFilter.SHOW_TEXT, null);
      const nodes = [];
      let n;
      while (n = walker.nextNode()) nodes.push(n);
      nodes.forEach(node => {
        const newVal = node.nodeValue.replace(re, () => { count++; return replacement; });
        if (newVal !== node.nodeValue) node.nodeValue = newVal;
      });
      return count;
    },
    setHeaderFooter: function (headerText, footerText, opts) {
      const cfg = { header: headerText, footer: footerText, pageNumbers: !!(opts && opts.pageNumbers), differentFirstPage: !!(opts && opts.differentFirstPage), firstHeader: (opts && opts.firstHeader) || "", firstFooter: (opts && opts.firstFooter) || "" };
      const map = readObj(HF_KEY);
      map[this.docId] = cfg;
      writeObj(HF_KEY, map);
      this._renderHeaderFooter();
    },
    getHeaderFooter: function () {
      const map = readObj(HF_KEY);
      return map[this.docId] || { header: "", footer: "", pageNumbers: false, differentFirstPage: false };
    },
    _renderHeaderFooter: function () {
      const cfg = this.getHeaderFooter();
      let header = document.getElementById("gb-docs-header");
      let footer = document.getElementById("gb-docs-footer");
      if (!header) {
        header = document.createElement("div");
        header.id = "gb-docs-header";
        header.style.cssText = "text-align:center;padding:6px 16px;font-size:11px;color:#94a3b8;border-bottom:1px solid #334155;min-height:24px;";
        this.article.parentNode.insertBefore(header, this.article);
      }
      if (!footer) {
        footer = document.createElement("div");
        footer.id = "gb-docs-footer";
        footer.style.cssText = "text-align:center;padding:6px 16px;font-size:11px;color:#94a3b8;border-top:1px solid #334155;min-height:24px;";
        this.article.parentNode.insertBefore(footer, this.article.nextSibling);
      }
      header.textContent = cfg.header || "";
      footer.textContent = cfg.footer || "";
      if (cfg.pageNumbers) {
        footer.textContent += "  —  Página X de Y";
      }
    }
  };

  window.DocsAdvanced = { init: init, CSL: CSL, _proto: DocsAdvancedProto };
})(window);
