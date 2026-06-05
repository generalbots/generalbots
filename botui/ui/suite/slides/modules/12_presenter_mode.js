"use strict";

/**
 * Module 12: Presenter Mode for Slides.
 * Provides: dual-view, slide preview, next/prev navigation, timer, notes panel,
 * black screen, laser pointer, audience view via window.open.
 */

function enterPresenterMode(state) {
  if (!state) state = {};
  state.presenter = {
    active: true,
    startedAt: Date.now(),
    currentSlide: state.currentSlide || 0,
    notes: state.notes || "",
    audienceWindow: null,
    timerInterval: null,
  };
  if (state.onPresenterEnter) state.onPresenterEnter(state.presenter);
  return state.presenter;
}

function exitPresenterMode(state) {
  if (!state || !state.presenter) return false;
  if (state.presenter.timerInterval) clearInterval(state.presenter.timerInterval);
  if (state.presenter.audienceWindow && !state.presenter.audienceWindow.closed) {
    state.presenter.audienceWindow.close();
  }
  state.presenter.active = false;
  if (state.onPresenterExit) state.onPresenterExit(state.presenter);
  return true;
}

function nextSlide(state) {
  if (!state || !state.slides) return null;
  const max = state.slides.length - 1;
  if (state.currentSlide < max) state.currentSlide += 1;
  if (state.presenter) state.presenter.currentSlide = state.currentSlide;
  broadcastToAudience(state);
  return state.currentSlide;
}

function prevSlide(state) {
  if (!state || !state.slides) return null;
  if (state.currentSlide > 0) state.currentSlide -= 1;
  if (state.presenter) state.presenter.currentSlide = state.currentSlide;
  broadcastToAudience(state);
  return state.currentSlide;
}

function gotoSlide(state, index) {
  if (!state || !state.slides) return null;
  if (index < 0 || index >= state.slides.length) return null;
  state.currentSlide = index;
  if (state.presenter) state.presenter.currentSlide = index;
  broadcastToAudience(state);
  return state.currentSlide;
}

function toggleBlackScreen(state) {
  if (!state || !state.presenter) return false;
  state.presenter.blackScreen = !state.presenter.blackScreen;
  return state.presenter.blackScreen;
}

function toggleWhiteScreen(state) {
  if (!state || !state.presenter) return false;
  state.presenter.whiteScreen = !state.presenter.whiteScreen;
  return state.presenter.whiteScreen;
}

function startTimer(state, tickInterval) {
  if (!state || !state.presenter) return null;
  if (state.presenter.timerInterval) clearInterval(state.presenter.timerInterval);
  state.presenter.timerStartedAt = state.presenter.timerStartedAt || Date.now();
  state.presenter.timerInterval = setInterval(function () {
    const elapsed = Date.now() - state.presenter.timerStartedAt;
    if (state.onTimerTick) state.onTimerTick(elapsed);
  }, tickInterval || 1000);
  return state.presenter.timerInterval;
}

function stopTimer(state) {
  if (!state || !state.presenter || !state.presenter.timerInterval) return false;
  clearInterval(state.presenter.timerInterval);
  state.presenter.timerInterval = null;
  return true;
}

function resetTimer(state) {
  if (!state || !state.presenter) return false;
  state.presenter.timerStartedAt = Date.now();
  return true;
}

function formatElapsed(ms) {
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${m}:${String(s).padStart(2, "0")}`;
}

function openAudienceView(state, url) {
  if (!state || !state.presenter) return null;
  const w = window.open(url || "/audience", "audience", "width=1024,height=768");
  state.presenter.audienceWindow = w;
  return w;
}

function broadcastToAudience(state) {
  if (!state || !state.presenter || !state.presenter.audienceWindow) return false;
  try {
    state.presenter.audienceWindow.postMessage(
      {
        type: "slide-change",
        index: state.currentSlide,
        slide: state.slides ? state.slides[state.currentSlide] : null,
      },
      "*"
    );
    return true;
  } catch (_e) {
    return false;
  }
}

function setLaserPointer(state, x, y) {
  if (!state || !state.presenter) return null;
  state.presenter.laser = { x, y, visible: true };
  if (state.onLaserMove) state.onLaserMove(x, y);
  return state.presenter.laser;
}

function hideLaserPointer(state) {
  if (!state || !state.presenter) return false;
  state.presenter.laser = { x: 0, y: 0, visible: false };
  return true;
}

function notesForSlide(state, slideIndex) {
  if (!state) return "";
  if (state.notesBySlide && state.notesBySlide[slideIndex]) {
    return state.notesBySlide[slideIndex];
  }
  const slide = state.slides && state.slides[slideIndex];
  if (slide && slide.notes) return slide.notes;
  return "";
}

function upcomingSlide(state) {
  if (!state || !state.slides) return null;
  if (state.currentSlide + 1 < state.slides.length) {
    return state.slides[state.currentSlide + 1];
  }
  return null;
}

window.SlidesPresenter = {
  enterPresenterMode,
  exitPresenterMode,
  nextSlide,
  prevSlide,
  gotoSlide,
  toggleBlackScreen,
  toggleWhiteScreen,
  startTimer,
  stopTimer,
  resetTimer,
  formatElapsed,
  openAudienceView,
  broadcastToAudience,
  setLaserPointer,
  hideLaserPointer,
  notesForSlide,
  upcomingSlide,
};
