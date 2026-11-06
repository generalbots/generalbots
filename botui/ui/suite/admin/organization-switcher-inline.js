        const OrganizationSwitcher = {
            currentOrg: null,
            organizations: [],
            isOpen: false,

            init() {
                this.bindEvents();
                this.loadOrganizations();
            },

            bindEvents() {
                const trigger = document.getElementById('org-trigger');
                const dropdown = document.getElementById('org-dropdown');
                const searchInput = document.getElementById('org-search');
                const createBtn = document.getElementById('create-org-btn');
                const modal = document.getElementById('create-org-modal');
                const closeModal = document.getElementById('close-modal');
                const cancelCreate = document.getElementById('cancel-create');
                const createForm = document.getElementById('create-org-form');
                const newOrgName = document.getElementById('new-org-name');
                const newOrgSlug = document.getElementById('new-org-slug');
                const slugPreview = document.getElementById('slug-preview');

                trigger.addEventListener('click', () => this.toggleDropdown());

                document.addEventListener('click', (e) => {
                    if (!e.target.closest('#org-switcher') && this.isOpen) {
                        this.closeDropdown();
                    }
                });

                document.addEventListener('keydown', (e) => {
                    if (e.key === 'Escape' && this.isOpen) {
                        this.closeDropdown();
                    }
                });

                searchInput.addEventListener('input', (e) => this.filterOrganizations(e.target.value));

                createBtn.addEventListener('click', () => this.openCreateModal());
                closeModal.addEventListener('click', () => this.closeCreateModal());
                cancelCreate.addEventListener('click', () => this.closeCreateModal());

                modal.querySelector('.modal-overlay').addEventListener('click', () => this.closeCreateModal());

                newOrgName.addEventListener('input', (e) => {
                    const slug = this.generateSlug(e.target.value);
                    newOrgSlug.value = slug;
                    slugPreview.textContent = slug || 'organization-slug';
                });

                newOrgSlug.addEventListener('input', (e) => {
                    slugPreview.textContent = e.target.value || 'organization-slug';
                });

                createForm.addEventListener('submit', (e) => this.handleCreateOrg(e));
            },

            async loadOrganizations() {
                const loading = document.getElementById('org-loading');
                const list = document.getElementById('org-list');

                try {
                    const response = await fetch('/api/organizations');
                    if (response.ok) {
                        this.organizations = await response.json();
                    } else {
                        this.organizations = [];
                    }
                } catch {
                    this.organizations = [];
                }

                loading.style.display = 'none';

                if (this.organizations.length === 0) {
                    list.innerHTML = `
                        <div class="org-empty">
                            <div class="org-empty-icon">⚠️</div>
                            <p>Unable to load organizations</p>
                        </div>
                    `;
                    return;
                }

                this.renderOrganizations(this.organizations);

                const storedOrgId = localStorage.getItem('currentOrganizationId');
                const currentOrg = this.organizations.find(o => o.id === storedOrgId) || this.organizations[0];
                if (currentOrg) {
                    this.selectOrganization(currentOrg, false);
                }
            },

            renderOrganizations(orgs) {
                const list = document.getElementById('org-list');

                if (orgs.length === 0) {
                    list.innerHTML = `
                        <div class="org-empty">
                            <div class="org-empty-icon">🏢</div>
                            <p>No organizations found</p>
                        </div>
                    `;
                    return;
                }

                list.innerHTML = orgs.map(org => `
                    <div class="org-list-item ${this.currentOrg?.id === org.id ? 'selected' : ''}"
                         data-org-id="${org.id}"
                         onclick="OrganizationSwitcher.selectOrganization(${JSON.stringify(org).replace(/"/g, '&quot;')})">
                        <div class="org-item-avatar ${org.isPersonal ? 'personal' : ''}">
                            ${org.isPersonal ? '👤' : (org.avatar || org.name.charAt(0).toUpperCase())}
                        </div>
                        <div class="org-item-info">
                            <div class="org-item-name">${this.escapeHtml(org.name)}</div>
                            <div class="org-item-meta">
                                <span class="org-item-role ${org.role}">${org.role}</span>
                                <span>${org.memberCount} member${org.memberCount !== 1 ? 's' : ''}</span>
                            </div>
                        </div>
                        <svg class="org-item-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="20,6 9,17 4,12"></polyline>
                        </svg>
                    </div>
                `).join('');
            },

            filterOrganizations(query) {
                const filtered = this.organizations.filter(org =>
                    org.name.toLowerCase().includes(query.toLowerCase()) ||
                    org.slug.toLowerCase().includes(query.toLowerCase())
                );
                this.renderOrganizations(filtered);
            },

            selectOrganization(org, closeAfter = true) {
                this.currentOrg = org;
                localStorage.setItem('currentOrganizationId', org.id);

                document.getElementById('current-org-avatar').textContent =
                    org.isPersonal ? '👤' : (org.avatar || org.name.charAt(0).toUpperCase());
                document.getElementById('current-org-avatar').className =
                    'org-avatar' + (org.isPersonal ? ' personal' : '');
                document.getElementById('current-org-name').textContent = org.name;
                document.getElementById('current-org-plan').textContent = org.plan + ' Plan';

                document.querySelectorAll('.org-list-item').forEach(item => {
                    item.classList.toggle('selected', item.dataset.orgId === org.id);
                });

                if (closeAfter) {
                    this.closeDropdown();
                }

                window.dispatchEvent(new CustomEvent('organizationChanged', {
                    detail: { organization: org }
                }));
            },

            toggleDropdown() {
                if (this.isOpen) {
                    this.closeDropdown();
                } else {
                    this.openDropdown();
                }
            },

            openDropdown() {
                const trigger = document.getElementById('org-trigger');
                const dropdown = document.getElementById('org-dropdown');

                trigger.classList.add('active');
                trigger.setAttribute('aria-expanded', 'true');
                dropdown.classList.add('show');
                this.isOpen = true;

                document.getElementById('org-search').value = '';
                this.renderOrganizations(this.organizations);
                document.getElementById('org-search').focus();
            },

            closeDropdown() {
                const trigger = document.getElementById('org-trigger');
                const dropdown = document.getElementById('org-dropdown');

                trigger.classList.remove('active');
                trigger.setAttribute('aria-expanded', 'false');
                dropdown.classList.remove('show');
                this.isOpen = false;
            },

            openCreateModal() {
                this.closeDropdown();
                document.getElementById('create-org-modal').style.display = 'flex';
                document.getElementById('new-org-name').focus();
            },

            closeCreateModal() {
                document.getElementById('create-org-modal').style.display = 'none';
                document.getElementById('create-org-form').reset();
                document.getElementById('slug-preview').textContent = 'organization-slug';
            },

            generateSlug(name) {
                return name
                    .toLowerCase()
                    .replace(/[^a-z0-9\s-]/g, '')
                    .replace(/\s+/g, '-')
                    .replace(/-+/g, '-')
                    .trim();
            },

            escapeHtml(text) {
                const div = document.createElement('div');
                div.textContent = text;
                return div.innerHTML;
            },

            async handleCreateOrg(e) {
                e.preventDefault();

                const name = document.getElementById('new-org-name').value;
                const slug = document.getElementById('new-org-slug').value;
                const plan = document.getElementById('new-org-plan').value;

                const newOrg = {
                    id: 'org-' + Date.now(),
                    name,
                    slug,
                    plan: plan.charAt(0).toUpperCase() + plan.slice(1),
                    role: 'owner',
                    avatar: name.charAt(0).toUpperCase(),
                    isPersonal: false,
                    memberCount: 1
                };

                try {
                    const response = await fetch('/api/organizations', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ name, slug, plan })
                    });

                    if (response.ok) {
                        const created = await response.json();
                        this.organizations.push(created);
                        this.selectOrganization(created);
                    } else {
                        this.organizations.push(newOrg);
                        this.selectOrganization(newOrg);
                    }
                } catch {
                    this.organizations.push(newOrg);
                    this.selectOrganization(newOrg);
                }

                this.closeCreateModal();
            }
        };

        document.addEventListener('DOMContentLoaded', () => OrganizationSwitcher.init());

        window.OrganizationSwitcher = OrganizationSwitcher;
    
