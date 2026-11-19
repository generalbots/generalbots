"use strict";
/* Slides Insert hub — dialog shell, category rail, grid and preview (#2428).
 * Built lazily on first open; the app shell is never reloaded. */

var SlidesInsertHub = {
  dialog: null,
  opener: null,
  category: "recent",
  query: "",
  selectedId: null,
  reuseItems: [],
  optionCache: null,
};

SlidesInsertHub.el = function (tag, className, text) {
  var node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined && text !== null) node.textContent = text;
  return node;
};

SlidesInsertHub.open = function (opener) {
  if (this.dialog) return;
  this.opener = opener || document.activeElement;
  this.category = "recent";
  this.query = "";
  this.selectedId = null;
  this.optionCache = null;
  var host = document.getElementById("slides-modal-container") || document.body;
  this.dialog = this.build();
  host.appendChild(this.dialog);
  this.renderCategories();
  this.renderGrid();
  this.search.focus();
  this.setStatus(siT("slides-insert-status-ready", "Choose an item to preview"));
};

SlidesInsertHub.close = function () {
  if (!this.dialog) return;
  this.dialog.remove();
  this.dialog = null;
  if (this.opener && this.opener.focus) {
    try { this.opener.focus(); } catch (e) { /* opener gone */ }
  }
  this.opener = null;
  this.optionCache = null;
};

SlidesInsertHub.build = function () {
  var self = this;
  var backdrop = this.el("div", "sl-insert-backdrop");
  var dialog = this.el("div", "sl-insert-dialog");
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "sl-insert-title");

  var header = this.el("div", "sl-insert-header");
  var title = this.el("h2", "sl-insert-title", siT("slides-insert-title", "Insert"));
  title.id = "sl-insert-title";
  this.search = this.el("input", "sl-insert-search");
  this.search.type = "search";
  this.search.setAttribute("aria-label", siT("slides-insert-search-label", "Search insertables"));
  this.search.setAttribute("placeholder", siT("slides-insert-search-placeholder", "Search layouts, media, shapes…"));
  var hint = this.el("span", "sl-insert-hint", siT("slides-insert-keyboard-hint", "Arrow keys to browse · Enter to select · Esc to close"));
  var close = this.el("button", "sl-insert-close", "×");
  close.type = "button";
  close.setAttribute("aria-label", siT("slides-insert-close", "Close"));
  close.addEventListener("click", function () { self.close(); });
  header.appendChild(title);
  header.appendChild(this.search);
  header.appendChild(hint);
  header.appendChild(close);

  var body = this.el("div", "sl-insert-body");
  this.cats = this.el("div", "sl-insert-cats");
  this.cats.setAttribute("role", "tablist");
  this.cats.setAttribute("aria-label", siT("slides-insert-categories", "Categories"));
  var main = this.el("div", "sl-insert-main");
  this.grid = this.el("div", "sl-insert-grid");
  this.grid.setAttribute("role", "listbox");
  this.grid.setAttribute("aria-label", siT("slides-insert-items", "Insertable items"));
  main.appendChild(this.grid);
  var side = this.el("div", "sl-insert-side");
  this.preview = this.el("div", "sl-insert-preview");
  this.options = this.el("div", "sl-insert-options");
  side.appendChild(this.preview);
  side.appendChild(this.options);
  body.appendChild(this.cats);
  body.appendChild(main);
  body.appendChild(side);

  var footer = this.el("div", "sl-insert-footer");
  this.status = this.el("p", "sl-insert-status");
  this.status.setAttribute("role", "status");
  this.status.setAttribute("aria-live", "polite");
  var cancel = this.el("button", "sl-insert-cancel", siT("slides-insert-cancel", "Cancel"));
  cancel.type = "button";
  cancel.addEventListener("click", function () { self.close(); });
  this.insertBtn = this.el("button", "sl-insert-insert", siT("slides-insert-insert", "Insert"));
  this.insertBtn.type = "button";
  this.insertBtn.disabled = true;
  this.insertBtn.addEventListener("click", function () { self.doInsert(); });
  footer.appendChild(this.status);
  footer.appendChild(cancel);
  footer.appendChild(this.insertBtn);

  dialog.appendChild(header);
  dialog.appendChild(body);
  dialog.appendChild(footer);
  backdrop.appendChild(dialog);

  this.search.addEventListener("input", function () {
    self.query = self.search.value || "";
    self.renderGrid();
  });
  this.search.addEventListener("keydown", function (event) {
    if (event.key === "ArrowDown" || event.key === "Enter") {
      var first = self.grid.querySelector(".sl-insert-card");
      if (first) { event.preventDefault(); first.focus(); }
    }
  });
  dialog.addEventListener("keydown", function (event) { self.onKeyDown(event); });
  backdrop.addEventListener("pointerdown", function (event) {
    if (event.target === backdrop) self.close();
  });
  self.bindCanvasDrop();
  return backdrop;
};

