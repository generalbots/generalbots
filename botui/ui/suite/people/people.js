    (function () {
        let currentContact = null;
        let contacts = [];

        document.addEventListener("DOMContentLoaded", () => {
            loadContacts();
        });

        async function loadContacts() {
            try {
                const response = await fetch("/api/contacts");
                if (response.ok) {
                    contacts = await response.json();
                    renderContacts(contacts);
                } else {
                    renderEmptyState();
                }
            } catch (error) {
                console.error("Failed to load contacts:", error);
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
            const name = `${contact.firstName} ${contact.lastName}`;

            return `<div class="contact-card" onclick="showContact('${contact.id}')">
        <div class="contact-avatar" style="background: ${getAvatarColor(name)}">${initials}</div>
        <div class="contact-info">
            <div class="contact-name">${name}</div>
            <div class="contact-detail">${contact.email || contact.phone || ""}</div>
        </div>
        <div class="contact-actions">
            <button class="icon-btn small" onclick="event.stopPropagation(); startChat('${contact.id}')" title="Chat">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
            </button>
            <button class="icon-btn small" onclick="event.stopPropagation(); sendEmail('${contact.email}')" title="Email">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                    <polyline points="22,6 12,13 2,6"/>
                </svg>
            </button>
        </div>
    </div>`;
        }

        function renderEmptyState() {
            document.getElementById("contacts-list").innerHTML = `
        <div class="empty-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                <circle cx="9" cy="7" r="4"/>
                <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
            <h3 data-i18n="people-empty-title">No contacts yet</h3>
            <p data-i18n="people-empty-desc">Add your first contact to get started</p>
            <button class="btn btn-primary" onclick="openAddContact()">
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
                const letter = (contact.lastName || contact.firstName || "#")
                    .charAt(0)
                    .toUpperCase();
                if (!grouped[letter]) grouped[letter] = [];
                grouped[letter].push(contact);
            }
            return Object.fromEntries(Object.entries(grouped).sort());
        }

        function getInitials(firstName, lastName) {
            return (
                (
                    (firstName?.charAt(0) || "") + (lastName?.charAt(0) || "")
                ).toUpperCase() || "?"
            );
        }

        function getAvatarColor(name) {
            const colors = [
                "#6366f1",
                "#8b5cf6",
                "#ec4899",
                "#ef4444",
                "#f97316",
                "#eab308",
                "#22c55e",
                "#14b8a6",
                "#06b6d4",
                "#3b82f6",
            ];
            let hash = 0;
            for (let i = 0; i < name.length; i++) {
                hash = name.charCodeAt(i) + ((hash << 5) - hash);
            }
            return colors[Math.abs(hash) % colors.length];
        }

        function showTab(tabId, btn) {
            document
                .querySelectorAll(".tab-content")
                .forEach((tab) => tab.classList.remove("active"));
            document.querySelectorAll(".tab-btn").forEach((b) => {
                b.classList.remove("active");
                b.setAttribute("aria-selected", "false");
            });

            document.getElementById(tabId + "-tab").classList.add("active");
            btn.classList.add("active");
            btn.setAttribute("aria-selected", "true");
        }

        function filterByLetter(letter, btn) {
            document
                .querySelectorAll(".alpha-btn")
                .forEach((b) => b.classList.remove("active"));
            btn.classList.add("active");

            document.querySelectorAll(".contact-group").forEach((group) => {
                if (letter === "all" || group.dataset.letter === letter) {
                    group.style.display = "";
                } else {
                    group.style.display = "none";
                }
            });
        }

        function showContact(id) {
            currentContact = contacts.find((c) => c.id === id);
            if (!currentContact) return;

            const panel = document.getElementById("contact-panel");
            const detail = document.getElementById("contact-detail");

            detail.innerHTML = `
        <div class="contact-header">
            <div class="contact-avatar large" style="background: ${getAvatarColor(currentContact.firstName + " " + currentContact.lastName)}">
                ${getInitials(currentContact.firstName, currentContact.lastName)}
            </div>
            <h2>${currentContact.firstName} ${currentContact.lastName}</h2>
            ${currentContact.title ? `<p class="contact-title">${currentContact.title}</p>` : ""}
            ${currentContact.company ? `<p class="contact-company">${currentContact.company}</p>` : ""}
        </div>
        <div class="contact-fields">
            ${
                currentContact.email
                    ? `
                <div class="field">
                    <label>Email</label>
                    <a href="mailto:${currentContact.email}">${currentContact.email}</a>
                </div>
            `
                    : ""
            }
            ${
                currentContact.phone
                    ? `
                <div class="field">
                    <label>Phone</label>
                    <a href="tel:${currentContact.phone}">${currentContact.phone}</a>
                </div>
            `
                    : ""
            }
            ${
                currentContact.notes
                    ? `
                <div class="field">
                    <label>Notes</label>
                    <p>${currentContact.notes}</p>
                </div>
            `
                    : ""
            }
        </div>
        <div class="contact-quick-actions">
            <button class="action-btn" onclick="startChat('${currentContact.id}')">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
                Chat
            </button>
            <button class="action-btn" onclick="sendEmail('${currentContact.email}')">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                    <polyline points="22,6 12,13 2,6"/>
                </svg>
                Email
            </button>
            <button class="action-btn" onclick="scheduleMeeting('${currentContact.id}')">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                    <line x1="16" y1="2" x2="16" y2="6"/>
                    <line x1="8" y1="2" x2="8" y2="6"/>
                    <line x1="3" y1="10" x2="21" y2="10"/>
                </svg>
                Meeting
            </button>
        </div>
    `;

            panel.classList.add("open");
        }

        function closeContactPanel() {
            document.getElementById("contact-panel").classList.remove("open");
            currentContact = null;
        }

        function openAddContact() {
            currentContact = null;
            document.getElementById("modal-title").textContent = "Add Contact";
            document.getElementById("contact-form").reset();
            document.getElementById("contact-modal").showModal();
        }

        function editContact() {
            if (!currentContact) return;
            document.getElementById("modal-title").textContent = "Edit Contact";
            const form = document.getElementById("contact-form");
            form.firstName.value = currentContact.firstName || "";
            form.lastName.value = currentContact.lastName || "";
            form.email.value = currentContact.email || "";
            form.phone.value = currentContact.phone || "";
            form.company.value = currentContact.company || "";
            form.title.value = currentContact.title || "";
            form.notes.value = currentContact.notes || "";
            document.getElementById("contact-modal").showModal();
        }

        function closeModal() {
            document.getElementById("contact-modal").close();
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
                const url = currentContact
                    ? `/api/contacts/${currentContact.id}`
                    : "/api/contacts";
                const method = currentContact ? "PUT" : "POST";

                const response = await fetch(url, {
                    method,
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify(data),
                });

                if (response.ok) {
                    closeModal();
                    loadContacts();
                    if (currentContact) closeContactPanel();
                }
            } catch (error) {
                console.error("Failed to save contact:", error);
            }
        }

        async function deleteContact() {
            if (!currentContact || !confirm("Delete this contact?")) return;

            try {
                const response = await fetch(
                    `/api/contacts/${currentContact.id}`,
                    {
                        method: "DELETE",
                    },
                );
                if (response.ok) {
                    closeContactPanel();
                    loadContacts();
                }
            } catch (error) {
                console.error("Failed to delete contact:", error);
            }
        }

        function startChat(contactId) {
            window.location.href = `/#chat?contact=${contactId}`;
        }

        function sendEmail(email) {
            if (email) window.location.href = `mailto:${email}`;
        }

        function scheduleMeeting(contactId) {
            window.location.href = `/#calendar?new=meeting&contact=${contactId}`;
        }

        document
            .getElementById("people-search")
            ?.addEventListener("input", (e) => {
                const query = e.target.value.toLowerCase();
                const filtered = contacts.filter(
                    (c) =>
                        (c.firstName + " " + c.lastName)
                            .toLowerCase()
                            .includes(query) ||
                        (c.email || "").toLowerCase().includes(query) ||
                        (c.company || "").toLowerCase().includes(query),
                );
                renderContacts(filtered);
            });

        window.showTab = showTab;
        window.filterByLetter = filterByLetter;
        window.showContact = showContact;
        window.closeContactPanel = closeContactPanel;
        window.openAddContact = openAddContact;
        window.editContact = editContact;
        window.closeModal = closeModal;
        window.saveContact = saveContact;
        window.deleteContact = deleteContact;
        window.startChat = startChat;
        window.sendEmail = sendEmail;
        window.scheduleMeeting = scheduleMeeting;

        window.exportContacts = function() {
            var csv = 'First Name,Last Name,Email,Phone,Company,Title\n' + contacts.map(function(c) {
                return '"' + (c.firstName||'') + '","' + (c.lastName||'') + '","' + (c.email||'') + '","' + (c.phone||'') + '","' + (c.company||'') + '","' + (c.title||'') + '"';
            }).join('\n');
            var blob = new Blob([csv], { type: 'text/csv' });
            var a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = 'contacts-export.csv';
            a.click();
        };

        window.importContacts = function() {
            var input = document.createElement('input');
            input.type = 'file';
            input.accept = '.csv';
            input.onchange = function(e) {
                var file = e.target.files[0];
                var reader = new FileReader();
                reader.onload = async function(ev) {
                    var lines = ev.target.result.split('\n').filter(function(l) { return l.trim(); });
                    var headers = lines[0].split(',').map(function(h) { return h.replace(/"/g, '').trim().toLowerCase(); });
                    for (var i = 1; i < lines.length; i++) {
                        var cols = lines[i].split(',').map(function(c) { return c.replace(/"/g, '').trim(); });
                        var data = {};
                        headers.forEach(function(h, idx) {
                            if (h === 'first name' || h === 'firstname') data.firstName = cols[idx];
                            else if (h === 'last name' || h === 'lastname') data.lastName = cols[idx];
                            else if (h === 'email') data.email = cols[idx];
                            else if (h === 'phone') data.phone = cols[idx];
                            else if (h === 'company') data.company = cols[idx];
                            else if (h === 'title') data.title = cols[idx];
                        });
                        if (data.firstName || data.email) {
                            await fetch('/api/contacts', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify(data)
                            });
                        }
                    }
                    loadContacts();
                };
                reader.readAsText(file);
            };
            input.click();
        };

        window.advancedSearch = function() {
            var q = document.getElementById('people-search').value.toLowerCase();
            var company = (document.getElementById('filter-company') || {}).value || '';
            var status = (document.getElementById('filter-status') || {}).value || '';
            var filtered = contacts.filter(function(c) {
                var matchText = !q || (c.firstName + ' ' + c.lastName).toLowerCase().includes(q) || (c.email || '').toLowerCase().includes(q);
                var matchCompany = !company || (c.company || '').toLowerCase().includes(company.toLowerCase());
                return matchText && matchCompany;
            });
            renderContacts(filtered);
        };

        window.mergeContacts = function(id1, id2) {
            if (!confirm('Merge these contacts? The first contact will be kept.')) return;
            fetch('/api/contacts/' + id1 + '/merge', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ merge_into: id2 })
            }).then(function() { loadContacts(); });
        };
        window.deleteContact = deleteContact;
        window.startChat = startChat;
        window.sendEmail = sendEmail;
        window.scheduleMeeting = scheduleMeeting;
    })();
