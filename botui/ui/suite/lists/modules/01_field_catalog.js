"use strict";
/* #2537 - static, local field catalog for structured audience segments.
   Wire contract (shared with the backend resolver):
     Segment = { match: "all" | "any", groups: Node[] }
     Node    = Condition | Group
     Condition = { field: <id below>, op: <op id below>, value: string|number }
     Group     = { match: "all" | "any", groups: Node[] }   // recursive, max depth 3
   Field ids and operator ids are stable API values; only labels are localized.
   Consent fields are per-channel: consent_email, consent_sms, consent_push,
   consent_whatsapp with values opted_in | opted_out | unknown. */
(function (global) {
  var ns = global.GBSegments;
  if (!ns) return;

  ns.MAX_DEPTH = 3;

  ns.OPERATORS = {
    eq: { key: "segments-op-eq", label: "is" },
    ne: { key: "segments-op-ne", label: "is not" },
    contains: { key: "segments-op-contains", label: "contains" },
    not_contains: { key: "segments-op-not-contains", label: "does not contain" },
    in: { key: "segments-op-in", label: "is one of" },
    not_in: { key: "segments-op-not-in", label: "is not one of" },
    gt: { key: "segments-op-gt", label: "is greater than" },
    gte: { key: "segments-op-gte", label: "is at least" },
    lt: { key: "segments-op-lt", label: "is less than" },
    lte: { key: "segments-op-lte", label: "is at most" },
    before: { key: "segments-op-before", label: "is before" },
    after: { key: "segments-op-after", label: "is after" },
    within_days: { key: "segments-op-within-days", label: "is within the last N days" }
  };

  var CONSENT_VALUES = [
    { value: "opted_in", key: "segments-value-opted-in", label: "Opted in" },
    { value: "opted_out", key: "segments-value-opted-out", label: "Opted out" },
    { value: "unknown", key: "segments-value-unknown", label: "Unknown" }
  ];

  var LIFECYCLE_VALUES = [
    { value: "lead", key: "segments-value-lead", label: "Lead" },
    { value: "prospect", key: "segments-value-prospect", label: "Prospect" },
    { value: "customer", key: "segments-value-customer", label: "Customer" },
    { value: "archived", key: "segments-value-archived", label: "Archived" }
  ];

  var STAGE_VALUES = [
    { value: "opportunity", key: "segments-value-opportunity", label: "Opportunity" },
    { value: "won", key: "segments-value-won", label: "Won" },
    { value: "lost", key: "segments-value-lost", label: "Lost" }
  ];

  ns.GROUPS = [
    { id: "contact", key: "segments-group-contact", label: "Contact fields" },
    { id: "tags", key: "segments-group-tags", label: "Tags" },
    { id: "owner", key: "segments-group-owner", label: "Owner" },
    { id: "lifecycle", key: "segments-group-lifecycle", label: "Lifecycle status" },
    { id: "deal", key: "segments-group-deal", label: "Deal" },
    { id: "activity", key: "segments-group-activity", label: "Activity" },
    { id: "consent", key: "segments-group-consent", label: "Consent per channel" },
    { id: "lists", key: "segments-group-lists", label: "List membership" }
  ];

  ns.FIELDS = [
    { id: "first_name", key: "segments-field-first-name", label: "First name", group: "contact", type: "text", ops: ["eq", "ne", "contains", "not_contains"] },
    { id: "last_name", key: "segments-field-last-name", label: "Last name", group: "contact", type: "text", ops: ["eq", "ne", "contains", "not_contains"] },
    { id: "email", key: "segments-field-email", label: "Email", group: "contact", type: "text", ops: ["eq", "ne", "contains", "not_contains"] },
    { id: "company", key: "segments-field-company", label: "Company", group: "contact", type: "text", ops: ["eq", "ne", "contains", "not_contains"] },
    { id: "city", key: "segments-field-city", label: "City", group: "contact", type: "text", ops: ["eq", "ne", "contains", "not_contains"] },
    { id: "country", key: "segments-field-country", label: "Country", group: "contact", type: "text", ops: ["eq", "ne", "contains", "not_contains"] },
    { id: "tags", key: "segments-field-tags", label: "Tags", group: "tags", type: "text", ops: ["contains", "not_contains"] },
    { id: "owner", key: "segments-field-owner", label: "Owner", group: "owner", type: "text", ops: ["eq", "ne"] },
    { id: "lifecycle_status", key: "segments-field-lifecycle-status", label: "Lifecycle status", group: "lifecycle", type: "select", ops: ["eq", "ne", "in", "not_in"], options: LIFECYCLE_VALUES },
    { id: "deal_value", key: "segments-field-deal-value", label: "Deal value", group: "deal", type: "number", ops: ["eq", "ne", "gt", "gte", "lt", "lte"] },
    { id: "deal_stage", key: "segments-field-deal-stage", label: "Deal stage", group: "deal", type: "select", ops: ["eq", "ne", "in", "not_in"], options: STAGE_VALUES },
    { id: "last_activity", key: "segments-field-last-activity", label: "Last activity", group: "activity", type: "date", ops: ["before", "after", "within_days"] },
    { id: "consent_email", key: "segments-field-consent-email", label: "Email consent", group: "consent", type: "select", ops: ["eq", "ne"], options: CONSENT_VALUES, consentChannel: "email" },
    { id: "consent_sms", key: "segments-field-consent-sms", label: "SMS consent", group: "consent", type: "select", ops: ["eq", "ne"], options: CONSENT_VALUES, consentChannel: "sms" },
    { id: "consent_push", key: "segments-field-consent-push", label: "Push consent", group: "consent", type: "select", ops: ["eq", "ne"], options: CONSENT_VALUES, consentChannel: "push" },
    { id: "consent_whatsapp", key: "segments-field-consent-whatsapp", label: "WhatsApp consent", group: "consent", type: "select", ops: ["eq", "ne"], options: CONSENT_VALUES, consentChannel: "whatsapp" },
    { id: "list_membership", key: "segments-field-list-membership", label: "List membership", group: "lists", type: "text", ops: ["in", "not_in"] }
  ];

  var fieldMap = {};
  ns.FIELDS.forEach(function (field) { fieldMap[field.id] = field; });

  ns.fieldById = function (id) { return fieldMap[id] || null; };

  ns.operatorLabel = function (op) {
    var def = ns.OPERATORS[op];
    return def ? ns.t(def.key, def.label) : op;
  };

  ns.fieldLabel = function (id) {
    var def = ns.fieldById(id);
    return def ? ns.t(def.key, def.label) : id;
  };

  ns.valueLabel = function (fieldId, value) {
    var def = ns.fieldById(fieldId);
    if (!def || !def.options) return String(value == null ? "" : value);
    var option = def.options.find(function (item) { return item.value === value; });
    return option ? ns.t(option.key, option.label) : String(value);
  };
})(window);