SlidesInsertHub.setStatus = function (message) {
  if (this.status) this.status.textContent = message;
};

SlidesInsertHub.cards = function () {
  return Array.prototype.slice.call(this.grid.querySelectorAll(".sl-insert-card"));
};

SlidesInsertHub.renderCategories = function () {
  var self = this;
  this.cats.textContent = "";
  SI_CATEGORIES.forEach(function (category) {
    var btn = self.el("button", "sl-insert-cat", siT(category.key, category.fallback));
    btn.type = "button";
    btn.setAttribute("role", "tab");
    btn.dataset.category = category.id;
    btn.setAttribute("aria-selected", category.id === self.category ? "true" : "false");
    btn.tabIndex = category.id === self.category ? 0 : -1;
    btn.addEventListener("click", function () { self.setCategory(category.id); });
    btn.addEventListener("keydown", function (event) {
      if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
      event.preventDefault();
      var at = SI_CATEGORIES.findIndex(function (c) { return c.id === category.id; });
      var next = SI_CATEGORIES[(at + (event.key === "ArrowRight" ? 1 : SI_CATEGORIES.length - 1)) % SI_CATEGORIES.length];
      var target = self.cats.querySelector('[data-category="' + next.id + '"]');
      if (target) { target.focus(); self.setCategory(next.id); }
    });
    self.cats.appendChild(btn);
  });
};

SlidesInsertHub.setCategory = function (id) {
  var self = this;
  this.category = id;
  Array.prototype.forEach.call(this.cats.children, function (btn) {
    var active = btn.dataset.category === id;
    btn.setAttribute("aria-selected", active ? "true" : "false");
    btn.tabIndex = active ? 0 : -1;
  });
  if (id === "reuse" && !this.reuseItems.length) {
    this.setStatus(siT("slides-insert-reuse-loading", "Loading deck…"));
    SlidesInsertPlace.reuseItems().then(function (items) {
      self.reuseItems = items;
      self.renderGrid();
      self.setStatus(items.length
        ? siT("slides-insert-status-ready", "Choose an item to preview")
        : siT("slides-insert-reuse-empty", "No reusable elements found."));
    }).catch(function (error) {
      self.setStatus(siT("slides-insert-status-failed", "Insert failed: {error}", { error: error.message }));
    });
  }
  this.renderGrid();
};

SlidesInsertHub.items = function () {
  if (this.category === "reuse") {
    var self = this;
    return this.reuseItems.filter(function (item) {
      var haystack = siT(item.key, item.fallback) + " " + item.keywords + " " + item.labelSuffix;
      return haystack.toLowerCase().indexOf(self.query.toLowerCase().trim()) >= 0;
    });
  }
  return siItemsFor(this.category, this.query);
};

