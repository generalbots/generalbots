    (function () {
        if (window.GBAppLifecycle) GBAppLifecycle.begin("people");
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

        function applyLetter(list) {
            if (!state.activeLetter || state.activeLetter === "all") return list;
            return (list || []).filter(function (c) {
                var ch = (c.lastName || c.firstName || "#").charAt(0).toUpperCase();
                return ch === state.activeLetter;
            });
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

        window.__peopleScope = {
            state: state, API_BASE: API_BASE, apiFetch: apiFetch,
            toSnakeBody: toSnakeBody, normalizeContact: normalizeContact,
            applyFilters: applyFilters, applyLetter: applyLetter,
            escapeHtml: escapeHtml, escapeAttr: escapeAttr,
            groupByLetter: groupByLetter, getInitials: getInitials,
            getAvatarColor: getAvatarColor,
            openDeepLinkedContact: openDeepLinkedContact,
        };
    })();
