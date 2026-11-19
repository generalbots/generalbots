"use strict";
/* #2537 - structured segment model: creation, normalization, validation and
   immutable-by-convention path mutation. No free-text parsing is performed
   here: the only accepted input is the documented JSON shape. */
(function (global) {
  var ns = global.GBSegments;
  if (!ns) return;

  ns.newCondition = function () { return { field: "", op: "", value: "" }; };

  ns.newGroup = function (match) {
    return { match: match === "any" ? "any" : "all", groups: [] };
  };

  ns.emptyModel = function () { return ns.newGroup("all"); };

  ns.isGroup = function (node) {
    return !!node && Array.isArray(node.groups);
  };

  // Defensive clone: unknown properties and malformed children are dropped so
  // a hand-edited payload can never inject DOM attributes or oversized trees.
  ns.normalize = function (value, depth) {
    var level = depth || 0;
    if (!ns.isGroup(value) || level > ns.MAX_DEPTH + 1) return ns.emptyModel();
    var out = ns.newGroup(value.match);
    var children = value.groups.slice(0, 50);
    for (var i = 0; i < children.length; i++) {
      var child = children[i];
      if (ns.isGroup(child)) {
        out.groups.push(ns.normalize(child, level + 1));
      } else if (child && typeof child === "object") {
        out.groups.push({
          field: typeof child.field === "string" ? child.field : "",
          op: typeof child.op === "string" ? child.op : "",
          value: child.value == null ? "" : String(child.value)
        });
      }
    }
    return out;
  };

  ns.fromJson = function (text) {
    if (!text) return null;
    try {
      var parsed = JSON.parse(text);
      return ns.isGroup(parsed) ? ns.normalize(parsed) : null;
    } catch (e) {
      return null;
    }
  };

  ns.toJson = function (model) { return JSON.stringify(ns.normalize(model)); };

  function error(path, key, fallback) {
    return { path: path.slice(), key: key, fallback: fallback };
  }

  function validateCondition(condition, path, errors) {
    var field = ns.fieldById(condition.field);
    if (!field) {
      errors.push(error(path, "segments-validate-invalid-field", "Choose a valid field."));
      return;
    }
    if (!field.ops.includes(condition.op)) {
      errors.push(error(path, "segments-validate-invalid-operator", "Choose a valid operator for this field."));
      return;
    }
    var value = String(condition.value == null ? "" : condition.value).trim();
    if (!value) {
      errors.push(error(path, "segments-validate-empty-value", "Enter a value for this condition."));
      return;
    }
    if (field.type === "number" && !isFinite(Number(value))) {
      errors.push(error(path, "segments-validate-number", "Enter a valid number."));
      return;
    }
    if (field.id === "last_activity" && condition.op === "within_days") {
      var days = Number(value);
      if (!isFinite(days) || days <= 0 || Math.floor(days) !== days) {
        errors.push(error(path, "segments-validate-days", "Enter a positive number of days."));
      }
      return;
    }
    if (field.options && ["in", "not_in"].includes(condition.op)) {
      var bad = value.split(",").some(function (part) {
        var needle = part.trim();
        return !field.options.some(function (option) { return option.value === needle; });
      });
      if (bad) errors.push(error(path, "segments-validate-invalid-field", "Choose a valid field."));
    }
  }

  function walk(node, path, depth, errors) {
    if (!ns.isGroup(node)) return;
    if (depth > ns.MAX_DEPTH) {
      errors.push(error(path, "segments-validate-nesting-depth", "Groups can be nested up to three levels."));
      return;
    }
    if (!node.groups.length) {
      errors.push(error(path, "segments-validate-empty-group", "A group must contain at least one condition."));
      return;
    }
    node.groups.forEach(function (child, index) {
      var childPath = path.concat(index);
      if (ns.isGroup(child)) walk(child, childPath, depth + 1, errors);
      else validateCondition(child, childPath, errors);
    });
  }

  ns.validate = function (model) {
    var errors = [];
    walk(ns.normalize(model), [], 1, errors);
    return errors;
  };

  ns.nodeAt = function (root, path) {
    var node = root;
    for (var i = 0; i < path.length; i++) {
      if (!ns.isGroup(node)) return null;
      node = node.groups[path[i]];
    }
    return node || null;
  };

  ns.replaceAt = function (root, path, replacement) {
    if (!path.length) return false;
    var parent = ns.nodeAt(root, path.slice(0, -1));
    if (!ns.isGroup(parent)) return false;
    parent.groups[path[path.length - 1]] = replacement;
    return true;
  };

  ns.removeAt = function (root, path) {
    if (!path.length) return false;
    var parent = ns.nodeAt(root, path.slice(0, -1));
    if (!ns.isGroup(parent)) return false;
    parent.groups.splice(path[path.length - 1], 1);
    return true;
  };

  ns.addTo = function (root, groupPath, node) {
    var group = groupPath.length ? ns.nodeAt(root, groupPath) : root;
    if (!ns.isGroup(group)) return false;
    group.groups.push(node);
    return true;
  };

  ns.countConditions = function (node) {
    if (!ns.isGroup(node)) return 1;
    return node.groups.reduce(function (sum, child) {
      return sum + (ns.isGroup(child) ? ns.countConditions(child) : 1);
    }, 0);
  };

  ns.consentChannels = function (node) {
    var channels = [];
    if (!ns.isGroup(node)) return channels;
    node.groups.forEach(function (child) {
      if (ns.isGroup(child)) {
        ns.consentChannels(child).forEach(function (c) {
          if (channels.indexOf(c) === -1) channels.push(c);
        });
        return;
      }
      var field = ns.fieldById(child.field);
      if (field && field.consentChannel && channels.indexOf(field.consentChannel) === -1) {
        channels.push(field.consentChannel);
      }
    });
    return channels;
  };
})(window);