SlidesInsertHub.renderGrid = function () {
  var self = this;
  this.grid.textContent = "";
  var items = this.items();
  if (!items.length) {
    this.grid.appendChild(this.el("p", "sl-insert-empty", siT("slides-insert-empty", "No matches. Try a different search.")));
    return;
  }
  items.forEach(function (item) {
    var card = self.el("div", "sl-insert-card");
    card.setAttribute("role", "option");
    card.setAttribute("aria-selected", "false");
    card.tabIndex = -1;
    card.dataset.itemId = item.id;
    card.draggable = true;
    card.appendChild(self.previewGlyph(item, "sl-insert-card-glyph"));
    var label = item.kind === "reuse"
      ? siT(item.key, item.fallback) + " · " + item.labelSuffix
      : siLabel(item);
    card.appendChild(self.el("span", "sl-insert-card-label", label));
    card.addEventListener("click", function () { self.select(item.id); });
    card.addEventListener("dblclick", function () { self.select(item.id); self.doInsert(); });
    card.addEventListener("dragstart", function (event) {
      event.dataTransfer.setData("application/x-slides-insert", item.id);
      event.dataTransfer.effectAllowed = "copy";
    });
    card.addEventListener("keydown", function (event) { self.onCardKey(event, card); });
    self.grid.appendChild(card);
  });
  if (this.selectedId) this.markSelected(this.selectedId);
};

SlidesInsertHub.markSelected = function (id) {
  Array.prototype.forEach.call(this.grid.children, function (card) {
    var active = card.dataset.itemId === id;
    card.setAttribute("aria-selected", active ? "true" : "false");
    card.classList.toggle("selected", active);
  });
};

SlidesInsertHub.previewGlyph = function (item, className) {
  var box = this.el("div", className);
  var glyph = item.glyph || item.kind;
  if (SI_GLYPHS[glyph]) {
    var svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("width", "22");
    svg.setAttribute("height", "22");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "1.8");
    var path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", SI_GLYPHS[glyph]);
    svg.appendChild(path);
    box.appendChild(svg);
    return box;
  }
  box.classList.add("sl-insert-glyph-" + String(glyph).replace(/[^a-z-]/gi, ""));
  box.textContent = glyph === "text" ? "T" : glyph === "heading" ? "H" : glyph === "equation" ? "ƒ" : "◆";
  return box;
};

SlidesInsertHub.renderPreview = function (item) {
  var self = this;
  this.preview.textContent = "";
  var frame = this.el("div", "sl-insert-preview-frame");
  var stage = this.el("div", "sl-insert-preview-stage");
  if (item.kind === "image") {
    var src = (this.optionCache && this.optionCache.src) || this.optionValue("url") || "";
    if (src) {
      var img = this.el("img", "sl-insert-preview-image");
      img.src = src;
      img.alt = "";
      stage.appendChild(img);
    } else {
      stage.appendChild(this.previewGlyph(item, "sl-insert-preview-icon"));
    }
  } else if (item.kind === "layout") {
    var mini = this.el("div", "sl-insert-mini-layout sl-insert-mini-" + item.layout);
    ["sl-insert-mini-a", "sl-insert-mini-b", "sl-insert-mini-c"].forEach(function (c) {
      mini.appendChild(self.el("div", c));
    });
    stage.appendChild(mini);
  } else if (item.kind === "table") {
    var table = this.el("div", "sl-insert-mini-table");
    for (var i = 0; i < 3; i++) {
      for (var j = 0; j < 3; j++) table.appendChild(this.el("span", "sl-insert-mini-cell"));
    }
    stage.appendChild(table);
  } else if (item.kind === "chart") {
    var bars = this.el("div", "sl-insert-mini-chart");
    [40, 70, 55, 85].forEach(function (h) {
      var bar = self.el("span", "sl-insert-mini-bar");
      bar.style.height = h + "%";
      bars.appendChild(bar);
    });
    stage.appendChild(bars);
  } else {
    stage.appendChild(this.previewGlyph(item, "sl-insert-preview-icon"));
    var sample = item.kind === "equation" ? siT(item.formulaKey, item.formula)
      : item.kind === "field" ? this.fieldPreview(item)
      : siLabel(item);
    stage.appendChild(this.el("span", "sl-insert-preview-caption", sample));
  }
  frame.appendChild(stage);
  this.preview.appendChild(frame);
};

