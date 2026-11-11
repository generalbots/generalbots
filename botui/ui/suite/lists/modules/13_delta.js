"use strict";
(function(){
  function apply(payload){
    var ul=document.querySelector("#lists-items, .list-items, ul.items, ul");
    (payload.ops||[]).forEach(function(op){
      if(op.op==="add_item" && ul){ var li=document.createElement("li"); if(op.id) li.setAttribute("data-delta-id",op.id); li.textContent=op.text||""; ul.appendChild(li); }
      if(op.op==="patch_item" && op.id){ var el=document.querySelector('[data-delta-id="'+op.id+'"]'); if(el && op.text!=null) el.textContent=op.text; }
      if(op.op==="check" && op.id){ var c=document.querySelector('[data-delta-id="'+op.id+'"]'); if(c) c.classList.toggle("checked", !!op.checked); }
      if(op.op==="delete_record" && op.id){ var d=document.querySelector('[data-delta-id="'+op.id+'"]'); if(d) d.remove(); }
    });
  }
  if(window.GBDocDelta) window.GBDocDelta.register("lists", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("lists", apply); });
})();
