/* Chat app guided tour. */
(function (window) {
  "use strict";
  if (!window.GBTour) return;

  window.GBTour.define("chat", {
    version: 1,
    root: "#chat-app",
    steps: [
      {
        target: '[data-tour="chat.messages"]',
        titleKey: "tour-chat-messages-title",
        title: "Your conversation",
        bodyKey: "tour-chat-messages-body",
        body: "Messages, tool results and files appear here. Scroll back at any time to review the thread.",
        placement: "center",
      },
      {
        target: '[data-tour="chat.composer"]',
        titleKey: "tour-chat-composer-title",
        title: "Ask anything",
        bodyKey: "tour-chat-composer-body",
        body: "Type a message, attach a file or use @ to reference a contact, file or bot. Press Enter to send.",
        placement: "above",
      },
      {
        target: '[data-tour="chat.model"]',
        titleKey: "tour-chat-model-title",
        title: "Choose the model",
        bodyKey: "tour-chat-model-body",
        body: "Auto picks the best balance of speed and cost. Switch modes when you need a faster or stronger answer.",
        placement: "above",
      },
    ],
  });

  window.GBTour.autoStart("chat");
})(window);
