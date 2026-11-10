"use strict";
/* Chat camera preview readiness — zero-dependency unit test
 * (run: node tests/chat_camera_preview.test.js)
 *
 * #2106 — the chat camera waited for `video.play()` to resolve before it
 * enabled the shutter. WebKitGTK can leave that promise pending forever, so the
 * dialog stayed on "Waiting for the camera to start…". Readiness now comes from
 * `GBCameraPreview.watch()` (frames + its own deadline). These assertions pin
 * the contract: resolve true on the first capturable frame, resolve false when
 * nothing arrives, and never leave listeners or timers behind. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(
  __dirname,
  "..",
  "ui",
  "suite",
  "chat",
  "modules",
  "77_camera_preview.js",
);
const SOURCE = fs.readFileSync(MODULE, "utf8");

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

function loadPreview() {
  const sandbox = { window: {}, setTimeout, clearTimeout, Promise };
  vm.createContext(sandbox);
  vm.runInContext(SOURCE, sandbox, { filename: MODULE });
  return sandbox.window.GBCameraPreview;
}

/* ── fake media element ─────────────────────────────────────────────────── */

class FakeVideo {
  constructor() {
    this.videoWidth = 0;
    this.videoHeight = 0;
    this.listeners = new Map();
    this.frameCallbacks = [];
    this.cancelledCallbacks = 0;
  }

  addEventListener(name, handler) {
    const list = this.listeners.get(name) || [];
    list.push(handler);
    this.listeners.set(name, list);
  }

  removeEventListener(name, handler) {
    const list = (this.listeners.get(name) || []).filter((fn) => fn !== handler);
    if (list.length) this.listeners.set(name, list);
    else this.listeners.delete(name);
  }

  listenerCount() {
    let total = 0;
    for (const list of this.listeners.values()) total += list.length;
    return total;
  }

  dispatch(name) {
    for (const handler of (this.listeners.get(name) || []).slice()) handler();
  }

  requestVideoFrameCallback(callback) {
    this.frameCallbacks.push(callback);
    return this.frameCallbacks.length;
  }

  cancelVideoFrameCallback() {
    this.cancelledCallbacks += 1;
  }

  presentFrame(width, height) {
    this.videoWidth = width;
    this.videoHeight = height;
    const pending = this.frameCallbacks.filter(Boolean);
    this.frameCallbacks = [];
    for (const callback of pending) callback(0, {});
  }
}

/* ── assertions ─────────────────────────────────────────────────────────── */

async function main() {
  const preview = loadPreview();
  check(!!preview && typeof preview.watch === "function", "the module exposes watch()");
  check(typeof preview.videoReady === "function", "the module exposes videoReady()");

  check(preview.videoReady(null) === false, "no element is not ready");
  check(preview.videoReady({ videoWidth: 0, videoHeight: 0 }) === false, "0x0 is not ready");
  check(preview.videoReady({ videoWidth: 1280, videoHeight: 720 }) === true, "1280x720 is ready");

  // 1. A frame that arrives before any event resolves the watch.
  const withFrame = new FakeVideo();
  withFrame.videoWidth = 640;
  withFrame.videoHeight = 480;
  const first = preview.watch(withFrame, { timeoutMs: 500, pollMs: 10 });
  check((await first.promise) === true, "an already-capturable element resolves true");
  check(withFrame.listenerCount() === 0, "listeners are removed after resolving");

  // 2. `playing` alone is enough, without waiting for the deadline.
  const evented = new FakeVideo();
  const second = preview.watch(evented, { timeoutMs: 500, pollMs: 10 });
  evented.presentFrame(320, 240);
  evented.dispatch("playing");
  check((await second.promise) === true, "the playing event resolves true once frames exist");

  // 3. No events at all: the poll still notices the frames (stalled play()).
  const polled = new FakeVideo();
  const third = preview.watch(polled, { timeoutMs: 500, pollMs: 10 });
  setTimeout(() => {
    polled.videoWidth = 1280;
    polled.videoHeight = 720;
  }, 30);
  check((await third.promise) === true, "polling resolves true without media events");

  // 4. A black element resolves false at the deadline instead of hanging.
  const black = new FakeVideo();
  const started = Date.now();
  const fourth = preview.watch(black, { timeoutMs: 60, pollMs: 10 });
  check((await fourth.promise) === false, "a silent element resolves false");
  check(Date.now() - started >= 50, "the deadline was actually waited for");
  check(black.listenerCount() === 0, "listeners are removed after the deadline");

  // 5. cancel() settles the promise and releases everything.
  const cancelled = new FakeVideo();
  const fifth = preview.watch(cancelled, { timeoutMs: 5000, pollMs: 10 });
  fifth.cancel();
  check((await fifth.promise) === false, "cancel() resolves false");
  check(cancelled.listenerCount() === 0, "cancel() removes the listeners");
  check(cancelled.cancelledCallbacks === 1, "cancel() cancels the frame callback");

  // 6. requestVideoFrameCallback is the strongest signal when it fires.
  const frameCallback = new FakeVideo();
  const sixth = preview.watch(frameCallback, { timeoutMs: 500, pollMs: 10 });
  frameCallback.presentFrame(800, 600);
  check((await sixth.promise) === true, "a presented frame resolves true");

  // 7. The camera module must not gate readiness on play() (#2106).
  const camera = fs.readFileSync(path.join(__dirname, "..", "ui", "suite", "chat", "modules", "77_camera.js"), "utf8");
  check(camera.includes("GBCameraPreview.watch("), "the camera uses the frame watcher");
  check(!/return Promise\.resolve\(played\)/.test(camera), "readiness is not chained on play()");

  console.log("chat_camera_preview: " + passed + " passed, " + failed + " failed");
  if (failed) process.exit(1);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
