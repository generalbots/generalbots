"use strict";

// #2147 — a bot image that cannot load must never stay on screen as a broken
// picture. The front-page captures are served by a token link (30 days) and the
// durable copy lives in Drive, so a failed load is a real possibility; degrade
// it to the alt text plus an actionable link instead of an empty box.
// Loaded after messages/40_artifact.js and before messages/50_message.js.
(function () {
  function degrade(img) {
    if (!img || img.getAttribute("data-gb-img-fallback") === "done") return;
    img.setAttribute("data-gb-img-fallback", "done");
    var src = img.getAttribute("src") || "";
    var label = img.getAttribute("alt") || "Image";
    var note = document.createElement("span");
    note.className = "gb-image-fallback";
    note.textContent = label + " — image unavailable";
    if (/^https?:\/\//i.test(src) || src.charAt(0) === "/") {
      var link = document.createElement("a");
      link.href = src;
      link.target = "_blank";
      link.rel = "noopener noreferrer";
      link.textContent = "Open";
      note.appendChild(document.createTextNode(" "));
      note.appendChild(link);
    }
    if (img.parentNode) img.parentNode.replaceChild(note, img);
  }

  // `error` does not bubble, but the capture phase sees it for every image.
  document.addEventListener(
    "error",
    function (event) {
      var target = event.target;
      if (!target || target.tagName !== "IMG") return;
      if (target.closest && !target.closest("#messages")) return;
      degrade(target);
    },
    true
  );

  // Images restored from history never fire `error` again: an already-complete
  // image with no intrinsic width failed while the bubble was being built.
  window.GBGuardBotImages = function (root) {
    if (!root || !root.querySelectorAll) return;
    var images = root.querySelectorAll("img");
    for (var i = 0; i < images.length; i++) {
      if (images[i].complete && images[i].naturalWidth === 0) degrade(images[i]);
    }
  };
})();
