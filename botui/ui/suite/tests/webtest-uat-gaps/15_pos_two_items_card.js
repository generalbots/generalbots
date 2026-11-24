/**
 * [UAT:BIZ-09] "Abra o caixa, registre R$ 89,90 em 2 itens e feche com cartão"
 * Steps exercised: (1) two real line items total R$ 89,90 in the cart;
 * (2) closing with the card method posts one order with those two items and
 * reports the confirmed receipt.
 * Step 1's chat phrase is LLM-driven; the register operations are driven here
 * through the shipped POS app functions (products, cart, payment, checkout).
 * pos.js keeps its controller private inside its IIFE; the test only exposes
 * that same object to the runner - no shipped function is reimplemented.
 */
"use strict";

var vm = require("vm");
var env = require("./00_env");
var h = env.h;

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

function makePos(products) {
  var calls = [];
  var toasts = [];
  var doc = h.makeDocument({
    readyState: "complete",
    ids: [
      "posSearch", "pos-grid", "pos-categories", "cart-items", "cart-count",
      "cart-subtotal", "cart-tax", "cart-total", "pos-confirm", "pos-live-status",
    ],
  });
  // Mirror the browser's textContent -> innerHTML behaviour so PosApp.esc(),
  // which escapes names through a detached div, renders real text.
  var originalCreate = doc.createElement;
  doc.createElement = function (tag) {
    var el = originalCreate(tag);
    var text = "";
    Object.defineProperty(el, "textContent", {
      get: function () { return text; },
      set: function (value) {
        text = String(value == null ? "" : value);
        el.innerHTML = text;
      },
      configurable: true,
    });
    return el;
  };
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve(/\/products$/.test(url) ? products : { id: "ord-1" }); },
    });
  }
  var sandbox = env.baseSandbox(doc, env.makeStore(), {
    fetch: fetchStub,
    GBToasts: {
      show: function (title, message, type) { toasts.push({ title: title, message: message, type: type }); },
    },
    // announce() uses a 20 ms timer; run it immediately so the receipt is seen.
    setTimeout: function (fn) { fn(); return 0; },
  });

  // Expose the shipped controller without altering any of its behaviour.
  var source = h.readFile("pos/pos.js");
  var closing = source.lastIndexOf("})();");
  h.assert(closing !== -1, "pos.js has no outer IIFE to expose the controller from");
  source = source.slice(0, closing) + "window.__PosAppUnderTest = PosApp;\n" + source.slice(closing);
  var context = vm.createContext(sandbox);
  vm.runInContext(source, context, { filename: "pos/pos.js" });
  return { app: sandbox.__PosAppUnderTest, doc: doc, calls: calls, toasts: toasts };
}

h.test("[UAT:BIZ-09] ring up two items totalling 89.90 and close with card", function () {
  var ctx = makePos([
    { id: "p1", name: "Cafe", price: 56.9, stock: 5 },
    { id: "p2", name: "Bolo", price: 33.0, stock: 5 },
  ]);
  var app = ctx.app;
  h.assert(!!app && typeof app.addToCart === "function", "POS controller not exposed");

  // Products load asynchronously on boot; wait before ringing anything up.
  return flush().then(function () {
    h.assert(app.state.products.length === 2, "POS products did not load");

    // Step 1 - two distinct line items whose prices total 89.90.
    app.addToCart("p1");
    app.addToCart("p2");
    h.assert(app.state.cart.length === 2, "expected two line items, got " + app.state.cart.length);
    h.assert(app.cartSubtotalCents() === 8990, "line items do not total 89.90: " + app.cartSubtotalCents());
    h.assert(
      ctx.doc.getElementById("cart-count").textContent === "2 items",
      "cart does not show two items",
    );
    var cart = ctx.doc.getElementById("cart-items").innerHTML;
    h.assert(cart.indexOf("Cafe") !== -1 && cart.indexOf("Bolo") !== -1, "line items are missing: " + cart);
    h.assert(ctx.doc.getElementById("cart-subtotal").textContent === "$89.90", "subtotal is not 89.90");

    // Step 2 - close the sale with the card method.
    var cardButton = h.makeElement("pay-card");
    app.setPayment("card", cardButton);
    h.assert(cardButton.classList.contains("active"), "card payment was not selected");
    return app.confirmOrder().then(function () {
      var orders = ctx.calls.filter(function (c) { return /\/api\/pos\/orders$/.test(c.url); });
      h.assert(orders.length === 1, "expected one cash-register order, got " + orders.length);
      var order = JSON.parse(orders[0].opts.body);
      h.assert(order.payment_method === "card", "sale was not closed with card: " + order.payment_method);
      h.assert(order.items.length === 2, "order does not carry two items: " + JSON.stringify(order.items));
      h.assert(order.subtotal === 89.9, "order subtotal is not 89.90: " + order.subtotal);
      h.assert(
        ctx.toasts.some(function (t) { return /Order confirmed/.test(t.message) && t.type === "success"; }),
        "no receipt/confirmation feedback was shown",
      );
      h.assert(ctx.doc.getElementById("cart-count").textContent === "0 items", "cart was not cleared after closing");
    });
  });
});
