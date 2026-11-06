"use strict";
/* GB Screensavers (#1151): idle overlay with gblogo / island / starfield /
 * pipes / blank scenes. Starts after 10 minutes of inactivity on the desktop
 * shell; any input exits. Picker: window.GBScreensaver.use(name).
 *
 * The default scene, "gblogo", is an SVG animation built from the General
 * Bots identity: interlocking gear rings rotate around a robot head that
 * assembles itself, over a drifting deep-space backdrop.
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
  /* General Bots emblem (SVG)                                           */
  /* ------------------------------------------------------------------ */

  function gearPath(cx, cy, teeth, rOut, rIn) {
    var d = "", step = Math.PI * 2 / teeth;
    for (var i = 0; i < teeth; i++) {
      var a = i * step;
      var pts = [
        [rIn, a],
        [rOut, a + step * 0.20],
        [rOut, a + step * 0.44],
        [rIn, a + step * 0.64]
      ];
      for (var j = 0; j < pts.length; j++) {
        var px = cx + pts[j][0] * Math.cos(pts[j][1]);
        var py = cy + pts[j][0] * Math.sin(pts[j][1]);
        d += (i === 0 && j === 0 ? "M" : "L") + px.toFixed(2) + " " + py.toFixed(2);
      }
    }
    return d + "Z";
  }

  function satellite(cx, cy, teeth, rOut, rIn, delay) {
    return '<g class="gb-ss-sat" style="transform-origin:' + cx + 'px ' + cy + 'px;animation-delay:' + delay + 's">' +
      '<path d="' + gearPath(cx, cy, teeth, rOut, rIn) + '" fill="none" stroke="url(#gbssEdge)" ' +
      'stroke-width="1.8" opacity="0.7"/>' +
      '<circle cx="' + cx + '" cy="' + cy + '" r="' + (rIn * 0.38).toFixed(1) + '" fill="none" ' +
      'stroke="url(#gbssEdge)" stroke-width="1.5" opacity="0.8"/>' +
      '<circle cx="' + cx + '" cy="' + cy + '" r="2" fill="#8ff0c8" opacity="0.9"/>' +
      "</g>";
  }

  function buildEmblem() {
    var stage = document.createElement("div");
    stage.className = "gb-ss-stage";

    var outerGear = gearPath(200, 200, 30, 178, 163);
    var innerGear = gearPath(200, 200, 20, 148, 136);

    var satMarkup =
      satellite(200, 24, 10, 30, 20, 1.15) +
      satellite(376, 200, 10, 30, 20, 1.3) +
      satellite(200, 376, 10, 30, 20, 1.45) +
      satellite(24, 200, 10, 30, 20, 1.6);

    stage.innerHTML =
      '<svg class="gb-ss-emblem" viewBox="0 0 400 400" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">' +
        '<defs>' +
          '<radialGradient id="gbssHalo" cx="50%" cy="50%" r="50%">' +
            '<stop offset="0%" stop-color="rgba(212,245,5,0.28)"/>' +
            '<stop offset="55%" stop-color="rgba(90,216,255,0.12)"/>' +
            '<stop offset="100%" stop-color="rgba(0,0,0,0)"/>' +
          "</radialGradient>" +
          '<linearGradient id="gbssEdge" x1="0" y1="0" x2="1" y2="1">' +
            '<stop offset="0%" stop-color="#d4f505"/>' +
            '<stop offset="55%" stop-color="#8ff0c8"/>' +
            '<stop offset="100%" stop-color="#5ad8ff"/>' +
          "</linearGradient>" +
          '<filter id="gbssGlow" x="-70%" y="-70%" width="240%" height="240%">' +
            '<feGaussianBlur stdDeviation="4.5" result="b"/>' +
            "<feMerge><feMergeNode in=\"b\"/><feMergeNode in=\"SourceGraphic\"/></feMerge>" +
          "</filter>" +
        "</defs>" +

        '<circle class="gb-ss-halo" cx="200" cy="200" r="158" fill="url(#gbssHalo)"/>' +
        '<circle class="gb-ss-orbit" cx="200" cy="200" r="196" fill="none" ' +
          'stroke="url(#gbssEdge)" stroke-width="0.7" stroke-dasharray="1 9" opacity="0.55"/>' +
        '<circle class="gb-ss-orbit gb-ss-orbit--inner" cx="200" cy="200" r="188" fill="none" ' +
          'stroke="url(#gbssEdge)" stroke-width="0.6" stroke-dasharray="1 15" opacity="0.32"/>' +

        '<g class="gb-ss-gear gb-ss-gear--outer">' +
          '<path d="' + outerGear + '" fill="none" stroke="url(#gbssEdge)" stroke-width="1.5" opacity="0.5"/>' +
        "</g>" +
        '<g class="gb-ss-gear gb-ss-gear--inner">' +
          '<path d="' + innerGear + '" fill="none" stroke="url(#gbssEdge)" stroke-width="1.3" opacity="0.42"/>' +
        "</g>" +
        satMarkup +

        '<g class="gb-ss-robot">' +
          '<g class="gb-ss-part gb-ss-part--antenna" style="animation-delay:0.15s">' +
            '<line x1="200" y1="132" x2="200" y2="106" stroke="url(#gbssEdge)" stroke-width="3" stroke-linecap="round"/>' +
            '<circle class="gb-ss-bulb" cx="200" cy="99" r="7" fill="#d4f505" filter="url(#gbssGlow)"/>' +
          "</g>" +
          '<g class="gb-ss-part gb-ss-part--ears" style="animation-delay:0.28s">' +
            '<rect x="124" y="178" width="12" height="34" rx="6" fill="#0b1118" stroke="url(#gbssEdge)" stroke-width="2"/>' +
            '<rect x="264" y="178" width="12" height="34" rx="6" fill="#0b1118" stroke="url(#gbssEdge)" stroke-width="2"/>' +
          "</g>" +
          '<g class="gb-ss-part gb-ss-part--head" style="animation-delay:0.45s">' +
            '<rect x="138" y="132" width="124" height="124" rx="30" fill="rgba(8,12,18,0.9)" ' +
              'stroke="url(#gbssEdge)" stroke-width="2.4"/>' +
          "</g>" +
          '<g class="gb-ss-part gb-ss-part--visor" style="animation-delay:0.7s">' +
            '<rect x="150" y="156" width="100" height="58" rx="18" fill="#04070d" ' +
              'stroke="rgba(212,245,5,0.35)" stroke-width="1.2"/>' +
          "</g>" +
          '<g class="gb-ss-part gb-ss-part--eyes" style="animation-delay:0.92s">' +
            '<rect class="gb-ss-eye" x="164" y="174" width="24" height="20" rx="7" fill="#d4f505" filter="url(#gbssGlow)"/>' +
            '<rect class="gb-ss-eye" x="212" y="174" width="24" height="20" rx="7" fill="#d4f505" filter="url(#gbssGlow)"/>' +
          "</g>" +
          '<g class="gb-ss-part gb-ss-part--mouth" style="animation-delay:1.08s">' +
            '<rect x="168" y="224" width="64" height="18" rx="9" fill="none" ' +
              'stroke="url(#gbssEdge)" stroke-width="1.6" opacity="0.85"/>' +
            '<rect class="gb-ss-scanbar" x="186" y="228" width="10" height="10" rx="3" fill="#5ad8ff"/>' +
          "</g>" +
          '<g class="gb-ss-part gb-ss-part--bolts" style="animation-delay:1.22s">' +
            '<circle cx="151" cy="145" r="2.4" fill="#8ff0c8"/>' +
            '<circle cx="249" cy="145" r="2.4" fill="#8ff0c8"/>' +
            '<circle cx="151" cy="243" r="2.4" fill="#8ff0c8"/>' +
            '<circle cx="249" cy="243" r="2.4" fill="#8ff0c8"/>' +
          "</g>" +
        "</g>" +
      "</svg>";

    return stage;
  }

  /* Measure each gear so the draw-in animation never leaves a dash gap. */
  function primeDraw(root) {
    var paths = root.querySelectorAll("path");
    for (var i = 0; i < paths.length; i++) {
      var len = 0;
      try { len = paths[i].getTotalLength(); } catch (e) { len = 0; }
      if (len > 0) {
        paths[i].style.strokeDasharray = len + " " + len;
        paths[i].style.setProperty("--gbss-len", String(len));
      }
    }
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
      var stage = buildEmblem();
      var hint = overlay.querySelector(".gb-screensaver-hint");
      overlay.insertBefore(stage, hint);
      primeDraw(stage);
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
