(function () {
        const scope = window.__peopleScope;
        const state = scope.state;
        const loadContacts = scope.loadContacts;
        const renderContacts = scope.renderContacts;
        const applyFilters = scope.applyFilters;
        const applyLetter = scope.applyLetter;
        const escapeHtml = scope.escapeHtml;
        const openDeepLinkedContact = scope.openDeepLinkedContact;

        function init() {
            bindAdvancedSearch();
            bindSearch();
            loadContacts();
            // Deep-link support: open the app contextualized to a person when the
            // desktop shell opened us via app://people?person_id=...
            const params = window.__gbAppParams__ || {};
            if (params.person_id) {
                openDeepLinkedContact(params.person_id);
            }
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

        function bindSearch() {
            var input = document.getElementById("people-search");
            if (input) {
                input.addEventListener("input", function () {
                    state.filters.name = (input.value || "").toLowerCase();
                    renderContacts(applyLetter(applyFilters(state.contacts)));
                });
            }
        }

        async function apiGetPeople(path) {
            var token = localStorage.getItem("gb_token");
            var resp = await fetch("/api/people" + path, {
                headers: { "Authorization": "Bearer " + (token || ""), "Content-Type": "application/json" },
            });
            if (!resp.ok) throw new Error("HTTP " + resp.status);
            var ct = resp.headers.get("content-type") || "";
            return ct.includes("application/json") ? await resp.json() : null;
        }

        function showTab(tab, btn) {
            document.querySelectorAll(".tab-btn").forEach(function (b) {
                b.classList.remove("active");
                b.setAttribute("aria-selected", "false");
            });
            document.querySelectorAll(".tab-content").forEach(function (c) {
                c.classList.remove("active");
            });
            var content = document.getElementById(tab + "-tab");
            if (content) content.classList.add("active");
            if (btn) {
                btn.classList.add("active");
                btn.setAttribute("aria-selected", "true");
            }
            if (tab === "contacts") {
                loadContacts();
            } else if (tab === "groups") {
                loadGroups();
            } else if (tab === "directory") {
                loadDirectory();
            } else if (tab === "recent") {
                loadRecent();
            }
        }

        function filterByLetter(letter, btn) {
            document.querySelectorAll(".alpha-btn").forEach(function (b) {
                b.classList.remove("active");
            });
            if (btn) btn.classList.add("active");
            state.activeLetter = letter;
            renderContacts(applyLetter(applyFilters(state.contacts)));
        }

        async function loadGroups() {
            var el = document.getElementById("groups-list");
            if (!el) return;
            try {
                var teams = await apiGetPeople("/teams");
                teams = Array.isArray(teams) ? teams : [];
                if (!teams.length) {
                    el.innerHTML = '<div class="empty-state"><p>No groups yet</p></div>';
                    return;
                }
                el.innerHTML = teams.map(function (t) {
                    var color = t.color || "#4ECDC4";
                    return '<div class="group-card" style="border-left:4px solid ' + color + '">' +
                        '<div class="group-name">' + escapeHtml(t.name || "") + "</div>" +
                        '<div class="group-desc">' + escapeHtml(t.description || "") + "</div>" +
                        "</div>";
                }).join("");
            } catch (e) {
                el.innerHTML = '<div class="empty-state"><p>Unable to load groups</p></div>';
            }
        }

        async function loadDirectory() {
            var el = document.getElementById("directory-tree");
            if (!el) return;
            try {
                var people = await apiGetPeople("");
                people = Array.isArray(people) ? people : [];
                if (!people.length) {
                    el.innerHTML = '<div class="empty-state"><p>No directory entries</p></div>';
                    return;
                }
                el.innerHTML = people.map(function (p) {
                    var name = ((p.first_name || "") + " " + (p.last_name || "")).trim();
                    return '<div class="directory-item">' +
                        '<div class="directory-name">' + escapeHtml(name) + "</div>" +
                        '<div class="directory-meta">' + escapeHtml(p.job_title || "") + " • " + escapeHtml(p.department || "") + "</div>" +
                        "</div>";
                }).join("");
            } catch (e) {
                el.innerHTML = '<div class="empty-state"><p>Unable to load directory</p></div>';
            }
        }

        async function loadRecent() {
            var el = document.getElementById("recent-list");
            if (!el) return;
            try {
                var people = await apiGetPeople("");
                people = Array.isArray(people) ? people : [];
                people.sort(function (a, b) {
                    return new Date(b.created_at || 0) - new Date(a.created_at || 0);
                });
                people = people.slice(0, 10);
                if (!people.length) {
                    el.innerHTML = '<div class="empty-state"><p>No recent contacts</p></div>';
                    return;
                }
                el.innerHTML = people.map(function (p) {
                    var name = ((p.first_name || "") + " " + (p.last_name || "")).trim();
                    return '<div class="recent-item">' +
                        '<div class="recent-name">' + escapeHtml(name) + "</div>" +
                        '<div class="recent-meta">' + escapeHtml(p.email || "") + "</div>" +
                        "</div>";
                }).join("");
            } catch (e) {
                el.innerHTML = '<div class="empty-state"><p>Unable to load recent contacts</p></div>';
            }
        }

        if (document.readyState === "loading") {
            (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
        } else {
            init();
        }

        window.showTab = showTab;
        window.filterByLetter = filterByLetter;
        Object.assign(window.__peopleScope, {
            init: init, showTab: showTab, filterByLetter: filterByLetter,
            bindAdvancedSearch: bindAdvancedSearch, bindSearch: bindSearch,
        });
    })();
