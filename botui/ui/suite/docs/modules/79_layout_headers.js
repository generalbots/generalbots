"use strict";
/* Docs layout preview (#2424), per-section headers/footers and fields.
 *
 * First/even/odd variants, page X of Y, date, title, author and watermark are
 * resolved against the COMPUTED page number and computed total — no stored
 * page counter is involved. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  NS.FIELD_RE = /\{\s*\$?\s*(page|total|date|title|author)\s*\}/gi;

  NS.pageVariant = function (page, section) {
    if (page.isSectionFirst && section && section.firstPageDifferent) return "first";
    return (Number(page.number) % 2 === 0) ? "even" : "odd";
  };

  NS.resolveFields = function (template, ctx) {
    var values = ctx || {};
    return String(template == null ? "" : template).replace(NS.FIELD_RE, function (match, name) {
      var value = values[name.toLowerCase()];
      return value == null ? "" : String(value);
    });
  };

  function variantValue(bucket, variant, fallbackBucket) {
    if (bucket && bucket[variant] != null) return bucket[variant];
    if (bucket && bucket.odd != null) return bucket.odd;
    if (fallbackBucket && fallbackBucket[variant] != null) return fallbackBucket[variant];
    return "";
  }
  NS.variantValue = variantValue;

  NS.resolvePageChrome = function (page, section, fields, total) {
    var sec = section || {};
    var values = fields || {};
    var ctx = {
      page: page.number,
      total: total,
      date: values.date || "",
      title: values.title || "",
      author: values.author || ""
    };
    var variant = NS.pageVariant(page, sec);
    page.variant = variant;
    page.headerLines = [NS.resolveFields(variantValue(sec.header, variant), ctx)];
    page.footerLines = [NS.resolveFields(variantValue(sec.footer, variant), ctx)];
    page.watermark = NS.resolveFields(sec.watermark || "", ctx);
    return page;
  };

  NS.fieldPreview = function (ctx) {
    return NS.resolveFields(
      NS.tr("docs-layout-ftr-odd", "Page { $page } of { $total }"), ctx
    );
  };
})(typeof window !== "undefined" ? window : globalThis);
