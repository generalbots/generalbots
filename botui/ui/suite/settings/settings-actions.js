    // #1304 — Integrations lives in its own app window; Settings just opens
    // it (window manager when available, desktop route otherwise).
    window.openIntegrationsWindow = function () {
        if (window.WindowManager && typeof window.WindowManager.openDeepLink === 'function') {
            window.WindowManager.openDeepLink('integrations', {});
        } else if (window.openDeepLink) {
            window.openDeepLink('integrations', {});
        } else {
            window.open('/suite/desktop.html?app=integrations', '_blank', 'noopener');
        }
        return false;
    };

    // About opens in its own window — same surface as the desktop About app.
    // The old raw hx-get hijacked #main-content (a target that does not exist
    // inside the Settings dialog) and blanked the whole dialog.
    window.openAboutWindow = function () {
        if (window.WindowManager && typeof window.WindowManager.openDeepLink === 'function') {
            window.WindowManager.openDeepLink('about', {});
        } else if (window.openDeepLink) {
            window.openDeepLink('about', {});
        } else {
            window.open('/suite/desktop.html?app=about', '_blank', 'noopener');
        }
        return false;
    };

    // #1546 — Profile/Security are directory-backed: link the user to the
    // Zitadel directory (console) for their identity, password, 2FA and
    // sessions. The issuer URL comes from the backend; without it the link
    // falls back to the local directory API so the button never dead-ends.
    window.__gbDirectoryIssuer = null;
    (function () {
        fetch('/api/auth/directory-config')
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (cfg) {
                if (cfg && cfg.issuer_url) window.__gbDirectoryIssuer = String(cfg.issuer_url).replace(/\/$/, '');
            })
            .catch(function () { /* link falls back below */ });
    })();
    window.openZitadelConsole = function () {
        var target = window.__gbDirectoryIssuer
            ? window.__gbDirectoryIssuer + (window.__gbDirectoryIssuer.indexOf('/ui') === -1 ? '/ui/console' : '')
            : null;
        if (target) {
            window.open(target, '_blank', 'noopener');
        } else if (window.WindowManager && typeof window.WindowManager.openDeepLink === 'function') {
            window.WindowManager.openDeepLink('admin', {});
        } else {
            window.open('/suite/settings/index.html?view=admin', '_blank', 'noopener');
        }
        return false;
    };

    function showSection(sectionId, element) {
        // Hide all sections
        document.querySelectorAll('.settings-section').forEach(section => {
            section.classList.remove('active');
        });

        // Show selected section
        document.getElementById(sectionId + '-section').classList.add('active');

        // Update nav items
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active');
        });
        element.classList.add('active');

        // Close mobile sidebar
        if (window.innerWidth <= 768) {
            toggleSettingsSidebar();
        }

        return false;
    }

    function toggleSettingsSidebar() {
        const sidebar = document.querySelector('.settings-sidebar');
        const overlay = document.getElementById('sidebar-overlay');
        sidebar.classList.toggle('open');
        overlay.classList.toggle('show');
    }

    function showToast(message, type = 'success') {
        if (window.showNotification) {
            window.showNotification(message, type);
        } else {
            const toast = document.createElement('div');
            toast.className = 'toast-notification toast-' + type;
            toast.textContent = message;
            toast.style.cssText = 'position:fixed;top:20px;right:20px;padding:12px 24px;background:#333;color:#fff;border-radius:8px;z-index:10000;animation:fadeIn 0.3s';
            document.body.appendChild(toast);
            setTimeout(() => toast.remove(), 3000);
        }
    }

    // Layout, locale and notification preferences. Every control persists to
    // localStorage so the choice survives reloads; the desktop clock reads the
    // date/time keys through the shared GBDateTime helper (#1339).
    function persistPref(key, value) {
        try {
            localStorage.setItem(key, String(value));
        } catch (e) {}
    }

    var PREFERENCE_KEYS = {
        compact_mode: 'gb-compact-mode',
        show_sidebar: 'gb-show-sidebar',
        animations: 'gb-animations',
        push_enabled: 'gb-push-enabled',
        push_sound: 'gb-push-sound',
        desktop_notifications: 'gb-desktop-notifications',
        badge_count: 'gb-badge-count'
    };

    function applyPreferenceToggles() {
        Object.keys(PREFERENCE_KEYS).forEach(function (name) {
            var input = document.querySelector('[name="' + name + '"]');
            if (!input) return;
            var saved = localStorage.getItem(PREFERENCE_KEYS[name]);
            if (saved === null) return;
            input.checked = saved === 'true';
        });
        document.body.classList.toggle('compact-mode', localStorage.getItem('gb-compact-mode') === 'true');
        document.body.classList.toggle('no-animations', localStorage.getItem('gb-animations') === 'false');
        document.body.classList.toggle('sidebar-hidden', localStorage.getItem('gb-show-sidebar') === 'false');
    }

    function restoreLocaleSelects() {
        var selects = {
            'date-format-select': 'gb-date-format',
            'time-format-select': 'gb-time-format',
            'timezone-select': 'gb-timezone',
            'first-day-select': 'gb-first-day'
        };
        Object.keys(selects).forEach(function (id) {
            var select = document.getElementById(id);
            var saved = localStorage.getItem(selects[id]);
            if (select && saved) select.value = saved;
        });
    }

    window.toggleCompactMode = function (checkbox) {
        persistPref('gb-compact-mode', checkbox.checked);
        document.body.classList.toggle('compact-mode', checkbox.checked);
        showToast(checkbox.checked ? 'Compact mode enabled' : 'Compact mode disabled');
    };

    window.toggleShowSidebar = function (checkbox) {
        persistPref('gb-show-sidebar', checkbox.checked);
        // #1433 — apply immediately to the desktop shell (the class is also
        // re-applied from localStorage on boot by restoreChatSidebar).
        document.body.classList.toggle('sidebar-hidden', !checkbox.checked);
        showToast(checkbox.checked ? 'Sidebar shown' : 'Sidebar hidden');
        window.dispatchEvent(new CustomEvent('gb-sidebar-visibility-changed', {
            detail: { show: checkbox.checked },
        }));
    };

    window.addEventListener('gb-sidebar-visibility-changed', function (e) {
        var show = !!(e && e.detail && e.detail.show);
        var input = document.querySelector('[name="show_sidebar"]');
        if (input) input.checked = show;
    });

    window.toggleAnimations = function (checkbox) {
        persistPref('gb-animations', checkbox.checked);
        document.body.classList.toggle('no-animations', !checkbox.checked);
        showToast(checkbox.checked ? 'Animations enabled' : 'Animations disabled');
    };

    window.changeDateFormat = function (format) {
        persistPref('gb-date-format', format);
        showToast('Date format updated');
    };

    window.changeTimeFormat = function (format) {
        persistPref('gb-time-format', format);
        showToast('Time format updated');
    };

    window.changeTimezone = function (tz) {
        if (!tz) return;
        persistPref('gb-timezone', tz);
        showToast('Timezone updated');
    };

    window.changeFirstDay = function (day) {
        if (!day) return;
        persistPref('gb-first-day', day);
        showToast('First day of week updated');
    };

    window.togglePushNotifications = function (checkbox) {
        persistPref('gb-push-enabled', checkbox.checked);
        if (checkbox.checked && window.Notification && Notification.permission === 'default') {
            Notification.requestPermission();
        }
        showToast(checkbox.checked ? 'Push notifications enabled' : 'Push notifications disabled');
    };

    // #1816 — Desktop notifications toggle: persists the preference and
    // exposes a helper other modules can consult before firing a Notification.
    window.toggleDesktopNotifications = function (checkbox) {
        persistPref('gb-desktop-notifications', checkbox.checked);
        showToast(checkbox.checked ? 'Desktop notifications enabled' : 'Desktop notifications disabled');
    };

    /** Returns true when desktop notifications are allowed by the user
     *  preference.  Defaults to true when no preference has been saved. */
    window.gbDesktopNotificationsEnabled = function () {
        var stored = localStorage.getItem('gb-desktop-notifications');
        return stored === null || stored === 'true';
    };

    // #1816 — Badge count toggle: persists preference and forwards to the
    // Tauri set_badge_count API when running inside the desktop shell.
    window.toggleBadgeCount = function (checkbox) {
        persistPref('gb-badge-count', checkbox.checked);
        // When disabling, clear the badge immediately via Tauri if available.
        if (!checkbox.checked) {
            try {
                var tauri = window.__TAURI__ || window.__TAURI_INTERNALS__;
                if (tauri && tauri.invoke) {
                    tauri.invoke('set_badge_count', { count: 0 }).catch(function () {});
                }
            } catch (e) { /* not in Tauri shell */ }
        }
        showToast(checkbox.checked ? 'Badge count enabled' : 'Badge count disabled');
    };

    /** Returns true when badge count display is allowed by the user. */
    window.gbBadgeCountEnabled = function () {
        var stored = localStorage.getItem('gb-badge-count');
        return stored === null || stored === 'true';
    };

    /** Set the app badge count, respecting the user preference. */
    window.gbSetBadgeCount = function (count) {
        if (!window.gbBadgeCountEnabled()) return;
        try {
            var tauri = window.__TAURI__ || window.__TAURI_INTERNALS__;
            if (tauri && tauri.invoke) {
                tauri.invoke('set_badge_count', { count: count || 0 }).catch(function () {});
            }
        } catch (e) { /* not in Tauri shell */ }
    };

    // Cloud portal deep-link for Billing (plans, subscription, invoices live
    // on the cloud host, not in the suite) (#1339).
    window.openCloudBilling = function () {
        var base = window.GB_CLOUD_URL;
        if (!base) {
            base = (window.location.origin || '').replace(/:(3000|8080)$/, ':4000');
        }
        window.open(base.replace(/\/$/, '') + '/cloud/billing', '_blank', 'noopener');
        return false;
    };

    // Controls marked with `data-pref` persist themselves under that key: the
    // Privacy page uses this so its selects and toggles stop resetting on every
    // visit (#1339).
    function bindDataPrefs() {
        var controls = document.querySelectorAll('[data-pref]');
        Array.prototype.forEach.call(controls, function (control) {
            var key = control.getAttribute('data-pref');
            var saved = localStorage.getItem(key);
            if (saved !== null) {
                if (control.type === 'checkbox') {
                    control.checked = saved === 'true';
                } else {
                    control.value = saved;
                }
            }
            control.addEventListener('change', function () {
                var value = control.type === 'checkbox' ? control.checked : control.value;
                persistPref(key, value);
                showToast('Preference saved');
            });
        });
    }

    function initPreferences() {
        applyPreferenceToggles();
        restoreLocaleSelects();
        bindDataPrefs();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initPreferences);
    } else {
        initPreferences();
    }
