"use strict";
/* Issue #2434 - date/time family for the local Sheets preview. Supports both
 * the 1900 and 1904 serial systems (including Excel's 1900 leap-year bug) and
 * NETWORKDAYS/WORKDAY weekend masks. */
(function () {
  var E = window.SheetFormulaErrors;
  var DAY = 86400000;
  var EPOCH_1900 = Date.UTC(1899, 11, 31);
  var EPOCH_1904 = Date.UTC(1904, 0, 1);

  function system() {
    var settings = window.SheetFunctionsSettings;
    return settings && settings.state().dateSystem === "1904" ? "1904" : "1900";
  }

  function epoch() {
    return system() === "1904" ? EPOCH_1904 : EPOCH_1900;
  }

  function serialFromDate(date) {
    var days = Math.floor((date.getTime() - epoch()) / DAY);
    if (system() === "1900" && days >= 60) days += 1;
    return days;
  }

  function dateFromSerial(serial) {
    var days = Math.floor(serial);
    if (system() === "1900" && days >= 61) days -= 1;
    return new Date(epoch() + days * DAY);
  }

  function parseText(text) {
    var parts = String(text).match(/\d+/g);
    if (!parts || parts.length < 3) return E.err("#VALUE!");
    var a = parts.map(Number);
    var year = a[0], month = a[1], day = a[2];
    if (year < 100) year += year < 30 ? 2000 : 1900;
    if (parts.length === 3 && String(text).indexOf("/") >= 0 && year >= 1000) {
      var swap = window.SheetFunctionsSettings && window.SheetFunctionsSettings.state().locale !== "en";
      if (swap) { day = a[0]; month = a[1]; year = a[2]; }
    }
    var date = new Date(Date.UTC(year, month - 1, day));
    if (date.getUTCMonth() !== month - 1 || date.getUTCDate() !== day) return E.err("#VALUE!");
    return serialFromDate(date);
  }

  function serial(value) {
    if (E.isError(value)) return value;
    if (typeof value === "number") return Number.isFinite(value) ? value : E.err("#NUM!");
    if (value instanceof Date) return serialFromDate(value);
    if (typeof value === "string") return parseText(value);
    if (value === null || value === undefined) return 0;
    return E.err("#VALUE!");
  }

  function numbers(value) {
    var failed = E.firstError(value);
    if (failed) return failed;
    return E.flatten([value]).map(function (item) {
      var converted = serial(item);
      return E.isError(converted) ? converted : converted;
    });
  }

  function holidaySet(values) {
    if (values === undefined || values === null) return {};
    var list = numbers(values);
    if (E.isError(list)) return list;
    var set = {};
    list.forEach(function (item) {
      if (E.isError(item)) return;
      set[Math.floor(item)] = true;
    });
    return set;
  }

  function weekendMask(value) {
    if (value === undefined || value === null || value === "" || value === 1) {
      return [false, false, false, false, false, true, true];
    }
    if (typeof value === "string" && value.length === 7) {
      return value.split("").map(function (flag) { return flag === "1"; });
    }
    var code = Number(value);
    if (!Number.isInteger(code) || code < 1 || code > 11) {
      return E.err("#NUM!");
    }
    var single = [6, 0, 1, 2, 3, 4, 5][code - 1] || 6;
    var second = code >= 7 ? (code === 7 ? 0 : single + 1 > 6 ? 0 : single + 1) : null;
    return [0, 1, 2, 3, 4, 5, 6].map(function (day) {
      return day === single || (second !== null && day === second);
    });
  }

  function isWeekend(date, mask) {
    var weekday = (date.getUTCDay() + 6) % 7;
    return Boolean(mask[weekday]);
  }

  function NETWORKDAYS(values) {
    var start = serial(values[0]);
    if (E.isError(start)) return start;
    var end = serial(values[1]);
    if (E.isError(end)) return end;
    var mask = weekendMask(values[3]);
    if (E.isError(mask)) return mask;
    var holidays = holidaySet(values[2]);
    if (E.isError(holidays)) return holidays;
    var step = start <= end ? 1 : -1;
    var count = 0;
    for (var s = start; step > 0 ? s <= end : s >= end; s += step) {
      var date = dateFromSerial(s);
      if (!isWeekend(date, mask) && !holidays[Math.floor(s)]) count += 1;
    }
    return E.finite(step > 0 ? count : -count);
  }

  function NETWORKDAYS_INTL(values) {
    return NETWORKDAYS([values[0], values[1], values[3], values[2]]);
  }

  function WORKDAY(values) {
    var start = serial(values[0]);
    if (E.isError(start)) return start;
    var days = E.num(values[1]);
    if (E.isError(days)) return days;
    var mask = weekendMask(values[3]);
    if (E.isError(mask)) return mask;
    var holidays = holidaySet(values[2]);
    if (E.isError(holidays)) return holidays;
    var step = days >= 0 ? 1 : -1;
    var current = Math.floor(start);
    var remaining = Math.abs(days);
    while (remaining > 0) {
      current += step;
      var date = dateFromSerial(current);
      if (!isWeekend(date, mask) && !holidays[current]) remaining -= 1;
      if (Math.abs(current) > 1e7) return E.err("#NUM!");
    }
    return E.finite(current);
  }

  function WORKDAY_INTL(values) {
    return WORKDAY([values[0], values[1], values[3], values[2]]);
  }

  function shiftMonths(value, months) {
    var start = serial(value);
    if (E.isError(start)) return start;
    var step = E.num(months);
    if (E.isError(step)) return step;
    var date = dateFromSerial(start);
    var day = date.getUTCDate();
    var target = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + Math.floor(step), 1));
    var last = new Date(Date.UTC(target.getUTCFullYear(), target.getUTCMonth() + 1, 0)).getUTCDate();
    target.setUTCDate(Math.min(day, last));
    return serialFromDate(target);
  }

  function EDATE(values) { return shiftMonths(values[0], values[1]); }

  function EOMONTH(values) {
    var shifted = shiftMonths(values[0], values[1]);
    if (E.isError(shifted)) return shifted;
    var date = dateFromSerial(shifted);
    var last = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 0));
    return serialFromDate(last);
  }

  function days360(a, b, european) {
    var dayA = a.getUTCDate(), dayB = b.getUTCDate();
    if (european) { dayA = Math.min(dayA, 30); dayB = Math.min(dayB, 30); }
    else {
      if (dayA === 31) dayA = 30;
      if (dayB === 31 && dayA === 30) dayB = 30;
    }
    return (b.getUTCFullYear() - a.getUTCFullYear()) * 360 +
      (b.getUTCMonth() - a.getUTCMonth()) * 30 + (dayB - dayA);
  }

  function YEARFRAC(values) {
    var start = serial(values[0]);
    if (E.isError(start)) return start;
    var end = serial(values[1]);
    if (E.isError(end)) return end;
    var basis = values.length > 2 ? E.num(values[2]) : 0;
    if (E.isError(basis)) return basis;
    var a = dateFromSerial(start), b = dateFromSerial(end);
    if (basis === 0 || basis === 4) return E.finite(days360(a, b, basis === 4) / 360);
    var actual = (b.getTime() - a.getTime()) / DAY;
    if (basis === 2) return E.finite(actual / 360);
    if (basis === 3) return E.finite(actual / 365);
    if (basis === 1) {
      var year = a.getUTCFullYear();
      var jan1 = new Date(Date.UTC(year, 0, 1));
      var jan1Next = new Date(Date.UTC(year + 1, 0, 1));
      var denominator = (jan1Next.getTime() - jan1.getTime()) / DAY;
      if (b.getUTCFullYear() !== year && a.getUTCMonth() === 0 && a.getUTCDate() === 1) {
        year = b.getUTCFullYear();
        jan1 = new Date(Date.UTC(year, 0, 1));
        jan1Next = new Date(Date.UTC(year + 1, 0, 1));
        denominator = (jan1Next.getTime() - jan1.getTime()) / DAY;
      }
      return E.finite(actual / denominator);
    }
    return E.err("#NUM!");
  }

  function WEEKNUM(values) {
    var value = serial(values[0]);
    if (E.isError(value)) return value;
    var type = values.length > 1 ? E.num(values[1]) : 1;
    if (E.isError(type)) return type;
    var date = dateFromSerial(value);
    if (type === 21) return ISOWEEKNUM([value]);
    var weekStart = type === 2 ? 1 : 0;
    var jan1 = new Date(Date.UTC(date.getUTCFullYear(), 0, 1));
    var offset = (jan1.getUTCDay() - weekStart + 7) % 7;
    return E.finite(Math.floor((value - Math.floor(serialFromDate(jan1)) + offset) / 7) + 1);
  }

  function ISOWEEKNUM(values) {
    var value = serial(values[0]);
    if (E.isError(value)) return value;
    var date = dateFromSerial(value);
    var target = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()));
    var weekday = (target.getUTCDay() + 6) % 7;
    target.setUTCDate(target.getUTCDate() + 3 - weekday);
    var yearStart = new Date(Date.UTC(target.getUTCFullYear(), 0, 4));
    var startWeekday = (yearStart.getUTCDay() + 6) % 7;
    yearStart.setUTCDate(yearStart.getUTCDate() - startWeekday);
    return E.finite(Math.floor((target.getTime() - yearStart.getTime()) / DAY / 7) + 1);
  }

  function DATEDIF(values) {
    var start = serial(values[0]);
    if (E.isError(start)) return start;
    var end = serial(values[1]);
    if (E.isError(end)) return end;
    var unit = String(values[2] || "").toUpperCase();
    if (start > end) return E.err("#NUM!");
    var a = dateFromSerial(start), b = dateFromSerial(end);
    var rawMonths = (b.getUTCFullYear() - a.getUTCFullYear()) * 12 + b.getUTCMonth() - a.getUTCMonth();
    var months = b.getUTCDate() < a.getUTCDate() ? rawMonths - 1 : rawMonths;
    switch (unit) {
      case "D": return E.finite(Math.floor(end) - Math.floor(start));
      case "M": return E.finite(months);
      case "Y": return E.finite(Math.floor(months / 12));
      case "YM": return E.finite(((months % 12) + 12) % 12);
      case "MD": return E.finite((b.getTime() - new Date(Date.UTC(b.getUTCFullYear(), b.getUTCMonth(), a.getUTCDate())).getTime()) / DAY);
      case "YD": return E.finite((b.getTime() - new Date(Date.UTC(b.getUTCFullYear(), a.getUTCMonth(), a.getUTCDate())).getTime()) / DAY);
      default: return E.err("#NUM!");
    }
  }

  window.SheetFunctionsDate = {
    serial: serial, numbers: numbers, dateFromSerial: dateFromSerial,
    NETWORKDAYS: NETWORKDAYS, NETWORKDAYS_INTL: NETWORKDAYS_INTL,
    WORKDAY: WORKDAY, WORKDAY_INTL: WORKDAY_INTL,
    EDATE: EDATE, EOMONTH: EOMONTH, YEARFRAC: YEARFRAC,
    WEEKNUM: WEEKNUM, ISOWEEKNUM: ISOWEEKNUM, DATEDIF: DATEDIF,
  };
})();
