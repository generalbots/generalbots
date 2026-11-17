"use strict";

// Settings → Appearance → Chat background. The picker writes the same
// `gb-chat-pattern` key the chat app reads (suite/js/chat-pattern.js), so the
// choice previews live and carries over to every conversation window.
(function () {
  function api() {
    return window.GBChatPattern || null;
  }

  function render() {
    var row = document.getElementById("settings-chatbg-row");
    var pattern = api();
    if (!row || !pattern) return;
    var current = pattern.get();
    var html = "";
    for (var index = 0; index < pattern.IDS.length; index++) {
      var id = pattern.IDS[index];
      var active = id === current;
      html += '<button type="button" role="radio" class="chatbg-btn' +
        (active ? " chatbg-btn-active" : "") + '" data-pattern="' + id + '"' +
        ' aria-checked="' + (active ? "true" : "false") + '">' +
        '<span class="chatbg-swatch chatbg-swatch-' + id + '" aria-hidden="true"></span>' +
        '<span class="chatbg-label">' + pattern.label(id) + "</span></button>";
    }
    row.innerHTML = html;
    var buttons = row.querySelectorAll(".chatbg-btn");
    for (var button = 0; button < buttons.length; button++) {
      buttons[button].addEventListener("click", function (event) {
        var next = event.currentTarget.getAttribute("data-pattern");
        if (api()) api().set(next);
        render();
      });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }

  // The Appearance panel arrives through HTMX (01 → 02 → 03) after first
  // paint, so the row is created later than this script runs.
  document.body.addEventListener("htmx:afterSwap", render);
  window.loadChatBgPanel = render;

  window.addEventListener("gb-chat-pattern-changed", function () {
    var row = document.getElementById("settings-chatbg-row");
    if (row && row.children.length) render();
  });
})();
