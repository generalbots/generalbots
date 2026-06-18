"use strict";
/* slides init — bootstrap on DOMContentLoaded */

window.addEventListener("DOMContentLoaded", function () {
  injectCanvasStyles();
  initSidebar();
  initAuth();
  initCollab();
  window.SlidesCanvas = SlideCanvas;
});
