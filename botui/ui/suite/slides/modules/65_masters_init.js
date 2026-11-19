"use strict";
/* Slides masters (#2429) — layer 3d: toolbar wiring and dialog assembly.
 * Everything opened here is local only; nothing is persisted to the server. */
(function (window) {
  var M = (window.GBSlidesMasters = window.GBSlidesMasters || {});
  var bound = false;

  M.open = function () {
    var api = M.openDialog({
      onBuild: function (dialog) {
        var state = M.init();
        dialog.body.appendChild(M.localNote());
        dialog.body.appendChild(M.buildThemePanel(dialog, state));
        dialog.body.appendChild(M.buildInheritPanel(dialog, state));
        dialog.body.appendChild(M.buildApplyPanel(dialog, state));
        dialog.body.appendChild(M.buildLayoutPanel(dialog, state));
        dialog.body.appendChild(M.buildBrandPanel(dialog, state));
        dialog.body.appendChild(M.buildTemplatesPanel(dialog, state));
      }
    });
    M.emit("open", api);
    return api;
  };

  function bind() {
    if (bound) return;
    var button = document.getElementById("slMastersBtn");
    if (!button) return;
    bound = true;
    button.addEventListener("click", function () { M.open(); });
  }

  M.initUI = function () {
    M.init();
    bind();
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", M.initUI);
  } else {
    M.initUI();
  }
})(typeof window !== "undefined" ? window : globalThis);
