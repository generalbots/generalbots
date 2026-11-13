"use strict";
/* GB Screensavers (#1151): idle overlay with gblogo / island / starfield /
 * pipes / blank scenes. Starts after 10 minutes of inactivity on the desktop
 * shell; any input exits. Picker: window.GBScreensaver.use(name).
 *
 * The default scene, "gblogo", shows the official General Bots logo artwork
 * (served as /logo.svg) over a drifting deep-space backdrop.
 */
(function () {
  if (window.GBScreensaver) return;

  var IDLE_MS = 10 * 60 * 1000;
  var DEFAULT_SCENE = "gblogo";
  var SCENES = ["gblogo", "island", "starfield", "pipes", "blank"];

  var current = readScene();
  var overlay = null;
  var raf = null;
  var timer = null;
  var cleanup = null;

  function readScene() {
    var saved = null;
    try {
      saved = localStorage.getItem("gb-screensaver");
      if (!localStorage.getItem("gb-screensaver-migrated")) {
        localStorage.setItem("gb-screensaver-migrated", "1");
        if (!saved || saved === "starfield") {
          saved = DEFAULT_SCENE;
          localStorage.setItem("gb-screensaver", saved);
        }
      }
    } catch (e) {
      saved = null;
    }
    if (!saved || SCENES.indexOf(saved) === -1) return DEFAULT_SCENE;
    return saved;
  }

  function createOverlay() {
    overlay = document.createElement("div");
    overlay.id = "gb-screensaver";
    overlay.innerHTML =
      '<canvas id="gb-screensaver-canvas"></canvas>' +
      '<div class="gb-screensaver-hint">Press any key or move the mouse</div>';
    document.body.appendChild(overlay);
    return overlay;
  }

  /* ------------------------------------------------------------------ */
  /* Space backdrop                                                      */
  /* ------------------------------------------------------------------ */

  function makeNebula(w, h) {
    var c = document.createElement("canvas");
    c.width = w; c.height = h;
    var nctx = c.getContext("2d");
    var blobs = [
      [0.30, 0.34, "rgba(90,216,255,0.13)"],
      [0.74, 0.60, "rgba(212,245,5,0.08)"],
      [0.52, 0.84, "rgba(126,86,255,0.11)"],
      [0.14, 0.78, "rgba(90,216,255,0.06)"]
    ];
    nctx.globalCompositeOperation = "lighter";
    for (var i = 0; i < blobs.length; i++) {
      var b = blobs[i];
      var cx = w * b[0], cy = h * b[1];
      var r = Math.max(w, h) * (0.34 + i * 0.05);
      var g = nctx.createRadialGradient(cx, cy, 0, cx, cy, r);
      g.addColorStop(0, b[2]);
      g.addColorStop(1, "rgba(0,0,0,0)");
      nctx.fillStyle = g;
      nctx.fillRect(0, 0, w, h);
    }
    return c;
  }

  function sceneCosmos(ctx, w, h) {
    var stars = [];
    for (var i = 0; i < 280; i++) {
      stars.push({
        x: Math.random() * w,
        y: Math.random() * h,
        r: 0.3 + Math.random() * 1.5,
        p: Math.random() * Math.PI * 2,
        s: 0.4 + Math.random() * 1.4,
        z: 0.3 + Math.random() * 0.7
      });
    }
    var nebula = makeNebula(w, h);
    var start = performance.now();
    var nextMeteor = start + 3500;
    var meteor = null;

    return function frame() {
      var t = performance.now() - start;
      ctx.globalCompositeOperation = "source-over";
      ctx.fillStyle = "#02040b";
      ctx.fillRect(0, 0, w, h);

      var drift = Math.sin(t * 0.00006) * 12;
      ctx.globalAlpha = 0.85;
      ctx.drawImage(nebula, drift - 12, Math.cos(t * 0.00005) * 10 - 12);
      ctx.globalAlpha = 1;

      for (var i = 0; i < stars.length; i++) {
        var st = stars[i];
        var tw = 0.35 + 0.65 * (0.5 + 0.5 * Math.sin(t * 0.0012 * st.s + st.p));
        ctx.globalAlpha = tw * (0.35 + st.z * 0.65);
        ctx.fillStyle = st.z > 0.8 ? "#eaf6ff" : "#9fc6e8";
        var sx = st.x + Math.sin(t * 0.00004 + st.p) * 7;
        var sy = st.y + Math.cos(t * 0.00003 + st.p) * 5;
        ctx.beginPath();
        ctx.arc(sx, sy, st.r * (0.7 + st.z * 0.6), 0, 6.2832);
        ctx.fill();
      }
      ctx.globalAlpha = 1;

      if (!meteor && t > nextMeteor - start) {
        meteor = {
          x: Math.random() * w * 0.7,
          y: Math.random() * h * 0.35,
          vx: 5 + Math.random() * 3,
          vy: 2 + Math.random() * 2,
          life: 1
        };
      }
      if (meteor) {
        var mx = meteor.x, my = meteor.y;
        var grad = ctx.createLinearGradient(mx, my, mx - meteor.vx * 16, my - meteor.vy * 16);
        grad.addColorStop(0, "rgba(234,246,255," + meteor.life + ")");
        grad.addColorStop(1, "rgba(90,216,255,0)");
        ctx.strokeStyle = grad;
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.moveTo(mx, my);
        ctx.lineTo(mx - meteor.vx * 16, my - meteor.vy * 16);
        ctx.stroke();
        meteor.x += meteor.vx;
        meteor.y += meteor.vy;
        meteor.life -= 0.012;
        if (meteor.life <= 0 || meteor.x > w + 40 || meteor.y > h + 40) {
          meteor = null;
          nextMeteor = t + 5000 + Math.random() * 9000;
        }
      }
    };
  }

  /* ------------------------------------------------------------------ */
  /* Official General Bots artwork                                       */
  /* ------------------------------------------------------------------ */

  function buildOfficialLogo() {
    var stage = document.createElement("div");
    stage.className = "gb-ss-stage";

    var img = document.createElement("img");
    img.className = "gb-ss-emblem gb-ss-logo";
    img.src = "/logo.svg";
    img.alt = "General Bots";
    img.decoding = "async";
    stage.appendChild(img);

    return stage;
  }

  /* ------------------------------------------------------------------ */
  /* Legacy scenes                                                       */
  /* ------------------------------------------------------------------ */

  function sceneStarfield(ctx, w, h) {
    var stars = [];
    for (var i = 0; i < 220; i++) {
      stars.push({ x: Math.random() * w - w / 2, y: Math.random() * h - h / 2, z: Math.random() * w });
    }
    return function frame() {
      ctx.fillStyle = "rgba(2,6,12,0.35)";
      ctx.fillRect(0, 0, w, h);
      ctx.fillStyle = "#e8f4ff";
      for (var i = 0; i < stars.length; i++) {
        var s = stars[i];
        s.z -= 2.2;
        if (s.z <= 1) { s.z = w; s.x = Math.random() * w - w / 2; s.y = Math.random() * h - h / 2; }
        var k = 128 / s.z;
        var px = s.x * k + w / 2, py = s.y * k + h / 2;
        var size = Math.max(0.4, (1 - s.z / w) * 2.4);
        if (px >= 0 && px < w && py >= 0 && py < h) ctx.fillRect(px, py, size, size);
      }
    };
  }

  function scenePipes(ctx, w, h) {
    var x = w / 2, y = h / 2, dir = 0, hue = Math.random() * 360;
    ctx.fillStyle = "rgba(4,8,14,0.08)";
    ctx.fillRect(0, 0, w, h);
    return function frame() {
      for (var step = 0; step < 3; step++) {
        if (Math.random() < 0.06) dir = Math.floor(Math.random() * 4);
        var len = 14;
        var nx = x + [len, 0, -len, 0][dir];
        var ny = y + [0, len, 0, -len][dir];
        ctx.strokeStyle = "hsl(" + hue + ",70%,55%)";
        ctx.lineWidth = 9;
        ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(nx, ny); ctx.stroke();
        x = nx; y = ny;
        if (x < 0 || x > w || y < 0 || y > h) { x = w / 2; y = h / 2; hue = (hue + 47) % 360; }
        hue = (hue + 0.35) % 360;
      }
    };
  }

  function sceneIsland(ctx, w, h, t) {
    var g = ctx.createLinearGradient(0, 0, 0, h);
    var night = (Math.sin(t / 9000) + 1) / 2;
    g.addColorStop(0, "rgb(" + (20 + 90 * (1-night)) + "," + (40 + 60*(1-night)) + "," + (90 + 80*(1-night)) + ")");
    g.addColorStop(0.7, "rgb(" + (30 + 120*night) + "," + (60 + 90*night) + "," + (110 + 90*night) + ")");
    g.addColorStop(0.72, "#123a2a");
    g.addColorStop(1, "#0b241a");
    ctx.fillStyle = g; ctx.fillRect(0, 0, w, h);
    ctx.fillStyle = "#0d2c20";
    ctx.beginPath();
    ctx.moveTo(w*0.18, h*0.74); ctx.quadraticCurveTo(w*0.5, h*0.5, w*0.82, h*0.74);
    ctx.quadraticCurveTo(w*0.5, h*0.82, w*0.18, h*0.74); ctx.fill();
    ctx.fillStyle = "rgba(255,255,255,0.75)";
    for (var i = 0; i < 3; i++) {
      var sx = ((t/60) + i*w/3) % (w+160) - 80;
      ctx.beginPath(); ctx.ellipse(sx, h*(0.22+0.04*i), 42, 13, 0, 0, 7); ctx.fill();
      ctx.beginPath(); ctx.ellipse(sx+30, h*(0.22+0.04*i), 34, 11, 0, 0, 7); ctx.fill();
    }
  }

  /* ------------------------------------------------------------------ */
  /* Lifecycle                                                           */
  /* ------------------------------------------------------------------ */

  function start() {
    if (overlay) return;
    createOverlay();

    var reduce = false;
    try { reduce = matchMedia("(prefers-reduced-motion: reduce)").matches; } catch (e) {}
    if (reduce) overlay.classList.add("gb-ss-static");

    var canvas = overlay.querySelector("canvas");
    var ctx = canvas.getContext("2d");
    function fit() { canvas.width = innerWidth; canvas.height = innerHeight; }
    fit(); addEventListener("resize", fit);

    var t0 = performance.now(), frame = null;
    if (current === "gblogo") {
      var stage = buildOfficialLogo();
      var hint = overlay.querySelector(".gb-screensaver-hint");
      overlay.insertBefore(stage, hint);
      var mark = document.createElement("div");
      mark.className = "gb-screensaver-wordmark";
      mark.innerHTML = "<span>General Bots</span><small>system idle</small>";
      overlay.insertBefore(mark, hint);
      frame = sceneCosmos(ctx, canvas.width, canvas.height);
    } else if (current === "starfield") {
      frame = sceneStarfield(ctx, canvas.width, canvas.height);
    } else if (current === "pipes") {
      frame = scenePipes(ctx, canvas.width, canvas.height);
    } else if (current === "island") {
      frame = function () { sceneIsland(ctx, canvas.width, canvas.height, performance.now() - t0); };
    }

    if (reduce) {
      if (frame) frame();
    } else {
      var loop = function () {
        if (frame) frame();
        raf = requestAnimationFrame(loop);
      };
      loop();
    }

    cleanup = function () {
      if (raf) cancelAnimationFrame(raf);
      raf = null;
      removeEventListener("resize", fit);
    };

    // Dismissing the overlay used to consume the click: `stop` removed it on
    // `mousedown`, so the following `mouseup`/`click` had no target and the
    // app never saw the first interaction (#2155 UI-REG-02: 0 click events
    // after the overlay dismissed). The overlay now forwards the dismissing
    // press to whatever sits under the cursor.
    function dismissAndForward(event) {
      var x = event.clientX;
      var y = event.clientY;
      var pointer = event.type === "mousedown" || event.type === "touchstart";
      stop();
      if (!pointer) return;
      var forward = function (upEvent) {
        document.removeEventListener("mouseup", forward, true);
        document.removeEventListener("touchend", forward, true);
        var px = typeof upEvent.clientX === "number" ? upEvent.clientX : x;
        var py = typeof upEvent.clientY === "number" ? upEvent.clientY : y;
        var target = document.elementFromPoint(px, py);
        if (!target) return;
        target.dispatchEvent(new MouseEvent("click", {
          bubbles: true,
          cancelable: true,
          clientX: px,
          clientY: py,
          view: window,
        }));
      };
      document.addEventListener("mouseup", forward, true);
      document.addEventListener("touchend", forward, true);
    }

    ["mousemove", "keydown", "wheel"].forEach(function (ev) {
      overlay.addEventListener(ev, stop, { once: true });
    });
    overlay.addEventListener("mousedown", dismissAndForward, { once: true });
    overlay.addEventListener("touchstart", dismissAndForward, { once: true });
  }

  function stop() {
    if (!overlay) return;
    if (cleanup) cleanup();
    overlay.remove(); overlay = null;
    armIdle();
  }

  function armIdle() {
    clearTimeout(timer);
    timer = setTimeout(start, IDLE_MS);
  }

  window.GBScreensaver = {
    start: start,
    stop: stop,
    use: function (name) {
      if (SCENES.indexOf(name) === -1) return false;
      current = name;
      try { localStorage.setItem("gb-screensaver", name); } catch (e) {}
      return true;
    },
    current: function () { return current; },
    list: function () { return SCENES.slice(); },
  };

  ["mousemove", "keydown", "mousedown", "touchstart", "wheel"].forEach(function (ev) {
    addEventListener(ev, armIdle, { passive: true });
  });
  armIdle();
})();
