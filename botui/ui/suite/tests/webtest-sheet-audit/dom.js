/**
 * Minimal DOM doubles for the shipped audit modules (#2491).
 * Only the element API the nine production IIFEs actually call is provided.
 * Sentinels are code-like tokens so the i18n literal scanner has no source
 * string to report. No dynamic i18n-key concatenation is used anywhere.
 */
"use strict";

function makeDataset() {
  return new Proxy({}, {
    set: function (target, key, value) { target[key] = String(value); return true; },
  });
}

function camel(name) {
  return name.replace(/-([a-z])/g, function (_, c) { return c.toUpperCase(); });
}

function attrValue(node, name) {
  if (name.indexOf("data-") === 0) return node.dataset[camel(name.slice(5))];
  if (name === "class") return node.className;
  if (name === "id") return node.id;
  if (name === "tabindex") return node.tabIndex;
  return node.attributes[name];
}

/* Minimal selector support for the exact selectors the shipped modules use:
 * .class, #id, tag, [attr], [attr="v"] and :not(...) — comma-separated. */
function matches(node, selector) {
  return String(selector || "").split(",").some(function (group) {
    var rest = group.trim();
    if (!rest) return false;
    if (rest.charAt(0) === "*") rest = rest.slice(1);
    var tag = /^[A-Za-z][\w-]*/.exec(rest);
    if (tag) {
      if (node.tagName !== tag[0].toUpperCase()) return false;
      rest = rest.slice(tag[0].length);
    }
    var guard = 0;
    while (rest.length && guard++ < 40) {
      var m;
      if ((m = /^\.([A-Za-z][\w-]*)/.exec(rest))) {
        if ((" " + node.className + " ").indexOf(" " + m[1] + " ") < 0) return false;
        rest = rest.slice(m[0].length);
      } else if ((m = /^#([\w-]+)/.exec(rest))) {
        if (node.id !== m[1]) return false;
        rest = rest.slice(m[0].length);
      } else if ((m = /^\[([^\]=]+)=(["'])([^\]]*?)\2\]/.exec(rest))) {
        if (String(attrValue(node, m[1])) !== m[3]) return false;
        rest = rest.slice(m[0].length);
      } else if ((m = /^\[([^\]]+)\]/.exec(rest))) {
        if (attrValue(node, m[1]) == null) return false;
        rest = rest.slice(m[0].length);
      } else if ((m = /^:not\(([^)]*)\)/.exec(rest))) {
        if (matches(node, m[1])) return false;
        rest = rest.slice(m[0].length);
      } else {
        return false;
      }
    }
    return true;
  });
}

function makeNode(tag) {
  var node = {
    tagName: String(tag || "<node>").toUpperCase(),
    className: "",
    id: "",
    type: "",
    value: "",
    title: "",
    tabIndex: 0,
    hidden: false,
    disabled: false,
    textContent: "",
    style: {},
    dataset: makeDataset(),
    attributes: {},
    children: [],
    listeners: {},
    isConnected: true,
    parentNode: null,
  };
  Object.defineProperty(node, "firstChild", {
    get: function () { return node.children[0] || null; },
  });
  Object.defineProperty(node, "offsetParent", {
    get: function () { return node.parentNode; },
  });
  Object.defineProperty(node, "classList", {
    get: function () {
      return {
        add: function (name) {
          if ((" " + node.className + " ").indexOf(" " + name + " ") < 0) {
            node.className = (node.className + " " + name).trim();
          }
        },
        remove: function (name) {
          node.className = node.className.split(/\s+/).filter(function (c) {
            return c && c !== name;
          }).join(" ");
        },
        contains: function (name) {
          return (" " + node.className + " ").indexOf(" " + name + " ") >= 0;
        },
      };
    },
  });
  node.appendChild = function (child) {
    child.parentNode = node;
    node.children.push(child);
    return child;
  };
  node.removeChild = function (child) {
    var i = node.children.indexOf(child);
    if (i >= 0) node.children.splice(i, 1);
    child.parentNode = null;
    return child;
  };
  node.setAttribute = function (name, value) { node.attributes[name] = String(value); };
  node.getAttribute = function (name) {
    return Object.prototype.hasOwnProperty.call(node.attributes, name)
      ? node.attributes[name] : null;
  };
  node.addEventListener = function (type, handler) {
    (node.listeners[type] || (node.listeners[type] = [])).push(handler);
  };
  node.dispatchEvent = function (event) {
    event.target = event.target || node;
    event.currentTarget = node;
    (node.listeners[event.type] || []).forEach(function (handler) { handler(event); });
    return true;
  };
  node.focus = function () { node.focused = true; };
  node.querySelectorAll = function (selector) {
    var out = [];
    (function walk(parent) {
      parent.children.forEach(function (child) { out.push(child); walk(child); });
    })(node);
    return out.filter(function (child) { return matches(child, selector); });
  };
  node.querySelector = function (selector) { return node.querySelectorAll(selector)[0] || null; };
  return node;
}

function makeFixedDate(fixedNow) {
  var RealDate = Date;
  function FixedDate() {
    if (!(this instanceof FixedDate)) return new RealDate(fixedNow).toString();
    if (arguments.length === 0) return new RealDate(fixedNow);
    return Reflect.construct(RealDate, Array.prototype.slice.call(arguments));
  }
  FixedDate.prototype = RealDate.prototype;
  FixedDate.now = function () { return fixedNow; };
  FixedDate.UTC = RealDate.UTC;
  FixedDate.parse = RealDate.parse;
  return FixedDate;
}

function makeStorage() {
  var map = {};
  return {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(map, key) ? map[key] : null;
    },
    setItem: function (key, value) { map[key] = String(value); },
    removeItem: function (key) { delete map[key]; },
    clear: function () { map = {}; },
    keys: function () { return Object.keys(map); },
  };
}

module.exports = {
  makeDataset: makeDataset, camel: camel, attrValue: attrValue,
  matches: matches, makeNode: makeNode,
  makeFixedDate: makeFixedDate, makeStorage: makeStorage,
};
