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
  function getEd(){ return document.querySelector("#gb-notepad-text, textarea[data-gb-field=\"text\"], #notepad-editor, textarea.notepad, [data-notepad], textarea"); }
  function apply(payload){
    var ed=getEd(); if(!ed) return;
    (payload.ops||[]).forEach(function(op){
      if(op.op==="clear") { if("value" in ed) ed.value=""; else ed.textContent=""; }
      if(op.op==="append_text") { var t=op.text||""; if("value" in ed) ed.value+=t; else ed.textContent+=t; }
      if(op.op==="patch_text" && op.text!=null) { if("value" in ed) ed.value=op.text; else ed.textContent=op.text; }
      if(op.op==="fill") { var t=op.text!=null?op.text:(typeof op.value==="string"?op.value:""); if(t && "value" in ed) ed.value=String(t); }
    });
  }
  registerWhenReady(0);
  if (window.GBDocDelta) document.addEventListener("gb-doc-delta", function (e) { apply((e && e.detail) || {}); });
})();
