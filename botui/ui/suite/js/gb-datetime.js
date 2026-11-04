"use strict";
/*
 * Shared date/time formatting driven by the locale preferences saved in
 * Settings -> Language (localStorage keys `gb-date-format` / `gb-time-format`).
 *
 * The Settings app writes the preferences and the desktop taskbar clock reads
 * them, so a format change is reflected in the clock at the bottom right
 * instead of being stored and ignored (#1339).
 */
(function () {
    var DEFAULT_TIME_FORMAT = "24h";

    function readPref(key, fallback) {
        try {
            return localStorage.getItem(key) || fallback;
        } catch (err) {
            return fallback;
        }
    }

    /* Locale-aware default. The long-standing MM/DD/YYYY default made a
       Portuguese UI print 11/04/2026 for 4 November. Settings can still
       override the value explicitly. */
    function defaultDateFormat() {
        var locale = "";
        try {
            locale = localStorage.getItem("gb-locale") || navigator.language || "en";
        } catch (err) {
            locale = "en";
        }
        if (/^en-US/i.test(locale)) return "MM/DD/YYYY";
        if (/^(ja|zh|ko)/i.test(locale)) return "YYYY-MM-DD";
        return "DD/MM/YYYY";
    }

    function pad(value) {
        return value < 10 ? "0" + value : String(value);
    }

    function formatTime(date, format) {
        if (format === "12h") {
            var hours = date.getHours();
            var suffix = hours < 12 ? "AM" : "PM";
            var hour12 = hours % 12;
            if (hour12 === 0) hour12 = 12;
            return hour12 + ":" + pad(date.getMinutes()) + " " + suffix;
        }
        return pad(date.getHours()) + ":" + pad(date.getMinutes());
    }

    function formatDate(date, format) {
        var day = pad(date.getDate());
        var month = pad(date.getMonth() + 1);
        var year = date.getFullYear();
        if (format === "DD/MM/YYYY") return day + "/" + month + "/" + year;
        if (format === "YYYY-MM-DD") return year + "-" + month + "-" + day;
        return month + "/" + day + "/" + year;
    }

    window.GBDateTime = {
        timeFormat: function () {
            return readPref("gb-time-format", DEFAULT_TIME_FORMAT);
        },
        dateFormat: function () {
            return readPref("gb-date-format", defaultDateFormat());
        },
        formatTime: function (date) {
            return formatTime(date, this.timeFormat());
        },
        formatDate: function (date) {
            return formatDate(date, this.dateFormat());
        },
        /* Current time/date as an object, ready to paint into the clock. */
        now: function () {
            var date = new Date();
            return {
                time: formatTime(date, this.timeFormat()),
                date: formatDate(date, this.dateFormat()),
            };
        },
    };
})();
