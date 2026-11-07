/* Drive app guided tour. */
(function (window) {
  "use strict";
  if (!window.GBTour) return;

  window.GBTour.define("drive", {
    version: 1,
    root: "#drive-app",
    steps: [
      {
        target: '[data-tour="drive.nav"]',
        titleKey: "tour-drive-nav-title",
        title: "Find your files",
        bodyKey: "tour-drive-nav-body",
        body: "Jump between My Drive, Recent, Starred and Trash from the sidebar.",
        placement: "right",
      },
      {
        target: '[data-tour="drive.upload"]',
        titleKey: "tour-drive-upload-title",
        title: "Add or create",
        bodyKey: "tour-drive-upload-body",
        body: "Upload files, upload a folder, or create a new folder without leaving the page.",
        placement: "right",
      },
      {
        target: '[data-tour="drive.tabs"]',
        titleKey: "tour-drive-tabs-title",
        title: "Switch workspaces",
        bodyKey: "tour-drive-tabs-body",
        body: "Move between your files, the bot workspace, shared items, connected storage and public files.",
        placement: "below",
      },
      {
        target: '[data-tour="drive.search"]',
        titleKey: "tour-drive-search-title",
        title: "Search every folder",
        bodyKey: "tour-drive-search-body",
        body: "One search box covers file names and extracted content across your buckets.",
        placement: "below",
      },
    ],
  });

  window.GBTour.autoStart("drive");
})(window);
