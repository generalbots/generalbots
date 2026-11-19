"use strict";
/* #2537 - clearly-labelled OFFLINE preview. These are synthetic contacts used
   only to exercise the builder UI when the server preview endpoint is absent.
   The UI must display the sample-data badge next to every number from here. */
(function (global) {
  var ns = global.GBSegments;
  if (!ns) return;

  function daysAgo(n) {
    var d = new Date();
    d.setDate(d.getDate() - n);
    return d.toISOString().slice(0, 10);
  }

  ns.SAMPLE_CONTACTS = [
    { first_name: "Ada", last_name: "Lovelace", email: "ada@example.test", company: "Analytical Engines", city: "London", country: "UK", tags: ["vip", "newsletter"], owner: "alice", lifecycle_status: "customer", deal_value: 12000, deal_stage: "won", last_activity: daysAgo(2), consent: { email: "opted_in", sms: "opted_out", push: "unknown", whatsapp: "opted_in" }, lists: "prospects,vip" },
    { first_name: "Grace", last_name: "Hopper", email: "grace@example.test", company: "Navy Labs", city: "New York", country: "US", tags: ["vip"], owner: "bob", lifecycle_status: "prospect", deal_value: 8500, deal_stage: "opportunity", last_activity: daysAgo(9), consent: { email: "opted_in", sms: "opted_in", push: "opted_in", whatsapp: "opted_in" }, lists: "prospects" },
    { first_name: "Alan", last_name: "Turing", email: "alan@example.test", company: "Bletchley", city: "London", country: "UK", tags: ["press"], owner: "alice", lifecycle_status: "lead", deal_value: 0, deal_stage: "opportunity", last_activity: daysAgo(41), consent: { email: "opted_out", sms: "unknown", push: "unknown", whatsapp: "unknown" }, lists: "newsletter" },
    { first_name: "Katherine", last_name: "Johnson", email: "katherine@example.test", company: "NASA", city: "Hampton", country: "US", tags: ["vip", "research"], owner: "carol", lifecycle_status: "customer", deal_value: 45000, deal_stage: "won", last_activity: daysAgo(1), consent: { email: "opted_in", sms: "opted_in", push: "opted_out", whatsapp: "opted_in" }, lists: "vip,customers" },
    { first_name: "Linus", last_name: "Torvalds", email: "linus@example.test", company: "Open Source", city: "Portland", country: "US", tags: ["dev"], owner: "bob", lifecycle_status: "lead", deal_value: 500, deal_stage: "opportunity", last_activity: daysAgo(120), consent: { email: "unknown", sms: "unknown", push: "unknown", whatsapp: "unknown" }, lists: "newsletter" },
    { first_name: "Margaret", last_name: "Hamilton", email: "margaret@example.test", company: "Apollo", city: "Cambridge", country: "US", tags: ["vip", "research"], owner: "alice", lifecycle_status: "customer", deal_value: 78000, deal_stage: "won", last_activity: daysAgo(6), consent: { email: "opted_in", sms: "unknown", push: "opted_in", whatsapp: "opted_in" }, lists: "customers" },
    { first_name: "Edsger", last_name: "Dijkstra", email: "edsger@example.test", company: "TU Delft", city: "Rotterdam", country: "NL", tags: ["research"], owner: "carol", lifecycle_status: "prospect", deal_value: 3200, deal_stage: "lost", last_activity: daysAgo(75), consent: { email: "opted_in", sms: "opted_out", push: "unknown", whatsapp: "unknown" }, lists: "prospects" },
    { first_name: "Barbara", last_name: "Liskov", email: "barbara@example.test", company: "MIT", city: "Boston", country: "US", tags: ["dev", "vip"], owner: "alice", lifecycle_status: "customer", deal_value: 21000, deal_stage: "won", last_activity: daysAgo(14), consent: { email: "opted_in", sms: "opted_in", push: "unknown", whatsapp: "opted_out" }, lists: "customers,vip" },
    { first_name: "Donald", last_name: "Knuth", email: "donald@example.test", company: "Stanford", city: "Palo Alto", country: "US", tags: ["research", "dev"], owner: "bob", lifecycle_status: "lead", deal_value: 1500, deal_stage: "opportunity", last_activity: daysAgo(3), consent: { email: "opted_in", sms: "unknown", push: "unknown", whatsapp: "unknown" }, lists: "newsletter" },
    { first_name: "Radia", last_name: "Perlman", email: "radia@example.test", company: "Networks Inc", city: "Seattle", country: "US", tags: ["dev"], owner: "carol", lifecycle_status: "prospect", deal_value: 9900, deal_stage: "opportunity", last_activity: daysAgo(22), consent: { email: "opted_out", sms: "opted_out", push: "opted_out", whatsapp: "opted_out" }, lists: "prospects" },
    { first_name: "Tim", last_name: "Berners-Lee", email: "tim@example.test", company: "CERN", city: "Geneva", country: "CH", tags: ["press", "vip"], owner: "alice", lifecycle_status: "customer", deal_value: 56000, deal_stage: "won", last_activity: daysAgo(5), consent: { email: "opted_in", sms: "opted_in", push: "opted_in", whatsapp: "opted_in" }, lists: "vip" },
    { first_name: "Frances", last_name: "Allen", email: "frances@example.test", company: "IBM", city: "Yorktown", country: "US", tags: ["research"], owner: "bob", lifecycle_status: "archived", deal_value: 700, deal_stage: "lost", last_activity: daysAgo(210), consent: { email: "unknown", sms: "opted_out", push: "unknown", whatsapp: "unknown" }, lists: "old" }
  ];

  function fieldValue(contact, condition) {
    var field = ns.fieldById(condition.field);
    if (!field) return undefined;
    if (field.consentChannel) return contact.consent ? contact.consent[field.consentChannel] : "unknown";
    if (field.id === "tags") return contact.tags || [];
    if (field.id === "list_membership") return contact.lists || "";
    return contact[condition.field];
  }

  function asList(value) {
    return String(value == null ? "" : value).split(",").map(function (part) {
      return part.trim().toLowerCase();
    }).filter(Boolean);
  }

  function evaluateCondition(condition, contact) {
    var raw = fieldValue(contact, condition);
    var target = String(condition.value == null ? "" : condition.value).trim().toLowerCase();
    var field = ns.fieldById(condition.field);
    if (field && field.id === "tags") {
      var tags = (raw || []).map(function (t) { return String(t).toLowerCase(); });
      var wanted = target;
      return condition.op === "not_contains" ? tags.indexOf(wanted) === -1 : tags.indexOf(wanted) !== -1;
    }
    if (["in", "not_in"].includes(condition.op)) {
      var found = asList(raw).indexOf(target) !== -1;
      return condition.op === "in" ? found : !found;
    }
    if (condition.op === "within_days") {
      var stamp = Date.parse(String(raw));
      if (!isFinite(stamp)) return false;
      var age = (Date.now() - stamp) / 86400000;
      return age >= 0 && age <= Number(condition.value);
    }
    if (["before", "after"].includes(condition.op)) {
      var a = Date.parse(String(raw));
      var b = Date.parse(String(condition.value));
      if (!isFinite(a) || !isFinite(b)) return false;
      return condition.op === "before" ? a < b : a > b;
    }
    if (["gt", "gte", "lt", "lte"].includes(condition.op)) {
      var left = Number(raw);
      var right = Number(condition.value);
      if (!isFinite(left) || !isFinite(right)) return false;
      if (condition.op === "gt") return left > right;
      if (condition.op === "gte") return left >= right;
      if (condition.op === "lt") return left < right;
      return left <= right;
    }
    var text = String(raw == null ? "" : raw).toLowerCase();
    if (condition.op === "contains") return text.indexOf(target) !== -1;
    if (condition.op === "not_contains") return text.indexOf(target) === -1;
    if (condition.op === "ne") return text !== target;
    return text === target;
  }

  function isConsentCondition(node) {
    if (ns.isGroup(node) || !node) return false;
    var field = ns.fieldById(node.field);
    return !!(field && field.consentChannel);
  }

  // Consent conditions do not *select* recipients here; they define the
  // channels being checked. Anyone not opted in to one of them is suppressed.
  function evaluateEligibility(node, contact) {
    if (!ns.isGroup(node)) return true;
    if (node.match === "any") {
      return node.groups.some(function (child) {
        return isConsentCondition(child) || evaluateNode(child, contact);
      });
    }
    return node.groups.every(function (child) {
      return isConsentCondition(child) || evaluateNode(child, contact);
    });
  }

  function evaluateNode(node, contact) {
    if (!ns.isGroup(node)) return evaluateCondition(node, contact);
    if (!node.groups.length) return false;
    var results = node.groups.map(function (child) {
      return ns.isGroup(child) ? evaluateEligibility(child, contact) : evaluateCondition(child, contact);
    });
    return node.match === "any" ? results.some(Boolean) : results.every(Boolean);
  }

  ns.evaluateSampleNode = evaluateNode;

  ns.offlinePreview = function (model) {
    var root = ns.normalize(model);
    var channels = ns.consentChannels(root);
    var eligible = ns.SAMPLE_CONTACTS.filter(function (contact) {
      return evaluateEligibility(root, contact);
    });
    var suppressed = eligible.filter(function (contact) {
      return channels.some(function (channel) {
        return !contact.consent || contact.consent[channel] !== "opted_in";
      });
    });
    var suppressedSet = new Set(suppressed);
    var items = eligible.filter(function (contact) { return !suppressedSet.has(contact); })
      .slice(0, 10)
      .map(function (contact) {
        return { name: contact.first_name + " " + contact.last_name, email: contact.email, status: contact.lifecycle_status };
      });
    return { source: "sample", matched: eligible.length, suppressed: suppressed.length, items: items };
  };
})(window);
