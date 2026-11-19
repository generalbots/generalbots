"use strict";

/* #2346 — provider-definition source/provenance helpers and CRUD calls.
   The CRUD route is the contract published in the issue; when a build does
   not expose it the view degrades to a truthful read-only transparency report
   built from the catalog response (which already exists). */
(function (namespace) {
    // Loaded after 01_state.js — safe to decorate its normalizers here.
    if (typeof namespace.installSourceEnrichment === "function") {
        namespace.installSourceEnrichment();
    }

    function t(key, fallback) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
    }

    namespace.providerSourceLabel = function (source) {
        switch (namespace.sourceKind(source)) {
            case "seeded:hardcoded":
                return t("integrations-source-seeded", "Compiled seed");
            case "operator":
                return t("integrations-source-operator", "Operator");
            case "community":
                return t("integrations-source-community", "Community");
            default:
                return t("integrations-source-unknown", "Unspecified");
        }
    };

    namespace.providerSourceDescription = function (source) {
        switch (namespace.sourceKind(source)) {
            case "seeded:hardcoded":
                return t("integrations-source-seeded-desc",
                    "Compiled into the server binary from Rust seed data. Changing it requires a code change and a redeploy.");
            case "operator":
                return t("integrations-source-operator-desc",
                    "Added by an operator through the provider editor and stored on the server.");
            case "community":
                return t("integrations-source-community-desc",
                    "Contributed by the community. Review its actions before storing credentials.");
            default:
                return t("integrations-source-unknown-desc",
                    "The catalog did not report where this definition came from.");
        }
    };

    function definitionFromCatalog(provider) {
        return {
            id: provider.id,
            name: provider.name,
            category: provider.category,
            module: provider.module,
            status: provider.status,
            source: provider.source,
            actions: provider.actions,
            readOnly: true,
            raw: provider
        };
    }

    namespace.normalizeProviderDefinition = function (raw) {
        var source = raw && typeof raw === "object" ? raw : {};
        return {
            id: namespace.text(source.id || source.provider_id || source.slug, ""),
            name: namespace.text(source.name || source.display_name, ""),
            category: namespace.text(source.category, ""),
            module: namespace.text(source.module, ""),
            status: namespace.text(source.status, ""),
            source: namespace.sourceKind(source.source || source.provenance),
            actions: Array.isArray(source.actions) ? source.actions : [],
            readOnly: false,
            raw: source
        };
    };

    function definitionsFromPayload(payload) {
        var rows = Array.isArray(payload) ? payload :
            (payload && Array.isArray(payload.providers) ? payload.providers :
                (payload && Array.isArray(payload.items) ? payload.items : []));
        return rows.map(namespace.normalizeProviderDefinition).filter(function (row) { return row.id; });
    }

    namespace.loadProviderDefinitions = async function (root) {
        var state = namespace.getState(root);
        if (!state || state.providerSourceStatus === "loading") {
            return;
        }
        state.providerSourceRequest += 1;
        var requestId = state.providerSourceRequest;
        state.providerSourceStatus = "loading";
        state.providerSourceMessage = "";
        namespace.renderProviderSource(root);
        try {
            var payload = await namespace.fetchJson(namespace.providersUrl);
            if (requestId !== state.providerSourceRequest) return;
            state.providerDefinitions = definitionsFromPayload(payload);
            state.providerSourceEditable = true;
            state.providerSourceStatus = "ready";
        } catch (error) {
            if (requestId !== state.providerSourceRequest) return;
            var status = error && typeof error.status === "number" ? error.status : 0;
            // 404/405 = route not shipped by this build. Other failures still
            // show the catalog as provenance evidence, plus a retry action.
            state.providerSourceEditable = false;
            state.providerSourceStatus = status === 404 || status === 405 ? "read-only" : "error";
            state.providerDefinitions = state.providers.map(definitionFromCatalog);
        }
        namespace.renderProviderSource(root);
    };

    function formValue(form, name) {
        var field = form.elements[name];
        return field ? String(field.value || "").trim() : "";
    }

    function buildDefinitionPayload(form, editing, state) {
        var id = formValue(form, "provider_id");
        var name = formValue(form, "name");
        if (!id || !name) {
            throw new Error(t("integrations-source-id-required", "Provider ID and name are required."));
        }
        var actionsText = formValue(form, "actions");
        var actions = [];
        if (actionsText) {
            try {
                var parsed = JSON.parse(actionsText);
                actions = Array.isArray(parsed) ? parsed : (parsed && Array.isArray(parsed.actions) ? parsed.actions : []);
            } catch (error) {
                throw new Error(t("integrations-source-actions-invalid", "Actions must be a valid JSON array."));
            }
        }
        var steps = formValue(form, "setup_steps").split("\n").map(function (line) {
            return line.trim();
        }).filter(Boolean);
        var payload = {
            id: id,
            name: name,
            category: formValue(form, "category") || "other",
            module: formValue(form, "module"),
            status: formValue(form, "status") || "planned",
            source: formValue(form, "source") || "operator",
            actions: actions,
            setup_steps: steps,
            credentials_url: formValue(form, "credentials_url"),
            official_docs: formValue(form, "official_docs"),
            guidance: formValue(form, "guidance")
        };
        if (editing) {
            // Keep fields this client does not model (params, auth, surfaces…)
            // so a round-trip never silently drops provider capabilities.
            var original = state.editingProviderOriginal || {};
            payload = Object.assign({}, original, payload);
        }
        return payload;
    }

    namespace.saveProviderDefinition = async function (root, form) {
        var state = namespace.getState(root);
        var editingId = state.editingProviderId === "__new__" ? "" : state.editingProviderId;
        var payload;
        try {
            payload = buildDefinitionPayload(form, !!editingId, state);
        } catch (error) {
            state.providerSourceMessage = error.message;
            namespace.renderProviderSource(root);
            return;
        }
        var url = namespace.providersUrl + (editingId ? "/" + encodeURIComponent(editingId) : "");
        try {
            await namespace.fetchJson(url, { method: editingId ? "PUT" : "POST", body: payload });
            state.editingProviderId = "";
            state.editingProviderOriginal = null;
            state.providerSourceMessage = t("integrations-source-saved", "Provider saved.");
            namespace.announce(root, state.providerSourceMessage);
            await namespace.loadProviderDefinitions(root);
            await namespace.loadCatalog(root);
        } catch (error) {
            state.providerSourceMessage = t("integrations-source-save-failed", "The provider could not be saved.");
            namespace.renderProviderSource(root);
        }
    };

    namespace.deleteProviderDefinition = async function (root, id) {
        var state = namespace.getState(root);
        var message = t("integrations-source-delete-confirm",
            "Delete this provider? It will disappear from the catalog immediately.");
        var title = t("integrations-source-delete-confirm-title", "Delete provider");
        var confirmed = await new Promise(function (resolve) {
            if (window.GBModal && typeof window.GBModal.confirm === "function") {
                window.GBModal.confirm(message, title, function () { resolve(true); }, function () { resolve(false); });
            } else {
                resolve(false);
            }
        });
        if (!confirmed) return;
        try {
            await namespace.fetchJson(namespace.providersUrl + "/" + encodeURIComponent(id), { method: "DELETE" });
            state.providerSourceMessage = t("integrations-source-deleted", "Provider deleted.");
            namespace.announce(root, state.providerSourceMessage);
            await namespace.loadProviderDefinitions(root);
            await namespace.loadCatalog(root);
        } catch (error) {
            state.providerSourceMessage = t("integrations-source-delete-failed", "The provider could not be deleted.");
            namespace.renderProviderSource(root);
        }
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
