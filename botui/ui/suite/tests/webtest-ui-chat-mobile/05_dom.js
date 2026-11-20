/**
 * Minimal DOM for the mobile-chat vm sandbox (05).
 *
 * Implements exactly the surface the four shipped chat modules touch: node
 * identity/connection, classes, data-* attributes, a cssText-capable style,
 * class/id/attribute selectors, listener dispatch, flex-gap geometry and
 * scrollTop. It is a stub, not a browser: no layout engine, no reflow timing.
 */
"use strict";

var vm = require("vm");
var h = require("./00_harness");

function makeStyle() {
  var props = {};
  var api = {
    setProperty: function (key, value) { props[key] = String(value); },
    getPropertyValue: function (key) { return props[key] || ""; },
  };
  Object.defineProperty(api, "cssText", {
    get: function () {
      return Object.keys(props).map(function (k) { return k + ":" + props[k]; }).join(";");
    },
    set: function (value) {
      String(value).split(";").forEach(function (decl) {
        var at = decl.indexOf(":");
        if (at < 0) return;
        props[decl.slice(0, at).trim()] = decl.slice(at + 1).trim();
      });
    },
  });
  return new Proxy(api, {
    get: function (target, key) {
      if (key in target) return target[key];
      return props[key] || "";
    },
    set: function (target, key, value) {
      if (key === "cssText") { target.cssText = value; return true; }
      props[key] = String(value);
      return true;
    },
  });
}

function makeClassList(node) {
  return {
    contains: function (name) { return !!node._classes[name]; },
    add: function () {
      for (var i = 0; i < arguments.length; i++) node._classes[arguments[i]] = true;
    },
    remove: function () {
      for (var i = 0; i < arguments.length; i++) delete node._classes[arguments[i]];
    },
    toggle: function (name) {
      if (node._classes[name]) { delete node._classes[name]; return false; }
      node._classes[name] = true;
      return true;
    },
  };
}

function camel(name) {
  return name.replace(/^data-/, "").replace(/-([a-z])/g, function (m, c) {
    return c.toUpperCase();
  });
}

function GNode(tag, engine, nodeType) {
  this.nodeType = nodeType || 1;
  this.tagName = String(tag || "div").toUpperCase();
  this._engine = engine;
  this.childNodes = [];
  this.parentNode = null;
  this._attrs = {};
  this._classes = {};
  this._listeners = {};
  this._id = "";
  this._h = 0;
  this._isScrollContainer = false;
  this.style = makeStyle();
  this.dataset = {};
  this.classList = makeClassList(this);
  this.textContent = "";
  this.className = "";
}
Object.defineProperty(GNode.prototype, "id", {
  get: function () { return this._id; },
  set: function (value) { this._id = String(value); },
});
Object.defineProperty(GNode.prototype, "className", {
  get: function () { return Object.keys(this._classes).join(" "); },
  set: function (value) {
    this._classes = {};
    String(value).split(/\s+/).forEach(function (name) {
      if (name) this._classes[name] = true;
    }, this);
  },
});
Object.defineProperty(GNode.prototype, "offsetHeight", {
  get: function () {
    if (this._classes["gb-vplaceholder"]) {
      var parsed = parseInt(this.style.height, 10);
      return parsed > 0 ? parsed : 24;
    }
    return this._h;
  },
});
Object.defineProperty(GNode.prototype, "isConnected", {
  get: function () {
    var node = this;
    while (node) {
      if (node.nodeType === 9) return true;
      node = node.parentNode;
    }
    return false;
  },
});
Object.defineProperty(GNode.prototype, "clientHeight", {
  get: function () { return this._clientHeight || 0; },
  set: function (value) { this._clientHeight = value; },
});
Object.defineProperty(GNode.prototype, "scrollTop", {
  get: function () { return this._scrollTop || 0; },
  set: function (value) { this._scrollTop = value; },
});
Object.defineProperty(GNode.prototype, "scrollHeight", {
  get: function () {
    var gap = this._engine ? this._engine.gap : 16;
    return this.childNodes.reduce(function (sum, child) {
      return sum + child.offsetHeight + gap;
    }, 0);
  },
});
Object.defineProperty(GNode.prototype, "scrollWidth", { get: function () { return 390; } });
Object.defineProperty(GNode.prototype, "innerHTML", {
  get: function () { return this._innerHTML || ""; },
  set: function (value) {
    this._innerHTML = String(value);
    this.childNodes.forEach(function (child) { child.parentNode = null; });
    this.childNodes = [];
  },
});

