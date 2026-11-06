/*
 * General Bots Cloud — shared page shell (single source of truth).
 *
 * Load this synchronously in <head> on every cloud page that uses the
 * .mgmt-* layout, in place of the per-page stylesheet block:
 *
 *   <script src="/cloud/js/cloud-shell.js"></script>
 *
 * The links are appended while <head> is still being parsed, so they land
 * before any page-specific <style> block: cascade order and render-blocking
 * behaviour are identical to the static <link> tags they replace.
 *
 * The shell wrapper (<div class="mgmt-shell"> around <main class="mgmt-main">)
 * is mounted by cloud.js (ensureShell), and the sidebar itself comes from
 * /cloud/partials/sidebar.html — pages never copy either.
 */
(function () {
  'use strict';

  var SHARED_STYLES = [
    '/cloud/css/cloud-layout.css',
    '/cloud/css/cloud-components.css',
    '/cloud/css/cloud-forms-utilities.css'
  ];

  var head = document.head;
  if (!head) return;

  SHARED_STYLES.forEach(function (href) {
    if (head.querySelector('link[data-cloud-shell][href="' + href + '"]')) return;
    var link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = href;
    link.setAttribute('data-cloud-shell', '');
    head.appendChild(link);
  });
})();
