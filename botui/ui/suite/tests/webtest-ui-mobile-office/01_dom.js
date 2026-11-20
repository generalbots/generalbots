/**
 * Minimal DOM stub for the #2515 shipped mobile IIFEs.
 *
 * Supports exactly the surface docs/19_mobile.js and slides/99_mobile.js use:
 * createElement, id/class attributes, classList.toggle(name, force), style
 * properties, style.setProperty/getPropertyValue, insertBefore, appendChild,
 * querySelector(All) including the two descendant selectors used by Slides,
 * contains, focus(options), getBoundingClientRect and listener dispatch.
 * DOM sentinels in tests are code-like tokens, never user-facing literals.
 */
"use strict";

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

/** querySelectorAll with comma-separated selector lists (CSS standard). */
function queryAll(root, selector) {
  return String(selector || "").split(",").reduce(function (out, part) {
    find(root, part, true).forEach(function (node) {
      if (out.indexOf(node) < 0) out.push(node);
    });
    return out;
  }, []);
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

function FakeNode(tag, ownerDoc) {
  this.tagName = String(tag || "").toUpperCase();
  this.children = [];
  this.parentNode = null;
  this._attrs = {};
  this._classes = [];
  this._props = {};
  this.style = {
    setProperty: function (name, value) { this._props[name] = String(value); }.bind(this),
    getPropertyValue: function (name) {
      return Object.prototype.hasOwnProperty.call(this._props, name) ? this._props[name] : "";
    }.bind(this),
  };
  this.listeners = {};
  this.value = "";
  this.hidden = false;
  this.disabled = false;
  this._text = "";
  this.rect = { top: 0, bottom: 0, left: 0, right: 0, width: 0, height: 0 };
  this.clientWidth = 0;
  this.scrollTop = 0;
  this.scrollLeft = 0;
  this.focusCalls = 0;
  this.lastFocusOptions = null;
  this._ownerDoc = ownerDoc || null;
  this.id = "";
  var self = this;
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
    if (this.tagName === "#TEXT" || !this.children.length) return this._text;
    return this.children.map(function (child) { return child.textContent; }).join("");
  },
  set: function (value) {
    this.children = [];
    this._text = value == null ? "" : String(value);
  },
});

Object.defineProperty(FakeNode.prototype, "firstChild", {
  get: function () { return this.children.length ? this.children[0] : null; },
});

Object.defineProperty(FakeNode.prototype, "isConnected", {
  get: function () {
    var node = this;
    while (node.parentNode) node = node.parentNode;
    return node._isDocumentBody === true;
  },
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
FakeNode.prototype.querySelectorAll = function (selector) { return queryAll(this, selector); };
FakeNode.prototype.closest = function (selector) {
  var node = this;
  while (node) {
    if (matchesSimple(node, selector)) return node;
    node = node.parentNode;
  }
  return null;
};
FakeNode.prototype.focus = function (options) {
  this.focusCalls++;
  this.lastFocusOptions = options || null;
  if (this._ownerDoc) this._ownerDoc.activeElement = this;
  return this;
};
FakeNode.prototype.getBoundingClientRect = function () { return this.rect; };

function makeDocument() {
  var doc = {
    readyState: "loading",
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
        if (!found && candidate.id === id) found = candidate;
      });
      return found;
    },
    querySelector: function (selector) { return find(doc.body, selector, false); },
    querySelectorAll: function (selector) { return queryAll(doc.body, selector); },
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
    else if (name === "text") node._text = attrs[name];
    else node.setAttribute(name, attrs[name]);
  });
  return node;
}

function byClass(node, cls) {
  return node.querySelectorAll("*").filter(function (candidate) {
    return candidate._classes.indexOf(cls) >= 0;
  });
}

module.exports = { FakeNode, makeDocument, el, walk, matchesSimple, find, byClass };
