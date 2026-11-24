/* Slides Insert hub — RUNTIME promise-path tests (#2428).
 * All work runs against the shipped hub/placement functions in the shared vm
 * sandbox. captureCall only observes the promise a real method already returns.
 */
"use strict";

var h = require("./00_harness");
var test = h.asyncTest;
var assert = h.assert;
var env = h.env;
var hub = env.hub;
var place = env.place;
var win = env.win;
var doc = env.doc;

function okResponse(json) {
  return {
    ok: true,
    status: 200,
    json: function () { return Promise.resolve(json || {}); },
  };
}

function captureCall(owner, name, invoke) {
  var original = owner[name];
  var args = null;
  var captured = null;
  owner[name] = function () {
    args = Array.prototype.slice.call(arguments);
    var result = original.apply(owner, args);
    captured = result && typeof result.then === "function" ? result : Promise.resolve(result);
    return result;
  };
  try {
    invoke();
  } catch (e) {
    owner[name] = original;
    throw e;
  }
  if (!captured) {
    owner[name] = original;
    throw new Error(name + " was never called");
  }
  return captured.then(function (value) {
    owner[name] = original;
    return { value: value, args: args };
  }, function (error) {
    owner[name] = original;
    throw error;
  });
}

test("[UAT:SLIDE-02] RUNTIME: layout placement posts slide/add with next position and refreshes", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse()); });
  return place.place(sFind("layout-title"), {}).then(function () {
    assert(env.fetchCalls.length === 1, "layout should issue one request");
    var call = env.fetchCalls[0];
    assert(call.url === "/api/slides/slide/add", "wrong layout endpoint");
    assert(call.init.method === "POST" && call.init.credentials === "same-origin", "wrong request options");
    var body = JSON.parse(call.init.body);
    assert(body.presentation_id === "<deck>" && body.layout === "title" && body.position === 1,
      "layout payload wrong");
    assert(env.refreshCalls.join(",") === "1", "layout refresh target wrong");
  });
});

function sFind(id) {
  return env.sandbox.siFind(id);
}

test("RUNTIME: doInsert remembers, closes, toasts and sends the chosen text", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse()); });
  hub.open();
  hub.setCategory("text");
  hub.select("text-body");
  hub.options.querySelector('[data-si-name="text"]').value = "<hello>";
  return captureCall(place, "place", function () { hub.doInsert(); }).then(function (seen) {
    assert(seen.args[0].id === "text-body", "insert sent the wrong catalog item");
    assert(seen.args[1].text === "<hello>", "insert dropped the entered text");
    assert(env.fetchCalls.length === 1, "insert did not use element/add");
    assert(env.fetchCalls[0].url === "/api/slides/element/add", "wrong element endpoint");
    assert(JSON.parse(env.fetchCalls[0].init.body).text === "<hello>", "posted text wrong");
    assert(env.sandbox.siRecentIds()[0] === "text-body", "inserted id not remembered");
    assert(hub.dialog === null, "hub did not close after insert");
    assert(env.toasts.length === 1 && env.toasts[0].kind === "success", "no success toast");
    assert(env.toasts[0].message === "<slides-insert-status-inserted>", "wrong toast message");
  });
});

test("RUNTIME: diagram insertion emits one post per cell with numbered steps", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse()); });
  return place.place(sFind("diagram-flow"), {}).then(function () {
    assert(env.fetchCalls.length === 3, "flow diagram should emit three shapes");
    var texts = env.fetchCalls.map(function (call) {
      return JSON.parse(call.init.body).text;
    });
    assert(texts.join(",") ===
      "<slides-insert-diagram-step>,<slides-insert-diagram-step>,<slides-insert-diagram-step>",
      "diagram step labels are not i18n keys");
    assert(env.refreshCalls.length === 1, "diagram should refresh once");
  });
});

test("RUNTIME: Drive paths resolve through share-link before element/add", function () {
  env.reset();
  env.setFetchHandler(function (url) {
    if (url === "/api/files/share-link") return Promise.resolve(okResponse({ url: "/link/p.png" }));
    return Promise.resolve(okResponse());
  });
  win.sessionStorage.setItem("drive-scope", "workspace");
  return place.place(sFind("media-image"), { drivePath: "/folder/p.png" }).then(function () {
    assert(env.fetchCalls.length === 2, "Drive insert should make two calls");
    assert(env.fetchCalls[0].url === "/api/files/share-link", "share-link was not first");
    var share = JSON.parse(env.fetchCalls[0].init.body);
    assert(share.path === "/folder/p.png" && share.scope === "workspace" && share.bucket === "",
      "share-link request wrong");
    assert(env.fetchCalls[1].url === "/api/slides/element/add", "image was not inserted");
    assert(JSON.parse(env.fetchCalls[1].init.body).src === "http://localhost/link/p.png",
      "resolved Drive URL not used as image src");
  });
});

test("RUNTIME: absolute media URLs skip share-link and a missing link rejects", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse()); });
  return place.place(sFind("media-image"), { src: "http://localhost/direct.png" }).then(function () {
    assert(env.fetchCalls.length === 1 && env.fetchCalls[0].url === "/api/slides/element/add",
      "absolute src must not call share-link");
    env.reset();
    env.setFetchHandler(function () { return Promise.resolve(okResponse({ url: "" })); });
    return place.place(sFind("media-image"), { drivePath: "/missing.png" }).then(function () {
      throw new Error("a missing Drive link should reject");
    }, function (error) {
      assert(error.message === "<slides-insert-drive-link-failed>", "unexpected link failure message");
    });
  });
});

