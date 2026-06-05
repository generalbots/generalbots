
"use strict";

// Advanced formula engine — 30+ functions, no eval/new Function
// This module provides a comprehensive evaluateFormula that overrides the basic one from 03.
// All helper functions (getCellValue, parseColName, safeEvalArithmetic, parseCellRef,
// parseRange, evaluateSum, evaluateAverage, evaluateCount, evaluateMax, evaluateMin,
// evaluateIf) come from modules 02, 03, and 04 loaded earlier.

function evaluateFormula(formula, sourceRow, sourceCol) {
    if (!formula || !formula.startsWith("=")) return formula;
    var expr = formula.substring(1).trim();

    // Replace cell references with values
    expr = expr.replace(/([A-Z]+)(\d+)/g, function(match, col, row) {
        var r = parseInt(row) - 1;
        var c = parseColName(col);
        var val = getCellValue(r, c);
        var num = parseFloat(val);
        return isNaN(num) ? '"' + val + '"' : num;
    });

    // Match function name
    var fnMatch = expr.match(/^(\w+)\((.+)\)$/);
    if (fnMatch) {
        var fnName = fnMatch[1].toUpperCase();
        var args = fnMatch[2];
        switch (fnName) {
            case "SUM": return evaluateSum(expr);
            case "AVERAGE": return evaluateAverage(expr);
            case "COUNT": return evaluateCount(expr);
            case "MAX": return evaluateMax(expr);
            case "MIN": return evaluateMin(expr);
            case "IF": return evaluateIf(expr);
            case "VLOOKUP": return evaluateVlookup(args);
            case "HLOOKUP": return evaluateHlookup(args);
            case "INDEX": return evaluateIndex(args);
            case "MATCH": return evaluateMatch(args);
            case "CONCATENATE": case "CONCAT": return evaluateConcat(args);
            case "LEFT": return evaluateLeft(args);
            case "RIGHT": return evaluateRight(args);
            case "MID": return evaluateMid(args);
            case "LEN": return evaluateLen(args);
            case "FIND": return evaluateFind(args);
            case "TRIM": return evaluateTrim(args);
            case "UPPER": return evaluateUpper(args);
            case "LOWER": return evaluateLower(args);
            case "PROPER": return evaluateProper(args);
            case "ROUND": return evaluateRound(args);
            case "ROUNDUP": return evaluateRoundup(args);
            case "ROUNDDOWN": return evaluateRounddown(args);
            case "INT": return evaluateInt(args);
            case "ABS": return evaluateAbs(args);
            case "MOD": return evaluateMod(args);
            case "POWER": return evaluatePower(args);
            case "SQRT": return evaluateSqrt(args);
            case "NOW": return new Date().getTime() / 86400000 + 25569;
            case "TODAY": return Math.floor(new Date().getTime() / 86400000 + 25569);
            case "YEAR": return new Date().getFullYear();
            case "MONTH": return new Date().getMonth() + 1;
            case "DAY": return new Date().getDate();
            case "AND": return evaluateAnd(args);
            case "OR": return evaluateOr(args);
            case "NOT": return evaluateNot(args);
            case "IFERROR": return evaluateIferror(args);
            case "IFNA": return evaluateIfna(args);
            case "ISBLANK": return evaluateIsblank(args);
            case "ISNUMBER": return evaluateIsnumber(args);
            case "ISTEXT": return evaluateIstext(args);
            case "SUMIF": return evaluateSumif(args);
            case "COUNTIF": return evaluateCountif(args);
            case "AVERAGEIF": return evaluateAverageif(args);
        }
        return "#ERROR";
    }

    // Simple arithmetic
    var result = safeEvalArithmetic(expr);
    return typeof result === "number" ? Math.round(result * 1000000) / 1000000 : result;
}

// Make globally accessible — overrides the basic version from module 03
window.evaluateFormula = evaluateFormula;

// --- Lookup & Reference Functions ---

function evaluateVlookup(args) {
    var parts = parseArgs(args);
    if (parts.length < 3) return "#ERROR";
    var lookupVal = parts[0];
    var rangeParts = parts[1].split(":");
    if (rangeParts.length < 2) return "#ERROR";
    var colIndex = parseInt(parts[2]);
    if (isNaN(colIndex)) return "#ERROR";
    var start = parseCellRef(rangeParts[0].trim());
    var end = parseCellRef(rangeParts[1].trim());
    if (!start || !end) return "#ERROR";
    for (var r = start.row; r <= end.row; r++) {
        var cellVal = getCellValue(r, start.col);
        if (String(cellVal).toLowerCase() === String(lookupVal).toLowerCase()) {
            var resultCol = start.col + colIndex - 1;
            if (resultCol <= end.col) return getCellValue(r, resultCol);
        }
    }
    var rangeMatch = parts[3];
    if (rangeMatch && rangeMatch.trim().toUpperCase() !== "TRUE") return "#N/A";
    return "#N/A";
}

