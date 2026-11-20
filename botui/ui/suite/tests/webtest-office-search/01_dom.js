/**
 * Minimal DOM stub for the #2505 shipped browser modules.
 *
 * Supports exactly the surface command-palette.js and gb-search.js use:
 * createElement, innerHTML assignment with a shallow id/class/data-index parse,
 * classList.toggle, style properties, querySelector(All), listeners, focus,
 * remove, setSelectionRange and scrollIntoView. Test fixtures use code-like
 * tokens, never user-facing literals, so the i18n scanner stays silent.
 */
"use strict";

function escapeText(value) {
  return String(value == null ? "" : value)
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

function walk(root, fn) {
  fn(root);
  root.children.forEach(function (child) { walk(child, fn); });
}

function hasAttr(node, name) {
  return Object.prototype.hasOwnProperty.call(node._attrs, name);
}

function matchesSimple(node, selector) {
  selector = String(selector || "").trim();
  if (selector === "*") return true;
  var m = selector.match(/^([a-zA-Z][\w-]*)\.([\w-]+)$/);
  if (m) return node.tagName === m[1].toUpperCase() && node._classes.indexOf(m[2]) >= 0;
  m = selector.match(/^\.([\w-]+)$/);
  if (m) return node._classes.indexOf(m[1]) >= 0;
  m = selector.match(/^#([\w-]+)$/);
  if (m) return node.id === m[1];
  m = selector.match(/^([a-zA-Z][\w-]*)\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node.tagName === m[1].toUpperCase() && node.getAttribute(m[2]) === m[3];
  m = selector.match(/^\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node.getAttribute(m[1]) === m[2];
  m = selector.match(/^([a-zA-Z][\w-]*)\[([^\]]+)\]$/);
  if (m) return node.tagName === m[1].toUpperCase() && hasAttr(node, m[2]);
  m = selector.match(/^\[([^\]=]+)\]$/);
  if (m) return hasAttr(node, m[1]);
  if (/^[a-zA-Z][\w-]*$/.test(selector)) return node.tagName === selector.toUpperCase();
  return false;
}

function find(root, selector, all) {
  var parts = String(selector || "").trim().split(/\s+/).filter(Boolean);
  var out = [];
  walk(root, function (candidate) {
    if (candidate === root || !matchesSimple(candidate, parts[parts.length - 1])) return;
    var index = parts.length - 2;
    var ancestor = candidate.parentNode;
    while (index >= 0 && ancestor) {
      if (matchesSimple(ancestor, parts[index])) index--;
      ancestor = ancestor.parentNode;
    }
    if (index < 0 && out.indexOf(candidate) < 0) out.push(candidate);
  });
  return all ? out : (out[0] || null);
}

/** Rebuild a flat child list from the start tags of a shipped HTML string. */
function parseChildren(node, html) {
  node.children = [];
  var re = /<([a-zA-Z][\w-]*)([^>]*)>/g;
  var match = re.exec(html);
  while (match) {
    var child = new FakeNode(match[1], node._ownerDoc);
    var attrs = match[2] || "";
    var idm = attrs.match(/\bid="([^"]*)"/);
    var clm = attrs.match(/\bclass="([^"]*)"/);
    var dim = attrs.match(/\bdata-index="([^"]*)"/);
    if (idm) child.id = idm[1];
    if (clm) child.className = clm[1];
    if (dim) child.setAttribute("data-index", dim[1]);
    child.parentNode = node;
    node.children.push(child);
    match = re.exec(html);
  }
}