SlidesInsertHub.fieldPreview = function (item) {
  if (item.field === "slide-number") return String(SlidesInsertPlace.slide() + 1);
  if (item.field === "date") return new Date().toLocaleDateString();
  return siT(item.valueKey, item.fallback);
};

SlidesInsertHub.optionValue = function (name) {
  var field = this.options.querySelector('[data-si-name="' + name + '"]');
  return field ? field.value : "";
};

SlidesInsertHub.renderOptions = function (item) {
  var self = this;
  this.options.textContent = "";
  var list = item.options || [];
  list.forEach(function (option) {
    var wrap = self.el("label", "sl-insert-field");
    wrap.appendChild(self.el("span", "sl-insert-field-label", siT(option.key, option.fallback)));
    var field;
    if (option.type === "textarea") {
      field = self.el("textarea", "sl-insert-input");
      field.rows = 3;
    } else if (option.type === "select") {
      field = self.el("select", "sl-insert-input");
      option.choices.forEach(function (choice) {
        var opt = self.el("option", "", siT(choice[1], choice[2]));
        opt.value = choice[0];
        field.appendChild(opt);
      });
      field.value = option.value;
    } else if (option.type === "file") {
      field = self.el("input", "sl-insert-input");
      field.type = "file";
      field.accept = "image/png,image/jpeg,image/gif,image/webp";
    } else {
      field = self.el("input", "sl-insert-input");
      field.type = option.type === "number" ? "number" : option.type === "url" ? "url" : "text";
      if (option.min !== undefined) field.min = option.min;
      if (option.max !== undefined) field.max = option.max;
      if (option.value !== undefined) field.value = option.value;
    }
    field.dataset.siName = option.name;
    if (option.placeholderKey) {
      field.setAttribute("placeholder", siT(option.placeholderKey, option.placeholderFallback));
    }
    if (option.type === "file") {
      field.addEventListener("change", function () {
        var file = field.files && field.files[0];
        if (!file) return;
        if (file.size > 1024 * 1024) {
          self.setStatus(siT("slides-insert-file-too-large", "Images larger than 1 MB cannot be inserted offline."));
          field.value = "";
          return;
        }
        var reader = new FileReader();
        reader.onload = function () {
          self.optionCache = self.optionCache || {};
          self.optionCache.src = String(reader.result || "");
          self.renderPreview(item);
        };
        reader.readAsDataURL(file);
      });
    } else {
      field.addEventListener("input", function () {
        if (option.name === "url" || option.name === "src") {
          self.optionCache = self.optionCache || {};
          self.optionCache.src = field.value;
          self.renderPreview(item);
        }
      });
    }
    wrap.appendChild(field);
    self.options.appendChild(wrap);
    if (option.type === "drive") {
      var resolve = self.el("button", "sl-insert-resolve", siT("slides-insert-resolve-drive", "Attach Drive file"));
      resolve.type = "button";
      resolve.addEventListener("click", function () {
        SlidesInsertPlace.resolveDrive(field.value).then(function (url) {
          if (!url) throw new Error("missing link");
          self.optionCache = self.optionCache || {};
          self.optionCache.src = url;
          self.renderPreview(item);
          self.setStatus(siT("slides-insert-drive-attached", "Drive file attached."));
        }).catch(function () {
          self.setStatus(siT("slides-insert-drive-link-failed", "Could not create a Drive link for this file."));
        });
      });
      self.options.appendChild(resolve);
    }
  });
  if (item.kind === "layout") {
    this.options.appendChild(this.el("p", "sl-insert-layout-note",
      siT("slides-insert-insert-after", "New slides are added after the current slide.")));
  }
  if (!list.length) {
    this.options.appendChild(this.el("p", "sl-insert-layout-note",
      siT("slides-insert-drop-hint", "Drop the card on the slide to place it at the pointer.")));
  }
};
