"use strict";
/* gb-doc-delta/v1 for the lists app (#2189).
   The app's main surface is the CRM Marketing Lists CRUD, which has no
   checklist container, so chat checklist ops used to land in a bare <ul> (or
   be dropped). This renderer owns a dedicated `#lists-items` checklist that is
   created on demand and is the target for add/check/reorder/patch ops. */
(function () {
  function ensureContainer() {
    var ul = document.getElementById("lists-items");
    if (ul) return ul;
    var host = document.querySelector(".crm-container") || document.body;
    var section = document.createElement("section");
    section.className = "lists-checklist";
    section.id = "lists-checklist";
    var heading = document.createElement("h2");
    heading.className = "lists-checklist-title";
    heading.textContent = "Checklist";
    var list = document.createElement("ul");
    list.id = "lists-items";
    list.className = "lists-checklist-items";
    section.appendChild(heading);
    section.appendChild(list);
    var grid = document.getElementById("listsList");
    if (grid && grid.parentElement === host) {
      host.insertBefore(section, grid);
    } else {
      host.appendChild(section);
    }
    return list;
  }

  function findItem(id) {
    if (id == null) return null;
    return document.querySelector('#lists-items [data-delta-id="' + id + '"]');
  }

  function makeItem(text, id) {
    var li = document.createElement("li");
    li.className = "lists-checklist-item";
    if (id != null) li.setAttribute("data-delta-id", id);
    var box = document.createElement("input");
    box.type = "checkbox";
    box.className = "lists-checklist-check";
    var label = document.createElement("span");
    label.className = "lists-checklist-text";
    label.textContent = text == null ? "" : String(text);
    li.appendChild(box);
    li.appendChild(label);
    box.addEventListener("change", function () {
      li.classList.toggle("checked", box.checked);
    });
    li.addEventListener("click", function (e) {
      if (e.target === box) return;
      box.checked = !box.checked;
      li.classList.toggle("checked", box.checked);
    });
    return li;
  }

  function apply(payload) {
    var ops = payload.ops || [];
    if (!ops.length) return;
    var ul = ensureContainer();
    ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "clear") {
        ul.innerHTML = "";
        return;
      }
      if (op.op === "add_item" || op.op === "append_item") {
        var id = op.id != null ? op.id : "item-" + Date.now() + "-" +
          Math.random().toString(36).slice(2, 7);
        if (findItem(id)) return;
        ul.appendChild(makeItem(op.text, id));
        return;
      }
      if (op.op === "patch_item") {
        var item = findItem(op.id);
        if (!item) return;
        var text = item.querySelector(".lists-checklist-text");
        if (text && op.text != null) text.textContent = String(op.text);
        if (op.checked != null) {
          var box = item.querySelector(".lists-checklist-check");
          if (box) box.checked = !!op.checked;
          item.classList.toggle("checked", !!op.checked);
        }
        return;
      }
      if (op.op === "check") {
        var checked = findItem(op.id);
        if (!checked) return;
        var checkbox = checked.querySelector(".lists-checklist-check");
        var isChecked = op.checked == null ? !checked.classList.contains("checked") : !!op.checked;
        if (checkbox) checkbox.checked = isChecked;
        checked.classList.toggle("checked", isChecked);
        return;
      }
      if (op.op === "delete_item" || op.op === "delete_record") {
        var doomed = findItem(op.id);
        if (doomed) doomed.remove();
        return;
      }
      if (op.op === "reorder") {
        var moved = findItem(op.id);
        if (!moved) return;
        if (typeof op.to === "number") {
          var siblings = Array.prototype.slice.call(ul.children);
          siblings.splice(siblings.indexOf(moved), 1);
          var index = Math.max(0, Math.min(op.to, siblings.length));
          if (index >= siblings.length) ul.appendChild(moved);
          else ul.insertBefore(moved, siblings[index]);
        } else {
          ul.appendChild(moved);
        }
        return;
      }
      if (op.op === "rename_list" || op.op === "finalize") {
        return;
      }
    });
    var section = document.getElementById("lists-checklist");
    if (section && ul.children.length && typeof section.scrollIntoView === "function") {
      section.scrollIntoView({ block: "nearest" });
    }
  }

  if (window.GBDocDelta) window.GBDocDelta.register("lists", apply);
  else document.addEventListener("DOMContentLoaded", function () {
    if (window.GBDocDelta) window.GBDocDelta.register("lists", apply);
  });
})();
