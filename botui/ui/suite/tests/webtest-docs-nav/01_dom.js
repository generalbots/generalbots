/**
 * Minimal faithful DOM stub for the #2485 NAV offline harness. Implements the
 * surface the shipped nav IIFEs touch: attributes, dataset, classList, text,
 * innerHTML (with a small parser), sibling/getElementById lookups, listeners,
 * selection-friendly text nodes and a live NamedNodeMap-like attributes view.
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
  var m = selector.match(/^([a-zA-Z]+)\.([\w-]+)$/);
  if (m) return node.tagName === m[1].toUpperCase() && node._classes.indexOf(m[2]) >= 0;
  m = selector.match(/^\.([\w-]+)\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node._classes.indexOf(m[1]) >= 0 && node.getAttribute(m[2]) === m[3];
  m = selector.match(/^\.([\w-]+)$/);
  if (m) return node._classes.indexOf(m[1]) >= 0;
  m = selector.match(/^#([\w-]+)$/);
  if (m) return node.getAttribute("id") === m[1];
  m = selector.match(/^([a-zA-Z]+)\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node.tagName === m[1].toUpperCase() && node.getAttribute(m[2]) === m[3];
  m = selector.match(/^\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node.getAttribute(m[1]) === m[2];
  m = selector.match(/^([a-zA-Z]+)\[([^\]]+)\]$/);
  if (m) return node.tagName === m[1].toUpperCase() && hasAttr(node, m[2]);
  m = selector.match(/^\[([^\]=]+)\]$/);
  if (m) return hasAttr(node, m[1]);
  if (/^[a-zA-Z][\w-]*$/.test(selector)) return node.tagName === selector.toUpperCase();
  return false;
}

function serialize(node) {
  return node.children.map(function (child) {
    if (child.tagName === "#TEXT") return child._text;
    var tag = child.tagName.toLowerCase();
    var attrs = Object.keys(child._attrs).map(function (key) {
      return " " + key + '="' + child._attrs[key] + '"';
    }).join("");
    return "<" + tag + attrs + ">" + serialize(child) + "</" + tag + ">";
  }).join("");
}

function parseHtml(html, ownerDoc) {
  var holder = new FakeNode("#frag", ownerDoc);
  var stack = [holder];
  var tagRe = /<\/?([a-zA-Z][\w-]*)((?:\s+[^<>]*?)?)(\/?)>|([^<]+)/g;
  var attrRe = /([\w:-]+)\s*=\s*"([^"]*)"|([\w:-]+)\s*=\s*'([^']*)'|([\w:-]+)/g;
  var voids = /^(br|img|input|hr|meta|link|area|base|col|embed|source|track|wbr)$/i;
  var match;
  while ((match = tagRe.exec(String(html || ""))) !== null) {
    if (match[4] !== undefined) {
      stack[stack.length - 1].appendChild(ownerDoc.createTextNode(match[4]));
      continue;
    }
    var tag = match[1];
    if (match[0].charAt(1) === "/") {
      if (stack.length > 1) stack.pop();
      continue;
    }
    var element = ownerDoc.createElement(tag);
    var attrMatch;
    while ((attrMatch = attrRe.exec(match[2] || "")) !== null) {
      var name = attrMatch[1] || attrMatch[3] || attrMatch[5];
      var value = attrMatch[2] !== undefined ? attrMatch[2]
        : (attrMatch[4] !== undefined ? attrMatch[4] : "");
      element.setAttribute(name, value);
    }
    stack[stack.length - 1].appendChild(element);
    if (!match[3] && !voids.test(tag)) stack.push(element);
  }
  return holder;
}

function FakeNode(tag, ownerDoc) {
  this.tagName = String(tag || "").toUpperCase();
  this.nodeType = this.tagName === "#TEXT" ? 3 : 1;
  this.children = [];
  this.parentNode = null;
  this._attrs = {};
  this._classes = [];
  this._text = "";
  this._ownerDoc = ownerDoc || null;
  var self = this;
  this.dataset = new Proxy({}, {
    set: function (target, prop, value) {
      target[prop] = value;
      self._attrs["data-" + String(prop).replace(/[A-Z]/g, function (m) {
        return "-" + m.toLowerCase();
      })] = String(value);
      return true;
    },
  });
  this.style = {};
  this.listeners = {};
  this.value = "";
  this.hidden = false;
  this.scrollTop = 0;
  var classes = {
    add: function (c) { if (self._classes.indexOf(c) < 0) self._classes.push(c); },
    remove: function (c) {
      var i = self._classes.indexOf(c);
      if (i >= 0) self._classes.splice(i, 1);
    },
    contains: function (c) { return self._classes.indexOf(c) >= 0; },
    toggle: function (c, force) {
      if (force === undefined) force = self._classes.indexOf(c) < 0;
      if (force) classes.add(c); else classes.remove(c);
      return !!force;
    },
  };
  this.classList = classes;
}

Object.defineProperty(FakeNode.prototype, "className", {
  get: function () { return this._classes.join(" "); },
  set: function (value) {
    this._classes = String(value || "").split(/\s+/).filter(function (c) { return !!c; });
    this._attrs.class = this._classes.join(" ");
  },
});
Object.defineProperty(FakeNode.prototype, "textContent", {
  get: function () {
    if (this.tagName === "#TEXT" || !this.children.length) return this._text;
    return this.children.map(function (child) { return child.textContent; }).join("");
  },
  set: function (value) { this.children = []; this._text = value == null ? "" : String(value); },
});
Object.defineProperty(FakeNode.prototype, "data", {
  get: function () { return this._text; },
  set: function (value) { this._text = value == null ? "" : String(value); },
});
Object.defineProperty(FakeNode.prototype, "id", {
  get: function () { return this.getAttribute("id") || ""; },
  set: function (value) { this.setAttribute("id", value); },
});
Object.defineProperty(FakeNode.prototype, "innerHTML", {
  get: function () { return this.children.length ? serialize(this) : (this._html || ""); },
  set: function (value) {
    this.children = [];
    this._html = String(value == null ? "" : value);
    var holder = parseHtml(this._html, this._ownerDoc);
    var self = this;
    holder.children.forEach(function (child) { self.appendChild(child); });
  },
});
Object.defineProperty(FakeNode.prototype, "parentElement", {
  get: function () {
    return this.parentNode && this.parentNode.tagName !== "#frag" ? this.parentNode : null;
  },
});
Object.defineProperty(FakeNode.prototype, "nextElementSibling", {
  get: function () {
    if (!this.parentNode) return null;
    var index = this.parentNode.children.indexOf(this);
    var next = this.parentNode.children[index + 1];
    return next && next.tagName !== "#TEXT" ? next : null;
  },
});
// Live NamedNodeMap-like view: removing an attribute shifts later indices,
// mirroring the browser collection that production code iterates.
Object.defineProperty(FakeNode.prototype, "attributes", {
  get: function () {
    var self = this;
    return new Proxy({}, {
      has: function (target, prop) {
        return typeof prop === "string" && /^\d+$/.test(prop)
          ? Number(prop) < Object.keys(self._attrs).length
          : false;
      },
      get: function (target, prop) {
        if (prop === "length") return Object.keys(self._attrs).length;
        if (prop === "item") {
          return function (index) {
            var key = Object.keys(self._attrs)[index];
            return key ? { name: key, value: self._attrs[key] } : null;
          };
        }
        if (typeof prop === "string" && /^\d+$/.test(prop)) {
          var name = Object.keys(self._attrs)[Number(prop)];
          return name ? { name: name, value: self._attrs[name] } : undefined;
        }
        return undefined;
      },
    });
  },
});

FakeNode.prototype.appendChild = function (child) {
  if (child.parentNode === this) {
    var existing = this.children.indexOf(child);
    if (existing >= 0) this.children.splice(existing, 1);
  }
  child.parentNode = this;
  this.children.push(child);
  return child;
};
FakeNode.prototype.removeChild = function (child) {
  var i = this.children.indexOf(child);
  if (i >= 0) this.children.splice(i, 1);
  child.parentNode = null;
  return child;
};
FakeNode.prototype.insertBefore = function (child, ref) {
  if (child.parentNode === this) {
    var existing = this.children.indexOf(child);
    if (existing >= 0) this.children.splice(existing, 1);
  }
  var index = ref ? this.children.indexOf(ref) : -1;
  child.parentNode = this;
  if (index < 0) this.children.push(child);
  else this.children.splice(index, 0, child);
  return child;
};
FakeNode.prototype.setAttribute = function (name, value) {
  this._attrs[name] = String(value);
  if (name === "class") this._classes = String(value).split(/\s+/).filter(function (c) { return !!c; });
};
FakeNode.prototype.getAttribute = function (name) { return hasAttr(this, name) ? this._attrs[name] : null; };
FakeNode.prototype.hasAttribute = function (name) { return hasAttr(this, name); };
FakeNode.prototype.removeAttribute = function (name) {
  delete this._attrs[name];
  if (name === "class") this._classes = [];
};
FakeNode.prototype.addEventListener = function (type, fn) {
  (this.listeners[type] || (this.listeners[type] = [])).push(fn);
};
FakeNode.prototype.removeEventListener = function (type, fn) {
  var list = this.listeners[type] || [];
  var i = list.indexOf(fn);
  if (i >= 0) list.splice(i, 1);
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
  var parts = String(selector).split(",").map(function (s) { return s.trim(); });
  var out = [];
  walk(this, function (candidate) {
    if (candidate === self) return;
    parts.forEach(function (part) {
      if (!matches(candidate, part)) return;
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
FakeNode.prototype.select = function () { return this; };
FakeNode.prototype.click = function () {
  this.dispatch("click", {
    target: this,
    preventDefault: function () {},
    stopPropagation: function () {},
  });
};
FakeNode.prototype.getBoundingClientRect = function () { return { top: 0, left: 0, bottom: 0, right: 0 }; };
FakeNode.prototype.scrollTo = function (options) { this._lastScrollTo = options; };

function makeDocument(readyState) {
  var doc = {
    readyState: readyState || "complete",
    body: new FakeNode("body", null),
    activeElement: null,
    _listeners: {},
    createElement: function (tag) { return new FakeNode(tag, doc); },
    createTextNode: function (text) {
      var node = new FakeNode("#text", doc);
      node._text = String(text);
      return node;
    },
    createRange: function () {
      return {
        startContainer: null,
        startOffset: 0,
        setStart: function (node, offset) { this.startContainer = node; this.startOffset = offset; },
        setStartAfter: function (node) {
          this.startContainer = node.parentNode;
          this.startOffset = this.startContainer ? this.startContainer.children.indexOf(node) + 1 : 0;
        },
        collapse: function () {},
      };
    },
    getElementById: function (id) {
      var found = null;
      walk(doc.body, function (node) {
        if (!found && node.getAttribute("id") === id) found = node;
      });
      return found;
    },
    querySelector: function (selector) { return doc.body.querySelector(selector); },
    addEventListener: function (type, fn) {
      (doc._listeners[type] || (doc._listeners[type] = [])).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = doc._listeners[type] || [];
      var i = list.indexOf(fn);
      if (i >= 0) list.splice(i, 1);
    },
    dispatch: function (type, event) {
      (doc._listeners[type] || []).slice().forEach(function (fn) { fn(event); });
    },
  };
  doc.body._ownerDoc = doc;
  return doc;
}

module.exports = { FakeNode: FakeNode, makeDocument: makeDocument, walk: walk };
