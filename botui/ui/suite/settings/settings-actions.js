    // #1304 — Integrations lives in its own app window; Settings just opens
    // it (window manager when available, desktop route otherwise).
    window.openIntegrationsWindow = function () {
        // #1884 — the Integrations app owns the MCP catalog; ask it for the
        // Sources tab directly so the deep link lands where the work happens.
        var params = { tab: 'sources' };
        if (window.WindowManager && typeof window.WindowManager.openDeepLink === 'function') {
            window.WindowManager.openDeepLink('integrations', params);
        } else if (window.openDeepLink) {
            window.openDeepLink('integrations', params);
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
        // #2133 — `/api/auth/directory-config` is no longer anonymous: it
        // exposes the identity-provider URL, so the request must carry the
        // suite token like every other authenticated read.
        var token = "";
        try {
            token = localStorage.getItem("gb-access-token") ||
                sessionStorage.getItem("gb-access-token") ||
                localStorage.getItem("management_token") ||
                "";
        } catch (e) { token = ""; }
        var headers = token ? { Authorization: "Bearer " + token } : {};
        fetch('/api/auth/directory-config', { headers: headers, cache: 'no-store' })
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (cfg) {
                if (cfg && cfg.issuer_url) {
            window.__gbDirectoryIssuer = String(cfg.issuer_url).replace(/\/$/, '');
            // #1887 — directory-backed deployments own password/2FA. The local
            // password form would mutate a credential the directory never sees.
            var passwordCard = document.getElementById('security-password-card');
            if (passwordCard) passwordCard.style.display = 'none';
        }
            })
            .catch(function () { /* link falls back below */ });
    })();
    window.openZitadelConsole = function () {
        var target = window.__gbDirectoryIssuer
            ? window.__gbDirectoryIssuer + (window.__gbDirectoryIssuer.indexOf('/ui') === -1 ? '/ui/console' : '')
            : null;
        if (target) {
            window.open(target, '_blank', 'noopener');
        } else {
            // #1887 — no issuer means the directory address is unavailable.
            // Never deep-link personal security into Administration; say what
            // happened and leave the button inert.
            if (typeof showToast === 'function') {
                showToast('Directory address unavailable', 'error');
            }
            console.warn('openZitadelConsole: no directory issuer configured');
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

    // #1887 — clearing the local cache: shell confirmation when available,
    // otherwise the action runs with a toast. No window.confirm.
    window.clearLocalCache = function () {
        function clear() {
            try {
                Object.keys(localStorage)
                    .filter(function (k) { return k.indexOf('gb_') === 0 || k.indexOf('gb-') === 0; })
                    .forEach(function (k) { localStorage.removeItem(k); });
                showToast('Local cache cleared');
            } catch (e) {
                showToast('Could not clear the cache', 'error');
            }
        }
        if (window.WindowManager && typeof window.WindowManager.confirmFloating === 'function') {
            window.WindowManager.confirmFloating(
                'Clear local cache',
                'Remove cached settings and drafts from this browser profile?',
                clear
            );
            return;
        }
        clear();
    };

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

    // #1888 — Language & region owns the only timezone control. The select is
    // populated from the runtime IANA list (with the shipped options as the
    // fallback), and the profile form mirrors the same value in a hidden field.
    var TIMEZONE_FALLBACKS = [
        'America/Sao_Paulo', 'America/New_York', 'America/Chicago',
        'America/Denver', 'America/Los_Angeles', 'Europe/London',
        'Europe/Paris', 'Europe/Berlin', 'Asia/Tokyo', 'Asia/Shanghai',
        'Australia/Sydney', 'UTC'
    ];

    function timezoneList() {
        try {
            if (typeof Intl.supportedValuesOf === 'function') {
                var zones = Intl.supportedValuesOf('timeZone');
                if (Array.isArray(zones) && zones.length) return zones;
            }
        } catch (e) { /* older runtimes keep the fallback list */ }
        return TIMEZONE_FALLBACKS;
    }

    function fillTimezoneSelect() {
        var select = document.getElementById('timezone-select');
        if (!select || select.getAttribute('data-filled') === '1') return;
        var saved = localStorage.getItem('gb-timezone') || 'UTC';
        var zones = timezoneList();
        if (zones.indexOf(saved) === -1) zones = zones.concat([saved]);
        select.innerHTML = zones.map(function (zone) {
            return '<option value="' + zone + '">' + zone + '</option>';
        }).join('');
        select.value = saved;
        select.setAttribute('data-filled', '1');
    }

    function syncProfileTimezone() {
        var hidden = document.getElementById('profile-timezone');
        if (!hidden) return;
        var saved = localStorage.getItem('gb-timezone');
        if (saved) hidden.value = saved;
    }

    function restoreLocaleSelects() {
        fillTimezoneSelect();
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
        syncProfileTimezone();
    }

    window.toggleCompactMode = function (checkbox) {
        persistPref('gb-compact-mode', checkbox.checked);
        document.body.classList.toggle('compact-mode', checkbox.checked);
        showToast(checkbox.checked ? 'Compact mode enabled' : 'Compact mode disabled');
        window.dispatchEvent(new CustomEvent('gb-compact-mode-changed', {
            detail: { compact: checkbox.checked },
        }));
    };
    window.addEventListener('gb-compact-mode-changed', function (e) {
        var enabled = !!(e && e.detail && e.detail.compact);
        var input = document.querySelector('[name="compact_mode"]');
        if (input) input.checked = enabled;
        document.body.classList.toggle('compact-mode', enabled);
    });

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
        // Keep the profile form's hidden field on the same value so the next
        // profile save writes the regional choice to the server (#1888).
        syncProfileTimezone();
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

    // #1821 — a failed API-key/webhook request used to leave the panel looking
    // empty (the 401 body has no list markup). Show the real status and an
    // actionable message instead.
    window.gbSettingsRequestError = function (target, event) {
        if (!target || !event || !event.detail) return;
        var status = event.detail.xhr ? event.detail.xhr.status : 0;
        var message = status === 401
            ? 'Your session is not authorized for this request. Sign in again from the desktop and retry.'
            : 'Request failed' + (status ? ' (HTTP ' + status + ')' : '') + '. Please retry.';
        target.innerHTML = '<div class="error-state"><p>' + message + '</p></div>';
    };
    if (!window.__gbSettingsRequestErrorBound) {
        window.__gbSettingsRequestErrorBound = true;
        document.addEventListener('htmx:responseError', function (event) {
            var path = event.detail && event.detail.pathInfo
                ? event.detail.pathInfo.requestPath
                : '';
            if (!/^\/api\/(user|admin)\//.test(path)) return;
            var target = event.detail && event.detail.target;
            if (target) window.gbSettingsRequestError(target, event);
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initPreferences);
    } else {
        initPreferences();
    }
    // Settings sections arrive through HTMX; re-restore when the profile or
    // regional partials are swapped in (#1888).
    document.addEventListener('htmx:afterSwap', function () {
        fillTimezoneSelect();
        syncProfileTimezone();
    });