function FakeNode(tag, ownerDoc) {
  this.tagName = String(tag || "").toUpperCase();
  this.children = [];
  this.parentNode = null;
  this._attrs = {};
  this._classes = [];
  this._text = "";
  this._html = "";
  this.listeners = {};
  this.value = "";
  this.id = "";
  this.hidden = false;
  this.clientWidth = 0;
  this.scrollTop = 0;
  this.scrollLeft = 0;
  this.focusCalls = 0;
  this.scrollIntoViewCalls = 0;
  this.selectionRange = null;
  this.rect = { top: 0, bottom: 0, left: 0, right: 0, width: 0, height: 0 };
  this._ownerDoc = ownerDoc || null;
  var self = this;
  this.style = {
    setProperty: function (name, value) { this[name] = String(value); },
    getPropertyValue: function (name) {
      return Object.prototype.hasOwnProperty.call(this, name) ? this[name] : "";
    },
  };
  this.dataset = new Proxy({}, {
    set: function (target, prop, value) {
      target[prop] = value;
      self._attrs["data-" + String(prop)] = String(value);
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
    toggle: function (name, force) {
      var present = self._classes.indexOf(name) >= 0;
      var next = force === undefined ? !present : !!force;
      if (next && !present) self._classes.push(name);
      if (!next && present) self._classes.splice(self._classes.indexOf(name), 1);
      return next;
    },
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
    if (!this.children.length) return this._text;
    return this.children.map(function (child) { return child.textContent; }).join("");
  },
  set: function (value) {
    this.children = [];
    this._html = "";
    this._text = value == null ? "" : String(value);
  },
});

Object.defineProperty(FakeNode.prototype, "innerHTML", {
  get: function () {
    if (this._html) return this._html;
    return this.children.length
      ? this.children.map(function (child) { return child.innerHTML; }).join("")
      : escapeText(this._text);
  },
  set: function (value) {
    this._text = "";
    this._html = String(value == null ? "" : value);
    parseChildren(this, this._html);
  },
});

Object.defineProperty(FakeNode.prototype, "firstChild", {
  get: function () { return this.children.length ? this.children[0] : null; },
});

FakeNode.prototype.appendChild = function (child) {
  child.parentNode = this;
  this.children.push(child);
  return child;
};

FakeNode.prototype.insertBefore = function (child, ref) {
  child.parentNode = this;
  var index = ref ? this.children.indexOf(ref) : -1;
  if (index < 0) this.children.push(child);
  else this.children.splice(index, 0, child);
  return child;
};

FakeNode.prototype.removeChild = function (child) {
  var index = this.children.indexOf(child);
  if (index >= 0) this.children.splice(index, 1);
  child.parentNode = null;
  return child;
};

FakeNode.prototype.remove = function () {
  if (this.parentNode) this.parentNode.removeChild(this);
};

FakeNode.prototype.setAttribute = function (name, value) {
  this._attrs[name] = String(value);
  if (name === "class") this.className = value;
  if (name === "id") this.id = String(value);
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
FakeNode.prototype.dispatchEvent = function (event) {
  (this.listeners[event.type] || []).slice().forEach(function (fn) { fn(event); });
  return true;
};
FakeNode.prototype.contains = function (node) {
  var found = false;
  walk(this, function (candidate) { if (candidate === node) found = true; });
  return found;
};
FakeNode.prototype.querySelector = function (selector) { return find(this, selector, false); };
FakeNode.prototype.querySelectorAll = function (selector) { return find(this, selector, true); };
FakeNode.prototype.closest = function (selector) {
  var node = this;
  while (node) {
    if (matchesSimple(node, selector)) return node;
    node = node.parentNode;
  }
  return null;
};
FakeNode.prototype.focus = function () {
  this.focusCalls++;
  if (this._ownerDoc) this._ownerDoc.activeElement = this;
  return this;
};
FakeNode.prototype.getBoundingClientRect = function () { return this.rect; };
FakeNode.prototype.setSelectionRange = function (start, end) {
  this.selectionRange = { start: start, end: end };
};
FakeNode.prototype.scrollIntoView = function (options) {
  this.scrollIntoViewCalls++;
  this.lastScrollOptions = options || null;
};

function makeDocument() {
  var doc = {
    readyState: "complete",
    activeElement: null,
    _nodes: {},
    _listeners: {},
    createElement: function (tag) { return new FakeNode(tag, doc); },
    createTextNode: function (value) {
      var node = new FakeNode("#text", doc);
      node._text = String(value);
      return node;
    },
    getElementById: function (id) {
      if (Object.prototype.hasOwnProperty.call(doc._nodes, id)) return doc._nodes[id];
      var found = null;
      walk(doc.body, function (candidate) {
        if (!found && candidate.id === id) found = candidate;
      });
      return found;
    },
    querySelector: function (selector) { return find(doc.body, selector, false); },
    querySelectorAll: function (selector) { return find(doc.body, selector, true); },
    addEventListener: function (type, fn) {
      (doc._listeners[type] || (doc._listeners[type] = [])).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = doc._listeners[type] || [];
      var index = list.indexOf(fn);
      if (index >= 0) list.splice(index, 1);
    },
    dispatchEvent: function (event) {
      (doc._listeners[event.type] || []).slice().forEach(function (fn) { fn(event); });
      return true;
    },
  };
  doc.body = new FakeNode("body", doc);
  doc.body._isDocumentBody = true;
  doc.documentElement = new FakeNode("html", doc);
  doc.head = new FakeNode("head", doc);
  return doc;
}

function el(tag, attrs) {
  var node = new FakeNode(tag, null);
  Object.keys(attrs || {}).forEach(function (name) {
    if (name === "class") node.className = attrs[name];
    else node.setAttribute(name, attrs[name]);
  });
  return node;
}

module.exports = { FakeNode, makeDocument, el, walk, escapeText, find };
