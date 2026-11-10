/* #2015 — Quotes & Orders tab.
 *
 * These two read-only lists were the only capability of the standalone "Sales"
 * app, which duplicated the CRM surface (same tabs, same title "Sales CRM").
 * The app is gone; its data lives here, behind the same `/api/sales/*`
 * endpoints and with the same column mapping the Sales page used. */
"use strict";
(function () {
    var loaded = false;

    function headers() {
        return window.authHeaders ? window.authHeaders() : {};
    }

    function esc(value) {
        return String(value == null ? '' : value).replace(/[&<>"']/g, function (c) {
            return ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c];
        });
    }

    function t(key, fallback) {
        return window.gbT ? window.gbT(key, fallback) : fallback;
    }

    // #2092 — money and dates follow the active locale; the currency comes
    // from the record (ISO payloads stay untouched).
    function money(value, currency) {
        var n = parseFloat(value);
        if (!isFinite(n)) return '-';
        if (window.i18n && window.i18n.formatCurrency) {
            return esc(window.i18n.formatCurrency(n, currency || 'USD'));
        }
        return esc(n.toFixed(2));
    }

    function day(value) {
        if (value == null || value === '') return '-';
        if (window.i18n && window.i18n.formatDate) {
            return esc(
                window.i18n.formatDate(value, {
                    year: 'numeric',
                    month: 'short',
                    day: 'numeric',
                }),
            );
        }
        return esc(String(value).slice(0, 10));
    }

    function items(data) {
        if (Array.isArray(data)) return data;
        if (data && Array.isArray(data.items)) return data.items;
        return [];
    }

    function fill(tbodyId, rows, emptyText, cols, render) {
        var tbody = document.getElementById(tbodyId);
        if (!tbody) return;
        if (!rows.length) {
            tbody.innerHTML = '<tr><td colspan="' + cols + '">' + emptyText + '</td></tr>';
            return;
        }
        tbody.innerHTML = rows.map(render).join('');
    }

    function loadOne(url, tbodyId, emptyText, errorText, cols, render) {
        return fetch(url, { headers: headers() })
            .then(function (res) {
                if (!res.ok) throw new Error('HTTP ' + res.status);
                return res.json();
            })
            .then(function (data) {
                fill(tbodyId, items(data), emptyText, cols, render);
            })
            .catch(function () {
                fill(tbodyId, [], errorText, cols, render);
                // Allow the next tab click to retry (the request may have
                // failed before the session was ready).
                loaded = false;
            });
    }

    function load() {
        if (loaded) return;
        loaded = true;
        loadOne('/api/sales/quotes', 'crm-quotes-body', t('crm-quotes-empty', 'No quotes yet'), t('crm-quotes-error', 'Could not load quotes.'), 6, function (q) {
            return '<tr>' +
                '<td>' + esc(q.quote_number || q.id) + '</td>' +
                '<td>' + esc(q.title || '-') + '</td>' +
                '<td>' + esc(q.customer || '-') + '</td>' +
                '<td>' + money(q.amount, q.currency) + '</td>' +
                '<td>' + day(q.valid_until) + '</td>' +
                '<td>' + esc(q.status || 'draft') + '</td>' +
                '</tr>';
        });
        loadOne('/api/sales/orders', 'crm-orders-body', t('crm-orders-empty', 'No orders yet'), t('crm-orders-error', 'Could not load orders.'), 6, function (o) {
            return '<tr>' +
                '<td>' + esc(o.order_number || o.id) + '</td>' +
                '<td>' + esc(o.customer || '-') + '</td>' +
                '<td>' + esc(o.items != null ? o.items : '-') + '</td>' +
                '<td>' + money(o.total, o.currency) + '</td>' +
                '<td>' + esc(o.status || 'pending') + '</td>' +
                '<td>' + day(o.delivery) + '</td>' +
                '</tr>';
        });
    }

    /* #2015 — deep-link entry (`openDeepLink("sales")` and old
       `/suite/sales/sales.html` bookmarks land on the CRM quotes tab). */
    function activate() {
        var tab = document.querySelector('.crm-tab[data-view="quotes"]');
        if (!tab) return;
        tab.click();
        load();
    }

    window.gbCrmQuotes = { load: load, activate: activate };

    // CRM is injected as an HTMX fragment, so DOMContentLoaded has already
    // fired by the time this module runs — wire up immediately.
    function wantedView() {
        var params = window.__gbAppParams__ || {};
        var view = params.view;
        if (!view) {
            try {
                view = new URLSearchParams(window.location.search).get('view');
            } catch (e) { view = null; }
        }
        if (!view && (params.record === 'quote' || params.record === 'order')) {
            view = 'quotes';
        }
        return view;
    }

    var tab = document.querySelector('.crm-tab[data-view="quotes"]');
    if (tab) tab.addEventListener('click', load);
    if (wantedView() === 'quotes') activate();

    document.addEventListener('gb:deep-link', function (event) {
        var params = (event && event.detail && event.detail.params) || {};
        if (params.view === 'quotes' || params.record === 'quote' || params.record === 'order') {
            activate();
        }
    });
})();
