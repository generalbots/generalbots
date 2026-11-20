/**
 * Minimal but faithful DOM stub for #2533 shipped docs-large IIFEs.
 *
 * Supports exactly the surface modules 84..91 touch: createElement, attributes,
 * dataset, classList/className, textContent, append/remove, closest,
 * querySelector(All), focus and listener dispatch. DOM sentinels in tests are
 * code-like tokens (angle brackets), never user-facing English literals.
 */
"use strict";

function walk(root, fn) {
  fn(root);
  root.children.forEach(function (child) { walk(child, fn); });
}

function hasAttr(node, name) {
  return Object.prototype.hasOwnProperty.call(node._attrs, name);
}

function matches(node, selector) {
  selector = String(selector || "").trim();
  if (selector === "*") return true;
  var match = selector.match(/^([a-zA-Z][\w-]*)\.([\w-]+)$/);
  if (match) return node.tagName === match[1].toUpperCase() && node._classes.indexOf(match[2]) >= 0;
  match = selector.match(/^\.([\w-]+)$/);
  if (match) return node._classes.indexOf(match[1]) >= 0;
  match = selector.match(/^#([\w-]+)$/);
  if (match) return node.id === match[1] || node.getAttribute("id") === match[1];
  match = selector.match(/^([a-zA-Z][\w-]*)\[([^=\]]+)="([^"]+)"\]$/);
  if (match) return node.tagName === match[1].toUpperCase() && node.getAttribute(match[2]) === match[3];
  match = selector.match(/^\[([^=\]]+)="([^"]+)"\]$/);
  if (match) return node.getAttribute(match[1]) === match[2];
  match = selector.match(/^([a-zA-Z][\w-]*)\[([^\]]+)\]$/);
  if (match) return node.tagName === match[1].toUpperCase() && hasAttr(node, match[2]);
  match = selector.match(/^\[([^\]=]+)\]$/);
  if (match) return hasAttr(node, match[1]);
  if (/^[a-zA-Z][\w-]*$/.test(selector)) return node.tagName === selector.toUpperCase();
  return false;
}

function FakeNode(tag, ownerDoc) {
  this.tagName = String(tag || "").toUpperCase();
  this.children = [];
  this.parentNode = null;
  this._attrs = {};
  this._classes = [];
  this.style = {};
  this.listeners = {};
  this.value = "";
  this.hidden = false;
  this.disabled = false;
  this._text = "";
  this._ownerDoc = ownerDoc || null;
  var self = this;
  // Mirror dataset writes into data-* attributes as a real DOM element does,
  // so shipped code using getAttribute("data-block-id") observes them.
  this.dataset = new Proxy({}, {
    set: function (target, prop, value) {
      target[prop] = value;
      self._attrs["data-" + String(prop).replace(/[A-Z]/g, function (letter) {
        return "-" + letter.toLowerCase();
      })] = String(value);
      return true;
    },
  });
  this.classList = {
    add: function (name) { if (self._classes.indexOf(name) < 0) self._classes.push(name); },
    remove: function (name) {
      var index = self._classes.indexOf(name);
      if (index >= 0) self._classes.splice(index, 1);
    },
    contains: function (name) { return self._classes.indexOf(name) >= 0; },
  };
}

Object.defineProperty(FakeNode.prototype, "className", {
  get: function () { return this._classes.join(" "); },
  set: function (value) {
    this._classes = String(value || "").split(/\s+/).filter(function (name) { return !!name; });
  },
});

Object.defineProperty(FakeNode.prototype, "textContent", {
  get: function () {
    if (this.tagName === "#TEXT" || !this.children.length) return this._text;
    return this.children.map(function (child) { return child.textContent; }).join("");
  },
  set: function (value) {
    this.children = [];
    this._text = value == null ? "" : String(value);
  },
});

FakeNode.prototype.appendChild = function (child) {
  child.parentNode = this;
  this.children.push(child);
  return child;
};

