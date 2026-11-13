(function () {
        const scope = window.__peopleScope;
        const state = scope.state;
        const apiFetch = scope.apiFetch;
        const normalizeContact = scope.normalizeContact;
        const applyFilters = scope.applyFilters;
        const applyLetter = scope.applyLetter;
        const escapeHtml = scope.escapeHtml;
        const escapeAttr = scope.escapeAttr;
        const groupByLetter = scope.groupByLetter;
        const getInitials = scope.getInitials;
        const getAvatarColor = scope.getAvatarColor;

        async function loadContacts() {
            try {
                const params = new URLSearchParams();
                if (state.filters.name) params.set("search", state.filters.name);
                const qs = params.toString();
                const data = await apiFetch(qs ? `?${qs}` : "");
                const list = Array.isArray(data) ? data : data.items || data.contacts || [];
                state.contacts = list.map(normalizeContact);
                renderContacts(applyLetter(applyFilters(state.contacts)));
            } catch (error) {
                console.error("Failed to load contacts:", error);
                state.contacts = [];
                renderEmptyState();
            }
        }


        function renderContacts(contactsList) {
            const container = document.getElementById("contacts-list");
            if (!contactsList || contactsList.length === 0) {
                renderEmptyState();
                return;
            }
            const grouped = groupByLetter(contactsList);
            let html = "";
            for (const [letter, group] of Object.entries(grouped)) {
                html += `<div class="contact-group" data-letter="${letter}">
            <div class="group-header">${letter}</div>
            <div class="group-contacts">`;
                for (const contact of group) {
                    html += renderContactCard(contact);
                }
                html += "</div></div>";
            }
            container.innerHTML = html;
        }

        function renderContactCard(contact) {
            const initials = getInitials(contact.firstName, contact.lastName);
            const name = `${contact.firstName || ""} ${contact.lastName || ""}`.trim() || "(no name)";

            return `<div class="contact-card" onclick="window.showContact('${contact.id}')">
        <div class="contact-avatar" style="background: ${getAvatarColor(name)}">${initials}</div>
        <div class="contact-info">
            <div class="contact-name">${escapeHtml(name)}</div>
            <div class="contact-detail">${escapeHtml(contact.email || contact.phone || "")}</div>
        </div>
        <div class="contact-actions">
            <button class="icon-btn small" onclick="event.stopPropagation(); window.startChat('${contact.id}')" title="Chat">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
            </button>
            <button class="icon-btn small" onclick="event.stopPropagation(); window.sendEmail('${escapeAttr(contact.email || "")}')" title="Email">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                    <polyline points="22,6 12,13 2,6"/>
                </svg>
            </button>
        </div>
    </div>`;
        }

        function renderEmptyState() {
            const container = document.getElementById("contacts-list");
            if (!container) return;
            container.innerHTML = `
        <div class="empty-state" role="status" aria-live="polite">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                <circle cx="9" cy="7" r="4"/>
                <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
            <h3 data-i18n="people-empty-title">No contacts yet</h3>
            <p data-i18n="people-empty-desc">Add your first contact to get started</p>
            <button class="btn btn-primary" onclick="window.openAddContact()">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <line x1="12" y1="5" x2="12" y2="19"/>
                    <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                <span data-i18n="people-add">Add Contact</span>
            </button>
        </div>
    `;
        }

        function showContact(id) {
            const contact = state.contacts.find((c) => c.id === id);
            if (!contact) return;
            state.currentContact = contact;
            const panel = document.getElementById("contact-panel");
            const detail = document.getElementById("contact-detail");
            if (!panel || !detail) return;
            const name = `${contact.firstName || ""} ${contact.lastName || ""}`.trim() || "(no name)";
            const tagList = Array.isArray(contact.tags) ? contact.tags : [];
            const tags = tagList.map((t) => `<span class="tag">${escapeHtml(t)}</span>`).join("");
            detail.innerHTML = `
                <div class="detail-name">${escapeHtml(name)}</div>
                <div class="detail-title">${escapeHtml(contact.title || "")}</div>
                <div class="detail-email">${escapeHtml(contact.email || "")}</div>
                <div class="detail-phone">${escapeHtml(contact.phone || "")}</div>
                <div class="detail-company">${escapeHtml(contact.company || "")}</div>
                ${contact.notes ? `<div class="detail-notes">${escapeHtml(contact.notes)}</div>` : ""}
                ${tags ? `<div class="detail-tags">${tags}</div>` : ""}
            `;
            panel.classList.add("open");
        }

        function closeContactPanel() {
            const panel = document.getElementById("contact-panel");
            if (panel) panel.classList.remove("open");
            state.currentContact = null;
        }

        function openAddContact() {
            state.currentContact = null;
            const titleEl = document.getElementById("modal-title");
            if (titleEl) titleEl.textContent = "Add Contact";
            const form = document.getElementById("contact-form");
            if (form) form.reset();
            const modal = document.getElementById("contact-modal");
            if (modal && modal.showModal) modal.showModal();
        }

        function editContact() {
            if (!state.currentContact) return;
            const titleEl = document.getElementById("modal-title");
            if (titleEl) titleEl.textContent = "Edit Contact";
            const form = document.getElementById("contact-form");
            if (!form) return;
            form.firstName.value = state.currentContact.firstName || "";
            form.lastName.value = state.currentContact.lastName || "";
            form.email.value = state.currentContact.email || "";
            form.phone.value = state.currentContact.phone || "";
            form.company.value = state.currentContact.company || "";
            form.title.value = state.currentContact.title || "";
            form.notes.value = state.currentContact.notes || "";
            const modal = document.getElementById("contact-modal");
            if (modal && modal.showModal) modal.showModal();
        }

        function closeModal() {
            const modal = document.getElementById("contact-modal");
            if (modal && modal.close) modal.close();
        }

        function showPeopleNotice(message, kind) {
            if (window.GBToasts && typeof window.GBToasts.show === "function") {
                window.GBToasts.show("People", message, kind || "info");
            } else if (typeof showToast === "function") {
                showToast(message, kind || "info");
            } else {
                console[kind === "error" ? "error" : "info"](message);
            }
        }

        function confirmPeopleAction(message, title) {
            if (window.GBModal && typeof window.GBModal.confirm === "function") {
                return new Promise((resolve) => {
                    try {
                        window.GBModal.confirm(message, title || "Confirm", () => resolve(true), () => resolve(false));
                    } catch (e) {
                        console.error("GBModal.confirm failed", e);
                        resolve(false);
                    }
                });
            }
            return Promise.resolve(false);
        }

        async function saveContact(event) {
            event.preventDefault();
            const form = event.target;
            const data = {
                firstName: form.firstName.value,
                lastName: form.lastName.value,
                email: form.email.value,
                phone: form.phone.value,
                company: form.company.value,
                title: form.title.value,
                notes: form.notes.value,
            };

            try {
                const payload = toSnakeBody(data);
                if (state.currentContact) {
                    await apiFetch(`/${state.currentContact.id}`, {
                        method: "PUT",
                        body: JSON.stringify(payload),
                    });
                } else {
                    await apiFetch("", {
                        method: "POST",
                        body: JSON.stringify(payload),
                    });
                }
                closeModal();
                await loadContacts();
                if (state.currentContact) closeContactPanel();
            } catch (error) {
                console.error("Failed to save contact:", error);
                showPeopleNotice("Failed to save contact: " + error.message, "error");
            }
        }

        async function deleteContact() {
            if (!state.currentContact) return;
            const confirmed = await confirmPeopleAction("Delete this contact?", "Confirm");
            if (!confirmed) return;

            try {
                await apiFetch(`/${state.currentContact.id}`, { method: "DELETE" });
                closeContactPanel();
                await loadContacts();
            } catch (error) {
                console.error("Failed to delete contact:", error);
                showPeopleNotice("Failed to delete contact: " + error.message, "error");
            }
        }

        function startChat(contactId) {
            window.location.href = `/#chat?contact=${contactId}`;
        }

        // E-mail surface removed (#1546): no mailto actions in the UI.
        function sendEmail(email) {
            if (!email) return;
            showToast ? showToast('E-mail actions are disabled', 'info') : console.info('E-mail actions are disabled');
        }

        function scheduleMeeting(contactId) {
            window.location.href = `/#calendar?contact=${contactId}`;
        }

        Object.assign(window.__peopleScope, {
            loadContacts: loadContacts, renderContacts: renderContacts,
            renderContactCard: renderContactCard, renderEmptyState: renderEmptyState,
            showContact: showContact, closeContactPanel: closeContactPanel,
            openAddContact: openAddContact, editContact: editContact,
            closeModal: closeModal, saveContact: saveContact,
            deleteContact: deleteContact, startChat: startChat,
            sendEmail: sendEmail, scheduleMeeting: scheduleMeeting,
        });
    })();
