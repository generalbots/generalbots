/**
 * Deterministic, dependency-free DOM stub for the slides review-tools tests.
 * Supports only the selector shapes the eight shipped modules use.
 */
"use strict";

var DOC = null;

function dash(name) {
  return String(name).replace(/([A-Z])/g, "-$1").toLowerCase();
}

function El(tag) {
  var self = this;
  this.nodeType = 1;
  this.tagName = String(tag || "div").toUpperCase();
  this.nodeName = this.tagName;
  this.childNodes = [];
  this.parentNode = null;
  this._attrs = Object.create(null);
  this._text = "";
  this._listeners = Object.create(null);
  this._datasetTarget = Object.create(null);
  this.style = {
    setProperty: function (name, value) { self.style[name] = value; },
    removeProperty: function (name) { delete self.style[name]; },
    getPropertyValue: function (name) { return self.style[name] || ""; }
  };
  this.dataset = new Proxy(this._datasetTarget, {
    set: function (target, prop, value) {
      self.setAttribute("data-" + dash(prop), String(value));
      return true;
    }
  });
  Object.defineProperty(this, "textContent", {
    get: function () {
      var extra = self.childNodes.map(function (child) {
        return child.nodeType === 3 ? child.data : child.textContent;
      }).join("");
      return self._text + extra;
    },
    set: function (value) {
      self.childNodes.forEach(function (child) { child.parentNode = null; });
      self.childNodes = [];
      self._text = value == null ? "" : String(value);
    }
  });
  Object.defineProperty(this, "id", {
    get: function () { return self.getAttribute("id") || ""; },
    set: function (value) { self.setAttribute("id", value); }
  });
  Object.defineProperty(this, "className", {
    get: function () { return self.getAttribute("class") || ""; },
    set: function (value) { self.setAttribute("class", value); }
  });
  Object.defineProperty(this, "type", {
    get: function () { return self.getAttribute("type") || ""; },
    set: function (value) { self.setAttribute("type", value); }
  });
  Object.defineProperty(this, "children", {
    get: function () {
      return self.childNodes.filter(function (child) { return child.nodeType === 1; });
    }
  });
  Object.defineProperty(this, "firstChild", {
    get: function () { return self.childNodes[0] || null; }
  });
  Object.defineProperty(this, "firstElementChild", {
    get: function () {
      for (var i = 0; i < self.childNodes.length; i++) {
        if (self.childNodes[i].nodeType === 1) return self.childNodes[i];
      }
      return null;
    }
  });
  this.classList = {
    add: function (cls) {
      var list = self.className.split(/\s+/).filter(Boolean);
      if (list.indexOf(cls) < 0) list.push(cls);
      self.className = list.join(" ");
    },
    remove: function (cls) {
      self.className = self.className.split(/\s+/).filter(function (item) {
        return item && item !== cls;
      }).join(" ");
    },
    contains: function (cls) {
      return self.className.split(/\s+/).indexOf(cls) >= 0;
    },
    toggle: function (cls) {
      if (self.classList.contains(cls)) self.classList.remove(cls);
      else self.classList.add(cls);
    }
  };
}

El.prototype.getAttribute = function (name) {
  var key = String(name).toLowerCase();
  return Object.prototype.hasOwnProperty.call(this._attrs, key) ? this._attrs[key] : null;
};
El.prototype.setAttribute = function (name, value) {
  var key = String(name).toLowerCase();
  var text = String(value);
  this._attrs[key] = text;
  if (key.indexOf("data-") === 0) {
    var camel = key.slice(5).replace(/-([a-z])/g, function (_m, ch) {
      return ch.toUpperCase();
    });
    this._datasetTarget[camel] = text;
  }
};
El.prototype.hasAttribute = function (name) {
  return this.getAttribute(name) !== null;
};
El.prototype.removeAttribute = function (name) {
  var key = String(name).toLowerCase();
  delete this._attrs[key];
  if (key.indexOf("data-") === 0) {
    var camel = key.slice(5).replace(/-([a-z])/g, function (_m, ch) {
      return ch.toUpperCase();
    });
    delete this._datasetTarget[camel];
  }
};
El.prototype.appendChild = function (child) {
  if (child.parentNode) child.parentNode.removeChild(child);
  child.parentNode = this;
  this.childNodes.push(child);
  return child;
};
El.prototype.insertBefore = function (child, ref) {
  if (!ref) return this.appendChild(child);
  var at = this.childNodes.indexOf(ref);
  if (child.parentNode) child.parentNode.removeChild(child);
  child.parentNode = this;
  this.childNodes.splice(at < 0 ? this.childNodes.length : at, 0, child);
  return child;
};
El.prototype.removeChild = function (child) {
  var at = this.childNodes.indexOf(child);
  if (at >= 0) this.childNodes.splice(at, 1);
  child.parentNode = null;
  return child;
};
El.prototype.addEventListener = function (type, fn) {
  if (!this._listeners[type]) this._listeners[type] = [];
  this._listeners[type].push(fn);
};
El.prototype.removeEventListener = function (type, fn) {
  var list = this._listeners[type] || [];
  var at = list.indexOf(fn);
  if (at >= 0) list.splice(at, 1);
};
El.prototype.dispatchEvent = function (event) {
  var ev = event || {};
  if (!ev.type) return true;
  ev.target = ev.target || this;
  if (!ev.preventDefault) ev.preventDefault = function () { ev.defaultPrevented = true; };
  if (!ev.stopPropagation) ev.stopPropagation = function () { ev.stopped = true; };
  var node = this;
  while (node && !ev.stopped) {
    (node._listeners[ev.type] || []).slice().forEach(function (fn) {
      fn.call(node, ev);
    });
    node = ev.bubbles === false ? null : node.parentNode;
  }
  return !ev.defaultPrevented;
};
El.prototype.click = function () {
  return this.dispatchEvent({ type: "click", bubbles: true });
};
El.prototype.focus = function () {
  if (DOC) DOC.activeElement = this;
};
El.prototype.blur = function () {
  if (DOC && DOC.activeElement === this) DOC.activeElement = null;
};
El.prototype.select = function () {};
El.prototype.scrollIntoView = function () { this.scrolledIntoView = true; };

