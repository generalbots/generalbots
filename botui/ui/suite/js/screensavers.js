"use strict";
/* GB Screensavers (#1151): idle overlay with island / starfield / pipes /
 * blank scenes. Starts after 10 minutes of inactivity on the desktop shell;
 * any input exits. Picker: window.GBScreensaver.use(name).
 */
(function () {
  if (window.GBScreensaver) return;

  var IDLE_MS = 10 * 60 * 1000;
  var current = localStorage.getItem("gb-screensaver") || "starfield";
  var overlay = null;
  var raf = null;
  var timer = null;
  var cleanup = null;

  function scenes() { return ["island", "starfield", "pipes", "blank"]; }

  function createOverlay() {
    overlay = document.createElement("div");
    overlay.id = "gb-screensaver";
    overlay.innerHTML =
      '<canvas id="gb-screensaver-canvas"></canvas>' +
      '<div class="gb-screensaver-hint">Press any key or move the mouse</div>';
    document.body.appendChild(overlay);
    return overlay;
  }

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

  function start() {
    if (overlay) return;
    createOverlay();
    var canvas = overlay.querySelector("canvas");
    var ctx = canvas.getContext("2d");
    function fit() { canvas.width = innerWidth; canvas.height = innerHeight; }
    fit(); addEventListener("resize", fit);
    var t0 = performance.now(), frame = null;
    if (current === "starfield") frame = sceneStarfield(ctx, canvas.width, canvas.height);
    else if (current === "pipes") frame = scenePipes(ctx, canvas.width, canvas.height);
    else if (current === "island") frame = function () { sceneIsland(ctx, canvas.width, canvas.height, performance.now() - t0); };
    else frame = null; // blank
    var loop = function () {
      if (frame) frame();
      raf = requestAnimationFrame(loop);
    };
    loop();
    cleanup = function () {
      cancelAnimationFrame(raf); raf = null;
      removeEventListener("resize", fit);
    };
    ["mousemove", "mousedown", "keydown", "touchstart", "wheel"].forEach(function (ev) {
      overlay.addEventListener(ev, stop, { once: true });
    });
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
      if (scenes().indexOf(name) === -1) return false;
      current = name;
      try { localStorage.setItem("gb-screensaver", name); } catch (e) {}
      return true;
    },
    list: scenes,
  };

  ["mousemove", "keydown", "mousedown", "touchstart", "wheel"].forEach(function (ev) {
    addEventListener(ev, armIdle, { passive: true });
  });
  armIdle();
})();
