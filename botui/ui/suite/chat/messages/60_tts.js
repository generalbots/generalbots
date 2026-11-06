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
  var btn = document.createElement("button");
  btn.className = "copy-msg-btn";
  btn.type = "button";
  btn.title = "Copy message";
  btn.setAttribute("aria-label", "Copy message");
  btn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  btn.addEventListener("click", function (e) {
    e.stopPropagation();
    var done = function () {
      btn.classList.add("copied");
      btn.title = "Copied";
      chatNotice("Message copied", "success");
      setTimeout(function () {
        btn.classList.remove("copied");
        btn.title = "Copy message";
      }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done).catch(function () {
        chatNotice("Copy failed", "warning");
      });
    } else {
      var area = document.createElement("textarea");
      area.value = text;
      document.body.appendChild(area);
      area.select();
      try { document.execCommand("copy"); done(); } catch (err) { /* ignore */ }
      area.remove();
    }
  });
  return btn;
}

function createSpeakButton(text) {
  var btn = document.createElement("button");
  btn.className = "tts-speak-btn";
  btn.type = "button";
  btn.title = "Listen (TTS)";
  btn.setAttribute("aria-label", "Listen to this message");
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