test("RUNTIME: resolveDrive handles empty, absolute and relative links", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse({ url: "/rel.png" })); });
  return Promise.all([
    place.resolveDrive(""),
    place.resolveDrive("http://localhost/abs.png"),
    place.resolveDrive("/rel.png"),
  ]).then(function (urls) {
    assert(urls.join("|") === "|http://localhost/abs.png|http://localhost/rel.png",
      "resolveDrive results wrong");
    assert(env.fetchCalls.length === 1, "only relative paths should call the server");
  });
});

test("RUNTIME: reuseItems maps deck elements and the reuse tab renders them", function () {
  env.reset();
  env.setFetchHandler(function () {
    return Promise.resolve(okResponse({
      slides: [
        { elements: [
          { element_type: "text", width: 100, height: 40, content: { text: "<body>" } },
          { element_type: "chart", width: 200, height: 120, content: { chart_data: { chart_type: "bar" } } },
        ] },
      ],
    }));
  });
  hub.open();
  return captureCall(place, "reuseItems", function () { hub.setCategory("reuse"); }).then(function (seen) {
    assert(seen.value.length === 2, "reuse list size wrong");
    assert(seen.value[0].id === "reuse-0-0" && seen.value[1].id === "reuse-0-1",
      "reuse ids are not slide/element addresses");
    assert(hub.reuseItems.length === 2, "hub did not adopt loaded reuse items");
    assert(itemIds(hub.cards()) === "reuse-0-0,reuse-0-1", "reuse grid not rendered");
    return place.place(seen.value[0], {});
  }).then(function () {
    var body = JSON.parse(env.fetchCalls[env.fetchCalls.length - 1].init.body);
    assert(body.text === "<body>" && body.element_type === "text", "reuse payload wrong");
    hub.close();
  });
});

function itemIds(items) {
  return items.map(function (item) { return item.id || item.dataset.itemId; }).join(",");
}

test("RUNTIME: canvas drag/drop converts pointer coordinates with canvas scale", function () {
  env.reset();
  hub.open();
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  canvas.setAttribute("data-slide-id", "2");
  doc.body.appendChild(canvas);
  win.SlideCanvas.scale = 2;
  var transfer = {
    types: ["application/x-slides-insert"],
    getData: function (type) { return type === "application/x-slides-insert" ? "shape-star" : ""; },
  };
  var over = h.fire(doc, "dragover", { target: canvas, dataTransfer: transfer });
  assert(over.defaultPrevented === true && transfer.dropEffect === "copy", "dragover not accepted");
  return captureCall(place, "place", function () {
    h.fire(doc, "drop", { target: canvas, dataTransfer: transfer, clientX: 80, clientY: 60 });
  }).then(function (seen) {
    assert(seen.args[0].id === "shape-star", "drop used the wrong insertable");
    assert(JSON.stringify(seen.args[2]) === JSON.stringify({ x: 40, y: 30 }),
      "drop coordinates ignored canvas scale");
    assert(hub.dialog === null, "drop did not close the hub");
  });
});

test("RUNTIME: Drive-file drop becomes a media insert with a drivePath option", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse({ url: "/dropped.png" })); });
  hub.open();
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  doc.body.appendChild(canvas);
  var transfer = {
    types: ["application/x-gb-drive-file"],
    getData: function (type) {
      return type === "application/x-gb-drive-file" ? JSON.stringify({ path: "/docs/a.png" }) : "";
    },
  };
  return captureCall(place, "place", function () {
    h.fire(doc, "drop", { target: canvas, dataTransfer: transfer, clientX: 10, clientY: 20 });
  }).then(function (seen) {
    assert(seen.args[0].id === "media-image", "Drive drop did not select image insert");
    assert(seen.args[1].drivePath === "/docs/a.png", "Drive drop lost the file path");
  });
});

test("RUNTIME: insertion failure re-enables the button and reports status", function () {
  env.reset();
  env.setFetchHandler(function () {
    return Promise.resolve({ ok: false, status: 503, json: function () { return Promise.resolve({}); } });
  });
  hub.open();
  hub.setCategory("text");
  hub.select("text-body");
  hub.doInsert();
  return new Promise(function (resolve) { setImmediate(resolve); }).then(function () {
    assert(hub.insertBtn.disabled === false, "failed insert left the button disabled");
    assert(h.text(hub.status) === "<slides-insert-status-failed>", "failure status missing");
    assert(env.toasts.length === 0, "failed insert must not show a success toast");
    hub.close();
  });
});

test("RUNTIME: Drive attach button previews the resolved file and reports status", function () {
  env.reset();
  env.setFetchHandler(function () { return Promise.resolve(okResponse({ url: "/attached.png" })); });
  hub.open();
  hub.setCategory("media");
  hub.select("media-drive");
  hub.options.querySelector('[data-si-name="drivePath"]').value = "/folder/a.png";
  var attach = hub.options.querySelector(".sl-insert-resolve");
  return captureCall(place, "resolveDrive", function () { h.fire(attach, "click", {}); }).then(function () {
    var image = hub.preview.querySelector(".sl-insert-preview-image");
    assert(image && image.src === "http://localhost/attached.png", "Drive preview not updated");
    assert(h.text(hub.status) === "<slides-insert-drive-attached>", "Drive attachment status wrong");
    hub.close();
  });
});