function attributeCondition(text, negate) {
  var m = /^([\w-]+)\s*(?:([~^$*]?=)\s*(?:"([^"]*)"|'([^']*)'))?$/.exec(text.trim());
  if (!m) return { kind: "attr", name: text, op: "", value: "", negate: negate };
  return {
    kind: "attr", name: m[1].toLowerCase(), op: m[2] || "",
    value: m[3] !== undefined ? m[3] : m[4], negate: negate
  };
}

function attributeMatches(el, cond) {
  var raw = el.getAttribute(cond.name);
  if (!cond.op) return raw !== null;
  if (raw === null) return false;
  if (cond.op === "=") return raw === cond.value;
  if (cond.op === "^=") return raw.indexOf(cond.value) === 0;
  if (cond.op === "$=") return raw.slice(-cond.value.length) === cond.value;
  return raw.indexOf(cond.value) >= 0;
}

function compound(token) {
  var out = { tag: "", conds: [] };
  var tag = /^[a-zA-Z][\w-]*/.exec(token);
  if (tag) { out.tag = tag[0].toUpperCase(); }
  var rest = token.slice(tag ? tag[0].length : 0);
  var re = /#([\w-]+)|\.([\w-]+)|\[([^\]]+)\]|:not\(([^)]+)\)/g;
  var match;
  while ((match = re.exec(rest))) {
    if (match[1]) out.conds.push({ kind: "id", value: match[1] });
    else if (match[2]) out.conds.push({ kind: "class", value: match[2] });
    else if (match[3]) out.conds.push(attributeCondition(match[3], false));
    else out.conds.push(attributeCondition(match[4].trim(), true));
  }
  return out;
}

function matchesCompound(el, spec) {
  if (spec.tag && el.tagName !== spec.tag) return false;
  return spec.conds.every(function (cond) {
    var ok;
    if (cond.kind === "id") ok = el.id === cond.value;
    else if (cond.kind === "class") ok = el.classList.contains(cond.value);
    else ok = attributeMatches(el, cond);
    return cond.negate ? !ok : ok;
  });
}

function parseSelector(selector) {
  return String(selector).split(",").map(function (group) {
    return group.trim().split(/\s+/).filter(Boolean).map(compound);
  });
}

function matchesPath(el, parts) {
  if (!matchesCompound(el, parts[parts.length - 1])) return false;
  var part = parts.length - 2;
  var node = el.parentNode;
  while (part >= 0) {
    var found = false;
    while (node && node.nodeType === 1) {
      if (matchesCompound(node, parts[part])) {
        found = true;
        node = node.parentNode;
        part--;
        break;
      }
      node = node.parentNode;
    }
    if (!found) return false;
  }
  return true;
}

El.prototype.matches = function (selector) {
  var self = this;
  return parseSelector(selector).some(function (parts) {
    return matchesPath(self, parts);
  });
};
El.prototype.querySelectorAll = function (selector) {
  var paths = parseSelector(selector);
  var found = [];
  function walk(node) {
    node.childNodes.forEach(function (child) {
      if (child.nodeType !== 1) return;
      if (paths.some(function (parts) { return matchesPath(child, parts); })) found.push(child);
      walk(child);
    });
  }
  walk(this);
  return found;
};
El.prototype.querySelector = function (selector) {
  return this.querySelectorAll(selector)[0] || null;
};
El.prototype.closest = function (selector) {
  var node = this;
  while (node && node.nodeType === 1) {
    if (node.matches(selector)) return node;
    node = node.parentNode;
  }
  return null;
};
El.prototype.contains = function (other) {
  var found = other === this;
  function walk(node) {
    node.childNodes.forEach(function (child) {
      if (child === other) found = true;
      else walk(child);
    });
  }
  if (!found) walk(this);
  return found;
};

DOC = new El("#document");
DOC.readyState = "loading";
DOC.body = new El("body");
DOC.appendChild(DOC.body);
DOC.createElement = function (tag) { return new El(tag); };
DOC.getElementById = function (id) { return DOC.querySelector("#" + id); };
DOC.activeElement = null;

module.exports = { El: El, document: DOC };
