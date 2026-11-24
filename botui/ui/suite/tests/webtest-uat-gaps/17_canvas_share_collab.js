/**
 * [UAT:CANVAS-05] "Share this canvas so a teammate can edit it with me"
 * Steps exercised: (1) the Share/Collaborate controls report the real link,
 * the real collaborator audience and the real connection state (not a static
 * hint); (2) a local edit is broadcast to a second browser and that browser's
 * move is applied to the first canvas through the shipped collab hooks.
 * Drives real canvas modules 02 (loadCollaborators) and 04 (shareCanvas,
 * toggleCollaborate, initCollab) with fetch/GBCollab/GBModal DOM stubs.
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

function makeSharedCanvas(opts) {
  opts = opts || {};
  var alerts = [];
  var copied = [];
  var collaboratorCalls = [];
  var connectCalls = [];
  var edits = [];
  var people = opts.people === undefined
    ? [{ name: "Ana Silva" }, { name: "Bruno Costa" }]
    : opts.people;
  var collab = {
    connected: opts.connected === true,
    connect: function (config) { connectCalls.push(config); },
    sendEdit: function (msg) { edits.push(msg); },
  };
  var stubs = {
    GBModal: { alert: function (msg) { alerts.push(String(msg)); } },
    GBCollab: collab,
    getGBAccessToken: function () { return "test-token"; },
    URLSearchParams: URLSearchParams,
    location: {
      origin: "http://localhost:3000",
      pathname: "/suite/desktop.html",
      search: "",
    },
    navigator: {
      clipboard: {
        writeText: function (text) { copied.push(text); return Promise.resolve(); },
      },
    },
    fetch: function (url, init) {
      collaboratorCalls.push({ url: url, init: init || {} });
      return Promise.resolve({
        ok: true,
        json: function () { return Promise.resolve(people); },
      });
    },
  };
  if (opts.filePath) stubs.__CANVAS_FILE_PATH = opts.filePath;
  var ctx = canvasStub.makeCanvas(stubs);
  ctx.alerts = alerts;
  ctx.copied = copied;
  ctx.collaboratorCalls = collaboratorCalls;
  ctx.connectCalls = connectCalls;
  ctx.edits = edits;
  ctx.collab = collab;
  return ctx;
}

h.test("[UAT:CANVAS-05] share reports real state and a second browser's edit is reflected", function () {
  var ctx = makeSharedCanvas({
    filePath: "designs/landing.draw",
    connected: true,
    people: [{ name: "Ana Silva" }, { name: "Bruno Costa" }],
  });
  var app = ctx.app;

  // Step 1 - the controls report the real link, audience and session state.
  app.toggleCollaborate();
  return flush().then(function () {
    var msg = ctx.alerts[ctx.alerts.length - 1] || "";
    h.assert(msg.indexOf("Share link ready for this canvas.") !== -1,
      "Share control does not report a real link: " + msg);
    h.assert(msg.indexOf("2 collaborator(s) have access.") !== -1,
      "Share control does not report the real audience: " + msg);
    h.assert(msg.indexOf("Real-time collaboration session active.") !== -1,
      "Share control does not report the real connection state: " + msg);
    h.assert(ctx.collaboratorCalls.some(function (c) {
      return /\/api\/canvas\/collaborators$/.test(c.url);
    }), "collaborators were not fetched from the API");

    app.loadCollaborators();
    return flush();
  }).then(function () {
    var panel = ctx.doc.getElementById("collaborators-list").innerHTML;
    h.assert(panel.indexOf('title="Ana Silva"') !== -1 && panel.indexOf(">A<") !== -1,
      "collaborators panel did not render Ana: " + panel);
    h.assert(panel.indexOf('title="Bruno Costa"') !== -1 && panel.indexOf(">B<") !== -1,
      "collaborators panel did not render Bruno: " + panel);

    app.shareCanvas();
    return flush();
  }).then(function () {
    h.assert(ctx.copied.length === 1 && ctx.copied[0].indexOf("app=canvas") !== -1,
      "Share did not copy a canvas link: " + JSON.stringify(ctx.copied));
    h.assert(ctx.copied[0].indexOf("designs%2Flanding.draw") !== -1,
      "Share link does not carry the drawing identity: " + ctx.copied[0]);

    // Step 2 - connect the real collab hooks, then edit and receive a remote move.
    app.initCollab();
    h.assert(ctx.connectCalls.length === 1 && ctx.connectCalls[0].app === "canvas",
      "collaboration session did not connect");
    h.assert(ctx.connectCalls[0].docId === "designs/landing.draw",
      "collaboration session did not bind to the saved drawing");

    app.setFillColor("#ff0000");
    canvasStub.draw(ctx, "rectangle", [10, 10], [40, 40]);
    h.assert(ctx.edits.length === 1, "local edit was not broadcast to the second browser");
    var sent = JSON.parse(ctx.edits[0].content);
    h.assert(sent.length === 1 && sent[0].fill === "#ff0000",
      "broadcast did not carry the drawn shape: " + ctx.edits[0].content);

    var remote = JSON.parse(JSON.stringify(sent));
    remote[0].x = 70;
    remote[0].y = 30;
    ctx.connectCalls[0].onEdit({ content: JSON.stringify(remote) });
    h.assert(app.elements[0].x === 70 && app.elements[0].y === 30,
      "second browser's move was not reflected on the first canvas");

    // An unsaved canvas reports its real state instead of a fake link.
    var unsaved = makeSharedCanvas({ people: [] });
    unsaved.app.toggleCollaborate();
    return flush().then(function () {
      var unsavedMsg = unsaved.alerts[unsaved.alerts.length - 1] || "";
      h.assert(unsavedMsg.indexOf("This canvas is not saved yet") !== -1,
        "unsaved canvas did not report its state: " + unsavedMsg);
      h.assert(unsavedMsg.indexOf("No collaborators have access yet.") !== -1,
        "empty audience was not reported: " + unsavedMsg);
    });
  });
});
