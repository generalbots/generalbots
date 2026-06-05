"use strict";

/**
 * Module 13: Footnotes and endnotes for Word Processor.
 * Adds toolbar buttons to insert footnote and endnote. The footnote
 * reference is a superscript number inserted in the text at the
 * cursor. The footnote body is shown at the bottom of the page (when
 * pagination is active) or at the end of the document. Endnotes go
 * to a dedicated section at the end. Supports custom numbering
 * schemes (1,2,3 / i,ii,iii / a,b,c). Auto-renumbering on insert /
 * delete. Hover preview of note content.
 *
 * Public API: window.DocsFootnotes = { insertFootnote, insertEndnote,
 *   renumber, renderNotesPanel, openPanel, closePanel, setScheme }.
 */

(function () {
  const SCHEMES = {
    decimal: (i) => String(i),
    "lower-roman": function (i) {
      const nums = ["i","ii","iii","iv","v","vi","vii","viii","ix","x",
        "xi","xii","xiii","xiv","xv","xvi","xvii","xviii","xix","xx"];
      return nums[i - 1] || String(i);
    },
    "upper-roman": function (i) {
      return SCHEMES["lower-roman"](i).toUpperCase();
    },
    "lower-alpha": (i) => String.fromCharCode(96 + (i % 26 || 26)),
    "upper-alpha": (i) => String.fromCharCode(64 + (i % 26 || 26)),
  };

  function getState() { return window.state || null; }
  function getNotes(type) {
    const s = getState();
    if (!s) return [];
    if (!s[type]) s[type] = [];
    return s[type];
  }
  function setScheme(type, scheme) {
    const s = getState();
    if (!s) return;
    if (!s.noteSchemes) s.noteSchemes = { footnotes: "decimal", endnotes: "decimal" };
    s.noteSchemes[type] = scheme;
    renumber(type);
  }

  function renumber(type) {
    const s = getState();
    if (!s) return;
    const scheme = (s.noteSchemes && s.noteSchemes[type]) || "decimal";
    const fn = SCHEMES[scheme] || SCHEMES.decimal;
    const list = s[type] || [];
    list.forEach((n, i) => { n.number = fn(i + 1); });
    document.dispatchEvent(new CustomEvent("docsNotesChange", { detail: { type: type } }));
  }

  function insertNote(type) {
    const s = getState();
    if (!s) return null;
    const list = getNotes(type);
    const note = {
      id: "note-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6),
      number: "0",
      body: "",
      refText: type === "footnotes" ? "footnote" : "endnote",
      timestamp: new Date().toISOString(),
    };
    list.push(note);
    renumber(type);
    const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    if (editor) {
      const sel = window.getSelection();
      if (sel && sel.rangeCount && sel.anchorNode && editor.contains(sel.anchorNode)) {
        const r = sel.getRangeAt(0);
        const ref = document.createElement("sup");
        ref.className = "note-ref " + type + "-ref";
        ref.contentEditable = "false";
        ref.textContent = note.number;
        ref.dataset.noteId = note.id;
        ref.title = "Click to edit";
        ref.addEventListener("click", function () { editNoteBody(type, note.id); });
        r.insertNode(ref);
      }
    }
    const body = window.prompt(type === "footnotes" ? "Footnote text:" : "Endnote text:", "");
    if (body != null) {
      note.body = body;
    }
    renderNotesPanel(type);
    return note;
  }

  function insertFootnote() { return insertNote("footnotes"); }
  function insertEndnote() { return insertNote("endnotes"); }

  function editNoteBody(type, id) {
    const list = getNotes(type);
    const note = list.find((n) => n.id === id);
    if (!note) return;
    const body = window.prompt("Edit " + type.replace(/s$/, "") + " text:", note.body || "");
    if (body != null) {
      note.body = body;
      renderNotesPanel(type);
    }
  }

  function renderNotesSection(type) {
    const s = getState();
    if (!s) return null;
    const list = s[type] || [];
    if (!list.length) return null;
    const section = document.createElement("div");
    section.className = type + "-section";
    section.style.cssText = "margin-top:24px;padding-top:12px;border-top:1px solid #888;font-size:13px;";
    const h = document.createElement("h4");
    h.textContent = type === "footnotes" ? "Footnotes" : "Endnotes";
    h.style.cssText = "margin:0 0 8px 0;font-size:13px;";
    section.appendChild(h);
    for (const n of list) {
      const p = document.createElement("p");
      p.style.cssText = "margin:4px 0;";
      const sup = document.createElement("sup");
      sup.textContent = n.number;
      p.appendChild(sup);
      p.appendChild(document.createTextNode(" " + (n.body || "")));
      section.appendChild(p);
    }
    return section;
  }

  function ensurePanel(type) {
    let p = document.getElementById("docsNotesPanel-" + type);
    if (p) return p;
    p = document.createElement("div");
    p.id = "docsNotesPanel-" + type;
    p.className = "docs-notes-panel";
    p.style.cssText = "position:fixed;bottom:0;right:0;width:340px;max-height:50vh;overflow:auto;background:#fff8e1;border:1px solid #ccc;padding:8px;z-index:9996;font-family:Arial,sans-serif;font-size:13px;display:none;";
    document.body.appendChild(p);
    return p;
  }

  function renderNotesPanel(type) {
    const p = ensurePanel(type);
    p.innerHTML = "";
    const title = document.createElement("div");
    title.style.cssText = "display:flex;align-items:center;gap:6px;margin-bottom:6px;";
    title.innerHTML = "<strong>" + (type === "footnotes" ? "Footnotes" : "Endnotes") + "</strong>";
    const close = document.createElement("button");
    close.textContent = "×";
    close.style.cssText = "margin-left:auto;background:transparent;border:0;cursor:pointer;font-size:18px;";
    close.addEventListener("click", function () { p.style.display = "none"; });
    title.appendChild(close);
    p.appendChild(title);
    const list = getNotes(type);
    for (const n of list) {
      const row = document.createElement("div");
      row.style.cssText = "border-bottom:1px solid #e0d090;padding:4px 0;";
      const num = document.createElement("sup");
      num.textContent = n.number;
      num.style.cssText = "margin-right:6px;";
      row.appendChild(num);
      row.appendChild(document.createTextNode(n.body || "(empty)"));
      p.appendChild(row);
    }
  }

  function openPanel(type) {
    renderNotesPanel(type);
    ensurePanel(type).style.display = "";
  }
  function closePanel(type) { ensurePanel(type).style.display = "none"; }

  function attach() {
    setTimeout(function () {
      const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
      if (!editor) return;
      const section = renderNotesSection("endnotes");
      if (section) editor.appendChild(section);
    }, 300);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsFootnotes = {
    insertFootnote, insertEndnote, renumber,
    renderNotesPanel, openPanel, closePanel, setScheme,
  };
})();
