"use strict";
/* Issue #2434 - financial function family for the local Sheets preview.
 * Rates match Excel to ~1e-9 on the standard vectors in the offline harness.
 * Every function receives raw evaluated arguments and returns a number or an
 * Excel error value. */
(function () {
  var E = window.SheetFormulaErrors;
  var TOL = 1e-11;

  function argsOf(values, count, optional) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var converted = E.numbers(values);
    if (E.isError(converted)) return converted;
    while (converted.length < count + (optional || 0)) converted.push(0);
    return converted;
  }

  function pow(rate, n) {
    var value = Math.pow(1 + rate, n);
    return Number.isFinite(value) && value > 0 ? value : E.err("#NUM!");
  }

  function payment(rate, nper, pv, fv, type) {
    if (E.isError(rate)) return rate;
    if (rate === 0) return E.finite(-(pv + fv) / nper);
    var factor = pow(rate, nper);
    if (E.isError(factor)) return factor;
    var result = -(pv * rate * factor + fv * rate) / (factor - 1);
    return E.finite(type ? result / (1 + rate) : result);
  }

  function PMT(values) {
    var a = argsOf(values, 3, 2);
    if (E.isError(a)) return a;
    return payment(a[0], a[1], a[2], a[3], a[4]);
  }

  function futureValue(rate, nper, pmt, pv, type) {
    if (rate === 0) return E.finite(-(pv + pmt * nper));
    var factor = pow(rate, nper);
    if (E.isError(factor)) return factor;
    return E.finite(-(pv * factor + pmt * (1 + rate * type) * (factor - 1) / rate));
  }

  function FV(values) {
    var a = argsOf(values, 3, 2);
    if (E.isError(a)) return a;
    return futureValue(a[0], a[1], a[2], a[3], a[4]);
  }

  function PV(values) {
    var a = argsOf(values, 3, 2);
    if (E.isError(a)) return a;
    if (a[0] === 0) return E.finite(-(a[3] + a[1] * a[2]));
    var factor = pow(a[0], a[1]);
    if (E.isError(factor)) return factor;
    return E.finite(-(a[3] + a[2] * (1 + a[0] * a[4]) * (factor - 1) / a[0]) / factor);
  }

  function IPMT(values) {
    var a = argsOf(values, 4, 2);
    if (E.isError(a)) return a;
    var rate = a[0], period = a[1], periods = a[2], pv = a[3], fv = a[4], type = a[5];
    if (period < 1 || period > periods) return E.err("#NUM!");
    if (rate === 0) return 0;
    var pmt = payment(rate, periods, pv, fv, type);
    if (E.isError(pmt)) return pmt;
    var balance = futureValue(rate, period - 1, pmt, pv, type);
    if (E.isError(balance)) return balance;
    var interest = balance * rate;
    return E.finite(type ? interest / (1 + rate) : interest);
  }

  function PPMT(values) {
    var a = argsOf(values, 4, 2);
    if (E.isError(a)) return a;
    var pmt = payment(a[0], a[2], a[3], a[4], a[5]);
    if (E.isError(pmt)) return pmt;
    var interest = IPMT(values);
    if (E.isError(interest)) return interest;
    return E.finite(pmt - interest);
  }

  function NPER(values) {
    var a = argsOf(values, 3, 2);
    if (E.isError(a)) return a;
    var rate = a[0], pmt = a[1], pv = a[2], fv = a[3], type = a[4];
    if (rate === 0) return pmt === 0 ? E.err("#NUM!") : E.finite(-(pv + fv) / pmt);
    var typeFactor = pmt * (1 + rate * type);
    var ratio = (typeFactor - fv * rate) / (pv * rate + typeFactor);
    var logRatio = E.checkedLog(ratio);
    if (E.isError(logRatio)) return logRatio;
    return E.finite(logRatio / Math.log(1 + rate));
  }

  function rootOf(fn, guess) {
    var seeds = [-0.999999, -0.9, -0.5, -0.1, 0, 0.001, 0.01, 0.05, 0.1, 0.2, 0.5, 1, 2, 10];
    if (Number.isFinite(guess)) seeds.unshift(guess);
    seeds.sort(function (a, b) { return a - b; });
    var previous = null;
    for (var i = 0; i < seeds.length; i += 1) {
      var x = seeds[i];
      var y = fn(x);
      if (!Number.isFinite(y)) continue;
      if (Math.abs(y) < TOL) return x;
      if (previous && Number.isFinite(previous.y) && previous.y * y < 0) {
        var lo = previous.x, hi = x, flo = previous.y;
        for (var j = 0; j < 200; j += 1) {
          var mid = (lo + hi) / 2;
          var fmid = fn(mid);
          if (!Number.isFinite(fmid)) break;
          if (Math.abs(fmid) < TOL || (hi - lo) / 2 < TOL) return mid;
          if (flo * fmid < 0) hi = mid;
          else { lo = mid; flo = fmid; }
        }
      }
      previous = { x: x, y: y };
    }
    return E.err("#NUM!");
  }

  function RATE(values) {
    var a = argsOf(values, 3, 3);
    if (E.isError(a)) return a;
    if (a[1] === 0) return E.err("#NUM!");
    var result = rootOf(function (rate) {
      var paymentValue = payment(rate, a[0], a[2], a[3], a[4]);
      return E.isError(paymentValue) ? NaN : paymentValue - a[1];
    }, a[5] || 0.1);
    return result;
  }

  function npvAt(rate, values, offset) {
    var shift = offset || 0;
    var total = 0;
    for (var i = 0; i < values.length; i += 1) {
      total += values[i] / Math.pow(1 + rate, i + shift);
    }
    return total;
  }

  function NPV(values) {
    var a = argsOf(values, 1);
    if (E.isError(a)) return a;
    var rate = a[0];
    if (rate <= -1) return E.err("#NUM!");
    return E.finite(npvAt(rate, a.slice(1), 1));
  }

  function IRR(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var flows = E.numbers(E.flatten([values[0]]));
    if (E.isError(flows) || flows.length < 2) return E.err("#NUM!");
    var guess = values.length > 1 ? E.num(values[1]) : 0.1;
    if (E.isError(guess)) return guess;
    return rootOf(function (rate) {
      if (rate <= -1) return NaN;
      return npvAt(rate, flows, 0);
    }, Number.isFinite(guess) ? guess : 0.1);
  }

  function MIRR(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var flows = E.numbers(E.flatten([values[0]]));
    if (E.isError(flows)) return flows;
    var finance = E.num(values[1]);
    var reinvest = E.num(values[2]);
    if (E.isError(finance)) return finance;
    if (E.isError(reinvest)) return reinvest;
    var n = flows.length;
    if (n < 2 || finance <= -1 || reinvest <= -1) return E.err("#NUM!");
    var negativePv = 0;
    var positiveFv = 0;
    flows.forEach(function (flow, index) {
      if (flow < 0) negativePv += flow / Math.pow(1 + finance, index);
      else positiveFv += flow * Math.pow(1 + reinvest, n - 1 - index);
    });
    if (negativePv >= 0) return E.err("#NUM!");
    return E.finite(Math.pow(-positiveFv / (negativePv * (1 + finance)), 1 / (n - 1)) - 1);
  }

  function flowParts(rate, flowArg, dateArg) {
    var dates = window.SheetFunctionsDate ? window.SheetFunctionsDate.numbers(dateArg) : null;
    if (!dates || E.isError(dates)) return E.err("#VALUE!");
    var flows = E.numbers(E.flatten([flowArg]));
    if (E.isError(flows)) return flows;
    if (dates.length !== flows.length || dates.length < 2) return E.err("#NUM!");
    return { rate: rate, dates: dates, flows: flows };
  }

  function XNPV(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var rate = E.num(values[0]);
    if (E.isError(rate)) return rate;
    var parts = flowParts(rate, values[1], values[2]);
    if (E.isError(parts)) return parts;
    if (parts.rate <= -1) return E.err("#NUM!");
    var base = parts.dates[0];
    var total = 0;
    for (var i = 0; i < parts.flows.length; i += 1) {
      total += parts.flows[i] / Math.pow(1 + parts.rate, (parts.dates[i] - base) / 365);
    }
    return E.finite(total);
  }

  function XIRR(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var parts = flowParts(0, values[0], values[1]);
    if (E.isError(parts)) return parts;
    var base = parts.dates[0];
    var guess = values.length > 2 ? E.num(values[2]) : 0.1;
    if (E.isError(guess)) return guess;
    return rootOf(function (rate) {
      if (rate <= -1) return NaN;
      var total = 0;
      for (var i = 0; i < parts.flows.length; i += 1) {
        total += parts.flows[i] / Math.pow(1 + rate, (parts.dates[i] - base) / 365);
      }
      return total;
    }, Number.isFinite(guess) ? guess : 0.1);
  }

  function SLN(values) {
    var a = argsOf(values, 3);
    if (E.isError(a)) return a;
    return E.divide(a[0] - a[1], a[2]);
  }

  function SYD(values) {
    var a = argsOf(values, 4);
    if (E.isError(a)) return a;
    if (a[2] <= 0 || a[3] < 1 || a[3] > a[2]) return E.err("#NUM!");
    return E.finite((a[0] - a[1]) * (a[2] - a[3] + 1) * 2 / (a[2] * (a[2] + 1)));
  }

  function DB(values) {
    var a = argsOf(values, 4, 1);
    if (E.isError(a)) return a;
    var life = a[2], period = a[3], month = a[4] || 12;
    if (life <= 0 || period < 1 || month < 1 || month > 12) return E.err("#NUM!");
    if (a[0] <= 0 || a[1] < 0) return E.err("#NUM!");
    var rate = Math.round((1 - Math.pow(a[1] / a[0], 1 / life)) * 1000) / 1000;
    var first = a[0] * rate * month / 12;
    if (period === 1) return E.finite(first);
    if (period === life + 1) return E.finite((a[0] - first - a[0] * rate * (life - 1)) * rate * (12 - month) / 12);
    if (period > life + 1) return E.err("#NUM!");
    return E.finite((a[0] - first) * rate * Math.pow(1 - rate, period - 2));
  }

  function DDB(values) {
    var a = argsOf(values, 4, 1);
    if (E.isError(a)) return a;
    var cost = a[0], salvage = a[1], life = a[2], period = a[3], factor = a[4] || 2;
    if (life <= 0 || period < 1 || period > life || factor <= 0) return E.err("#NUM!");
    var total = 0;
    var depreciation = 0;
    for (var i = 1; i <= Math.floor(period); i += 1) {
      var allowance = Math.min((cost - total) * factor / life, cost - salvage - total);
      depreciation = Math.max(0, allowance);
      total += depreciation;
    }
    return E.finite(depreciation);
  }

  function EFFECT(values) {
    var a = argsOf(values, 2);
    if (E.isError(a)) return a;
    if (a[0] <= 0 || a[1] < 1) return E.err("#NUM!");
    return E.finite(Math.pow(1 + a[0] / a[1], a[1]) - 1);
  }

  function NOMINAL(values) {
    var a = argsOf(values, 2);
    if (E.isError(a)) return a;
    if (a[0] <= 0 || a[1] < 1) return E.err("#NUM!");
    return E.finite(a[1] * (Math.pow(1 + a[0], 1 / a[1]) - 1));
  }

  window.SheetFunctionsFinancial = {
    PMT: PMT, IPMT: IPMT, PPMT: PPMT, FV: FV, PV: PV, RATE: RATE,
    NPER: NPER, NPV: NPV, IRR: IRR, MIRR: MIRR, XIRR: XIRR, XNPV: XNPV,
    SLN: SLN, SYD: SYD, DB: DB, DDB: DDB, EFFECT: EFFECT, NOMINAL: NOMINAL,
  };
})();
