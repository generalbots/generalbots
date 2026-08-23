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
            var runButton = action.implemented === true ?
                '<button class="integrations-button integrations-button-subtle integrations-run-button" type="button" data-run-action="' +
                namespace.escapeHtml(action.name) + '" data-provider-slug="' + namespace.escapeHtml(provider.id) + '">Run</button>' : "";
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
                var metadata = (field.secret ? "<span>Secret</span>" : "<span>Configuration</span>") +
                    '<span>' + (field.required ? "Required" : "Optional") + '</span>';
                var placeholder = field.secret ? "Stored in the secrets vault" : "Value stored with the connection";
                return '<label class="integrations-field"><span class="integrations-field-label"><span>' + namespace.escapeHtml(field.label) + '</span>' +
                    '<span class="integrations-field-meta">' + metadata + '</span></span>' +
                    '<input type="' + type + '" id="' + namespace.escapeHtml(inputId) + '" name="' + namespace.escapeHtml(field.key) + '"' +
                    (field.required ? " required" : "") + ' autocomplete="' + (field.secret ? "new-password" : "off") + '"' +
                    ' placeholder="' + namespace.escapeHtml(placeholder) + '"' +
                    ' aria-label="' + namespace.escapeHtml(field.label + ", " + (field.secret ? "secret, " : "") + (field.required ? "required" : "optional")) + '">' +
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
            return '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Connect securely</h4>' +
                '<span>OAuth 2.0</span></div>' +
                '<p class="integrations-connect-help">Authorization happens on the provider consent screen; the exchanged token is written to the server-side vault and never displayed again.</p>' +
                '<a class="integrations-button" data-oauth-start href="' + namespace.escapeHtml(startUrl) + '">Authorize ' +
                namespace.escapeHtml(provider.name || provider.id) + '</a>' +
                '</section>';
        }
        if (!authKind || !connectInputs(provider, uid)) {
            return null;
        }
        return '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Connect securely</h4>' +
            '<span>Vault-backed</span></div>' +
            '<form class="integrations-connect-form" data-connect-form novalidate>' +
            '<p class="integrations-connect-help">Values travel once over an authenticated request; secret keys are written to the server-side vault and never displayed again.</p>' +
            '<label class="integrations-field"><span class="integrations-field-label"><span>Display name</span>' +
            '<span class="integrations-field-meta"><span>Required</span></span></span>' +
            '<input type="text" name="display_name" required maxlength="255" autocomplete="off" value="' +
            namespace.escapeHtml(provider.name + " connection") + '" aria-label="Display name, required"></label>' +
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
            if (!input || !String(input.value).trim()) {
                return;
            }
            if (field.secret) {
                secrets[field.key] = String(input.value);
            } else {
                configuration[field.key] = String(input.value);
            }
        });
    }

    function clearFormInputs(form) {
        Array.prototype.forEach.call(form.querySelectorAll("input"), function (input) {
            input.value = "";
        });
        form.reset();
    }

    function connectErrorMessage(error) {
        var status = error && typeof error.status === "number" ? error.status : 0;
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
            return "Some values were rejected by the control plane. Review the fields and try again.";
        }
        return "Connection could not be created. The control plane may be unreachable; try again.";
    }

    namespace.submitConnectForm = async function (root, form) {
        var state = namespace.getState(root);
        var provider = state.providerMap.get(state.selectedProviderId);
        if (!provider || !provider.llm_available || !form.matches("[data-connect-form]")) {
            return;
        }
        var progress = form.querySelector("[data-connect-progress]");
        var errorLine = form.querySelector("[data-connect-error]");
        var successLine = form.querySelector("[data-connect-success]");
        var submitButton = form.querySelector('button[type="submit"]');
        errorLine.hidden = true;
        successLine.hidden = true;
        progress.hidden = false;
        if (submitButton) {
            submitButton.disabled = true;
        }
        var displayName = namespace.text(form.elements.display_name ? form.elements.display_name.value : "", "");
        var secrets = {};
        var configuration = {};
        collectFieldValues(provider, form, secrets, configuration);
        try {
            var payload = await namespace.fetchJson(namespace.connectionsUrl(), {
                method: "POST",
                body: {
                    provider_slug: provider.id,
                    display_name: displayName,
                    auth_kind: authKindFor(provider),
                    secrets: secrets,
                    configuration: configuration,
                    granted_scopes: []
                }
            });
            clearFormInputs(form);
            progress.hidden = true;
            successLine.textContent = "Connected. Connection " + namespace.text(payload.id, "(id pending)") + " is active.";
            successLine.hidden = false;
            namespace.announce(root, provider.name + " connected");
            if (typeof namespace.loadConnected === "function") {
                namespace.loadConnected(root);
            }
        } catch (error) {
            clearFormInputs(form);
            progress.hidden = true;
            errorLine.textContent = connectErrorMessage(error);
            errorLine.hidden = false;
            namespace.announce(root, provider.name + " connection failed");
        } finally {
            if (submitButton) {
                submitButton.disabled = false;
            }
        }
    };

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
        var adapterTitle = "Configuration requires adapter";
        var adapterCopy = "Planned setup only. This control plane does not submit or persist credentials.";
        var adapterBlock = provider.llm_available ?
            (renderConnectSection(provider, state.uid) ||
                '<div class="integrations-adapter-state"><strong>Available to assistants</strong><p>The language model can use implemented actions, but this provider declares no connectable fields yet.</p></div>') :
            '<div class="integrations-adapter-state"><strong>' + namespace.escapeHtml(adapterTitle) + '</strong><p>' + namespace.escapeHtml(adapterCopy) + '</p></div>';
        var warning = state.detailStatus === "error" ? '<div class="integrations-inline-warning"><strong>Latest detail could not be loaded</strong>' +
            '<p>Showing the provider information already available from the catalog.</p><button class="integrations-button" type="button" data-action="retry-detail">Try detail again</button></div>' :
            (state.detailStatus === "loading" ? '<p class="integrations-readonly-note" role="status">Refreshing provider detail...</p>' : '');
        body.innerHTML = '<div class="integrations-detail-summary">' + providerMark(provider) + '<div><h3>' + namespace.escapeHtml(provider.name) + '</h3>' +
            '<p>' + namespace.escapeHtml(provider.strategy) + '</p></div></div>' +
            '<dl class="integrations-detail-meta"><div><dt>Category</dt><dd>' + namespace.escapeHtml(namespace.titleCase(provider.category)) + '</dd></div>' +
            '<div><dt>Priority</dt><dd>' + namespace.escapeHtml(namespace.titleCase(provider.priority)) + '</dd></div>' +
            '<div><dt>Module</dt><dd>' + namespace.escapeHtml(provider.module) + '</dd></div></dl>' + warning +
            adapterBlock +
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
        var focusable = Array.from(root.querySelectorAll("[data-detail-panel] button:not([disabled]), [data-detail-panel] a[href], [data-detail-panel] summary, [data-detail-panel] input:not([disabled]), [data-detail-panel][tabindex]"));
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
