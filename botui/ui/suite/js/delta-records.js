"use strict";
/* #2176 — generic record-shaped delta handler for Tier-3 apps.
   Supports upsert_record, patch_field, delete_record, reorder, link. */
(function(){
  function applyToContainer(container, payload){
    if(!container) return;
    (payload.ops||[]).forEach(function(op){
      if(!op) return;
      if(op.op==="upsert_record"){
        var id=op.id || (op.entity+"-"+Math.random().toString(36).slice(2));
        var el=container.querySelector('[data-record-id="'+id+'"]');
        if(!el){ el=document.createElement("div"); el.className="delta-record"; el.setAttribute("data-record-id",id); container.appendChild(el); }
        if(op.fields) el.dataset.fields = JSON.stringify(op.fields);
        el.textContent = op.fields ? (op.fields.title || op.fields.name || op.fields.summary || JSON.stringify(op.fields)) : (op.title||op.text||id);
      }
      if(op.op==="patch_field" && op.id){
        var el2=container.querySelector('[data-record-id="'+op.id+'"]');
        if(el2){ var f={}; try{f=JSON.parse(el2.dataset.fields||"{}");}catch(e){} f[op.field]=op.value; el2.dataset.fields=JSON.stringify(f); if(op.field==="title"||op.field==="name") el2.textContent=String(op.value); }
      }
      if(op.op==="delete_record" && op.id){ var d=container.querySelector('[data-record-id="'+op.id+'"]'); if(d) d.remove(); }
      if(op.op==="reorder" && op.id && container){ var el3=container.querySelector('[data-record-id="'+op.id+'"]'); if(el3 && typeof op.index==="number"){ var ref=container.children[op.index]; if(ref) container.insertBefore(el3, ref); else container.appendChild(el3); } }
    });
  }
  function registerSimple(appId, selector){
    if(!window.GBDocDelta) return;
    window.GBDocDelta.register(appId, function(payload){
      // Records must land inside the app window that is already open; the
      // old body fallback wrote them off-screen behind the desktop (#2176).
      var c=document.querySelector(selector);
      if(!c){ c=document.querySelector("#"+appId+"-list, ."+appId+"-list, [data-app='"+appId+"']"); }
      if(!c){
        var win=document.querySelector("#window-"+appId+" .gb-window-content, #window-"+appId);
        if(!win) return;
        c=document.createElement("div");
        c.id=appId+"-list";
        c.className="delta-record-list";
        win.insertBefore(c, win.firstChild);
      }
      applyToContainer(c, payload);
    });
  }
  window.GBDeltaRecords = { applyToContainer: applyToContainer, registerSimple: registerSimple };
  var apps = ["goals","plan","project","tasks","crm","people","tickets","hr","billing","products","tax","calendar","campaigns","social","mail","templates","automations","workspace","vibe"];
  function registerAll(){
    if(!window.GBDocDelta){
      if(document.readyState === "loading"){
        document.addEventListener("DOMContentLoaded", registerAll, { once: true });
      }
      return;
    }
    apps.forEach(function(id){ registerSimple(id, "#"+id+"-list, ."+id+"-list, [data-"+id+"-records]"); });
  }
  registerAll();
})();
