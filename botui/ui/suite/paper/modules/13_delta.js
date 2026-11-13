"use strict";
/* #2172/#2175 — gb-doc-delta for paper (block model). */
(function () {
  // Markdown-lite → blocks. Only the constructs the plan prompt promises:
  // '# '/'## ' headings, '- ' bullets, '1. ' numbered items, blank-line
  // paragraphs, **bold**.
  function paperMarkdown(text) {
    var html = "";
    var list = null;
    function closeList() { if (list) { html += "</" + list + ">"; list = null; } }
    String(text).split(/\r?\n/).forEach(function (line) {
      var trimmed = line.trim();
      if (!trimmed) { closeList(); return; }
      var bold = function (value) {
        return value.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
      };
      if (/^#{1,3}\s+/.test(trimmed)) {
        closeList();
        html += "<h2>" + bold(trimmed.replace(/^#{1,3}\s+/, "")) + "</h2>";
      } else if (/^[-*]\s+/.test(trimmed)) {
        if (list !== "ul") { closeList(); html += "<ul>"; list = "ul"; }
        html += "<li>" + bold(trimmed.replace(/^[-*]\s+/, "")) + "</li>";
      } else if (/^\d+[.)]\s+/.test(trimmed)) {
        if (list !== "ol") { closeList(); html += "<ol>"; list = "ol"; }
        html += "<li>" + bold(trimmed.replace(/^\d+[.)]\s+/, "")) + "</li>";
      } else {
        closeList();
        html += "<p>" + bold(trimmed) + "</p>";
      }
    });
    closeList();
    return html;
  }

  // Persists the note the delta just wrote. The chat used to claim it saved
  // while every attempt created another empty "Untitled" note; one stable
  // id per open paper app keeps repeated turns on the same document (#2186).
  // #2315 — the status bar shipped a static "Saved" that never changed, and a
  // failed autosave was swallowed. A rejected save looked exactly like a saved
  // one, so the user closed the tab and lost the note.
  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }

  function saveStatus(state, label, retry) {
    var el = document.getElementById("save-status");
    if (!el) return;
    el.setAttribute("data-state", state);
    el.textContent = label;
    el.title = retry ? t("paper-save-retry-hint", "Click to retry saving") : label;
    el.style.cursor = retry ? "pointer" : "";
    el.onclick = retry || null;
  }

  function persist(appRoot, root) {
    var titleEl = document.getElementById("paper-title");
    var title = titleEl && titleEl.innerText.trim();
    if (!title) {
      var heading = root.querySelector("h1, h2, h3");
      title = heading && heading.textContent.trim();
    }
    if (!titleEl.innerText.trim() && title) titleEl.textContent = title;
    var noteId = appRoot.getAttribute("data-note-id");
    if (!noteId) {
      noteId = "note-" + Date.now().toString(36) + "-" +
        Math.random().toString(36).slice(2, 8);
      appRoot.setAttribute("data-note-id", noteId);
    }
    saveStatus("syncing", t("paper-saving", "Saving…"));
    return fetch("/api/ui/paper/autosave", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id: noteId,
        title: title || "Untitled",
        content: root.innerHTML,
      }),
    }).then(function (resp) {
      if (!resp.ok) throw new Error("autosave failed (" + resp.status + ")");
      saveStatus("saved", t("paper-saved", "Saved"));
      if (window.htmx && document.getElementById("paper-list")) {
        window.htmx.ajax("GET", "/api/ui/paper/list", { target: "#paper-list" });
      }
    }).catch(function (error) {
      // Surface it, and make the indicator itself the retry affordance.
      if (window.console && console.warn) console.warn("[paper] autosave failed:", error);
      saveStatus("error", t("paper-save-failed", "Not saved — retry"),
        function () { persist(appRoot, root); });
    });
  }

  function apply(payload) {
    // The document body is #editor-content; `[contenteditable=true]` alone
    // matched #paper-title first, so every block landed in the title.
    var root = document.getElementById("editor-content") ||
      document.querySelector(".editor-content") ||
      document.querySelector("#paper-editor, .paper-editor");
    if (!root) return;
    (payload.ops || []).forEach(function (op) {
      if (op.op === "append_block" || op.op === "update_block") {
        var el = document.createElement(op.type === "heading" ? "h2" : "p");
        if (op.id) el.setAttribute("data-delta-id", op.id);
        el.textContent = (typeof op.text === "string") ? op.text : (Array.isArray(op.runs) ? op.runs.map(function(r){return typeof r==="string"?r:(r.text||"");}).join("") : "");
        var ex = op.id && root.querySelector('[data-delta-id="'+op.id+'"]');
        if (ex) ex.replaceWith(el); else root.appendChild(el);
      }
      if (op.op === "delete_block" && op.id) { var d=root.querySelector('[data-delta-id="'+op.id+'"]'); if(d)d.remove(); }
      if (op.op === "clear") root.innerHTML = "";
      // The web plan sends the docs-style semantic fill
      // ({"op":"fill","field":"document body","value":"<markdown-lite>"}); paper
      // only understood append_block, so the planner failed with
      // "Field 'document body' not found" and the note stayed empty (#2186).
      if (op.op === "fill") {
        var field = String(op.field || "").toLowerCase();
        var target = field.indexOf("title") >= 0
          ? (document.getElementById("paper-title") || document.querySelector("#paper-title, .paper-title"))
          : root;
        if (target) {
          if (target === root) {
            target.innerHTML = paperMarkdown(op.value || "");
          } else {
            target.textContent = String(op.value || "");
          }
        }
      }
    });
    var appRoot = document.getElementById("paper-app");
    if (appRoot && root.innerHTML.trim()) persist(appRoot, root);
  }

  if (window.GBDocDelta) window.GBDocDelta.register("paper", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("paper", apply); });
})();