GNode.prototype.setAttribute = function (name, value) {
  var text = String(value);
  this._attrs[name] = text;
  if (name.indexOf("data-") === 0) this.dataset[camel(name)] = text;
};
GNode.prototype.getAttribute = function (name) {
  return Object.prototype.hasOwnProperty.call(this._attrs, name) ? this._attrs[name] : null;
};
GNode.prototype.hasAttribute = function (name) {
  return Object.prototype.hasOwnProperty.call(this._attrs, name);
};
GNode.prototype.removeAttribute = function (name) {
  delete this._attrs[name];
  if (name.indexOf("data-") === 0) delete this.dataset[camel(name)];
};
GNode.prototype.contains = function (other) {
  var node = other;
  while (node) {
    if (node === this) return true;
    node = node.parentNode;
  }
  return false;
};
GNode.prototype.appendChild = function (child) {
  child.parentNode = this;
  this.childNodes.push(child);
  return child;
};
GNode.prototype.insertBefore = function (child, before) {
  var at = before ? this.childNodes.indexOf(before) : -1;
  if (at < 0) return this.appendChild(child);
  child.parentNode = this;
  this.childNodes.splice(at, 0, child);
  return child;
};
GNode.prototype.replaceChild = function (fresh, old) {
  var at = this.childNodes.indexOf(old);
  if (at < 0) return old;
  this.childNodes[at] = fresh;
  fresh.parentNode = this;
  old.parentNode = null;
  return old;
};
GNode.prototype.removeChild = function (child) {
  var at = this.childNodes.indexOf(child);
  if (at < 0) return child;
  this.childNodes.splice(at, 1);
  child.parentNode = null;
  return child;
};
GNode.prototype.remove = function () {
  if (this.parentNode) this.parentNode.removeChild(this);
};
GNode.prototype.addEventListener = function (type, fn) {
  (this._listeners[type] || (this._listeners[type] = [])).push(fn);
};
GNode.prototype.removeEventListener = function (type, fn) {
  var list = this._listeners[type] || [];
  var at = list.indexOf(fn);
  if (at >= 0) list.splice(at, 1);
};
GNode.prototype.dispatch = function (type, detail) {
  var engine = this._engine;
  var event = new engine.sandbox.Event(type, { detail: detail });
  event.target = this;
  (this._listeners[type] || []).slice().forEach(function (fn) { fn(event); });
  return event;
};
GNode.prototype.listenerTypes = function () { return Object.keys(this._listeners).sort(); };
GNode.prototype.scrollTo = function (options) {
  if (options && typeof options.top === "number" && options.behavior !== "smooth") {
    this._scrollTop = options.top;
  }
  this._scrollCalls = this._scrollCalls || [];
  this._scrollCalls.push(options || {});
};
GNode.prototype.getBoundingClientRect = function () {
  if (this._isScrollContainer) {
    return { top: 100, bottom: 100 + this.clientHeight, left: 0, right: 400,
      width: 400, height: this.clientHeight };
  }
  var offset = 0;
  var node = this;
  var host = null;
  while (node.parentNode && node.parentNode.nodeType === 1) {
    var siblings = node.parentNode.childNodes;
    var at = siblings.indexOf(node);
    for (var i = 0; i < at; i++) {
      offset += siblings[i].offsetHeight + (node._engine ? node._engine.gap : 16);
    }
    node = node.parentNode;
    if (node._isScrollContainer) { host = node; break; }
  }
  var base = host ? host.getBoundingClientRect().top - host.scrollTop : 100;
  var top = base + offset;
  return { top: top, bottom: top + this.offsetHeight, left: 0, right: 400,
    width: 400, height: this.offsetHeight };
};

