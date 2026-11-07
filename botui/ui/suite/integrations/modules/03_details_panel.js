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
            // #1823: never POST to the current page when workspace context is missing.
            if (typeof namespace.fetchContext === "function") {
                await namespace.fetchContext(root);
            }
            var url = namespace.connectionsUrl();
            if (!url) {
                progress.hidden = true;
                errorLine.textContent = "No active bot/workspace context. Open a workspace with an active bot before connecting integrations.";
                errorLine.hidden = false;
                namespace.announce(root, provider.name + " needs a workspace context");
                return;
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
        // #1859: send users straight to the page that issues the credential,
        // not just to the API reference. Unsupported/unknown providers and
        // user-supplied endpoints publish no credentials_url.
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
