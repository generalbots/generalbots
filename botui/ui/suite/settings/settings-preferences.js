    // Handle hash navigation on load
    document.addEventListener('DOMContentLoaded', function () {
        const hash = window.location.hash.substring(1);
        if (hash) {
            const navItem = document.querySelector(`a[href="#${hash}"]`);
            if (navItem) {
                showSection(hash, navItem);
            }
        }
    });

    // #1295 — Admin Panel deep-links (formerly /suite/admin/index.html) land
    // here with ?view=admin: open the first admin-only section after RBAC
    // has run. Admin entries may be hidden for non-admins, so resolve the
    // target at that point, not before.
    (function () {
        var params = new URLSearchParams(window.location.search);
        // #2013 — the `admin` deep link resolves to Settings with
        // `{ view: "admin" }` in `__gbAppParams__` (no query string on the
        // shell URL), so accept both sources.
        var appParams = window.__gbAppParams__ || {};
        if (params.get('view') === 'admin' || appParams.view === 'admin') {
            var openAdminView = function () {
                // Only land on an admin section that survived RBAC gating;
                // hidden entries (non-admin) fall back to Profile.
                var adminNav = Array.prototype.filter.call(
                    document.querySelectorAll('.nav-item[data-admin-only="true"]'),
                    function (n) { return n.offsetParent !== null; });
                var target = adminNav[0]
                    || document.querySelector('.nav-item[href="#profile"]');
                if (target) target.click();
            };
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', function () {
                    setTimeout(openAdminView, 0);
                });
            } else {
                setTimeout(openAdminView, 0);
            }
        }
    })();

    // Theme selection - handle both .theme-card and .theme-option elements
    document.querySelectorAll('.theme-card').forEach(card => {
        card.addEventListener('click', function () {
            const theme = this.dataset.theme;
            document.body.setAttribute('data-theme', theme);
            localStorage.setItem('gb-theme', theme);

            document.querySelectorAll('.theme-card').forEach(c => c.classList.remove('active'));
            this.classList.add('active');
        });
    });

    // Desktop wallpaper picker — the single implementation lives in
    // settings-main.js (WALLPAPERS/getWallpaper/applyWallpaper/setWallpaper/
    // initWallpapers, #1293). The inline copy that used to run here
    // overwrote window.setWallpaper with a version that skipped the
    // gb-wallpaper-changed event, so the desktop never repainted (#1339).

    // #1295 — RBAC gating for admin-only sections. Signal precedence:
    // gb-user roles in localStorage, then management_is_admin; refreshed
    // from /api/auth/me when available. The backend API still enforces
    // authorization — this only hides navigation.
    function currentUserIsAdmin() {
        try {
            var raw = localStorage.getItem('gb-user');
            if (raw) {
                var u = JSON.parse(raw);
                if (u && u.roles && u.roles.length) {
                    return u.roles.some(function (r) {
                        var rl = (r || '').toLowerCase();
                        return rl.indexOf('admin') !== -1 || rl.indexOf('super') !== -1;
                    });
                }
            }
        } catch (e) {}
        return localStorage.getItem('management_is_admin') === 'true';
    }

    function applySettingsRbac() {
        var admin = currentUserIsAdmin();
        document.querySelectorAll('[data-admin-only="true"]').forEach(function (el) {
            el.style.display = admin ? '' : 'none';
        });
        if (!admin) {
            var active = document.querySelector('.settings-section.active');
            if (active && active.dataset.adminOnly === 'true') {
                var first = document.querySelector('.nav-item[href="#profile"]');
                if (first) first.click();
            }
        }
    }

    applySettingsRbac();
    fetch('/api/auth/me')
        .then(function (r) { return r.ok ? r.json() : null; })
        .then(function (me) {
            var u = me && (me.user || me);
            if (u && u.roles && u.roles.length) {
                try {
                    var stored = {};
                    try { stored = JSON.parse(localStorage.getItem('gb-user') || '{}') || {}; } catch (e) {}
                    stored.roles = u.roles;
                    localStorage.setItem('gb-user', JSON.stringify(stored));
                } catch (e) {}
                applySettingsRbac();
            }
        })
        .catch(function () {});

    // Live-update desktop wallpaper when changed in another tab
    window.addEventListener('storage', function (e) {
        if (e.key === 'gb-wallpaper' && e.newValue) {
            applyWallpaper(e.newValue);
        }
    });

    // Versions tab - load when shown
    var versionsLoaded = false;
    window.loadVersions = function () {
      if (versionsLoaded) return;
      var container = document.getElementById("versionsTableContainer");
      if (!container) return;
      versionsLoaded = true;
      fetch("/api/system/versions")
        .then(function (r) {
          if (!r.ok) throw new Error("HTTP " + r.status);
          return r.json();
        })
        .then(function (versions) {
          var comps = [
            { id: "botserver", name: "BotServer" },
            { id: "botui", name: "BotUI" },
            { id: "rust", name: "Rust" },
            { id: "postgresql", name: "PostgreSQL" },
            { id: "valkey", name: "Valkey / Redis" },
            { id: "minio", name: "MinIO" },
            { id: "qdrant", name: "Qdrant" },
            { id: "vault", name: "Vault" },
            { id: "zitadel", name: "Zitadel" }
          ];
          var html =
            '<table style="width:100%;border-collapse:collapse">' +
            '<thead><tr style="text-align:left;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.5px;color:var(--text-secondary,#888);border-bottom:1px solid var(--border,#2a2a2a)">' +
            '<th style="padding:14px 16px">Component</th>' +
            '<th style="padding:14px 16px">Version</th></tr></thead><tbody>';
          comps.forEach(function (c) {
            var ver = versions[c.id] || "N/A";
            html +=
              '<tr style="border-bottom:1px solid var(--border,#2a2a2a)">' +
              '<td style="padding:14px 16px;font-size:14px;color:var(--text,#e5e5e5)">' +
              escapeHtml(c.name) + "</td>" +
              '<td style="padding:14px 16px;font-size:14px;font-family:\'Fira Code\',monospace;color:var(--text-secondary,#aaa)">' +
              escapeHtml(ver) + "</td></tr>";
          });
          html += "</tbody></table>";
          container.innerHTML = html;
        })
        .catch(function () {
          container.innerHTML =
            '<div style="padding:24px;text-align:center;color:var(--text-secondary,#888)">' +
            "<p>Could not load version information.</p>" +
            '<button class="btn-secondary btn-sm" onclick="window.loadVersions()">Retry</button></div>';
        });
    };

    // Override showSection to trigger versions loading
    var origShowSection = window.showSection;
    window.showSection = function (sectionId, navElement) {
      if (typeof origShowSection === "function") {
        origShowSection(sectionId, navElement);
      }
      if (sectionId === "versions") {
        setTimeout(window.loadVersions, 100);
      }
    };

    // Initialize theme options - mark current theme as active
    (function initThemeOptions() {
        const savedTheme = localStorage.getItem('gb-theme') || 'sentient';

        // Mark the active theme option
        document.querySelectorAll('.theme-option').forEach(opt => {
            if (opt.dataset.theme === savedTheme) {
                opt.classList.add('active');
            } else {
                opt.classList.remove('active');
            }
        });

        // Also mark theme-card if present
        document.querySelectorAll('.theme-card').forEach(card => {
            if (card.dataset.theme === savedTheme) {
                card.classList.add('active');
            } else {
                card.classList.remove('active');
            }
        });
    })();

    // #1304 — the theme card is fed by the shared ThemeManager so Settings and
    // the sidebar combobox can never drift apart. Clicking an option applies
    // the theme immediately through the same code path the combobox uses.
    (function initSettingsThemeGrid() {
        function render() {
            var grid = document.getElementById('settings-theme-grid');
            var tm = window.ThemeManager;
            if (!grid || !tm || typeof tm.getAvailableThemes !== 'function') return false;

            var themes = tm.getAvailableThemes() || [];
            var active = document.documentElement.getAttribute('data-theme') || '';
            grid.innerHTML = '';
            themes.forEach(function (theme) {
                var id = theme.id || theme.name || '';
                var label = theme.label || theme.name || id;
                var btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'theme-option' + (id === active ? ' active' : '');
                btn.setAttribute('data-theme', id);
                btn.innerHTML = '<span>' + label + '</span>';
                btn.addEventListener('click', function () {
                    tm.loadTheme(id);
                    Array.prototype.forEach.call(grid.children, function (other) {
                        other.classList.toggle('active', other === btn);
                    });
                });
                grid.appendChild(btn);
            });
            return true;
        }

        if (!render()) {
            var tries = 0;
            var timer = setInterval(function () {
                tries += 1;
                if (render() || tries > 20) clearInterval(timer);
            }, 150);
        }
        // Keep the active highlight in sync when the theme changes elsewhere.
        window.addEventListener('gb:themechange', function () {
            var grid = document.getElementById('settings-theme-grid');
            if (!grid) return;
            var active = document.documentElement.getAttribute('data-theme') || '';
            Array.prototype.forEach.call(grid.children, function (btn) {
                btn.classList.toggle('active', btn.getAttribute('data-theme') === active);
            });
        });
    })();

    // Load translation script and translate page
    (function loadTranslations() {
        var script = document.createElement('script');
        script.src = '/suite/js/translations.js';
        script.onload = function () {
            if (window.gbTranslations) {
                window.gbTranslations.translatePage();
            }
        };
        document.head.appendChild(script);
    })();
