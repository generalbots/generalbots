"use strict";
(function () {
  function getEd(){ return document.querySelector("#notepad-editor, textarea.notepad, [data-notepad], textarea"); }
  function apply(payload){
    var ed=getEd(); if(!ed) return;
    (payload.ops||[]).forEach(function(op){
      if(op.op==="clear") { if("value" in ed) ed.value=""; else ed.textContent=""; }
      if(op.op==="append_text") { var t=op.text||""; if("value" in ed) ed.value+=t; else ed.textContent+=t; }
      if(op.op==="patch_text" && op.text!=null) { if("value" in ed) ed.value=op.text; else ed.textContent=op.text; }
    });
  }
  if(window.GBDocDelta) window.GBDocDelta.register("notepad", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("notepad", apply); });
})();
