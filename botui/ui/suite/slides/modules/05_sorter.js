"use strict";
/* slides sorter — reorder slides and persist the new order (#1748).
 *
 * The panel is loaded into #slides-modal-container by the toolbar. Items keep
 * their original index in `data-index`; the visual order is the DOM order, so
 * saving walks `.sl-sorter-item` and posts that list as `slide_order`.
 * The server used to render a static hx-vals `order` string, which the typed
 * API (ReorderSlidesRequest) ignored — the request must be JSON.
 */
(function () {
  var dragItem = null;

  function panel() {
    return document.getElementById("slide-sorter-panel");
  }

  function items(root) {
    return Array.prototype.slice.call(root.querySelectorAll(".sl-sorter-item"));
  }

  function syncLabels(root) {
    items(root).forEach(function (item, index) {
      var label = item.querySelector(".sl-sorter-order");
      if (label) label.textContent = String(index + 1);
      item.dataset.position = String(index);
    });
  }

  function move(item, delta) {
    var root = item.closest ? item.closest("#slide-sorter-panel") : null;
    if (!root) return;
    var list = items(root);
    var from = list.indexOf(item);
    var to = from + delta;
    if (from < 0 || to < 0 || to >= list.length) return;
    var after = delta > 0 ? list[to].nextSibling : list[to];
    item.parentNode.insertBefore(item, after);
    syncLabels(root);
  }

  function authHeaders() {
    if (window.GBShared && window.GBShared.auth) return window.GBShared.auth.headers({ "Content-Type": "application/json" });
    var token =
      (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
      window.localStorage.getItem("gb-access-token") ||
      window.sessionStorage.getItem("gb-access-token") ||
      "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }

  function save(button) {
    var root = panel();
    if (!root) return;
    var deck =
      (window.getSlidesPresentationId && window.getSlidesPresentationId()) ||
      root.dataset.presentationId ||
      "current";
    var order = items(root).map(function (item) {
      return parseInt(item.dataset.index, 10);
    });
    var status = root.querySelector("#slide-sorter-status");
    if (button) button.disabled = true;
    if (status) status.textContent = "Salvando…";
    fetch("/api/slides/slide/reorder", {
      method: "POST",
      headers: authHeaders(),
      body: JSON.stringify({ presentation_id: deck, slide_order: order }),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function () {
        if (window.refreshSlideView) window.refreshSlideView();
        var host = document.getElementById("slides-modal-container");
        if (host) host.innerHTML = "";
      })
      .catch(function (error) {
        if (button) button.disabled = false;
        if (status) status.textContent = "Falha ao salvar: " + error.message;
      });
  }

  // Up/down buttons (fallback/reliable path) and Save.
  document.addEventListener("click", function (event) {
    var target = event.target;
    if (!target || !target.closest) return;
    if (target.closest("#slide-sorter-save")) {
      event.preventDefault();
      save(target.closest("#slide-sorter-save"));
      return;
    }
    var item = target.closest(".sl-sorter-item");
    if (!item || !item.closest("#slide-sorter-panel")) return;
    if (target.closest(".sl-sorter-up")) {
      event.preventDefault();
      move(item, -1);
    } else if (target.closest(".sl-sorter-down")) {
      event.preventDefault();
      move(item, 1);
    }
  });

  // HTML5 drag reordering; the DOM order is the source of truth.
  document.addEventListener("dragstart", function (event) {
    var item = event.target.closest ? event.target.closest(".sl-sorter-item") : null;
    if (!item || !item.closest("#slide-sorter-panel")) return;
    dragItem = item;
    item.style.opacity = "0.5";
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/plain", item.dataset.index || "");
    }
  });

  document.addEventListener("dragover", function (event) {
    if (!dragItem) return;
    var root = dragItem.closest("#slide-sorter-panel");
    if (!root) return;
    var target = event.target.closest ? event.target.closest(".sl-sorter-item") : null;
    if (!target || target === dragItem) return;
    event.preventDefault();
    var rect = target.getBoundingClientRect();
    var after = event.clientY > rect.top + rect.height / 2;
    target.parentNode.insertBefore(dragItem, after ? target.nextSibling : target);
    syncLabels(root);
  });

  document.addEventListener("drop", function (event) {
    if (!dragItem) return;
    event.preventDefault();
  });

  document.addEventListener("dragend", function () {
    if (dragItem) dragItem.style.opacity = "";
    dragItem = null;
    var root = panel();
    if (root) syncLabels(root);
  });

  // The fragment is swapped in after page load; label the initial order.
  document.addEventListener("htmx:afterSwap", function () {
    var root = panel();
    if (root) syncLabels(root);
  });
})();
