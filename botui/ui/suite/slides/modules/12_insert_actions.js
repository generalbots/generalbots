"use strict";
/* Slides Insert hub — keyboard operation, insertion, drag-to-canvas (#2428). */

SlidesInsertHub.onCardKey = function (event, card) {
  var cards = this.cards();
  var at = cards.indexOf(card);
  if (event.key === "ArrowDown" || event.key === "ArrowRight") {
    event.preventDefault();
    if (cards[at + 1]) cards[at + 1].focus();
  } else if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
    event.preventDefault();
    if (cards[at - 1]) cards[at - 1].focus();
  } else if (event.key === "Home") {
    event.preventDefault();
    if (cards[0]) cards[0].focus();
  } else if (event.key === "End") {
    event.preventDefault();
    if (cards.length) cards[cards.length - 1].focus();
  } else if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    this.select(card.dataset.itemId);
  }
};

SlidesInsertHub.findItem = function (id) {
  if (!id) return null;
  if (id.indexOf("reuse-") === 0) {
    return this.reuseItems.filter(function (item) { return item.id === id; })[0] || null;
  }
  return siFind(id);
};

SlidesInsertHub.select = function (id) {
  var item = this.findItem(id);
  if (!item) return;
  this.selectedId = id;
  this.markSelected(id);
  this.insertBtn.disabled = false;
  this.renderPreview(item);
  this.renderOptions(item);
  this.setStatus(siT("slides-insert-status-selected", "{name} selected", {
    name: item.kind === "reuse" ? siT(item.key, item.fallback) : siLabel(item),
  }));
};

SlidesInsertHub.optionValues = function () {
  var values = {};
  Array.prototype.forEach.call(this.options.querySelectorAll("[data-si-name]"), function (field) {
    values[field.dataset.siName] = field.value;
  });
  return Object.assign(values, this.optionCache || {});
};

SlidesInsertHub.doInsert = function () {
  var self = this;
  var item = this.findItem(this.selectedId);
  if (!item) {
    this.setStatus(siT("slides-insert-status-ready", "Choose an item to preview"));
    return;
  }
  var options = this.optionValues();
  this.insertBtn.disabled = true;
  this.setStatus(siT("slides-insert-inserting", "Inserting…"));
  SlidesInsertPlace.place(item, options).then(function () {
    siRemember(item.id);
    var name = item.kind === "reuse" ? siT(item.key, item.fallback) : siLabel(item);
    self.close();
    if (window.GBToasts && window.GBToasts.show) {
      window.GBToasts.show(siT("slides-insert-title", "Insert"),
        siT("slides-insert-status-inserted", "{name} inserted", { name: name }), "success");
    }
  }).catch(function (error) {
    self.insertBtn.disabled = false;
    self.setStatus(siT("slides-insert-status-failed", "Insert failed: {error}", { error: error.message }));
  });
};

SlidesInsertHub.focusables = function () {
  if (!this.dialog) return [];
  return Array.prototype.filter.call(
    this.dialog.querySelectorAll("button:not([disabled]), input, textarea, select, [tabindex]:not([tabindex='-1'])"),
    function (node) { return node.offsetParent !== null || node === document.activeElement; }
  );
};

SlidesInsertHub.onKeyDown = function (event) {
  if (event.key === "Escape") {
    event.preventDefault();
    this.close();
    return;
  }
  if (event.key !== "Tab") return;
  var nodes = this.focusables();
  if (!nodes.length) return;
  var first = nodes[0];
  var last = nodes[nodes.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
};

SlidesInsertHub.bindCanvasDrop = function () {
  if (SlidesInsertHub._dropBound) return;
  SlidesInsertHub._dropBound = true;
  document.addEventListener("dragover", function (event) {
    if (!event.target.closest || !event.target.closest("#slide-canvas")) return;
    var types = event.dataTransfer ? event.dataTransfer.types : [];
    if (types.indexOf("application/x-slides-insert") >= 0 ||
        types.indexOf("application/x-gb-drive-file") >= 0) {
      event.preventDefault();
      event.dataTransfer.dropEffect = "copy";
    }
  });
  document.addEventListener("drop", function (event) {
    var canvas = event.target.closest && event.target.closest("#slide-canvas");
    if (!canvas || !event.dataTransfer) return;
    var rawInsert = event.dataTransfer.getData("application/x-slides-insert");
    var rawDrive = event.dataTransfer.getData("application/x-gb-drive-file");
    if (!rawInsert && !rawDrive) return;
    event.preventDefault();
    var scale = (window.SlideCanvas && window.SlideCanvas.scale) || 1;
    var rect = canvas.getBoundingClientRect();
    var at = { x: (event.clientX - rect.left) / scale, y: (event.clientY - rect.top) / scale };
    var options = {};
    var item;
    if (rawInsert) {
      item = SlidesInsertHub.findItem(rawInsert);
    } else {
      var data = {};
      try { data = JSON.parse(rawDrive); } catch (e) { return; }
      item = siFind("media-image");
      options.drivePath = data.path || "";
    }
    if (!item) return;
    if (SlidesInsertHub.dialog) SlidesInsertHub.close();
    SlidesInsertPlace.place(item, options, at).catch(function (error) {
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show(siT("slides-insert-title", "Insert"), error.message, "error");
      }
    });
  });
};

SlidesInsertHub.adoptLegacy = function () {
  Array.prototype.forEach.call(
    document.querySelectorAll('#slides-content [hx-post="/suite/slides/modals/insert-element"]'),
    function (button) {
      if (button.dataset.siAdopted) return;
      button.dataset.siAdopted = "1";
      button.removeAttribute("hx-post");
      button.removeAttribute("hx-vals");
      button.textContent = siT("slides-insert-open", "Insert");
      button.addEventListener("click", function (event) {
        event.preventDefault();
        event.stopPropagation();
        SlidesInsertHub.open(button);
      });
    }
  );
};

(function () {
  function init() {
    var toolbar = document.getElementById("slidesInsertBtn");
    if (toolbar && !toolbar.dataset.siBound) {
      toolbar.dataset.siBound = "1";
      toolbar.addEventListener("click", function () { SlidesInsertHub.open(toolbar); });
    }
    SlidesInsertHub.adoptLegacy();
    document.addEventListener("htmx:afterSwap", function (event) {
      var target = event.target;
      if (target && target.id === "slides-content") {
        SlidesInsertHub.adoptLegacy();
        SlidesInsertPlace.applyImageExtras();
      }
    });
    document.addEventListener("keydown", function (event) {
      if (!(event.ctrlKey || event.metaKey) || !event.altKey) return;
      if (event.key !== "i" && event.key !== "I") return;
      event.preventDefault();
      if (SlidesInsertHub.dialog) SlidesInsertHub.close();
      else SlidesInsertHub.open(document.getElementById("slidesInsertBtn"));
    });
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
