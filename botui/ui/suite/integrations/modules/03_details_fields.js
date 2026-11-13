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

    // #1854: the catalog ships auth.setup_steps as an ordered credential
    // walkthrough. Render it as a numbered list so every provider shows a
    // concrete 1-2-3 path instead of one prose paragraph.
    function renderSetupSteps(value, fallback) {
        var items = collectGuidance(value, []);
        if (!items.length && fallback) {
            items.push(fallback);
        }
        if (!items.length) {
            return '<p class="integrations-readonly-note">No setup steps were published for this provider.</p>';
        }
        return '<ol class="integrations-guidance integrations-setup-steps">' + items.map(function (item) {
            return '<li>' + namespace.escapeHtml(item) + '</li>';
        }).join("") + '</ol>';
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
            var runButton = '<button class="integrations-button integrations-button-subtle integrations-run-button" type="button" data-run-action="' +
                namespace.escapeHtml(action.name) + '" data-provider-slug="' + namespace.escapeHtml(provider.id) + '"' +
                (action.implemented === true ? '' : ' data-planned="true"') + '>Run</button>';
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
                '<strong>' + namespace.escapeHtml(title) + '</strong></div><div class="integrations-action-badges">' + badges + runButton + '</div></div>' +
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

    function authKindFor(provider) {
        var method = String(provider.auth.method || "").toLowerCase();
        return namespace.allowedAuthKinds.indexOf(method) !== -1 ? method : "";
    }

    function connectInputs(provider, uid) {
        return fieldSource(provider).map(function (field) { return normalizeField(field, provider); })
            .map(function (field, index) {
                var type = field.secret ? "password" : "text";
                var inputId = uid + "-connect-" + index;
                var errorId = inputId + "-error";
                var metadata = (field.secret ? "<span>Secret</span>" : "<span>Configuration</span>") +
                    '<span>' + (field.required ? "Required" : "Optional") + '</span>';
                var placeholder = field.secret ? "Stored in the secrets vault" : "Value stored with the connection";
                return '<label class="integrations-field"><span class="integrations-field-label"><span>' + namespace.escapeHtml(field.label) + '</span>' +
                    '<span class="integrations-field-meta">' + metadata + '</span></span>' +
                    '<input type="' + type + '" id="' + namespace.escapeHtml(inputId) + '" name="' + namespace.escapeHtml(field.key) + '"' +
                    (field.required ? " required" : "") + ' autocomplete="' + (field.secret ? "new-password" : "off") + '"' +
                    ' placeholder="' + namespace.escapeHtml(placeholder) + '"' +
                    ' aria-label="' + namespace.escapeHtml(field.label + ", " + (field.secret ? "secret, " : "") + (field.required ? "required" : "optional")) + '"' +
                    ' aria-describedby="' + namespace.escapeHtml(errorId) + '"' +
                    ' data-connect-input="true">' +
                    '<span class="integrations-field-error" id="' + namespace.escapeHtml(errorId) + '" data-field-error="' + namespace.escapeHtml(field.key) + '" hidden></span>' +
                    (field.description ? '<span class="integrations-field-help">' + namespace.escapeHtml(field.description) + '</span>' : '') + '</label>';
            }).join("");
    }

    function renderConnectSection(provider, uid) {
        var authKind = authKindFor(provider);
        var botId = (namespace.context && namespace.context.botId) || "";
        if (authKind === "oauth2") {
            if (!botId) return null;
            var startUrl = "/api/bots/" + encodeURIComponent(botId) +
                "/integrations/oauth/" + encodeURIComponent(provider.id) + "/start?return_to=/integrations";
            var origin = (window.location && window.location.origin) || "";
            var redirectUri = origin + "/api/integrations/oauth/" + encodeURIComponent(provider.id) + "/callback";
            return '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Connect securely</h4>' +
                '<span>OAuth 2.0</span></div>' +
                '<p class="integrations-connect-help">Authorization happens on the provider consent screen; the exchanged token is written to the server-side vault and never displayed again.</p>' +
                '<a class="integrations-button" data-oauth-start href="' + namespace.escapeHtml(startUrl) + '">Authorize ' +
                namespace.escapeHtml(provider.name || provider.id) + '</a>' +
                '<details class="integrations-oauth-setup"><summary>App configuration (only without a managed app)</summary>' +
                '<p class="integrations-connect-help">A packaged desktop ships a managed public client, so Authorize works as-is. Self-hosted installations register one app at the provider console with this redirect URI and paste its client id here; no client secret is needed because the flow uses PKCE.</p>' +
                '<p class="integrations-connect-help"><code data-oauth-redirect>' + namespace.escapeHtml(redirectUri) + '</code></p>' +
                '<form class="integrations-connect-form" data-oauth-client-form>' +
                '<label class="integrations-field"><span class="integrations-field-label"><span>Client ID</span></span>' +
                '<input type="text" name="oauth_client_id" required autocomplete="off" aria-label="OAuth client id"></label>' +
                '<label class="integrations-field"><span class="integrations-field-label"><span>Client secret</span>' +
                '<span class="integrations-field-meta"><span>Optional for public clients</span></span></span>' +
                '<input type="password" name="oauth_client_secret" autocomplete="new-password" aria-label="OAuth client secret"></label>' +
                '<p class="integrations-inline-progress" data-oauth-progress hidden role="status">Saving client…</p>' +
                '<p class="integrations-inline-error" data-oauth-error hidden role="alert"></p>' +
                '<p class="integrations-connect-success" data-oauth-success hidden role="status"></p>' +
                '<div class="integrations-connect-actions"><button class="integrations-button" type="submit" data-action="save-oauth-client">Save app configuration</button></div>' +
                '</form></details>' +
                '</section>';
        }
        if (!authKind || !connectInputs(provider, uid)) {
            return null;
        }
        return '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Connect securely</h4>' +
            '<span>Vault-backed</span></div>' +
            '<form class="integrations-connect-form" data-connect-form novalidate>' +
            '<label class="integrations-connect-field"><span>Visibility</span>' +
            '<select name="visibility" data-visibility>' +
            '<option value="private">Only me (private)</option>' +
            '<option value="branch">Everyone in this branch workspace</option>' +
            '</select></label>'+
            '<p class="integrations-connect-help">Values travel once over an authenticated request; secret keys are written to the server-side vault and never displayed again.</p>' +
            '<label class="integrations-field"><span class="integrations-field-label"><span>Display name</span>' +
            '<span class="integrations-field-meta"><span>Required</span></span></span>' +
            '<input type="text" id="' + namespace.escapeHtml(uid + "-connect-display-name") + '" name="display_name" required maxlength="255" autocomplete="off" value="' +
            namespace.escapeHtml(provider.name + " connection") + '" aria-label="Display name, required" aria-describedby="' + namespace.escapeHtml(uid + "-connect-display-name-error") + '" data-connect-input="true">' +
            '<span class="integrations-field-error" id="' + namespace.escapeHtml(uid + "-connect-display-name-error") + '" data-field-error="display_name" hidden></span></label>' +
            connectInputs(provider, uid) +
            '<p class="integrations-inline-progress" data-connect-progress hidden role="status">Storing credentials...</p>' +
            '<p class="integrations-inline-error" data-connect-error hidden role="alert"></p>' +
            '<p class="integrations-connect-success" data-connect-success hidden role="status"></p>' +
            '<div class="integrations-connect-actions">' +
            '<button class="integrations-button" type="submit" data-action="connect-provider">Connect</button>' +
            '</div></form></section>';
    }

    function collectFieldValues(provider, form, secrets, configuration) {
        fieldSource(provider).forEach(function (rawField) {
            var field = normalizeField(rawField, provider);
            var input = form.elements[field.key];
            if (!input) {
                return;
            }
            var value = String(input.value || "").trim();
            if (!value) {
                return;
            }
            if (field.secret) {
                secrets[field.key] = String(input.value);
            } else {
                configuration[field.key] = String(input.value);
            }
        });
    }

    function clearFieldErrors(form) {
        Array.prototype.forEach.call(form.querySelectorAll("[data-field-error]"), function (node) {
            node.textContent = "";
            node.hidden = true;
        });
        Array.prototype.forEach.call(form.querySelectorAll("input[aria-invalid], select[aria-invalid], textarea[aria-invalid]"), function (input) {
            input.removeAttribute("aria-invalid");
        });
    }

    function setFieldError(form, fieldKey, message) {
        var errorNode = form.querySelector('[data-field-error="' + fieldKey.replace(/"/g, '\\"') + '"]');
        var input = form.elements[fieldKey];
        if (errorNode) {
            errorNode.textContent = message;
            errorNode.hidden = !message;
        }
        if (input && input.setAttribute) {
            if (message) {
                input.setAttribute("aria-invalid", "true");
            } else {
                input.removeAttribute("aria-invalid");
            }
        }
    }

    function requiredFieldLabel(provider, fieldKey) {
        if (fieldKey === "display_name") {
            return "Display name";
        }
        var match = fieldSource(provider).map(function (field) { return normalizeField(field, provider); })
            .find(function (field) { return field.key === fieldKey; });
        return match ? match.label : namespace.titleCase(fieldKey);
    }

    function validateConnectForm(provider, form) {
        clearFieldErrors(form);
        var missing = [];
        var displayInput = form.elements.display_name;
        if (displayInput && displayInput.hasAttribute("required") && !String(displayInput.value || "").trim()) {
            missing.push("display_name");
            setFieldError(form, "display_name", "Display name is required.");
        }
        fieldSource(provider).forEach(function (rawField) {
            var field = normalizeField(rawField, provider);
            var input = form.elements[field.key];
            if (!input || !field.required) {
                return;
            }
            if (!String(input.value || "").trim()) {
                missing.push(field.key);
                setFieldError(form, field.key, requiredFieldLabel(provider, field.key) + " is required.");
            }
        });
        if (missing.length && typeof form.checkValidity === "function") {
            form.checkValidity();
        }
        return { valid: missing.length === 0, missing: missing };
    }

    function clearFormInputs(form, options) {
        var opts = options || {};
        var secretsOnly = opts.secretsOnly === true;
        Array.prototype.forEach.call(form.querySelectorAll("input"), function (input) {
            if (input.type === "hidden" || input.type === "submit" || input.type === "button") {
                return;
            }
            if (secretsOnly && input.type !== "password") {
                return;
            }
            input.value = "";
        });
        if (!secretsOnly && typeof form.reset === "function") {
            form.reset();
        }
        if (secretsOnly) {
            // Keep non-secret values and validation state consistent after clearing secrets.
            clearFieldErrors(form);
        }
    }

    function connectErrorMessage(error, context) {
        var status = error && typeof error.status === "number" ? error.status : 0;
        if (error && (error.code === "CSRF_MISSING" || /CSRF token/i.test(String(error.message || "")))) {
            return "Security token missing. Reload this page and try again.";
        }
        if (status === 409) {
            return "An active connection for this provider already exists.";
        }
        if (status === 401 || status === 403) {
            return "Your session could not be authorized for this workspace. Sign in again or ask an administrator for access.";
        }
        if (status === 503) {
            return "Credential vault unavailable. Nothing was stored; try again shortly.";
        }
        if (status === 400) {
            if (context && context.fieldName) {
                return context.fieldName + " was rejected by the control plane. Review it and try again.";
            }
            if (context && Array.isArray(context.missingFields) && context.missingFields.length) {
                return "Missing required fields: " + context.missingFields.join(", ") + ".";
            }
            return "Some values were rejected by the control plane. Review the highlighted fields and try again.";
        }
        return "Connection could not be created. The control plane may be unreachable; try again.";
    }

    namespace.detailsHelpers = {
        isAwsProvider: isAwsProvider,
        collectGuidance: collectGuidance,
        renderFields: renderFields,
        renderGuidance: renderGuidance,
        renderSetupSteps: renderSetupSteps,
        renderActions: renderActions,
        renderConnectSection: renderConnectSection,
        providerMark: providerMark,
        authKindFor: authKindFor,
        collectFieldValues: collectFieldValues,
        clearFieldErrors: clearFieldErrors,
        validateConnectForm: validateConnectForm,
        requiredFieldLabel: requiredFieldLabel,
        clearFormInputs: clearFormInputs,
        connectErrorMessage: connectErrorMessage,
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
