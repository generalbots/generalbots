    (function () {
        const API_BASE = "/api/crm/contacts";

        const state = {
            currentContact: null,
            contacts: [],
            filters: {
                name: "",
                company: "",
                title: "",
                tags: "",
                source: "",

            },
        };

        function snakeToCamel(key) {
            return key.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
        }

        function normalizeContact(c) {
            if (!c || typeof c !== "object") return c;
            const out = {};
            for (const k of Object.keys(c)) {
                out[snakeToCamel(k)] = c[k];
            }
            if (out.jobTitle && !out.title) out.title = out.jobTitle;
            return out;
        }

        function toSnakeBody(c) {
            const map = {
                firstName: "first_name",
                lastName: "last_name",
                jobTitle: "job_title",
                organizationId: "organization_id",
                ownerId: "owner_id",
                firstName: "first_name",
                lastName: "last_name",
                mobile: "mobile",
                phone: "phone",
                email: "email",
                company: "company",
                title: "job_title",
                notes: "notes",
                tags: "tags",
                customFields: "custom_fields",
                addressLine1: "address_line1",
                addressLine2: "address_line2",
                postalCode: "postal_code",
                linkedin: "linkedin",
                twitter: "twitter",
            };
            const out = {};
            for (const k of Object.keys(c)) {
                out[map[k] || k] = c[k];
            }
            return out;
        }

        async function apiFetch(path, options = {}) {
            const url = path.startsWith("http") ? path : `${API_BASE}${path}`;
            const token = localStorage.getItem("gb_token");
            const opts = {
                headers: {
                    "Content-Type": "application/json",
                    ...(token ? { "Authorization": `Bearer ${token}` } : {}),
                },
                ...options,
            };
            const resp = await fetch(url, opts);
            if (!resp.ok) {
                const text = await resp.text();
                throw new Error(`HTTP ${resp.status}: ${text || resp.statusText}`);
            }
            const ct = resp.headers.get("content-type") || "";
            if (ct.includes("application/json")) {
                return await resp.json();
            }
            return null;
        }

        function init() {
            bindAdvancedSearch();
            loadContacts();
            // Deep-link support: open the app contextualized to a person when the
            // desktop shell opened us via app://people?person_id=...
            const params = window.__gbAppParams__ || {};
            if (params.person_id) {
                openDeepLinkedContact(params.person_id);
            }
        }

        // Fetches a single contact by id and opens the detail panel directly.
        // This works even when the contacts list is scoped to a different branch
        // (the list endpoint uses the global default bot scope) — the id endpoint
        // returns the record regardless.
        async function openDeepLinkedContact(contactId) {
            const panel = document.getElementById("contact-panel");
            const detail = document.getElementById("contact-detail");
            if (panel && detail) {
                detail.textContent = "Loading contact...";
                panel.classList.add("open");
            }
            try {
                const data = await apiFetch(`/${contactId}`);
                const contact = normalizeContact(Array.isArray(data) ? data[0] : data);
                state.contacts = state.contacts || [];
                if (!state.contacts.some((c) => c.id === contact.id)) {
                    state.contacts.push(contact);
                }
                state.currentContact = contact;
                if (typeof window.showContact === "function") {
                    window.showContact(contact.id);
                }
            } catch (error) {
                console.error("Deep-link contact load failed:", error);
                if (detail) detail.textContent = "Contact not found.";
            }
        }

        if (document.readyState === "loading") {
            (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
        } else {
            init();
        }

        async function loadContacts() {
            try {
                const params = new URLSearchParams();
                if (state.filters.name) params.set("search", state.filters.name);
                const qs = params.toString();
                const data = await apiFetch(qs ? `?${qs}` : "");
                const list = Array.isArray(data) ? data : data.items || data.contacts || [];
                state.contacts = list.map(normalizeContact);
                renderContacts(applyFilters(state.contacts));
            } catch (error) {
                console.error("Failed to load contacts:", error);
                state.contacts = [];
                renderEmptyState();
            }
        }

        function applyFilters(list) {
            const f = state.filters;
            return list.filter((c) => {
                if (f.company && (c.company || "").toLowerCase() !== f.company.toLowerCase()) return false;
                if (f.title && (c.title || "").toLowerCase() !== f.title.toLowerCase()) return false;
                if (f.source && c.source !== f.source) return false;
                if (f.tags) {
                    const wanted = f.tags.split(",").map((t) => t.trim().toLowerCase()).filter(Boolean);
                    const have = (c.tags || []).map((t) => (t || "").toLowerCase());
                    if (!wanted.every((t) => have.includes(t))) return false;
                }
                return true;
            });
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

        function escapeHtml(s) {
            return String(s)
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#39;");
        }

        function escapeAttr(s) {
            return escapeHtml(s);
        }

        function renderEmptyState() {
            const container = document.getElementById("contacts-list");
            if (!container) return;
            container.innerHTML = `
        <div class="empty-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                <circle cx="9" cy="7" r="4"/>
                <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
            <h3 data-i18n="people-empty-title">No contacts yet</h3>
            <p data-i18n="people-empty-desc">Add your first contact to get started</p>
            <button class="btn btn-primary" onclick="window.openAddContact()">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="12" y1="5" x2="12" y2="19"/>
                    <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                <span data-i18n="people-add">Add Contact</span>
            </button>
        </div>
    `;
        }

        function groupByLetter(contactsList) {
            const grouped = {};
            for (const contact of contactsList) {
                const letter = (contact.lastName || contact.firstName || "#").charAt(0).toUpperCase();
                if (!grouped[letter]) grouped[letter] = [];
                grouped[letter].push(contact);
            }
            const sorted = Object.keys(grouped).sort();
            const out = {};
            for (const k of sorted) out[k] = grouped[k];
            return out;
        }

        function getInitials(firstName, lastName) {
            const f = (firstName || "").charAt(0).toUpperCase();
            const l = (lastName || "").charAt(0).toUpperCase();
            return f + l || "?";
        }

        function getAvatarColor(name) {
            const colors = ["#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E2"];
            let hash = 0;
            for (let i = 0; i < name.length; i++) hash = name.charCodeAt(i) + ((hash << 5) - hash);
            return colors[Math.abs(hash) % colors.length];
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
                alert("Failed to save contact: " + error.message);
            }
        }

        async function deleteContact() {
            if (!state.currentContact || !confirm("Delete this contact?")) return;

            try {
                await apiFetch(`/${state.currentContact.id}`, { method: "DELETE" });
                closeContactPanel();
                await loadContacts();
            } catch (error) {
                console.error("Failed to delete contact:", error);
                alert("Failed to delete contact: " + error.message);
            }
        }

        function startChat(contactId) {
            window.location.href = `/#chat?contact=${contactId}`;
        }

        function sendEmail(email) {
            if (!email) return;
            window.location.href = `mailto:${email}`;
        }

        function scheduleMeeting(contactId) {
            window.location.href = `/#calendar?contact=${contactId}`;
        }

        function bindAdvancedSearch() {
            const runBtn = document.getElementById("adv-search-run");
            if (runBtn) {
                runBtn.addEventListener("click", () => {
                    state.filters.name = (document.getElementById("adv-search-name") || {}).value || "";
                    state.filters.company = (document.getElementById("adv-search-company") || {}).value || "";
                    state.filters.title = (document.getElementById("adv-search-title") || {}).value || "";
                    state.filters.tags = (document.getElementById("adv-search-tags") || {}).value || "";
                    state.filters.source = (document.getElementById("adv-search-source") || {}).value || "";
                    loadContacts();
                });
            }
            const exportBtn = document.getElementById("adv-search-export-csv");
            if (exportBtn) exportBtn.addEventListener("click", exportCsv);
            const importBtn = document.getElementById("adv-search-import-csv");
            const importInput = document.getElementById("adv-search-import-input");
            if (importBtn && importInput) {
                importBtn.addEventListener("click", () => importInput.click());
                importInput.addEventListener("change", importCsv);
            }

        }

        function exportCsv() {
            const rows = state.contacts.map((c) => [
                c.firstName || "",
                c.lastName || "",
                c.email || "",
                c.phone || "",
                c.company || "",
                c.title || "",
            ]);
            const header = ["first_name", "last_name", "email", "phone", "company", "title"];
            const csv = [header, ...rows]
                .map((r) => r.map((v) => `"${String(v).replace(/"/g, '""')}"`).join(","))
                .join("\n");
            const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = "contacts.csv";
            a.click();
            URL.revokeObjectURL(url);
        }

        async function importCsv(event) {
            const file = event.target.files && event.target.files[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = async (e) => {
                const text = String(e.target.result || "");
                const lines = text.split(/\r?\n/).filter((l) => l.trim());
                if (lines.length < 2) return;
                const headers = parseCsvLine(lines[0]).map((h) => h.toLowerCase().trim());
                let imported = 0;
                for (let i = 1; i < lines.length; i++) {
                    const cols = parseCsvLine(lines[i]);
                    const row = {};
                    headers.forEach((h, idx) => {
                        row[h] = cols[idx];
                    });
                    const data = {
                        firstName: row["first name"] || row["firstname"] || row["first_name"] || "",
                        lastName: row["last name"] || row["lastname"] || row["last_name"] || "",
                        email: row["email"] || "",
                        phone: row["phone"] || "",
                        company: row["company"] || "",
                        title: row["title"] || row["job title"] || "",
                    };
                    if (!data.firstName && !data.email) continue;
                    try {
                        await apiFetch("", {
                            method: "POST",
                            body: JSON.stringify(toSnakeBody(data)),
                        });
                        imported++;
                    } catch (err) {
                        console.error("CSV import row failed", err);
                    }
                }
                alert(`Imported ${imported} contacts`);
                await loadContacts();
            };
            reader.readAsText(file);
        }

        function parseCsvLine(line) {
            const out = [];
            let cur = "";
            let inQ = false;
            for (let i = 0; i < line.length; i++) {
                const ch = line[i];
                if (inQ) {
                    if (ch === '"' && line[i + 1] === '"') {
                        cur += '"';
                        i++;
                    } else if (ch === '"') {
                        inQ = false;
                    } else {
                        cur += ch;
                    }
                } else if (ch === '"') {
                    inQ = true;
                } else if (ch === ",") {
                    out.push(cur);
                    cur = "";
                } else {
                    cur += ch;
                }
            }
            out.push(cur);
            return out;
        }

        function advancedSearch() {
            const f = state.filters;
            const q = (document.getElementById("people-search") || {}).value || "";
            f.name = q.toLowerCase();
            loadContacts();
        }

        async function mergeContacts(id1, id2) {
            if (!confirm("Merge these contacts? The first contact will be kept.")) return;
            try {
                await apiFetch(`/${id1}`, {
                    method: "PUT",
                    body: JSON.stringify(toSnakeBody({
                        mergeInto: id2,
                        mergedAt: new Date().toISOString(),
                    })),
                });
                await loadContacts();
            } catch (err) {
                console.error("Merge failed", err);
                alert("Merge failed: " + err.message);
            }
        }

        window.showContact = showContact;
        window.openAddContact = openAddContact;
        window.editContact = editContact;
        window.closeContactPanel = closeContactPanel;
        window.closeModal = closeModal;
        window.saveContact = saveContact;
        window.deleteContact = deleteContact;
        window.startChat = startChat;
        window.sendEmail = sendEmail;
        window.scheduleMeeting = scheduleMeeting;
        window.advancedSearch = advancedSearch;
        window.exportCsv = exportCsv;
        window.importCsv = importCsv;
        window.mergeContacts = mergeContacts;
    })();
