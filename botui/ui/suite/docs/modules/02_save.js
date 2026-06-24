"use strict";
/* docs save — auto-save, status, AI driver */

function setSaveStatus(text, isError) {
  var el = document.getElementById("saveStatus");
  if (el) {
    el.textContent = text;
    el.style.color = isError ? SAVE_ERR_COLOR : SAVE_OK_COLOR;
  }
}

var saveTimer = null;
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

var DocumentAIDriver = {
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
