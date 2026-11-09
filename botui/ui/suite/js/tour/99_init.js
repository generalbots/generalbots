/* Guided tour engine — public API and boot (3/3, last). */
(function (window) {
  "use strict";

  var T = window.__GBTour;
  if (!T) return;

  var api = {
    define: T.define,
    start: T.start,
    autoStart: T.autoStart,
    next: T.next,
    prev: T.prev,
    skip: T.skip,
    close: T.close,
    status: T.status,
    isComplete: T.isComplete,
    reset: T.reset,
    current: T.current,
    definitionIds: T.definitionIds,
  };

  if (typeof window.__GBTourBoot === "function") window.__GBTourBoot(api);
  else window.GBTour = api;
})(window);
