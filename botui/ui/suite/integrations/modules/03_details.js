"use strict";

(function (namespace) {
    function isAwsProvider(provider) {
        var id = provider.id.toLowerCase();
        var name = provider.name.toLowerCase();
        return id === "aws" || id.indexOf("aws-") === 0 || id === "amazon-web-services" ||
            name === "aws" || name.indexOf("aws ") === 0 || name === "amazon web services";
    }

    function collectGuidance(value, output) {
        if (typeof value === "string" && value.trim()) {
            output.push(value.trim());
        } else if (Array.isArray(value)) {
            value.forEach(function (entry) { collectGuidance(entry, output); });
        } else if (value && typeof value === "object") {
            Object.keys(value).forEach(function (key) { collectGuidance(value[key], output); });
        }
        return output;
    }

    function awsFallbackFields() {
        return [
            { name: "access_key_id", label: "Access Key ID", required: true, secret: false },
            { name: "secret_access_key", label: "Secret Access Key", required: true, secret: true },
            { name: "session_token", label: "Session Token", required: false, secret: true },
            { name: "region", label: "Region", required: true, secret: false }
        ];
    }

    function fieldSource(provider) {
        var fields = provider.auth.fields;
        if (Array.isArray(fields)) {
            return fields;
        }
        if (fields && typeof fields === "object") {
            return Object.keys(fields).map(function (key) {
                var field = fields[key];
                return field && typeof field === "object" ? Object.assign({ name: key }, field) : { name: key, type: field };
            });
        }
        return isAwsProvider(provider) ? awsFallbackFields() : [];
    }

    function normalizeField(field, provider) {
        var source = typeof field === "string" ? { name: field } :
            (field && typeof field === "object" ? field : {});
        var key = namespace.text(source.name || source.key || source.id, "credential");
        var awsLabels = {
            "access_key_id": "Access Key ID",
            "aws_access_key_id": "Access Key ID",
            "secret_access_key": "Secret Access Key",
            "aws_secret_access_key": "Secret Access Key",
            "session_token": "Session Token",
            "aws_session_token": "Session Token",
            "region": "Region",
            "aws_region": "Region"
        };
        var isAws = isAwsProvider(provider);
        var label = namespace.text(source.label, isAws && awsLabels[key.toLowerCase()] ? awsLabels[key.toLowerCase()] : namespace.titleCase(key));
        var secret = source.secret === true || source.sensitive === true || source.is_secret === true ||
            String(source.type || "").toLowerCase() === "password" ||
            /(secret|password|token|private[_-]?key|api[_-]?key)/i.test(key);
        var sessionToken = isAws && ["session_token", "aws_session_token"].indexOf(key.toLowerCase()) !== -1;
        var required = typeof source.required === "boolean" ? source.required :
            (typeof source.optional === "boolean" ? !source.optional : !sessionToken);
        return {
            key: key,
            label: label,
            secret: secret,
            required: required,
            description: namespace.text(source.description || source.help || source.instructions, "")
        };
    }

    function renderFields(provider) {
        var fields = fieldSource(provider).map(function (field) { return normalizeField(field, provider); });
        if (!fields.length) {
            return '<p class="integrations-readonly-note">No credential fields are declared for this provider.</p>';
        }
        return '<div class="integrations-field-list">' + fields.map(function (field) {
            var type = field.secret ? "password" : "text";
            var metadata = (field.secret ? "<span>Secret</span>" : "<span>Configuration</span>") +
                '<span>' + (field.required ? "Required" : "Optional") + '</span>';
            var placeholder = field.secret ? "Secret value is never displayed" : "Configured by secure adapter";
            return '<label class="integrations-field"><span class="integrations-field-label"><span>' + namespace.escapeHtml(field.label) + '</span>' +
                '<span class="integrations-field-meta">' + metadata + '</span></span>' +
                '<input type="' + type + '" disabled autocomplete="' + (field.secret ? "new-password" : "off") + '" ' +
                'placeholder="' + namespace.escapeHtml(placeholder) + '" aria-label="' + namespace.escapeHtml(field.label + ", " + (field.secret ? "secret, " : "") + (field.required ? "required" : "optional")) + '">' +
                (field.description ? '<span class="integrations-field-help">' + namespace.escapeHtml(field.description) + '</span>' : '') + '</label>';
        }).join("") + '</div>';
    }

    function renderGuidance(value, fallback) {
        var items = collectGuidance(value, []);
        if (!items.length && fallback) {
            items.push(fallback);
        }
        if (!items.length) {
            return '<p class="integrations-readonly-note">No guidance was published for this provider.</p>';
        }
        return '<ul class="integrations-guidance">' + items.map(function (item) {
            return '<li>' + namespace.escapeHtml(item) + '</li>';
        }).join("") + '</ul>';
    }

    function normalizeParams(params) {
        if (params && typeof params === "object" && !Array.isArray(params) && params.properties && typeof params.properties === "object") {
            var requiredNames = Array.isArray(params.required) ? params.required : [];
            return Object.keys(params.properties).map(function (key) {
                var definition = params.properties[key];
                var normalized = definition && typeof definition === "object" ? Object.assign({ name: key }, definition) : { name: key, type: definition };
                normalized.required = requiredNames.indexOf(key) !== -1;
                return normalized;
            });
        }
        if (Array.isArray(params)) {
            return params.map(function (param) {
                return typeof param === "string" ? { name: param } :
                    (param && typeof param === "object" ? param : {});
            });
        }
        if (params && typeof params === "object") {
            return Object.keys(params).map(function (key) {
                var param = params[key];
                return param && typeof param === "object" ? Object.assign({ name: key }, param) : { name: key, type: param };
            });
        }
        return [];
    }

    function actionPill(label, className) {
        return '<span class="integrations-pill' + (className ? " " + className : "") + '">' + namespace.escapeHtml(label) + '</span>';
    }

    function renderActions(provider) {
        if (!provider.actions.length) {
            return '<p class="integrations-readonly-note">No actions are declared for this provider.</p>';
        }
        return '<div class="integrations-action-list">' + provider.actions.map(function (action) {
            var verb = namespace.text(action.verb, "ACTION").toUpperCase();
            var title = namespace.text(action.label || action.name, verb);
            var risk = namespace.titleCase(action.risk, "Unspecified risk");
            var params = normalizeParams(action.params);
            var surfaces = Array.isArray(action.surfaces) ? action.surfaces : [];
            var badges = actionPill(risk, "") +
                actionPill(action.requires_approval === true ? "Approval required" : "No approval", "") +
                actionPill(action.implemented === true ? "Implemented" : "Planned", action.implemented === true ? "is-must" : "");
            var detailParts = params.map(function (param) {
                var name = namespace.text(param.name || param.key, "parameter");
                var type = namespace.text(param.type, "value");
                var required = param.required === true ? ", required" : "";
                return actionPill(name + ": " + type + required, "");
            }).join("") + surfaces.map(function (surface) {
                return actionPill("Surface: " + namespace.titleCase(surface), "");
            }).join("");
            return '<article class="integrations-action"><div class="integrations-action-head">' +
                '<div class="integrations-action-title"><span class="integrations-action-verb">' + namespace.escapeHtml(verb) + '</span>' +
                '<strong>' + namespace.escapeHtml(title) + '</strong></div><div class="integrations-action-badges">' + badges + '</div></div>' +
                (action.summary ? '<p>' + namespace.escapeHtml(action.summary) + '</p>' : '') +
                (detailParts ? '<details class="integrations-action-details"><summary>Parameters and surfaces</summary><div class="integrations-param-list">' + detailParts + '</div></details>' : '') +
                '</article>';
        }).join("") + '</div>';
    }

    function providerMark(provider) {
        var name = namespace.text(provider.name, "IN");
        var mark = name.split(/\s+/).slice(0, 2).map(function (part) { return part.charAt(0); }).join("").toUpperCase();
        return '<span class="integrations-provider-mark" aria-hidden="true">' + namespace.escapeHtml(mark) + '</span>';
    }

    namespace.renderDetails = function (root) {
        var state = namespace.getState(root);
        var provider = state.providerMap.get(state.selectedProviderId);
        var body = root.querySelector("[data-detail-body]");
        if (!provider) {
            body.innerHTML = '<div class="integrations-state is-error"><span class="integrations-state-mark">!</span><h3>Provider unavailable</h3><p>This provider is no longer present in the catalog.</p></div>';
            return;
        }
        root.querySelector("[data-detail-title]").textContent = provider.name;
        var isAws = isAwsProvider(provider);
        var leastPrivilegeFallback = isAws ?
            "Use a dedicated least-privilege IAM principal with only the actions required by this provider. Prefer temporary session credentials and rotate long-lived keys." : "";
        var leastPrivilege = collectGuidance(provider.auth.least_privilege, []);
        if (isAws && !leastPrivilege.some(function (item) { return /\bIAM\b/i.test(item); })) {
            leastPrivilege.push(leastPrivilegeFallback);
        }
        var docsUrl = namespace.safeUrl(provider.official_docs);
        var adapterTitle = provider.llm_available ? "Available to assistants" : "Configuration requires adapter";
        var adapterCopy = provider.llm_available ?
            "The language model can use implemented actions. Credential setup remains read-only until a secure save endpoint is documented." :
            "Planned setup only. This control plane does not submit or persist credentials.";
        var warning = state.detailStatus === "error" ? '<div class="integrations-inline-warning"><strong>Latest detail could not be loaded</strong>' +
            '<p>Showing the provider information already available from the catalog.</p><button class="integrations-button" type="button" data-action="retry-detail">Try detail again</button></div>' :
            (state.detailStatus === "loading" ? '<p class="integrations-readonly-note" role="status">Refreshing provider detail...</p>' : '');
        body.innerHTML = '<div class="integrations-detail-summary">' + providerMark(provider) + '<div><h3>' + namespace.escapeHtml(provider.name) + '</h3>' +
            '<p>' + namespace.escapeHtml(provider.strategy) + '</p></div></div>' +
            '<dl class="integrations-detail-meta"><div><dt>Category</dt><dd>' + namespace.escapeHtml(namespace.titleCase(provider.category)) + '</dd></div>' +
            '<div><dt>Priority</dt><dd>' + namespace.escapeHtml(namespace.titleCase(provider.priority)) + '</dd></div>' +
            '<div><dt>Module</dt><dd>' + namespace.escapeHtml(provider.module) + '</dd></div></dl>' + warning +
            '<div class="integrations-adapter-state"><strong>' + namespace.escapeHtml(adapterTitle) + '</strong><p>' + namespace.escapeHtml(adapterCopy) + '</p></div>' +
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Authentication</h4>' +
            '<span>' + namespace.escapeHtml(namespace.titleCase(provider.auth.method, "Not specified")) + '</span></div>' + renderFields(provider) + '</section>' +
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Setup guidance</h4><span>Read only</span></div>' +
            renderGuidance(provider.auth.instructions, "") + '</section>' +
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Least-privilege guidance</h4><span>Recommended</span></div>' +
            renderGuidance(leastPrivilege, leastPrivilegeFallback) + '</section>' +
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Actions</h4><span>' + provider.action_count + ' declared</span></div>' +
            renderActions(provider) + '</section>' +
            '<section class="integrations-detail-section">' +
            (docsUrl ? '<a class="integrations-docs-link" href="' + namespace.escapeHtml(docsUrl) + '" target="_blank" rel="noopener noreferrer">Open official documentation</a>' :
                '<p class="integrations-readonly-note">Official documentation was not provided for this catalog entry.</p>') + '</section>';
    };

    namespace.openDetails = function (root, providerId) {
        var state = namespace.getState(root);
        var provider = state.providerMap.get(providerId);
        if (!provider) {
            return;
        }
        state.exploreTouched = true;
        state.selectedProviderId = providerId;
        state.detailStatus = "loading";
        state.detailRequest += 1;
        var requestId = state.detailRequest;
        var layer = root.querySelector("[data-detail-layer]");
        var panel = root.querySelector("[data-detail-panel]");
        if (layer.hidden) {
            state.lastFocus = root.contains(document.activeElement) ? document.activeElement : null;
        }
        layer.hidden = false;
        layer.setAttribute("aria-hidden", "false");
        namespace.renderDetails(root);
        window.requestAnimationFrame(function () { panel.focus(); });
        namespace.fetchJson(namespace.catalogUrl + "/" + encodeURIComponent(providerId)).then(function (payload) {
            if (requestId !== state.detailRequest || state.selectedProviderId !== providerId) {
                return;
            }
            var detail = payload.provider && typeof payload.provider === "object" ? payload.provider : payload;
            var normalized = namespace.normalizeProvider(detail);
            var index = state.providers.findIndex(function (item) { return item.id === providerId; });
            normalized.id = providerId;
            if (index !== -1) {
                state.providers[index] = normalized;
            }
            state.providerMap.set(providerId, normalized);
            state.detailStatus = "ready";
            namespace.renderDetails(root);
        }).catch(function () {
            if (requestId === state.detailRequest && state.selectedProviderId === providerId) {
                state.detailStatus = "error";
                namespace.renderDetails(root);
            }
        });
    };

    namespace.closeDetails = function (root) {
        var state = namespace.getState(root);
        var layer = root.querySelector("[data-detail-layer]");
        state.detailRequest += 1;
        state.selectedProviderId = "";
        state.detailStatus = "idle";
        layer.hidden = true;
        layer.setAttribute("aria-hidden", "true");
        if (state.lastFocus && state.lastFocus.isConnected) {
            state.lastFocus.focus();
        }
    };

    namespace.trapDetailFocus = function (root, event) {
        var layer = root.querySelector("[data-detail-layer]");
        if (layer.hidden) {
            return;
        }
        if (event.key === "Escape") {
            event.preventDefault();
            namespace.closeDetails(root);
            return;
        }
        if (event.key !== "Tab") {
            return;
        }
        var focusable = Array.from(root.querySelectorAll("[data-detail-panel] button:not([disabled]), [data-detail-panel] a[href], [data-detail-panel] summary, [data-detail-panel][tabindex]"));
        if (!focusable.length) {
            event.preventDefault();
            return;
        }
        var first = focusable[0];
        var last = focusable[focusable.length - 1];
        if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first.focus();
        }
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
