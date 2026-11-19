"use strict";
/* Issue #2434 - function registry, help metadata, examples and locale aliases.
 * `badge` records whether a function exists in the Rust engine ("native") or
 * is only emulated by this local preview ("local"). */
(function () {
  function entry(name, category, signature, args, helpKey, example, expected, badge) {
    return {
      name: name, category: category, signature: signature, args: args,
      helpKey: helpKey, example: example, expected: expected, badge: badge,
    };
  }

  var FUNCTIONS = [
    entry("PMT", "financial", "PMT(rate, nper, pv, [fv], [type])", ["rate", "nper", "pv", "fv", "type"], "sheet-fn-help-pmt", "PMT(0.05/12,360,100000)", -536.8216230121398, "local"),
    entry("IPMT", "financial", "IPMT(rate, per, nper, pv, [fv], [type])", ["rate", "per", "nper", "pv", "fv", "type"], "sheet-fn-help-ipmt", "IPMT(0.1/12,1,36,8000)", -66.3341127119215, "local"),
    entry("PPMT", "financial", "PPMT(rate, per, nper, pv, [fv], [type])", ["rate", "per", "nper", "pv", "fv", "type"], "sheet-fn-help-ppmt", "PPMT(0.1/12,1,36,8000)", -190.696896763988, "local"),
    entry("FV", "financial", "FV(rate, nper, pmt, [pv], [type])", ["rate", "nper", "pmt", "pv", "type"], "sheet-fn-help-fv", "FV(0.05/12,12,-100,-1000)", -103888.3042390117, "local"),
    entry("PV", "financial", "PV(rate, nper, pmt, [fv], [type])", ["rate", "nper", "pmt", "fv", "type"], "sheet-fn-help-pv", "PV(0.05/12,12,-100)", 1168.122200429816, "local"),
    entry("RATE", "financial", "RATE(nper, pmt, pv, [fv], [type], [guess])", ["nper", "pmt", "pv", "fv", "type", "guess"], "sheet-fn-help-rate", "RATE(36,-257.026,8000)", 0.008333333, "local"),
    entry("NPER", "financial", "NPER(rate, pmt, pv, [fv], [type])", ["rate", "pmt", "pv", "fv", "type"], "sheet-fn-help-nper", "NPER(0.05/12,-100,1000)", 10.103, "local"),
    entry("NPV", "financial", "NPV(rate, value1, [value2], ...)", ["rate", "values"], "sheet-fn-help-npv", "NPV(0.1,-1000,100,200,300,400,500)", 65.25883105351687, "local"),
    entry("IRR", "financial", "IRR(values, [guess])", ["values", "guess"], "sheet-fn-help-irr", "IRR([-1000,100,200,300,400,500])", 0.1200576195419627, "local"),
    entry("MIRR", "financial", "MIRR(values, finance_rate, reinvest_rate)", ["values", "finance", "reinvest"], "sheet-fn-help-mirr", "MIRR([-1000,100,200,300,400,500],0.1,0.1)", 0.09296228706421239, "local"),
    entry("XIRR", "financial", "XIRR(values, dates, [guess])", ["values", "dates", "guess"], "sheet-fn-help-xirr", "XIRR([-1000,1100],[46022,46387])", 0.109, "local"),
    entry("XNPV", "financial", "XNPV(rate, values, dates)", ["rate", "values", "dates"], "sheet-fn-help-xnpv", "XNPV(0.1,[-1000,1100],[46022,46387])", 0, "local"),
    entry("SLN", "financial", "SLN(cost, salvage, life)", ["cost", "salvage", "life"], "sheet-fn-help-sln", "SLN(10000,1000,10)", 900, "local"),
    entry("SYD", "financial", "SYD(cost, salvage, life, period)", ["cost", "salvage", "life", "period"], "sheet-fn-help-syd", "SYD(10000,1000,10,1)", 1636.3636363636363, "local"),
    entry("DB", "financial", "DB(cost, salvage, life, period, [month])", ["cost", "salvage", "life", "period", "month"], "sheet-fn-help-db", "DB(10000,1000,10,1)", 2060, "local"),
    entry("DDB", "financial", "DDB(cost, salvage, life, period, [factor])", ["cost", "salvage", "life", "period", "factor"], "sheet-fn-help-ddb", "DDB(10000,1000,10,1)", 2000, "local"),
    entry("EFFECT", "financial", "EFFECT(nominal_rate, npery)", ["nominal", "npery"], "sheet-fn-help-effect", "EFFECT(0.05,12)", 0.05116189788173298, "local"),
    entry("NOMINAL", "financial", "NOMINAL(effect_rate, npery)", ["effect", "npery"], "sheet-fn-help-nominal", "NOMINAL(0.051161897881733,12)", 0.05, "local"),

    entry("NETWORKDAYS", "date", "NETWORKDAYS(start, end, [holidays])", ["start", "end", "holidays"], "sheet-fn-help-networkdays", "NETWORKDAYS(\"2026-03-02\",\"2026-03-08\")", 5, "local"),
    entry("NETWORKDAYS.INTL", "date", "NETWORKDAYS.INTL(start, end, [weekend], [holidays])", ["start", "end", "weekend", "holidays"], "sheet-fn-help-networkdays-intl", "NETWORKDAYS.INTL(\"2026-03-02\",\"2026-03-08\",1)", 5, "local"),
    entry("WORKDAY", "date", "WORKDAY(start, days, [holidays])", ["start", "days", "holidays"], "sheet-fn-help-workday", "WORKDAY(\"2026-03-06\",1)", 46085, "local"),
    entry("WORKDAY.INTL", "date", "WORKDAY.INTL(start, days, [weekend], [holidays])", ["start", "days", "weekend", "holidays"], "sheet-fn-help-workday-intl", "WORKDAY.INTL(\"2026-03-06\",1,1)", 46085, "local"),
    entry("EOMONTH", "date", "EOMONTH(start, months)", ["start", "months"], "sheet-fn-help-eomonth", "EOMONTH(\"2026-02-10\",1)", 46086, "local"),
    entry("EDATE", "date", "EDATE(start, months)", ["start", "months"], "sheet-fn-help-edate", "EDATE(\"2026-01-31\",1)", 46079, "local"),
    entry("YEARFRAC", "date", "YEARFRAC(start, end, [basis])", ["start", "end", "basis"], "sheet-fn-help-yearfrac", "YEARFRAC(\"2026-01-01\",\"2026-07-01\",1)", 0.4958904109589041, "local"),
    entry("WEEKNUM", "date", "WEEKNUM(serial, [return_type])", ["serial", "type"], "sheet-fn-help-weeknum", "WEEKNUM(\"2026-01-05\",2)", 2, "local"),
    entry("ISOWEEKNUM", "date", "ISOWEEKNUM(serial)", ["serial"], "sheet-fn-help-isoweeknum", "ISOWEEKNUM(\"2026-01-01\")", 1, "local"),
    entry("DATEDIF", "date", "DATEDIF(start, end, unit)", ["start", "end", "unit"], "sheet-fn-help-datedif", "DATEDIF(\"2026-01-01\",\"2027-03-01\",\"Y\")", 1, "local"),

    entry("CORREL", "statistical", "CORREL(array1, array2)", ["array1", "array2"], "sheet-fn-help-correl", "CORREL([1,2,3],[2,4,6])", 1, "local"),
    entry("SLOPE", "statistical", "SLOPE(known_y, known_x)", ["known_y", "known_x"], "sheet-fn-help-slope", "SLOPE([1,2,3],[2,4,6])", 2, "local"),
    entry("INTERCEPT", "statistical", "INTERCEPT(known_y, known_x)", ["known_y", "known_x"], "sheet-fn-help-intercept", "INTERCEPT([1,2,3],[2,4,6])", 0, "local"),
    entry("FORECAST.LINEAR", "statistical", "FORECAST.LINEAR(x, known_y, known_x)", ["x", "known_y", "known_x"], "sheet-fn-help-forecast-linear", "FORECAST.LINEAR(4,[1,2,3],[2,4,6])", 8, "local"),
    entry("STDEV.S", "statistical", "STDEV.S(number1, [number2], ...)", ["values"], "sheet-fn-help-stdev-s", "STDEV.S([1,2,3,4])", 1.2909944487358056, "local"),
    entry("STDEV.P", "statistical", "STDEV.P(number1, [number2], ...)", ["values"], "sheet-fn-help-stdev-p", "STDEV.P([1,2,3,4])", 1.118033988749895, "local"),
    entry("VAR.S", "statistical", "VAR.S(number1, [number2], ...)", ["values"], "sheet-fn-help-var-s", "VAR.S([1,2,3,4])", 1.6666666666666667, "local"),
    entry("VAR.P", "statistical", "VAR.P(number1, [number2], ...)", ["values"], "sheet-fn-help-var-p", "VAR.P([1,2,3,4])", 1.25, "local"),
    entry("MODE.SNGL", "statistical", "MODE.SNGL(number1, [number2], ...)", ["values"], "sheet-fn-help-mode-sngl", "MODE.SNGL([1,2,2,3])", 2, "local"),
    entry("NORM.DIST", "statistical", "NORM.DIST(x, mean, sd, cumulative)", ["x", "mean", "sd", "cumulative"], "sheet-fn-help-norm-dist", "NORM.DIST(0,0,1,TRUE)", 0.5, "local"),
    entry("NORM.INV", "statistical", "NORM.INV(probability, mean, sd)", ["probability", "mean", "sd"], "sheet-fn-help-norm-inv", "NORM.INV(0.5,0,1)", 0, "local"),

    entry("TEXTJOIN", "text", "TEXTJOIN(delimiter, ignore_empty, text1, ...)", ["delimiter", "ignore", "values"], "sheet-fn-help-textjoin", "TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")", "a-b", "local"),
    entry("CONCAT", "text", "CONCAT(text1, [text2], ...)", ["values"], "sheet-fn-help-concat", "CONCAT(\"a\",\"b\")", "ab", "local"),
    entry("NUMBERVALUE", "text", "NUMBERVALUE(text, [decimal], [group])", ["text", "decimal", "group"], "sheet-fn-help-numbervalue", "NUMBERVALUE(\"1.234,56\",\",\",\".\")", 1234.56, "local"),

    entry("SUM", "logic", "SUM(number1, [number2], ...)", ["values"], "sheet-fn-help-sum", "SUM(1,2,3)", 6, "native"),
    entry("AVERAGE", "logic", "AVERAGE(number1, [number2], ...)", ["values"], "sheet-fn-help-average", "AVERAGE(1,2,3)", 2, "native"),
    entry("COUNT", "logic", "COUNT(value1, [value2], ...)", ["values"], "sheet-fn-help-count", "COUNT(1,\"x\",2)", 2, "native"),
    entry("COUNTIF", "logic", "COUNTIF(range, criterion)", ["range", "criterion"], "sheet-fn-help-countif", "COUNTIF([1,2,3],\">1\")", 2, "native"),
    entry("IF", "logic", "IF(condition, true_value, [false_value])", ["condition", "true", "false"], "sheet-fn-help-if", "IF(TRUE,1,2)", 1, "native"),
    entry("IFERROR", "logic", "IFERROR(value, value_if_error)", ["value", "fallback"], "sheet-fn-help-iferror", "IFERROR(1/0,0)", 0, "native"),
    entry("IFNA", "logic", "IFNA(value, value_if_na)", ["value", "fallback"], "sheet-fn-help-ifna", "IFNA(#N/A,9)", 9, "native"),
    entry("ISERROR", "logic", "ISERROR(value)", ["value"], "sheet-fn-help-iserror", "ISERROR(1/0)", true, "native"),
    entry("ISNA", "logic", "ISNA(value)", ["value"], "sheet-fn-help-isna", "ISNA(1/0)", false, "native"),
    entry("MAX", "logic", "MAX(number1, [number2], ...)", ["values"], "sheet-fn-help-max", "MAX(1,5,3)", 5, "native"),
    entry("MIN", "logic", "MIN(number1, [number2], ...)", ["values"], "sheet-fn-help-min", "MIN(1,5,3)", 1, "native"),
    entry("ROUND", "logic", "ROUND(number, digits)", ["number", "digits"], "sheet-fn-help-round", "ROUND(1.2345,2)", 1.23, "native"),
  ];

  var ALIASES = {
    en: {},
    "pt-BR": { SOMA: "SUM", MEDIA: "AVERAGE", "MÉDIA": "AVERAGE", CONT: "COUNT", "CONT.SE": "COUNTIF", MAXIMO: "MAX", "MÁXIMO": "MAX", MINIMO: "MIN", "MÍNIMO": "MIN", SE: "IF", ARRED: "ROUND" },
    es: { SUMA: "SUM", PROMEDIO: "AVERAGE", CONTAR: "COUNT", "CONTAR.SI": "COUNTIF", MAXIMO: "MAX", MINIMO: "MIN", SI: "IF", REDONDEAR: "ROUND" },
    fr: { SOMME: "SUM", MOYENNE: "AVERAGE", NB: "COUNT", "NB.SI": "COUNTIF", MAX: "MAX", MIN: "MIN", SI: "IF", ARRONDI: "ROUND" },
    de: { SUMME: "SUM", MITTELWERT: "AVERAGE", ANZAHL: "COUNT", "ZÄHLENWENN": "COUNTIF", MAX: "MAX", MIN: "MIN", WENN: "IF", RUNDEN: "ROUND" },
    ja: { 合計: "SUM", 平均: "AVERAGE", 件数: "COUNT", "COUNTIF": "COUNTIF", 最大: "MAX", 最小: "MIN", もし: "IF", 四捨五入: "ROUND" },
    ko: { 합계: "SUM", 평균: "AVERAGE", 개수: "COUNT", "COUNTIF": "COUNTIF", 최대: "MAX", 최소: "MIN", 만약: "IF", 반올림: "ROUND" },
    "zh-CN": { 求和: "SUM", 平均值: "AVERAGE", 计数: "COUNT", "COUNTIF": "COUNTIF", 最大值: "MAX", 最小值: "MIN", 如果: "IF", 四舍五入: "ROUND" },
  };

  var BY_NAME = {};
  FUNCTIONS.forEach(function (item) { BY_NAME[item.name] = item; });

  function all() { return FUNCTIONS.slice(); }

  function get(name) { return BY_NAME[name] || null; }

  function resolve(name, locale) {
    var upper = String(name).toUpperCase();
    if (BY_NAME[upper]) return BY_NAME[upper];
    var table = ALIASES[locale] || ALIASES.en;
    var canonical = table[upper] || table[name] || null;
    return canonical ? BY_NAME[canonical] || null : null;
  }

  function categories() { return ["financial", "date", "statistical", "text", "logic"]; }

  function search(query, locale) {
    var needle = String(query || "").trim().toUpperCase();
    if (!needle) return all();
    return all().filter(function (item) {
      var aliasText = Object.keys(ALIASES[locale] || {}).filter(function (key) {
        return ALIASES[locale][key] === item.name;
      }).join(" ");
      return item.name.toUpperCase().indexOf(needle) >= 0 || aliasText.toUpperCase().indexOf(needle) >= 0;
    });
  }

  window.SheetFunctionsCatalog = {
    FUNCTIONS: FUNCTIONS, ALIASES: ALIASES, all: all, get: get,
    resolve: resolve, categories: categories, search: search,
  };
})();
