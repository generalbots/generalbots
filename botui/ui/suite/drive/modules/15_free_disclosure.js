"use strict";
/* Assistant parity: Drive uses the shared free-model disclosure module.
 * Drive has no chat composer, so the reminder renders only when a composer
 * surface is present (for example a chat window injected beside Drive). */
(function () {
  var D = window.GBFreeDisclosure;
  if (!D) return;
  D.renderComposerBar();
})();
