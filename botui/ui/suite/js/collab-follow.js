"use strict";
/* GBCollabFollow — shared collaborators panel with click-to-follow.
 *
 * The presence modules (DocsPresence / SlidesPresence) expose list() + follow();
 * this helper renders a dropdown of the people currently online and jumps the
 * local viewport to a collaborator's cursor/element when they are clicked.
 *
 * Public API (window.GBCollabFollow):
 *   toggle(button, getPresence) — show/hide the panel anchored to `button`
 *   hide()                     — dismiss the panel
 *
 * `getPresence` must be a function returning the presence API object.
 */
(function (window) {
  var panel = null;

  function build() {
    if (panel && panel.parentNode) return panel;
    panel = document.createElement("div");
    panel.className = "gb-follow-panel";
    panel.style.cssText =
      "position:fixed;top:0;left:0;display:none;background:#1e293b;border:1px solid #334155;" +
      "border-radius:8px;padding:6px;z-index:100000;min-width:200px;max-height:320px;overflow-y:auto;" +
      "box-shadow:0 8px 24px rgba(0,0,0,.4);";
    document.body.appendChild(panel);
    return panel;
  }

  function render(getPresence) {
    var p = build();
    p.innerHTML = "";
    var list = [];
    try { list = (getPresence && getPresence().list()) || []; } catch (_) {}

    var header = document.createElement("div");
    header.textContent = "Collaborators (" + list.length + ")";
    header.style.cssText = "color:#94a3b8;font-size:11px;font-weight:600;padding:4px 8px;";
    p.appendChild(header);

    if (!list.length) {
      var empty = document.createElement("div");
      empty.textContent = "No one else is here";
      empty.style.cssText = "color:#64748b;font-size:12px;padding:8px;";
      p.appendChild(empty);
      return;
    }

    list.forEach(function (u) {
      var row = document.createElement("div");
      row.style.cssText =
        "display:flex;align-items:center;gap:8px;padding:6px 8px;border-radius:6px;" +
        "cursor:pointer;color:#f8fafc;font-size:13px;";
      row.addEventListener("mouseenter", function () { row.style.background = "#334155"; });
      row.addEventListener("mouseleave", function () { row.style.background = ""; });

      var dot = document.createElement("span");
      dot.style.cssText =
        "width:10px;height:10px;border-radius:50%;flex-shrink:0;background:" +
        (u.user_color || u.color || "#3b82f6") + ";";
      var name = document.createElement("span");
      name.textContent = u.user_name || u.name || u.user_id;
      var hint = document.createElement("span");
      hint.textContent = "→";
      hint.style.cssText = "margin-left:auto;color:#64748b;font-size:12px;";

      row.appendChild(dot);
      row.appendChild(name);
      row.appendChild(hint);
      row.addEventListener("click", function () {
        try { getPresence().follow(u.user_id); } catch (_) {}
        hide();
      });
      p.appendChild(row);
    });
  }

  function show(button, getPresence) {
    var p = build();
    render(getPresence);
    var r = button.getBoundingClientRect();
    p.style.display = "block";
    p.style.top = Math.min(r.bottom + 6, window.innerHeight - 220) + "px";
    p.style.left = Math.max(8, r.right - 200) + "px";
  }

  function hide() {
    if (panel) panel.style.display = "none";
  }

  function toggle(button, getPresence) {
    if (panel && panel.style.display === "block") hide();
    else show(button, getPresence);
  }

  document.addEventListener("click", function (e) {
    if (panel && panel.style.display === "block" && !panel.contains(e.target)) hide();
  });

  window.GBCollabFollow = { toggle: toggle, hide: hide };
})(window);
