/* Chat messages module: Text-to-speech toggle, copy/speak buttons and speech synthesis. */
// ── TTS state + auto-read toggle (issue #708) ──
var TTSState = {
  autoSpeak: localStorage.getItem("gb_tts_auto") === "1"
};

function initTtsToggle() {
  var btn = document.getElementById("ttsToggle");
  if (!btn || btn.dataset.bound === "1") return;
  btn.dataset.bound = "1";
  function refresh() {
    btn.classList.toggle("active", TTSState.autoSpeak);
    btn.setAttribute("aria-pressed", TTSState.autoSpeak ? "true" : "false");
    btn.title = TTSState.autoSpeak ? "Stop auto-reading bot replies" : "Auto-read bot replies (TTS)";
    btn.setAttribute("aria-label", btn.title);
  }
  btn.addEventListener("click", function () {
    TTSState.autoSpeak = !TTSState.autoSpeak;
    localStorage.setItem("gb_tts_auto", TTSState.autoSpeak ? "1" : "0");
    refresh();
    if (!TTSState.autoSpeak && "speechSynthesis" in window) {
      window.speechSynthesis.cancel();
    }
  });
  refresh();
}

// The chat partial is injected by htmx after DOMContentLoaded, so the
// listener alone would never bind the toggle (#1693 follow-up).
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initTtsToggle);
} else {
  initTtsToggle();
}

// ── TTS (issue #708) — speak a bot message aloud ──
function createCopyButton(text) {
  var copyLabel = (typeof gbT === "function") ? gbT("chat-copy-message", "Copy message") : "Copy message";
  var copiedLabel = (typeof gbT === "function") ? gbT("chat-copied", "Copied") : "Copied";
  var copiedNotice = (typeof gbT === "function") ? gbT("chat-message-copied", "Message copied") : "Message copied";
  var copyFailedNotice = (typeof gbT === "function") ? gbT("chat-copy-failed", "Copy failed") : "Copy failed";
  var btn = document.createElement("button");
  btn.className = "copy-msg-btn";
  btn.type = "button";
  btn.title = copyLabel;
  btn.setAttribute("aria-label", copyLabel);
  btn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  btn.addEventListener("click", function (e) {
    e.stopPropagation();
    var done = function () {
      btn.classList.add("copied");
      btn.title = copiedLabel;
      btn.setAttribute("aria-label", copiedLabel);
      chatNotice(copiedNotice, "success");
      setTimeout(function () {
        btn.classList.remove("copied");
        btn.title = copyLabel;
        btn.setAttribute("aria-label", copyLabel);
      }, 1500);
    };
    gbCopyText(text).then(function (copied) {
      if (copied) done();
      else chatNotice(copyFailedNotice, "warning");
    });
  });
  return btn;
}

function createSpeakButton(text) {
  var ttsLabel = (typeof gbT === "function") ? gbT("chat-listen-tts", "Listen (TTS)") : "Listen (TTS)";
  var ttsAria = (typeof gbT === "function") ? gbT("chat-listen-message", "Listen to this message") : "Listen to this message";
  var btn = document.createElement("button");
  btn.className = "tts-speak-btn";
  btn.type = "button";
  btn.title = ttsLabel;
  btn.setAttribute("aria-label", ttsAria);
  btn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/></svg>';
  btn.addEventListener("click", function (e) {
    e.stopPropagation();
    speakText(text);
  });
  return btn;
}

function speakText(text) {
  try {
    if (!("speechSynthesis" in window)) {
      chatNotice("Text-to-speech is not supported in this browser", "warning");
      return;
    }
    window.speechSynthesis.cancel();
    var clean = text.replace(/[*_#`~]/g, "").replace(/\s+/g, " ").trim();
    if (!clean) return;
    var utterance = new SpeechSynthesisUtterance(clean);
    var preferred = "pt-BR";
    var voices = window.speechSynthesis.getVoices();
    if (voices.length) {
      var match = voices.filter(function (v) { return v.lang === preferred; })[0] || voices[0];
      utterance.voice = match;
    }
    utterance.rate = 1;
    utterance.pitch = 1;
    window.speechSynthesis.speak(utterance);
  } catch (err) {
    console.error("TTS failed:", err);
  }
}