function descendants(root) {
  var out = [];
  (function walk(parent) {
    parent.childNodes.forEach(function (child) {
      out.push(child);
      walk(child);
    });
  })(root);
  return out;
}
function matchesSimple(node, token) {
  while (token) {
    var match = /^(\[[^\]]+\]|[.#]?[\w-]+)/.exec(token);
    if (!match) return false;
    var part = match[1];
    token = token.slice(part.length);
    if (part.charAt(0) === ".") {
      if (!node.classList.contains(part.slice(1))) return false;
    } else if (part.charAt(0) === "#") {
      if (node._id !== part.slice(1)) return false;
    } else if (part.charAt(0) === "[") {
      var attr = /^\[([\w-]+)(?:[~^$*]?=\s*['"]?([^\]'"]*)['"]?)?\]$/.exec(part);
      if (!attr || !node.hasAttribute(attr[1])) return false;
      if (attr[2] !== undefined && node.getAttribute(attr[1]) !== attr[2]) return false;
    } else if (node.tagName !== part.toUpperCase()) {
      return false;
    }
  }
  return true;
}
GNode.prototype.querySelectorAll = function (selector) {
  var text = selector.trim();
  var direct = false;
  if (/^:scope\s*>\s*/.test(text)) {
    direct = true;
    text = text.replace(/^:scope\s*>\s*/, "");
  }
  var parts = text.split(/\s+/);
  var roots = [this];
  var result = [];
  parts.forEach(function (part, index) {
    var next = [];
    roots.forEach(function (root) {
      var pool = index === 0 && direct ? root.childNodes : descendants(root);
      pool.forEach(function (node) {
        if (matchesSimple(node, part) && next.indexOf(node) < 0) next.push(node);
      });
    });
    roots = next;
  });
  return roots;
};
GNode.prototype.querySelector = function (selector) {
  var found = this.querySelectorAll(selector);
  return found.length ? found[0] : null;
};

function buildDom(engine) {
  var doc = new GNode("#document", engine, 9);
  doc.readyState = "complete";
  doc.documentElement = new GNode("html", engine);
  doc.documentElement.parentNode = doc;
  doc.childNodes.push(doc.documentElement);
  doc.createElement = function (tag) { return new GNode(tag, engine); };
  doc.createDocumentFragment = function () { return new GNode("#fragment", engine, 11); };
  doc.getElementById = function (id) {
    return descendants(doc).filter(function (node) { return node._id === id; })[0] || null;
  };

  var messages = new GNode("div", engine);
  messages.id = "messages";
  messages._isScrollContainer = true;
  messages.clientHeight = 800;
  messages.scrollTop = 0;
  doc.documentElement.appendChild(messages);

  var chatApp = new GNode("div", engine);
  chatApp.id = "chat-app";
  doc.documentElement.appendChild(chatApp);

  var input = new GNode("textarea", engine);
  input.id = "messageInput";
  input._h = 43;
  doc.documentElement.appendChild(input);

  ["sendBtn", "scrollToBottom", "gbChatStatus"].forEach(function (id) {
    var node = new GNode("button", engine);
    node.id = id;
    doc.documentElement.appendChild(node);
  });

  engine.sandbox.document = doc;
  engine.window.document = doc;
  engine.dom = {
    document: doc,
    messages: messages,
    chatApp: chatApp,
    input: input,
    byId: function (id) {
      return descendants(doc).filter(function (node) { return node._id === id; })[0] || null;
    },
  };
  return engine.dom;
}

function makeMessage(engine, index, height, kind) {
  var el = new GNode("div", engine);
  el.className = "message " + (kind || "bot");
  el.id = "msg-" + index;
  el._h = height === undefined ? 60 : height;
  var content = new GNode("div", engine);
  content.className = "message-content";
  el.appendChild(content);
  return el;
}

function loadModules(engine) {
  h.MODULES.forEach(function (rel) {
    vm.runInContext(h.readFile(rel), engine.context, { filename: rel });
  });
}

function fresh() {
  var engine = h.createEngine();
  buildDom(engine);
  loadModules(engine);
  return engine;
}

var active = fresh();

exports.fresh = fresh;
exports.active = function () { return active; };
exports.buildDom = buildDom;
exports.loadModules = loadModules;
exports.makeMessage = makeMessage;
exports.GNode = GNode;
exports.descendants = descendants;