FakeNode.prototype.removeChild = function (child) {
  var index = this.children.indexOf(child);
  if (index >= 0) this.children.splice(index, 1);
  child.parentNode = null;
  return child;
};

FakeNode.prototype.setAttribute = function (name, value) {
  this._attrs[name] = String(value);
  if (name === "class") this.className = value;
};
FakeNode.prototype.getAttribute = function (name) {
  return hasAttr(this, name) ? this._attrs[name] : null;
};
FakeNode.prototype.hasAttribute = function (name) { return hasAttr(this, name); };
FakeNode.prototype.addEventListener = function (type, fn) {
  (this.listeners[type] || (this.listeners[type] = [])).push(fn);
};
FakeNode.prototype.removeEventListener = function (type, fn) {
  var list = this.listeners[type] || [];
  var index = list.indexOf(fn);
  if (index >= 0) list.splice(index, 1);
};
FakeNode.prototype.dispatch = function (type, event) {
  (this.listeners[type] || []).slice().forEach(function (fn) { fn(event); });
};
FakeNode.prototype.contains = function (node) {
  var found = false;
  walk(this, function (candidate) { if (candidate === node) found = true; });
  return found;
};
FakeNode.prototype.querySelector = function (selector) {
  var found = null;
  var self = this;
  walk(this, function (candidate) {
    if (found || candidate === self) return;
    if (matches(candidate, selector)) found = candidate;
  });
  return found;
};
FakeNode.prototype.querySelectorAll = function (selector) {
  var self = this;
  var parts = String(selector).split(",").map(function (part) { return part.trim(); });
  var out = [];
  walk(this, function (candidate) {
    if (candidate === self) return;
    parts.forEach(function (raw) {
      var negated = raw.indexOf(":not(") >= 0;
      var part = negated ? raw.slice(0, raw.indexOf(":not(")) : raw;
      if (!matches(candidate, part)) return;
      if (negated && candidate.getAttribute("tabindex") === "-1") return;
      if (out.indexOf(candidate) < 0) out.push(candidate);
    });
  });
  return out;
};
FakeNode.prototype.closest = function (selector) {
  var node = this;
  while (node) {
    if (matches(node, selector)) return node;
    node = node.parentNode;
  }
  return null;
};
FakeNode.prototype.focus = function () {
  if (this._ownerDoc) this._ownerDoc.activeElement = this;
  return this;
};

function makeDocument() {
  var doc = {
    readyState: "loading",
    body: new FakeNode("body", null),
    activeElement: null,
    _nodes: {},
    _listeners: {},
    createElement: function (tag) { return new FakeNode(tag, doc); },
    createTextNode: function (text) {
      var node = new FakeNode("#text", doc);
      node._text = String(text);
      return node;
    },
    getElementById: function (id) {
      if (Object.prototype.hasOwnProperty.call(doc._nodes, id)) return doc._nodes[id];
      var found = null;
      walk(doc.body, function (candidate) {
        if (found) return;
        if (candidate.id === id || candidate.getAttribute("id") === id) found = candidate;
      });
      return found;
    },
    querySelector: function (selector) { return doc.body.querySelector(selector); },
    querySelectorAll: function (selector) { return doc.body.querySelectorAll(selector); },
    addEventListener: function (type, fn) {
      (doc._listeners[type] || (doc._listeners[type] = [])).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = doc._listeners[type] || [];
      var index = list.indexOf(fn);
      if (index >= 0) list.splice(index, 1);
    },
    dispatch: function (type, event) {
      (doc._listeners[type] || []).slice().forEach(function (fn) { fn(event); });
    },
  };
  doc.body._ownerDoc = doc;
  return doc;
}

function byClass(node, cls) {
  return node.querySelectorAll("*").filter(function (candidate) {
    return candidate._classes.indexOf(cls) >= 0;
  });
}

function withAttr(node, name) {
  return node.querySelectorAll("*").filter(function (candidate) {
    return candidate.hasAttribute(name);
  });
}

module.exports = {
  FakeNode,
  makeDocument,
  walk,
  matches,
  byClass,
  withAttr,
};
