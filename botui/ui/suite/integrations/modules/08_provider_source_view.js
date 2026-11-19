"use strict";

/* #2346 — catalog-source view: provenance report + provider editor.
   The editor is enabled only when the backend exposes the documented CRUD
   route; otherwise every provider is still shown with an honest source label
   and an explanation of where its definition lives. */
(function (namespace) {
    function t(key, fallback) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
    }

    function esc(value) {
        return namespace.escapeHtml(value);
    }

    function countLabel(count) {
        return typeof count === "number" ? String(count) : String((count || []).length);
    }

    function sourceCell(row) {
        var kind = namespace.sourceKind(row.source);
        return '<span class="integrations-source-pill is-source-' + esc(kind.replace(/:/g, "-")) + '">' +
            esc(namespace.providerSourceLabel(kind)) + '</span>' +
            '<span class="integrations-cell-subtext">' + esc(namespace.providerSourceDescription(kind)) + '</span>';
    }

    function providerRows(root, state) {
        if (!state.providerDefinitions.length) {
            return '<tr><td colspan="6">' + esc(t("integrations-source-none", "No provider definitions are available.")) + '</td></tr>';
        }
        return state.providerDefinitions.map(function (row) {
            var manage = "";
            if (state.providerSourceEditable) {
                manage = '<button class="integrations-button integrations-button-subtle" type="button" ' +
                    'data-provider-source-action="edit-provider" data-provider-source-id="' + esc(row.id) + '">' +
                    esc(t("integrations-source-edit-provider", "Edit")) + '</button>' +
                    '<button class="integrations-button integrations-button-subtle" type="button" ' +
                    'data-provider-source-action="delete-provider" data-provider-source-id="' + esc(row.id) + '">' +
                    esc(t("integrations-source-delete-provider", "Delete")) + '</button>';
            }
            return '<tr><td data-label="' + esc(t("integrations-source-col-provider", "Provider")) + '">' +
                '<strong>' + esc(row.name || row.id) + '</strong>' +
                '<span class="integrations-cell-subtext">' + esc(row.id) + '</span></td>' +
                '<td data-label="' + esc(t("integrations-source-col-category", "Category")) + '">' + esc(row.category) + '</td>' +
                '<td data-label="' + esc(t("integrations-source-col-source", "Source")) + '">' + sourceCell(row) + '</td>' +
                '<td data-label="' + esc(t("integrations-source-col-module", "Module")) + '">' + esc(row.module) + '</td>' +
                '<td data-label="' + esc(t("integrations-source-actions", "Actions")) + '">' + countLabel(row.actions) + '</td>' +
                '<td data-label="' + esc(t("integrations-source-col-manage", "Manage")) + '">' + manage + '</td></tr>';
        }).join("");
    }

    function definitionTable(root, state) {
        return '<div class="integrations-source-table-wrap"><table class="integrations-source-table">' +
            '<thead><tr>' +
            '<th>' + esc(t("integrations-source-col-provider", "Provider")) + '</th>' +
            '<th>' + esc(t("integrations-source-col-category", "Category")) + '</th>' +
            '<th>' + esc(t("integrations-source-col-source", "Source")) + '</th>' +
            '<th>' + esc(t("integrations-source-col-module", "Module")) + '</th>' +
            '<th>' + esc(t("integrations-source-actions", "Actions")) + '</th>' +
            '<th>' + esc(t("integrations-source-col-manage", "Manage")) + '</th>' +
            '</tr></thead><tbody>' + providerRows(root, state) + '</tbody></table></div>';
    }

    function field(name, label, value, type) {
        return '<label class="integrations-source-field"><span>' + esc(label) + '</span>' +
            '<input type="' + (type || "text") + '" name="' + esc(name) + '" value="' + esc(value) + '" ' +
            'autocomplete="off"></label>';
    }

    function textarea(name, label, value, rows) {
        return '<label class="integrations-source-field is-wide"><span>' + esc(label) + '</span>' +
            '<textarea name="' + esc(name) + '" rows="' + rows + '">' + esc(value) + '</textarea></label>';
    }

    function jsonText(value) {
        if (Array.isArray(value)) return JSON.stringify(value, null, 2);
        if (value && typeof value === "object") return JSON.stringify(value, null, 2);
        return typeof value === "string" ? value : "";
    }

    function stepsText(value) {
        if (Array.isArray(value)) return value.join("\n");
        if (typeof value === "string") return value;
        return "";
    }

    function editorForm(state) {
        var editing = !!state.editingProviderId && state.editingProviderId !== "__new__";
        var raw = state.editingProviderOriginal || {};
        var source = editing ? namespace.sourceKind(raw.source || raw.provenance) : "operator";
        var options = ["operator", "seeded:hardcoded", "community"].map(function (kind) {
            return '<option value="' + esc(kind) + '"' + (kind === source ? " selected" : "") + '>' +
                esc(namespace.providerSourceLabel(kind)) + '</option>';
        }).join("");
        return '<form class="integrations-source-form" data-provider-form novalidate>' +
            '<div class="integrations-source-form-grid">' +
            field("provider_id", t("integrations-source-id", "Provider ID"), namespace.text(raw.id || raw.provider_id, "")) +
            field("name", t("integrations-source-name", "Name"), namespace.text(raw.name, "")) +
            field("category", t("integrations-source-category", "Category"), namespace.text(raw.category, "")) +
            field("module", t("integrations-source-module", "Module"), namespace.text(raw.module, "")) +
            field("status", t("integrations-source-status", "Status"), namespace.text(raw.status, "")) +
            '<label class="integrations-source-field"><span>' + esc(t("integrations-source-col-source", "Source")) + '</span>' +
            '<select name="source">' + options + '</select></label>' +
            field("credentials_url", t("integrations-source-credentials-url", "Credentials URL"), namespace.text(raw.credentials_url, ""), "url") +
            field("official_docs", t("integrations-source-docs-url", "Official docs URL"), namespace.text(raw.official_docs, ""), "url") +
            '</div>' +
            textarea("actions", t("integrations-source-actions", "Actions (JSON)"), jsonText(raw.actions), 6) +
            textarea("setup_steps", t("integrations-source-setup-steps", "Setup steps (one per line)"), stepsText(raw.setup_steps), 4) +
            textarea("guidance", t("integrations-source-guidance", "Guidance"), stepsText(raw.guidance), 3) +
            '<div class="integrations-connect-actions">' +
            '<button class="integrations-button" type="submit">' +
            esc(editing ? t("integrations-source-save-provider", "Save provider") : t("integrations-source-new-provider", "New provider")) + '</button>' +
            '<button class="integrations-button integrations-button-subtle" type="button" ' +
            'data-provider-source-action="cancel-provider-edit">' + esc(t("action-cancel", "Cancel")) + '</button>' +
            '</div></form>';
    }

    function notice(state) {
        if (state.providerSourceEditable) {
            return '<p class="integrations-readonly-note">' +
                esc(t("integrations-source-editor-hint",
                    "Provider definitions are stored on the server. Changes appear in the catalog immediately after saving.")) + '</p>';
        }
        var title = state.providerSourceStatus === "error" ?
            t("integrations-source-error-title", "Provider definitions unavailable") :
            t("integrations-source-readonly-title", "Source transparency");
        var copy = state.providerSourceStatus === "error" ?
            t("integrations-source-error",
                "The provider definitions endpoint did not answer. The list below is the compiled catalog, shown so the source of every provider stays visible.") :
            t("integrations-source-readonly-desc",
                "This build serves a compile-time catalog. Provider definitions live in botserver/src/apps/integration_catalog/providers/*.rs and are expanded at boot; adding one needs a code change and a redeploy.");
        return '<div class="integrations-inline-warning"><strong>' + esc(title) + '</strong><p>' + esc(copy) + '</p></div>';
    }

    namespace.renderProviderSource = function (root) {
        var state = namespace.getState(root);
        var view = root.querySelector("[data-provider-source-view]");
        var status = root.querySelector("[data-provider-source-status]");
        if (!view || !state) return;
        if (state.providerSourceStatus === "idle" || state.providerSourceStatus === "loading") {
            status.textContent = t("integrations-loading-catalog-source", "Loading catalog source");
            view.setAttribute("aria-busy", "true");
            view.innerHTML = '<p class="integrations-readonly-note" role="status">' +
                esc(t("integrations-loading-catalog-source", "Loading catalog source")) + '</p>';
            return;
        }
        view.setAttribute("aria-busy", "false");
        status.textContent = t("integrations-source-provider-list-title", "Provider definitions") + " · " +
            state.providerDefinitions.length;
        var html = notice(state) + definitionTable(root, state);
        if (state.providerSourceEditable && state.editingProviderId) {
            html += editorForm(state);
        } else if (state.providerSourceEditable) {
            html += '<div class="integrations-source-add"><button class="integrations-button" type="button" ' +
                'data-provider-source-action="new-provider">' +
                esc(t("integrations-source-new-provider", "New provider")) + '</button></div>';
        }
        html += '<p class="integrations-inline-error" data-provider-source-message role="status" aria-live="polite"' +
            (state.providerSourceMessage ? "" : " hidden") + '>' + esc(state.providerSourceMessage) + '</p>';
        view.innerHTML = html;
    };

    function openEditor(root, id) {
        var state = namespace.getState(root);
        var definition = state.providerDefinitions.find(function (row) { return row.id === id; });
        if (!definition) return;
        state.editingProviderId = id;
        state.editingProviderOriginal = definition.raw;
        state.providerSourceMessage = "";
        namespace.renderProviderSource(root);
        var input = root.querySelector("[data-provider-form] input[name='provider_id']");
        if (input && typeof input.focus === "function") input.focus();
    }

    namespace.installProviderSourceEvents = function (root) {
        if (root.dataset.providerSourceReady === "true") return;
        root.dataset.providerSourceReady = "true";
        root.addEventListener("click", function (event) {
            var button = event.target.closest("[data-provider-source-action]");
            if (!button || !root.contains(button)) return;
            var state = namespace.getState(root);
            var action = button.dataset.providerSourceAction;
            if (action === "refresh-source") {
                namespace.loadProviderDefinitions(root);
            } else if (action === "new-provider") {
                state.editingProviderId = "__new__";
                state.editingProviderOriginal = {};
                state.providerSourceMessage = "";
                namespace.renderProviderSource(root);
                var input = root.querySelector("[data-provider-form] input[name='provider_id']");
                if (input && typeof input.focus === "function") input.focus();
            } else if (action === "edit-provider") {
                openEditor(root, button.dataset.providerSourceId);
            } else if (action === "delete-provider") {
                namespace.deleteProviderDefinition(root, button.dataset.providerSourceId);
            } else if (action === "cancel-provider-edit") {
                state.editingProviderId = "";
                state.editingProviderOriginal = null;
                state.providerSourceMessage = "";
                namespace.renderProviderSource(root);
            }
        });
        root.addEventListener("submit", function (event) {
            if (!event.target.matches("[data-provider-form]")) return;
            event.preventDefault();
            namespace.saveProviderDefinition(root, event.target);
        });
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
