"use strict";

/* #2339 — schema-assisted automation authoring. Parameter definitions come
   from catalog action metadata already loaded in the provider map; the raw
   JSON box stays available as an advanced fallback. */
(function (namespace) {
    var fieldCounter = 0;

    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    function normalizeParams(action) {
        if (!action) {
            return [];
        }
        var raw = action.params;
        if (raw && typeof raw === "object" && !Array.isArray(raw) && raw.properties && typeof raw.properties === "object") {
            var required = Array.isArray(raw.required) ? raw.required : [];
            return Object.keys(raw.properties).map(function (name) {
                var def = raw.properties[name] && typeof raw.properties[name] === "object"
                    ? raw.properties[name]
                    : { type: raw.properties[name] };
                var example = def.example !== undefined
                    ? def.example
                    : (Array.isArray(def.examples) ? def.examples[0] : undefined);
                return {
                    name: name,
                    type: String(def.type || "string"),
                    required: required.indexOf(name) !== -1,
                    example: example,
                    description: namespace.text(def.description || def.help, "")
                };
            });
        }
        if (Array.isArray(raw)) {
            return raw.filter(function (param) { return param && typeof param === "object"; })
                .map(function (param) {
                    return {
                        name: namespace.text(param.name || param.key, ""),
                        type: String(param.type || "string"),
                        required: param.required === true,
                        example: param.example,
                        description: namespace.text(param.description || param.help, "")
                    };
                })
                .filter(function (param) { return !!param.name; });
        }
        return [];
    }

    function exampleText(param) {
        if (param.example === undefined || param.example === null) {
            return "";
        }
        if (typeof param.example === "object") {
            try { return JSON.stringify(param.example); } catch (error) { return ""; }
        }
        return String(param.example);
    }

    function inputMarkup(param) {
        fieldCounter += 1;
        var id = "integrations-automation-field-" + fieldCounter;
        var type = param.type.toLowerCase();
        var kind = ["number", "integer", "boolean", "json", "object", "array"].indexOf(type) !== -1 ? type : "string";
        var label = namespace.escapeHtml(param.name + (param.required ? " *" : ""));
        var placeholder = namespace.escapeHtml(exampleText(param));
        var required = param.required ? ' data-required="true"' : '';
        var control;
        if (kind === "boolean") {
            control = '<select id="' + id + '" data-schema-param="' + namespace.escapeHtml(param.name) + '" data-kind="boolean"' + required + '>' +
                '<option value=""></option><option value="true">true</option><option value="false">false</option></select>';
        } else if (kind === "json" || kind === "object" || kind === "array") {
            control = '<textarea id="' + id + '" rows="3" data-schema-param="' + namespace.escapeHtml(param.name) + '" ' +
                'data-kind="' + kind + '"' + required + ' placeholder="' + placeholder + '"></textarea>';
        } else {
            var inputType = kind === "number" || kind === "integer" ? "number" : "text";
            control = '<input id="' + id + '" type="' + inputType + '" data-schema-param="' + namespace.escapeHtml(param.name) + '" ' +
                'data-kind="' + kind + '"' + required + ' autocomplete="off" placeholder="' + placeholder + '">';
        }
        var help = param.description
            ? '<span class="integrations-field-help">' + namespace.escapeHtml(param.description) + '</span>'
            : '';
        return '<label class="integrations-run-field" for="' + id + '">' + label + control + help + '</label>';
    }

    namespace.renderAutomationSchema = function (form, action) {
        var container = form && form.querySelector("[data-automation-schema]");
        if (!container) {
            return;
        }
        var params = normalizeParams(action);
        container.innerHTML = params.length
            ? '<div class="integrations-automation-schema-grid">' + params.map(inputMarkup).join("") + '</div>'
            : "";
    };

    function invalid(field, kind) {
        return t("integrations-automation-param-invalid", "Check the value for { $field } ({ $type }).", {
            field: field, type: kind
        });
    }

    function parseTyped(input, kind, fieldName) {
        var value = input.value.trim();
        if (!value) {
            return { present: false };
        }
        if (kind === "boolean") {
            return { present: true, value: value === "true" };
        }
        if (kind === "number" || kind === "integer") {
            var numeric = Number(value);
            if (!Number.isFinite(numeric) || (kind === "integer" && !Number.isInteger(numeric))) {
                return { present: true, error: invalid(fieldName, kind) };
            }
            return { present: true, value: numeric };
        }
        if (kind === "json" || kind === "object" || kind === "array") {
            var parsed;
            try {
                parsed = JSON.parse(value);
            } catch (error) {
                return { present: true, error: invalid(fieldName, kind) };
            }
            if (kind === "object" && (!parsed || Array.isArray(parsed) || typeof parsed !== "object")) {
                return { present: true, error: invalid(fieldName, kind) };
            }
            if (kind === "array" && !Array.isArray(parsed)) {
                return { present: true, error: invalid(fieldName, kind) };
            }
            return { present: true, value: parsed };
        }
        return { present: true, value: value };
    }

    namespace.collectAutomationParams = function (form) {
        if (!form) {
            return { params: {}, error: "" };
        }
        var advanced = form.querySelector('[name="params"]');
        var raw = advanced ? String(advanced.value || "").trim() : "";
        if (raw) {
            var parsed;
            try {
                parsed = JSON.parse(raw);
            } catch (error) {
                return { params: {}, error: t("integrations-automation-json-invalid", "Parameters must be valid JSON.") };
            }
            if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
                return { params: {}, error: t("integrations-automation-json-invalid", "Parameters must be valid JSON.") };
            }
            return { params: parsed, error: "" };
        }
        var params = {};
        var inputs = Array.prototype.slice.call(form.querySelectorAll("[data-schema-param]"));
        for (var index = 0; index < inputs.length; index += 1) {
            var input = inputs[index];
            var name = input.dataset.schemaParam;
            var result = parseTyped(input, input.dataset.kind || "string", name);
            if (result.error) {
                return { params: {}, error: result.error };
            }
            if (!result.present) {
                if (input.dataset.required === "true") {
                    return {
                        params: {},
                        error: t("integrations-automation-param-required", "{ $field } is required.", { field: name })
                    };
                }
                continue;
            }
            params[name] = result.value;
        }
        return { params: params, error: "" };
    };

    function actionFor(provider, name) {
        if (!provider || !Array.isArray(provider.actions) || !name) {
            return null;
        }
        return provider.actions.find(function (item) { return item && item.name === name; }) || null;
    }

    function syncSchema(form) {
        if (!form) {
            return;
        }
        var app = form.closest("[data-integrations-app]");
        var state = app && namespace.getState(app);
        var providerSelect = form.querySelector("[data-automation-provider]");
        var actionSelect = form.querySelector("[data-automation-action]");
        var provider = providerSelect && state && state.providerMap ? state.providerMap.get(providerSelect.value) : null;
        namespace.renderAutomationSchema(form, actionFor(provider, actionSelect && actionSelect.value));
    }

    namespace.installAutomationSchema = function () {
        document.addEventListener("change", function (event) {
            var input = event.target;
            if (!input || !input.matches || !input.matches("[data-automation-provider], [data-automation-action]")) {
                return;
            }
            var form = input.closest("[data-automation-form]");
            if (form && !form.hidden) {
                syncSchema(form);
            }
        });
        document.addEventListener("click", function (event) {
            var button = event.target && event.target.closest
                ? event.target.closest('[data-action="new-automation"]')
                : null;
            if (!button) {
                return;
            }
            window.requestAnimationFrame(function () {
                var app = button.closest("[data-integrations-app]");
                if (app) {
                    syncSchema(app.querySelector("[data-automation-form]"));
                }
            });
        });
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
