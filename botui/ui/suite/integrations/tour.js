/* Integrations app guided tour. */
(function (window) {
  "use strict";
  if (!window.GBTour) return;

  window.GBTour.define("integrations", {
    version: 1,
    root: "[data-integrations-app]",
    steps: [
      {
        target: '[data-tour="integrations.header"]',
        titleKey: "tour-integrations-header-title",
        title: "Trusted connections",
        bodyKey: "tour-integrations-header-body",
        body: "Every integration lists the capabilities and permissions it needs before you connect it.",
        placement: "below",
      },
      {
        target: '[data-tour="integrations.tabs"]',
        titleKey: "tour-integrations-tabs-title",
        title: "Browse by state",
        bodyKey: "tour-integrations-tabs-body",
        body: "Explore the catalog, review connected services, manage automations or inspect data sources.",
        placement: "below",
      },
      {
        target: '[data-tour="integrations.search"]',
        titleKey: "tour-integrations-search-title",
        title: "Find a provider",
        bodyKey: "tour-integrations-search-body",
        body: "Search by provider, action or category, then open a card to see the full setup.",
        placement: "below",
      },
      {
        target: '[data-tour="integrations.catalog"]',
        titleKey: "tour-integrations-catalog-title",
        title: "Review before you connect",
        bodyKey: "tour-integrations-catalog-body",
        body: "Cards show availability and health. Open one to inspect actions and credential requirements.",
        placement: "above",
      },
    ],
  });

  window.GBTour.autoStart("integrations");
})(window);
