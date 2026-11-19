"use strict";
/* Content controls (#2427): plain text, dropdown, date, checkbox and repeating
 * section. A control is a local inline element carrying data-* attributes; the
 * inspector edits those attributes. There is no DOCX w:sdt round-trip. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var C = (NS.controls = NS.controls || {});

  C.TYPES = [
    { id: "text", key: "docs-forms-control-type-text", fallback: "Plain text" },
    { id: "dropdown", key: "docs-forms-control-type-dropdown", fallback: "Dropdown list" },
    { id: "date", key: "docs-forms-control-type-date", fallback: "Date" },
    { id: "checkbox", key: "docs-forms-control-type-checkbox", fallback: "Checkbox" },
    { id: "repeat", key: "docs-forms-control-type-repeat", fallback: "Repeating section" }
  ];

  C.typeName = function (type) {
    for (var i = 0; i < C.TYPES.length; i += 1) {
      if (C.TYPES[i].id === type) return NS.tr(C.TYPES[i].key, C.TYPES[i].fallback);
    }
    return String(type || "");
  };

  function list(value) {
    if (Array.isArray(value)) return value.slice();
    if (typeof value === "string" && value) {
      return value.split(/[,\n]/).map(function (part) { return part.trim(); }).filter(Boolean);
    }
    return [];
  }

  C.create = function (type, options) {
    var opts = options || {};
    var id = opts.id || NS.uid("ctl");
    return {
      id: id,
      type: type || "text",
      label: typeof opts.label === "string" ? opts.label : C.typeName(type),
      options: list(opts.options),
      value: typeof opts.value === "string" ? opts.value : "",
      required: opts.required === true,
      placeholder: typeof opts.placeholder === "string" ? opts.placeholder : ""
    };
  };

  function attrs(descriptor) {
    return " data-gb-form-control=\"1\" data-form-id=\"" + NS.esc(descriptor.id) + "\"" +
      " data-form-type=\"" + NS.esc(descriptor.type) + "\"" +
      " data-form-label=\"" + NS.esc(descriptor.label) + "\"" +
      " data-form-options=\"" + NS.esc(descriptor.options.join(",")) + "\"" +
      " data-form-default=\"" + NS.esc(descriptor.value) + "\"" +
      " data-form-required=\"" + (descriptor.required ? "true" : "false") + "\"" +
      " data-form-placeholder=\"" + NS.esc(descriptor.placeholder) + "\"";
  }
  C.attrs = attrs;

  function display(descriptor) {
    if (descriptor.type === "checkbox") return descriptor.value === "true" ? "[x]" : "[ ]";
    if (descriptor.type === "repeat") return "\u2026";
    if (descriptor.value) return descriptor.value;
    return descriptor.placeholder || descriptor.options[0] || "\u2026";
  }

  C.render = function (descriptor) {
    return "<span class=\"gb-forms-control gb-forms-control-" + NS.esc(descriptor.type) + "\"" +
      attrs(descriptor) + "><span class=\"gb-forms-control-label\">" + NS.esc(descriptor.label) +
      "</span><span class=\"gb-forms-control-value\">" + NS.esc(display(descriptor)) + "</span></span>";
  };

  C.parseFromElement = function (element) {
    if (!element || !element.getAttribute) return null;
    var get = function (name) { return element.getAttribute(name) || ""; };
    return C.create(get("data-form-type"), {
      id: get("data-form-id") || NS.uid("ctl"),
      label: get("data-form-label"),
      options: get("data-form-options"),
      value: get("data-form-default"),
      required: get("data-form-required") === "true",
      placeholder: get("data-form-placeholder")
    });
  };

  C.validate = function (descriptor, value) {
    var current = value == null ? descriptor.value : value;
    var empty = current == null || String(current).trim() === "";
    if (descriptor.type === "checkbox") empty = current !== true && current !== "true";
    if (descriptor.required && empty) {
      return {
        ok: false,
        key: "docs-forms-control-missing-required",
        fallback: "Required field is empty: { $label }",
        params: { label: descriptor.label || descriptor.id }
      };
    }
    return { ok: true };
  };

  C.validateAll = function (descriptors, values) {
    var errors = [];
    (descriptors || []).forEach(function (descriptor) {
      var result = C.validate(descriptor, values ? values[descriptor.id] : descriptor.value);
      if (!result.ok) errors.push(result);
    });
    return errors;
  };

  C.syncFromDom = function (host) {
    if (!host || !host.querySelectorAll) return NS.state.controls;
    var nodes = host.querySelectorAll("[data-gb-form-control]");
    var found = [];
    Array.prototype.forEach.call(nodes, function (node) {
      var descriptor = C.parseFromElement(node);
      if (descriptor) found.push(descriptor);
    });
    NS.state.controls = found;
    return found;
  };

  C.insert = function (type, host) {
    var target = host || NS.getArticle();
    if (!target || !target.insertAdjacentHTML) return null;
    var descriptor = C.create(type, { label: C.typeName(type) });
    target.insertAdjacentHTML("beforeend", "<p>" + C.render(descriptor) + "</p>");
    C.syncFromDom(target);
    NS.persist();
    NS.statusKey("docs-forms-control-inserted", "Control inserted into the document");
    return descriptor;
  };

  C.updateInDom = function (id, patch, host) {
    var target = host || NS.getArticle();
    if (!target || !target.querySelectorAll) return false;
    var nodes = target.querySelectorAll("[data-gb-form-control]");
    for (var i = 0; i < nodes.length; i += 1) {
      if (nodes[i].getAttribute("data-form-id") !== id) continue;
      var merged = C.parseFromElement(nodes[i]);
      Object.keys(patch || {}).forEach(function (name) { merged[name] = patch[name]; });
      if (nodes[i].outerHTML != null) nodes[i].outerHTML = C.render(merged);
      C.syncFromDom(target);
      NS.persist();
      return true;
    }
    return false;
  };
})(window);
