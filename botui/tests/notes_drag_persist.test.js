"use strict";
/* Sticky-notes drag persistence (#2261 NOTE-01) — zero-dependency unit test.
 * Run: node tests/notes_drag_persist.test.js
 *
 * Loads ui/suite/notes/notes.js against a minimal board shim: the board's
 * innerHTML setter turns the rendered markup back into fake cards, so the
 * real drag handlers, the real localStorage record and the real re-render run.
 * Asserts the position is written into the note record on release and is
 * restored by a fresh render (i.e. after a reload). */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const SRC = path.join(__dirname, "..", "ui", "suite", "notes", "notes.js");

let passed = 0;
let failed = 0;

function assertEqual(actual, expected, label) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    passed += 1;
  } else {
    failed += 1;
    console.error("FAIL " + label + "\n  expected: " + e + "\n  actual:   " + a);
  }
}

function assertTrue(cond, label) {
  if (cond) passed += 1;
  else {
    failed += 1;
    console.error("FAIL " + label);
  }
}

function makeCard(index, ts, html) {
  const listeners = {};
  const left = /left:(-?\d+)px/.exec(html || "");
  const top = /top:(-?\d+)px/.exec(html || "");
  const card = {
    index: index,
    ts: ts,
    style: { left: left ? left[1] + "px" : "", top: top ? top[1] + "px" : "" },
    attributes: { "data-i": String(index), "data-ts": String(ts) },
    getAttribute: function (name) { return this.attributes[name]; },
    getBoundingClientRect: function () {
      return { left: 100, top: 200, width: 220, height: 120 };
    },
    setPointerCapture: function () {},
    addEventListener: function (name, fn) { (listeners[name] = listeners[name] || []).push(fn); },
    removeEventListener: function (name, fn) {
      const list = listeners[name] || [];
      const at = list.indexOf(fn);
      if (at > -1) list.splice(at, 1);
    },
    fire: function (name, ev) { (listeners[name] || []).slice().forEach(function (fn) { fn(ev); }); },
    listeners: listeners,
  };
  return card;
}

function load() {
  const store = {};
  const boardListeners = {};
  let cards = [];

  const board = {
    style: {},
    scrollLeft: 0,
    scrollTop: 0,
    clientLeft: 0,
    clientTop: 0,
    clientWidth: 1200,
    innerHTML: "",
    getBoundingClientRect: function () { return { left: 0, top: 0, width: 1200, height: 600 }; },
    addEventListener: function (name, fn) { (boardListeners[name] = boardListeners[name] || []).push(fn); },
    querySelectorAll: function (sel) {
      if (sel === ".note-card") return cards;
      return [];
    },
  };
  Object.defineProperty(board, "innerHTML", {
    get: function () { return this._html || ""; },
    set: function (value) {
      this._html = value;
      cards = [];
      const re = /data-i="(\d+)" data-ts="(\d+)"/g;
      let m = re.exec(value);
      while (m) {
        const fragment = value.slice(m.index, value.indexOf("</div>", m.index));
        cards.push(makeCard(parseInt(m[1], 10), m[2], fragment));
        m = re.exec(value);
      }
    },
  });

  const sandbox = {
    window: {},
    document: {
      readyState: "complete",
      addEventListener: function () {},
      getElementById: function (id) {
        if (id === "notesBoard") return board;
        return { textContent: "", addEventListener: function () {} };
      },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
    },
    localStorage: {
      getItem: function (k) { return Object.prototype.hasOwnProperty.call(store, k) ? store[k] : null; },
      setItem: function (k, v) { store[k] = String(v); },
      removeItem: function (k) { delete store[k]; },
    },
    console: console,
    Math: Math,
    String: String,
    Number: Number,
    JSON: JSON,
    Date: Date,
  };
  sandbox.window = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(SRC, "utf8"), sandbox, { filename: SRC });
  return { notes: sandbox.GBNotes, store: store, board: board, cards: function () { return cards; } };
}

function record(store) {
  return JSON.parse(store["gb-sticky-notes"] || "[]");
}

function pointerTarget() {
  return { closest: function () { return null; } };
}

// A pointer event with the fields the drag path reads.
function pointerEvent(x, y) {
  return { target: pointerTarget(), clientX: x, clientY: y, pointerId: 1, preventDefault: function () {} };
}

// A board with one note, created before the module boots.
const env = load();
env.notes.add("buy milk");
assertTrue(env.cards().length === 1, "the note rendered as a card");
assertEqual(record(env.store)[0].x, undefined, "a fresh note has no position yet");

// Drag it 140px right / 60px down from the card's laid-out spot.
const card = env.cards()[0];
card.fire("pointerdown", pointerEvent(400, 300));
assertTrue(env.board.style.position === "relative", "the board becomes the positioning context while dragging");
card.fire("pointermove", { clientX: 540, clientY: 360 });
card.fire("pointerup", { clientX: 540, clientY: 360 });

const saved = record(env.store)[0];
assertEqual(typeof saved.x, "number", "x is persisted in the note record");
assertEqual(typeof saved.y, "number", "y is persisted in the note record");
assertEqual(saved.x, 240, "x matches the drag delta (100 + 140)");
assertEqual(saved.y, 260, "y matches the drag delta (200 + 60)");
assertEqual(saved.text, "buy milk", "the note text survives the drag");

// A reload re-reads the same record: the position must come back as absolutely
// positioned markup, not as a transient inline style on a detached card.
env.notes.render();
const rerendered = env.cards()[0];
assertTrue(/position:absolute/.test(env.board.innerHTML), "the restored card is absolutely positioned");
assertTrue(env.board.innerHTML.indexOf("left:240px") > -1, "left:240px comes from the record");
assertTrue(env.board.innerHTML.indexOf("top:260px") > -1, "top:260px comes from the record");
assertTrue(env.board.innerHTML.indexOf("data-ts=\"" + saved.ts + "\"") > -1, "the note keeps its ts key");
assertEqual(rerendered.ts, String(saved.ts), "the card is the persisted note");

// A second drag updates the same record instead of adding a field elsewhere.
env.notes.render();
const again = env.cards()[0];
again.fire("pointerdown", pointerEvent(500, 500));
again.fire("pointermove", { clientX: 520, clientY: 530 });
again.fire("pointerup", { clientX: 520, clientY: 530 });
assertEqual(record(env.store)[0].x, 260, "a second drag moves the note right");
assertEqual(record(env.store)[0].y, 290, "a second drag moves the note down");
assertEqual(record(env.store).length, 1, "dragging never duplicates the note");

console.log((failed ? "FAILED " : "OK ") + passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
