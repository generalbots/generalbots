"use strict";
(function(){
  function apply(payload){
    var list=document.querySelector("#notes-list, .notes-list");
    (payload.ops||[]).forEach(function(op){
      if(op.op==="add_note" && list){
        var card=document.createElement("div"); card.className="note-card"; if(op.id) card.setAttribute("data-delta-id",op.id);
        card.textContent=op.text||""; list.prepend(card);
      }
      if(op.op==="patch_note" && op.id){ var el=document.querySelector('[data-delta-id="'+op.id+'"]'); if(el && op.text!=null) el.textContent=op.text; }
      if(op.op==="delete_note" && op.id){ var d=document.querySelector('[data-delta-id="'+op.id+'"]'); if(d) d.remove(); }
    });
  }
  if(window.GBDocDelta) window.GBDocDelta.register("notes", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("notes", apply); });
})();