function evaluateHlookup(args) {
    var parts = parseArgs(args);
    if (parts.length < 3) return "#ERROR";
    var lookupVal = parts[0];
    var rangeParts = parts[1].split(":");
    if (rangeParts.length < 2) return "#ERROR";
    var rowIndex = parseInt(parts[2]);
    if (isNaN(rowIndex)) return "#ERROR";
    var start = parseCellRef(rangeParts[0].trim());
    var end = parseCellRef(rangeParts[1].trim());
    if (!start || !end) return "#ERROR";
    for (var c = start.col; c <= end.col; c++) {
        var cellVal = getCellValue(start.row, c);
        if (String(cellVal).toLowerCase() === String(lookupVal).toLowerCase()) {
            var resultRow = start.row + rowIndex - 1;
            if (resultRow <= end.row) return getCellValue(resultRow, c);
        }
    }
    return "#N/A";
}

function evaluateIndex(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var rangeParts = parts[0].split(":");
    var rowNum = parseInt(parts[1]) - 1;
    if (isNaN(rowNum)) return "#ERROR";
    if (rangeParts.length === 2) {
        var start = parseCellRef(rangeParts[0].trim());
        var end = parseCellRef(rangeParts[1].trim());
        if (start && end) {
            var col = start.col;
            if (parts.length >= 3) {
                col = start.col + parseInt(parts[2]) - 1;
            }
            return getCellValue(start.row + rowNum, col);
        }
    }
    return "#ERROR";
}

function evaluateMatch(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var lookupVal = parts[0];
    var rangeParts = parts[1].split(":");
    if (rangeParts.length < 2) return "#ERROR";
    var start = parseCellRef(rangeParts[0].trim());
    var end = parseCellRef(rangeParts[1].trim());
    if (!start || !end) return "#ERROR";
    if (start.row === end.row) {
        for (var c = start.col; c <= end.col; c++) {
            if (String(getCellValue(start.row, c)).toLowerCase() === String(lookupVal).toLowerCase()) {
                return c - start.col + 1;
            }
        }
    } else if (start.col === end.col) {
        for (var r = start.row; r <= end.row; r++) {
            if (String(getCellValue(r, start.col)).toLowerCase() === String(lookupVal).toLowerCase()) {
                return r - start.row + 1;
            }
        }
    }
    return "#N/A";
}

// --- Text Functions ---

function evaluateConcat(args) {
    var parts = parseArgs(args);
    return parts.map(function(p) { return String(p).replace(/^"(.*)"$/, "$1"); }).join("");
}

function evaluateLeft(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var text = String(parts[0]).replace(/^"(.*)"$/, "$1");
    var n = parseInt(parts[1]);
    return text.substring(0, n);
}

function evaluateRight(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var text = String(parts[0]).replace(/^"(.*)"$/, "$1");
    var n = parseInt(parts[1]);
    return text.substring(text.length - n);
}

function evaluateMid(args) {
    var parts = parseArgs(args);
    if (parts.length < 3) return "#ERROR";
    var text = String(parts[0]).replace(/^"(.*)"$/, "$1");
    var start = parseInt(parts[1]) - 1;
    var n = parseInt(parts[2]);
    return text.substring(start, start + n);
}

function evaluateLen(args) {
    var text = String(parseArgs(args)[0] || "").replace(/^"(.*)"$/, "$1");
    return text.length;
}

function evaluateFind(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var findText = String(parts[0]).replace(/^"(.*)"$/, "$1");
    var withinText = String(parts[1]).replace(/^"(.*)"$/, "$1");
    return withinText.indexOf(findText) + 1;
}

function evaluateTrim(args) {
    var text = String(parseArgs(args)[0] || "").replace(/^"(.*)"$/, "$1");
    return text.trim().replace(/\s+/g, " ");
}

function evaluateUpper(args) {
    var text = String(parseArgs(args)[0] || "").replace(/^"(.*)"$/, "$1");
    return text.toUpperCase();
}

function evaluateLower(args) {
    var text = String(parseArgs(args)[0] || "").replace(/^"(.*)"$/, "$1");
    return text.toLowerCase();
}

function evaluateProper(args) {
    var text = String(parseArgs(args)[0] || "").replace(/^"(.*)"$/, "$1");
    return text.replace(/\w\S*/g, function(w) { return w.charAt(0).toUpperCase() + w.substring(1).toLowerCase(); });
}

// --- Math & Trig Functions ---

function evaluateRound(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var num = parseFloat(parts[0]);
    var digits = parseInt(parts[1]);
    return Math.round(num * Math.pow(10, digits)) / Math.pow(10, digits);
}

function evaluateRoundup(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var num = parseFloat(parts[0]);
    var digits = parseInt(parts[1]);
    var factor = Math.pow(10, digits);
    return Math.ceil(num * factor) / factor;
}

function evaluateRounddown(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var num = parseFloat(parts[0]);
    var digits = parseInt(parts[1]);
    var factor = Math.pow(10, digits);
    return Math.floor(num * factor) / factor;
}

