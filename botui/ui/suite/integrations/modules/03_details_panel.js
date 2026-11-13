"use strict";

(function (namespace) {
    var helpers = namespace.detailsHelpers || {};
    var isAwsProvider = helpers.isAwsProvider;
    var collectGuidance = helpers.collectGuidance;
    var renderFields = helpers.renderFields;
    var renderGuidance = helpers.renderGuidance;
    var renderSetupSteps = helpers.renderSetupSteps;
    var renderActions = helpers.renderActions;
    var renderConnectSection = helpers.renderConnectSection;
    var providerMark = helpers.providerMark;
    var authKindFor = helpers.authKindFor;
    var collectFieldValues = helpers.collectFieldValues;
    var validateConnectForm = helpers.validateConnectForm;
    var requiredFieldLabel = helpers.requiredFieldLabel;
    var clearFieldErrors = helpers.clearFieldErrors;
    var clearFormInputs = helpers.clearFormInputs;
    var connectErrorMessage = helpers.connectErrorMessage;

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
        var rotate = state.rotateTarget && state.rotateTarget.providerSlug === provider.id ? state.rotateTarget : null;
        errorLine.hidden = true;
        errorLine.textContent = "";
        successLine.hidden = true;
        if (typeof clearFieldErrors === "function") {
            clearFieldErrors(form);
        }

        var validation = { valid: true, missing: [] };
        if (rotate) {
            if (typeof clearFieldErrors === "function") {
                clearFieldErrors(form);
            }
        } else if (typeof validateConnectForm === "function") {
            validation = validateConnectForm(provider, form);
        } else {
            validation = { valid: typeof form.checkValidity !== "function" || form.checkValidity(), missing: [] };
        }
        if (!validation.valid) {
            if (typeof form.reportValidity === "function") {
                form.reportValidity();
            }
            var labels = (validation.missing || []).map(function (key) {
                return typeof requiredFieldLabel === "function" ? requiredFieldLabel(provider, key) : key;
            });
            errorLine.textContent = labels.length
                ? "Missing required fields: " + labels.join(", ") + "."
                : "Please review the highlighted fields and try again.";
            errorLine.hidden = false;
            if (validation.missing && validation.missing[0] && form.elements[validation.missing[0]] && typeof form.elements[validation.missing[0]].focus === "function") {
                form.elements[validation.missing[0]].focus();
            }
            namespace.announce(root, provider.name + " connection form has missing required fields");
            return;
        }

        progress.hidden = false;
        if (submitButton) {
            submitButton.disabled = true;
        }
        var displayName = namespace.text(form.elements.display_name ? form.elements.display_name.value : "", "");
        var secrets = {};
        var configuration = {};
        collectFieldValues(provider, form, secrets, configuration);
        try {
            if (typeof namespace.fetchContext === "function") {
                await namespace.fetchContext(root);
            }
            if (rotate) {
                if (!Object.keys(secrets).length) {
                    throw Object.assign(new Error("Rotation requires new credential values."), { status: 400 });
                }
                var rotateUrl = namespace.connectionActionUrl(rotate.connectionId, "/rotate");
                if (!rotateUrl) {
                    throw new Error("Connection rotation needs a workspace context");
                }
                var rotated = await namespace.fetchJson(rotateUrl, { method: "POST", body: { secrets: secrets } });
                clearFormInputs(form, { secretsOnly: true });
                progress.hidden = true;
                successLine.textContent = "Credentials rotated. Credential version is now v" + namespace.text(rotated.credential_version, "?") + ".";
                successLine.hidden = false;
                namespace.announce(root, provider.name + " credentials rotated");
                state.rotateTarget = null;
            } else {
                var url = namespace.connectionsUrl();
                if (!url) {
                    throw new Error("No active bot/workspace context. Open a workspace with an active bot before connecting integrations.");
                }
                var payload = await namespace.fetchJson(url, {
                    method: "POST",
                    body: {
                        provider_slug: provider.id,
                        display_name: displayName,
                        auth_kind: authKindFor(provider),
                        secrets: secrets,
                        configuration: configuration,
                        granted_scopes: [],
                        visibility: form.elements.visibility ? form.elements.visibility.value : "private"
                    }
                });
                clearFormInputs(form, { secretsOnly: true });
                progress.hidden = true;
                successLine.textContent = "Connected. Connection " + namespace.text(payload.id, "(id pending)") + " is active.";
                successLine.hidden = false;
                namespace.announce(root, provider.name + " connected");
            }
            if (typeof namespace.loadConnected === "function") {
                namespace.loadConnected(root);
            }
            namespace.renderDetails(root);
        } catch (error) {
            progress.hidden = true;
            var fieldContext = null;
            if (error && error.body && typeof error.body === "object") {
                var fieldName = error.body.field || error.body.field_name || (Array.isArray(error.body.missing) ? error.body.missing[0] : "");
                if (fieldName) {
                    fieldContext = { fieldName: typeof requiredFieldLabel === "function" ? requiredFieldLabel(provider, String(fieldName)) : String(fieldName) };
                } else if (Array.isArray(error.body.missing_fields) && error.body.missing_fields.length) {
                    fieldContext = { missingFields: error.body.missing_fields.map(function (key) {
                        return typeof requiredFieldLabel === "function" ? requiredFieldLabel(provider, String(key)) : String(key);
                    }) };
                }
            }
            var message = connectErrorMessage(error, fieldContext);
            if (rotate && error && /credential material/i.test(String(error.message || ""))) {
                message = "Enter new secret credential values to rotate this connection.";
            }
            errorLine.textContent = message;
            errorLine.hidden = false;
            namespace.announce(root, provider.name + (rotate ? " credential rotation failed" : " connection failed"));
        } finally {
            if (submitButton) {
                submitButton.disabled = false;
            }
            progress.hidden = true;
        }
    };

    namespace.saveOAuthClient = async function (root, form) {
        var state = namespace.getState(root);
        var provider = state.providerMap.get(state.selectedProviderId);
        var botId = (namespace.context && namespace.context.botId) || "";
        var progress = form.querySelector("[data-oauth-progress]");
        var errorLine = form.querySelector("[data-oauth-error]");
        var successLine = form.querySelector("[data-oauth-success]");
        if (!provider || !botId) return;
        errorLine.hidden = true;
        successLine.hidden = true;
        progress.hidden = false;
        var clientId = namespace.text(form.elements.oauth_client_id.value, "");
        var clientSecret = namespace.text(form.elements.oauth_client_secret.value, "");
        try {
            await namespace.fetchJson(
                "/api/bots/" + encodeURIComponent(botId) +
                "/integrations/oauth/" + encodeURIComponent(provider.id) + "/client",
                { method: "POST", body: { client_id: clientId, client_secret: clientSecret || null } }
            );
            form.elements.oauth_client_secret.value = "";
            progress.hidden = true;
            successLine.textContent = "App configuration saved. You can authorize now.";
            successLine.hidden = false;
            namespace.announce(root, provider.name + " OAuth app saved");
        } catch (error) {
            progress.hidden = true;
            var status = error && typeof error.status === "number" ? error.status : 0;
            errorLine.textContent = status === 403
                ? "Only an administrator of this workspace can store the app configuration."
                : "Could not save the app configuration (HTTP " + (status || "?") + ").";
            errorLine.hidden = false;
        }
    };

    function expiryBanner(root, provider) {
        var state = namespace.getState(root);
        var rows = (state.connectors || []).filter(function (row) { return row.provider_slug === provider.id; });
        var soon = rows.filter(function (row) {
            var s = typeof namespace.expiryState === "function" ? namespace.expiryState(row.expires_at) : "";
            return s === "soon" || s === "expired";
        });
        if (!soon.length) return "";
        var row = soon[0];
        var s = typeof namespace.expiryState === "function" ? namespace.expiryState(row.expires_at) : "";
        var when = typeof namespace.relativeDate === "function" ? namespace.relativeDate(row.expires_at) : row.expires_at;
        var title = s === "expired" ? "Credential expired" : "Credential expires soon";
        var copy = (row.display_name || provider.name) + (s === "expired" ? " expired " : " expires ") + namespace.escapeHtml(String(when || "")) + ". Rotate or renew it to avoid service interruption.";
        return '<div class="integrations-inline-warning integrations-expiry-banner"><strong>' + namespace.escapeHtml(title) + '</strong>' +
            '<p>' + copy + '</p>' +
            '<button class="integrations-button integrations-button-subtle" type="button" data-action="rotate-connection" data-connection-id="' + namespace.escapeHtml(row.id) + '" data-provider-slug="' + namespace.escapeHtml(provider.id) + '">Rotate / renew credentials</button></div>';
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
        var credentialsUrl = namespace.safeUrl(provider.credentials_url);
        var credentialsBlock = credentialsUrl ?
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Get credentials</h4><span>Vendor console</span></div>' +
            '<p class="integrations-readonly-note">Create the token, key or OAuth app on the vendor page, then paste the value into the connect form below.</p>' +
            '<a class="integrations-button" href="' + namespace.escapeHtml(credentialsUrl) + '" target="_blank" rel="noopener noreferrer">Open credential page</a></section>' : '';
        var setupSteps = renderSetupSteps(provider.auth.setup_steps, provider.auth.instructions);
        var setupNote = provider.auth.instructions ?
            '<p class="integrations-readonly-note">' + namespace.escapeHtml(provider.auth.instructions) + '</p>' : '';
        var adapterTitle = "Configuration requires adapter";
        var adapterCopy = "Planned setup only. This control plane does not submit or persist credentials.";
        var connectSection = renderConnectSection(provider, state.uid);
        if (connectSection && state.rotateTarget && state.rotateTarget.providerSlug === provider.id) {
            connectSection = connectSection.replace('<h4>Connect securely</h4>', '<h4>Rotate credentials</h4>');
            connectSection = connectSection.replace('<span>Vault-backed</span>', '<span>Updates stored secrets</span>');
            connectSection = connectSection.replace('data-action="connect-provider">Connect</button>', 'data-action="connect-provider">Rotate credentials</button>');
            connectSection = connectSection.replace('<p class="integrations-connect-help">Values travel once over an authenticated request; secret keys are written to the server-side vault and never displayed again.</p>',
                '<p class="integrations-connect-help" data-rotate-note>Enter new secret values for the selected connection. Secrets are written to the server-side vault and the credential version increments after a successful rotation.</p>');
        }
        var adapterBlock = provider.llm_available ?
            (connectSection ||
                '<div class="integrations-adapter-state"><strong>Available to assistants</strong><p>The language model can use implemented actions, but this provider declares no connectable fields yet.</p></div>') :
            '<div class="integrations-adapter-state"><strong>' + namespace.escapeHtml(adapterTitle) + '</strong><p>' + namespace.escapeHtml(adapterCopy) + '</p></div>';
        var warning = state.detailStatus === "error" ? '<div class="integrations-inline-warning"><strong>Latest detail could not be loaded</strong>' +
            '<p>Showing the provider information already available from the catalog.</p><button class="integrations-button" type="button" data-action="retry-detail">Try detail again</button></div>' :
            (state.detailStatus === "loading" ? '<p class="integrations-readonly-note" role="status" aria-live="polite">Refreshing provider detail...</p>' : '');
        body.innerHTML = '<div class="integrations-detail-summary">' + providerMark(provider) + '<div><h3>' + namespace.escapeHtml(provider.name) + '</h3>' +
            '<p>' + namespace.escapeHtml(provider.strategy) + '</p></div></div>' +
            '<dl class="integrations-detail-meta"><div><dt>Category</dt><dd>' + namespace.escapeHtml(namespace.titleCase(provider.category)) + '</dd></div>' +
            '<div><dt>Priority</dt><dd>' + namespace.escapeHtml(namespace.titleCase(provider.priority)) + '</dd></div>' +
            '<div><dt>Module</dt><dd>' + namespace.escapeHtml(provider.module) + '</dd></div></dl>' + warning + expiryBanner(root, provider) +
            credentialsBlock + adapterBlock +
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Authentication</h4>' +
            '<span>' + namespace.escapeHtml(namespace.titleCase(provider.auth.method, "Not specified")) + '</span></div>' + renderFields(provider) + '</section>' +
            '<section class="integrations-detail-section"><div class="integrations-detail-section-head"><h4>Setup guidance</h4><span>Read only</span></div>' +
            setupSteps + setupNote + '</section>' +
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
        if (!state.rotateTarget || state.rotateTarget.providerSlug !== providerId) {
            state.rotateTarget = null;
        }
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
        state.rotateTarget = null;
        layer.hidden = true;
        layer.setAttribute("aria-hidden", "true");
        if (state.lastFocus && state.lastFocus.isConnected) {
            state.lastFocus.focus();
        }
    };

    namespace.openRotateConnection = function (root, connectionId, providerSlug) {
        var state = namespace.getState(root);
        var connection = (state.connectors || []).find(function (row) { return row.id === connectionId; });
        if (!connection) return namespace.announce(root, "Connection no longer available for rotation");
        var slug = providerSlug || connection.provider_slug;
        if (!slug || !state.providerMap || !state.providerMap.has(slug)) {
            namespace.announce(root, "Open the provider in Explore to rotate this credential");
            if (slug && namespace.openDetails) namespace.openDetails(root, slug);
            return;
        }
        state.rotateTarget = { connectionId: connectionId, providerSlug: slug };
        namespace.openDetails(root, slug);
        window.requestAnimationFrame(function () {
            var form = root.querySelector("[data-connect-form]");
            if (!form) return;
            var submit = form.querySelector('[type="submit"]');
            if (submit) submit.textContent = "Rotate credentials";
            var first = form.querySelector('input[type="password"]') || form.querySelector('[data-connect-input="true"]');
            if (first && typeof first.focus === "function") first.focus();
        });
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
