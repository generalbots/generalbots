"use strict";
/* Docs i18n helper (#2091).
 *
 * The docs modules build their own DOM, so they cannot rely on `data-i18n`
 * attributes. This wrapper keeps a missing catalogue entry from leaking a raw
 * identifier into the UI: it always returns the authored English fallback. */
function docsT(key, fallback, params) {
  if (window.gbT) return window.gbT(key, fallback, params);
  return fallback || key;
}
