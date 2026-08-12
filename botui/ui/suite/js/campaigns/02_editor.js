"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.editor = {
  editEl: null,

  init: function () {
    var self = this;
    this.editEl = document.getElementById("studio-editor");
    if (!this.editEl) return;
    this.editEl.addEventListener("input", function () {
      window.CampStudio.state.content = self.editEl.innerHTML;
      window.CampStudio.events.emit("content-changed", self.editEl.innerHTML);
    });
  },

  setContent: function (html) {
    if (!this.editEl) return;
    this.editEl.innerHTML = html || "";
    window.CampStudio.state.content = this.editEl.innerHTML || "";
  },

  getContent: function () {
    return this.editEl ? this.editEl.innerHTML : window.CampStudio.state.content;
  },

  exec: function (command, value) {
    this.editEl.focus();
    document.execCommand(command, false, value || null);
    this.editEl.dispatchEvent(new Event("input"));
  },

  insertVariable: function (name) {
    var self = this;
    if (!this.editEl) return;
    this.editEl.focus();
    var content = this.editEl.innerHTML;
    this.editEl.innerHTML = content + ' <span class="studio-var">' + name + "</span>";
    this.editEl.dispatchEvent(new Event("input"));
    window.CampStudio.events.emit("content-changed", this.editEl.innerHTML);
  },

  bindToolbar: function () {
    var self = this;
    document.querySelectorAll("[data-editor-cmd]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        self.exec(btn.dataset.editorCmd, btn.dataset.editorValue || undefined);
      });
    });
    document.querySelectorAll("[data-var]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        self.insertVariable(btn.dataset.var);
      });
    });
  },
};

window.CampStudio.editor.init();
window.CampStudio.editor.bindToolbar();