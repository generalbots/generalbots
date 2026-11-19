"use strict";
/* Slides notes workspace (#2433): per-slide local notes list, search and rich
 * text editing through the existing Notes modal. Notes are local-only metadata;
 * the server notes endpoint is not reimplemented here. */
(function () {
  var O = window.GBSlidesOutline;
  var STORE = "notes";

  function all() { return O.load(STORE, {}); }
  function save(map) { O.save(STORE, map); }
  function ids(map) { return Object.keys(map).sort(function (a, b) { return parseInt(a, 10) - parseInt(b, 10); }); }

  function noteDialog(titleKey, title, value, onSave) {
    var dialog = O.dialog({ titleKey: titleKey, title: title });
    var label = O.el("label", "slides-tools-field-label",
      O.t("slides-outline-notes-text", "Note"), "slides-outline-notes-text");
    var area = document.createElement("textarea");
    area.rows = 5;
    area.value = value || "";
    area.setAttribute("data-i18n-placeholder", "slides-outline-notes-text");
    area.placeholder = O.t("slides-outline-notes-text", "Note");
    label.appendChild(area);
    dialog.body.appendChild(label);
    dialog.body.appendChild(O.localBadge());
    var cancel = O.el("button", "slides-tools-btn", O.t("slides-outline-cancel", "Cancel"), "slides-outline-cancel");
    var saveBtn = O.el("button", "slides-tools-btn primary", O.t("slides-outline-save", "Save"), "slides-outline-save");
    cancel.type = "button"; saveBtn.type = "button";
    cancel.addEventListener("click", function () { dialog.dismiss(false); });
    saveBtn.addEventListener("click", function () {
      var text = (area.value || "").trim();
      if (!text) { O.status(O.t("slides-outline-notes-required", "Enter some note text")); return; }
      dialog.dismiss(true);
      onSave(text);
    });
    dialog.foot.appendChild(cancel);
    dialog.foot.appendChild(saveBtn);
    window.setTimeout(function () { area.focus(); }, 0);
  }

  function openRichNotes(slideIndex) {
    var button = document.getElementById("notesBtn");
    if (!button || !window.htmx) {
      O.status(O.t("slides-outline-notes-modal-unavailable", "Notes modal is unavailable"));
      return;
    }
    button.setAttribute("hx-vals", JSON.stringify({
      presentation_id: O.deckId(), slide_index: String(slideIndex)
    }));
    window.htmx.process(button);
    button.click();
    var host = document.getElementById("slides-modal-container");
    if (!host) return;
    var onSave = function (event) {
      if (event.detail && event.detail.successful === false) return;
      var area = host.querySelector('textarea[name="notes"]');
      if (!area) return;
      host.removeEventListener("htmx:afterRequest", onSave);
      var text = (area.value || "").trim();
      if (!text) return;
      var map = all();
      var key = String(slideIndex);
      var list = map[key] || [];
      list.push({ id: "rich-" + Date.now().toString(36), text: text, ts: Date.now(), rich: true });
      map[key] = list; save(map);
      O.status(O.t("slides-outline-notes-synced", "Rich notes copied into the local workspace"));
    };
    host.addEventListener("htmx:afterRequest", onSave);
  }

  function addNote(slideIndex, render) {
    noteDialog("slides-outline-notes-add", "Add note", "", function (text) {
      var map = all();
      var key = String(slideIndex);
      var list = map[key] || [];
      var before = list.slice();
      var note = { id: "n" + Date.now().toString(36), text: text, ts: Date.now() };
      list.push(note); map[key] = list; save(map); render();
      O.pushUndo({
        label: O.t("slides-outline-action-note-add", "add note"),
        undo: function () { map[key] = before; save(map); render(); },
        redo: function () { map[key] = before.concat([note]); save(map); render(); }
      });
      O.status(O.t("slides-outline-notes-added", "Note added locally"));
    });
  }

  function noteRow(slideIndex, note, render, host) {
    var row = O.el("div", "slides-notes-row");
    row.dataset.slide = String(slideIndex);
    row.dataset.note = note.id;
    var jump = O.el("button", "slides-notes-jump",
      O.t("slides-outline-untitled", "Slide {n}", { n: slideIndex + 1 }), "slides-outline-untitled");
    jump.type = "button";
    jump.addEventListener("click", function () { O.jump(slideIndex); });
    row.appendChild(jump);
    row.appendChild(O.el("span", "slides-notes-text", note.text));
    if (note.rich) row.appendChild(O.el("span", "slides-notes-rich-flag",
      O.t("slides-outline-notes-rich", "Rich"), "slides-outline-notes-rich"));
    var actions = O.el("div", "slides-notes-actions");
    var edit = O.el("button", "slides-tools-btn", O.t("slides-outline-edit", "Edit"), "slides-outline-edit");
    var rich = O.el("button", "slides-tools-btn", O.t("slides-outline-notes-rich-edit", "Edit rich text"), "slides-outline-notes-rich-edit");
    var del = O.el("button", "slides-tools-btn danger", O.t("slides-outline-delete", "Delete"), "slides-outline-delete");
    edit.type = "button"; rich.type = "button"; del.type = "button";
    edit.addEventListener("click", function () {
      noteDialog("slides-outline-notes-edit", "Edit note", note.text, function (text) {
        var before = note.text;
        var map = all();
        map[String(slideIndex)].forEach(function (item) { if (item.id === note.id) item.text = text; });
        save(map); render();
        O.pushUndo({
          label: O.t("slides-outline-action-note-edit", "edit note"),
          undo: function () { var m = all(); m[String(slideIndex)].forEach(function (i) { if (i.id === note.id) i.text = before; }); save(m); render(); },
          redo: function () { var m = all(); m[String(slideIndex)].forEach(function (i) { if (i.id === note.id) i.text = text; }); save(m); render(); }
        });
      });
    });
    rich.addEventListener("click", function () { openRichNotes(slideIndex); });
    del.addEventListener("click", function () {
      var map = all();
      var key = String(slideIndex);
      var before = map[key].slice();
      map[key] = map[key].filter(function (item) { return item.id !== note.id; });
      save(map); render();
      O.pushUndo({
        label: O.t("slides-outline-action-note-delete", "delete note"),
        undo: function () { var m = all(); m[key] = before; save(m); render(); },
        redo: function () { var m = all(); m[key] = before.filter(function (i) { return i.id !== note.id; }); save(m); render(); }
      });
      O.status(O.t("slides-outline-notes-deleted", "Note deleted locally"));
    });
    actions.appendChild(edit); actions.appendChild(rich); actions.appendChild(del);
    row.appendChild(actions);
    host.appendChild(row);
  }

  function open() {
    var panel = O.openPanel("slides-notes-panel", function () {
      var root = O.el("section", "slides-tools-panel slides-notes-panel");
      var head = O.el("div", "slides-tools-panel-head");
      head.appendChild(O.el("h2", "", O.t("slides-outline-notes-title", "Notes workspace"), "slides-outline-notes-title"));
      head.appendChild(O.localBadge());
      var close = O.el("button", "slides-tools-close", O.t("slides-outline-close", "Close"), "slides-outline-close");
      close.type = "button"; close.addEventListener("click", O.closePanels);
      head.appendChild(close);
      root.appendChild(head);

      var toolbar = O.el("div", "slides-tools-toolbar");
      var search = document.createElement("input");
      search.type = "search";
      search.setAttribute("data-i18n-placeholder", "slides-outline-notes-search");
      search.placeholder = O.t("slides-outline-notes-search", "Search notes");
      search.setAttribute("aria-label", O.t("slides-outline-notes-search", "Search notes"));
      var add = O.el("button", "slides-tools-btn", O.t("slides-outline-notes-add", "Add note"), "slides-outline-notes-add");
      add.type = "button";
      var undo = O.el("button", "slides-tools-btn slides-tools-undo", O.t("action-undo", "Undo"), "action-undo");
      undo.type = "button"; undo.addEventListener("click", O.undo);
      toolbar.appendChild(search); toolbar.appendChild(add); toolbar.appendChild(undo);
      root.appendChild(toolbar);
      var list = O.el("div", "slides-notes-list");
      root.appendChild(list);
      return root;
    });
    var list = panel.querySelector(".slides-notes-list");
    var search = panel.querySelector('input[type="search"]');
    function render() {
      list.textContent = "";
      var map = all();
      var query = (search.value || "").toLowerCase();
      var count = 0;
      ids(map).forEach(function (slideKey) {
        map[slideKey].forEach(function (note) {
          if (query && (note.text || "").toLowerCase().indexOf(query) === -1) return;
          noteRow(parseInt(slideKey, 10), note, render, list);
          count += 1;
        });
      });
      if (!count) {
        list.appendChild(O.el("p", "slides-tools-empty",
          O.t("slides-outline-notes-empty", "No notes found"), "slides-outline-notes-empty"));
      }
    }
    panel.querySelector('[data-i18n="slides-outline-notes-add"]').addEventListener("click", function () {
      var canvas = O.$("#slide-canvas") || document.getElementById("slide-canvas");
      var index = canvas ? parseInt(canvas.getAttribute("data-slide-id") || "0", 10) || 0 : 0;
      addNote(index, render);
    });
    search.addEventListener("input", render);
    render();
    O.updateUndoUI();
    return panel;
  }

  window.GBSlidesNotes = { open: open };
})();
