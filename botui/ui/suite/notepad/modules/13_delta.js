"use strict";
(function () {
  function registerWhenReady(attempt) {
    if (window.GBDocDelta && typeof window.GBDocDelta.register === "function") {
      window.GBDocDelta.register("notepad", apply);
      return;
    }
    if ((attempt || 0) >= 20) return;
    setTimeout(function () { registerWhenReady((attempt || 0) + 1); }, 250);
  }
  // Scope the lookup to the notepad surface: a bare `textarea` fallback
  // matched the sidebar feedback box (#sidebarFeedbackText) first, so note
  // text was appended off-app.
  function getEd(){
    var root = document.getElementById("gb-notepad-root") ||
      document.querySelector("#window-notepad .gb-window-content");
    return (root && root.querySelector("#gb-notepad-text, textarea[data-gb-field=\"text\"], textarea")) ||
      document.getElementById("gb-notepad-text");
  }
  function apply(payload){
    var ed=getEd(); if(!ed) return;
    // A delta that lands before notepad.js finishes booting must survive the
    // initial seed (the boot pass used to overwrite it with the stored note).
    if("value" in ed) ed.dataset.gbDelta="1";
    var changed = false;
    (payload.ops||[]).forEach(function(op){
      if(op.op==="clear") { if("value" in ed) ed.value=""; else ed.textContent=""; changed = true; }
      if(op.op==="append_text") { var t=op.text||""; if("value" in ed) ed.value+=t; else ed.textContent+=t; changed = true; }
      if(op.op==="patch_text" && op.text!=null) { if("value" in ed) ed.value=op.text; else ed.textContent=op.text; changed = true; }
      if(op.op==="fill") { var t=op.text!=null?op.text:(typeof op.value==="string"?op.value:""); if(t && "value" in ed) { ed.value=String(t); changed = true; } }
    });
    // Persist through notepad.js autosave (it listens for input events).
    if (changed) {
      try { ed.dispatchEvent(new Event("input", { bubbles: true })); } catch (e) {}
    }
  }
  registerWhenReady(0);
  // Standalone fallback only: with GBDocDelta present, register() already
  // routes every batch to apply(), so a second listener applied it twice.
  if (!window.GBDocDelta) {
    document.addEventListener("gb-doc-delta", function (e) { apply((e && e.detail) || {}); });
  }
})();
