"use strict";
(function(){
  function apply(payload){
    var root=document.querySelector("#minutes-content, .minutes-content, main");
    (payload.ops||[]).forEach(function(op){
      if(op.op==="append_section" && root){ var sec=document.createElement("section"); sec.innerHTML="<h3>"+(op.title||"")+"</h3><p>"+(op.text||"")+"</p>"; root.appendChild(sec); }
      if(op.op==="set_action_item" && root){ var li=document.createElement("div"); li.className="action-item"; li.textContent=(op.owner?op.owner+": ":"")+(op.text||"")+(op.due?" ("+op.due+")":""); root.appendChild(li); }
    });
  }
  if(window.GBDocDelta) window.GBDocDelta.register("minutes", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("minutes", apply); });
})();