function evaluateInt(args) {
    var num = parseFloat(parseArgs(args)[0]);
    return Math.floor(num);
}

function evaluateAbs(args) {
    return Math.abs(parseFloat(parseArgs(args)[0]));
}

function evaluateMod(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    return parseFloat(parts[0]) % parseFloat(parts[1]);
}

function evaluatePower(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    return Math.pow(parseFloat(parts[0]), parseFloat(parts[1]));
}

function evaluateSqrt(args) {
    var num = parseFloat(parseArgs(args)[0]);
    return num >= 0 ? Math.sqrt(num) : "#NUM!";
}

// --- Logical Functions ---

function evaluateAnd(args) {
    var parts = parseArgs(args);
    for (var i = 0; i < parts.length; i++) {
        if (typeof parts[i] === 'string' && parts[i].startsWith('"')) continue;
        if (!parts[i]) return 0;
    }
    return 1;
}

function evaluateOr(args) {
    var parts = parseArgs(args);
    for (var i = 0; i < parts.length; i++) {
        if (parts[i]) return 1;
    }
    return 0;
}

function evaluateNot(args) {
    return parseArgs(args)[0] ? 0 : 1;
}

function evaluateIferror(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var val = parts[0];
    if (typeof val === 'string' && val.startsWith("#")) return parts[1];
    return val;
}

function evaluateIfna(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    return parts[0] === "#N/A" ? parts[1] : parts[0];
}

function evaluateIsblank(args) {
    var val = parseArgs(args)[0];
    return val === "" || val === undefined || val === null ? 1 : 0;
}

function evaluateIsnumber(args) {
    return typeof parseFloat(parseArgs(args)[0]) === 'number' && !isNaN(parseFloat(parseArgs(args)[0])) ? 1 : 0;
}

function evaluateIstext(args) {
    return typeof parseArgs(args)[0] === 'string' && isNaN(parseFloat(parseArgs(args)[0])) ? 1 : 0;
}

// --- Conditional Aggregation ---

function evaluateSumif(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var rangeParts = parts[0].split(":");
    var criteria = String(parts[1]).replace(/^"(.*)"$/, "$1");
    var sumRange = parts.length >= 3 ? parts[2] : parts[0];
    var sumParts = sumRange.split(":");
    var start = parseCellRef(rangeParts[0].trim());
    if (!start) return "#ERROR";
    var sumStart = parseCellRef(sumParts[0].trim());
    if (!sumStart) return "#ERROR";
    var end = rangeParts.length >= 2 ? parseCellRef(rangeParts[1].trim()) : start;
    var sumEnd = sumParts.length >= 2 ? parseCellRef(sumParts[1].trim()) : sumStart;
    if (!end || !sumEnd) return "#ERROR";
    var total = 0;
    for (var r = start.row; r <= end.row; r++) {
        var cellVal = String(getCellValue(r, start.col));
        var matches = criteria.includes("*")
            ? cellVal.toLowerCase().startsWith(criteria.replace("*", "").toLowerCase())
            : cellVal.toLowerCase() === criteria.toLowerCase();
        if (matches) {
            var sv = parseFloat(getCellValue(r, sumStart.col));
            if (!isNaN(sv)) total += sv;
        }
    }
    return total;
}

function evaluateCountif(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var rangeParts = parts[0].split(":");
    var criteria = String(parts[1]).replace(/^"(.*)"$/, "$1");
    var start = parseCellRef(rangeParts[0].trim());
    if (!start) return "#ERROR";
    var end = rangeParts.length >= 2 ? parseCellRef(rangeParts[1].trim()) : start;
    if (!end) return "#ERROR";
    var count = 0;
    for (var r = start.row; r <= end.row; r++) {
        for (var c = start.col; c <= end.col; c++) {
            var cellVal = String(getCellValue(r, c));
            var matches = criteria.includes("*")
                ? cellVal.toLowerCase().startsWith(criteria.replace("*", "").toLowerCase())
                : cellVal.toLowerCase() === criteria.toLowerCase();
            if (matches) count++;
        }
    }
    return count;
}

function evaluateAverageif(args) {
    var parts = parseArgs(args);
    if (parts.length < 2) return "#ERROR";
    var sum = evaluateSumif(args);
    var count = evaluateCountif(args);
    if (count === 0) return "#DIV/0!";
    return sum / count;
}

// --- Argument Parser ---

function parseArgs(argsStr) {
    var parts = [];
    var depth = 0;
    var current = "";
    var inString = false;
    for (var i = 0; i < argsStr.length; i++) {
        var c = argsStr[i];
        if (c === '"' && (i === 0 || argsStr[i-1] !== '\\')) { inString = !inString; current += c; }
        else if (c === ',' && depth === 0 && !inString) { parts.push(current.trim()); current = ""; }
        else if (c === '(') { depth++; current += c; }
        else if (c === ')') { depth--; current += c; }
        else { current += c; }
    }
    if (current.trim()) parts.push(current.trim());
    return parts;
}
