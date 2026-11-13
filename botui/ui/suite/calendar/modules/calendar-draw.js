(function () {
    'use strict';
    var CS = window.__calCS;

    function escapeHtml(str) {
        if (str === null || str === undefined) return '';
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    function getWeekStart(date) {
        var base = window.CalendarModule || {};
        return base.getWeekStart ? base.getWeekStart(date) : (function () {
            var d = new Date(date);
            d.setDate(d.getDate() - d.getDay());
            return d;
        })();
    }

    function drawEventsMonth() {
        var cells = document.querySelectorAll('.month-day');
        cells.forEach(function (cell) {
            var dateStr = cell.dataset.date;
            if (!dateStr) return;
            var parts = dateStr.split('-');
            var cellYear = parseInt(parts[0], 10);
            var cellMonth = parseInt(parts[1], 10) - 1;
            var cellDay = parseInt(parts[2], 10);

            var dayEvents = CS.events.filter(function (evt) {
                var d = new Date(evt.start_time);
                return d.getFullYear() === cellYear && d.getMonth() === cellMonth && d.getDate() === cellDay;
            });

            var container = cell.querySelector('.month-day-events');
            if (container) {
                container.innerHTML = dayEvents.map(function (evt) {
                    var color = evt.color || '#3b82f6';
                    return '<div class="event-chip" data-id="' + evt.id + '" style="background:' + color + '20; border-left:3px solid ' + color + '; color:' + color + '; font-size:10px; padding:2px 4px; margin-top:2px; border-radius:3px; cursor:pointer;" onclick="event.stopPropagation(); window.CalendarModule.showEventPopup(event, \'' + evt.id + '\')">' + escapeHtml(evt.title) + '</div>';
                }).join('');
            }
        });
    }

    function drawEventsWeek() {
        var columns = document.querySelectorAll('#week-grid .day-column');
        columns.forEach(function (col) {
            col.querySelectorAll('.event-chip-week').forEach(function (chip) { chip.remove(); });
        });

        var weekStart = getWeekStart(CS.selectedDate);
        CS.events.forEach(function (evt) {
            var start = new Date(evt.start_time);
            var end = new Date(evt.end_time);
            var diffDays = Math.floor((start.getTime() - weekStart.getTime()) / (1000 * 60 * 60 * 24));
            if (diffDays >= 0 && diffDays < 7) {
                var col = columns[diffDays];
                if (col) {
                    var startMinutes = start.getHours() * 60 + start.getMinutes();
                    var durationMinutes = (end.getTime() - start.getTime()) / (1000 * 60);
                    var top = (startMinutes / 60) * 48;
                    var height = (durationMinutes / 60) * 48;
                    var color = evt.color || '#3b82f6';
                    var chip = document.createElement('div');
                    chip.className = 'event-chip-week';
                    chip.style.cssText = 'position:absolute; top:' + top + 'px; left:4px; right:4px; height:' + height + 'px; background:' + color + '20; border-left:3px solid ' + color + '; color:' + color + '; font-size:11px; padding:4px; border-radius:4px; cursor:pointer; overflow:hidden; z-index:2;';
                    chip.innerHTML = '<b>' + escapeHtml(evt.title) + '</b>';
                    chip.onclick = function (e) { e.stopPropagation(); showEventPopup(e, evt.id); };
                    col.appendChild(chip);
                }
            }
        });
    }

    function drawEventsDay() {
        var container = document.getElementById('day-events');
        if (!container) return;
        container.innerHTML = '';

        var cellYear = CS.selectedDate.getFullYear();
        var cellMonth = CS.selectedDate.getMonth();
        var cellDay = CS.selectedDate.getDate();

        var dayEvents = CS.events.filter(function (evt) {
            var d = new Date(evt.start_time);
            return d.getFullYear() === cellYear && d.getMonth() === cellMonth && d.getDate() === cellDay;
        });

        dayEvents.forEach(function (evt) {
            var start = new Date(evt.start_time);
            var end = new Date(evt.end_time);
            var startMinutes = start.getHours() * 60 + start.getMinutes();
            var durationMinutes = (end.getTime() - start.getTime()) / (1000 * 60);
            var top = (startMinutes / 60) * 48;
            var height = (durationMinutes / 60) * 48;
            var color = evt.color || '#3b82f6';
            var chip = document.createElement('div');
            chip.className = 'event-chip-day';
            chip.style.cssText = 'position:absolute; top:' + top + 'px; left:4px; right:4px; height:' + height + 'px; background:' + color + '20; border-left:3px solid ' + color + '; color:' + color + '; font-size:12px; padding:6px; border-radius:4px; cursor:pointer; overflow:hidden; z-index:2;';
            chip.innerHTML = '<b>' + escapeHtml(evt.title) + '</b><br><span style="font-size:10px;">' + escapeHtml(evt.location || '') + '</span>';
            chip.onclick = function (e) { e.stopPropagation(); showEventPopup(e, evt.id); };
            container.appendChild(chip);
        });
    }

    function drawEvents() {
        if (CS.currentView === 'month') return drawEventsMonth();
        if (CS.currentView === 'week') return drawEventsWeek();
        return drawEventsDay();
    }

    function checkConflict(start, end) {
        var s = new Date(start).getTime();
        var e = new Date(end).getTime();
        return CS.events.some(function (evt) {
            var evtS = new Date(evt.start_time).getTime();
            var evtE = new Date(evt.end_time).getTime();
            return (s < evtE && e > evtS);
        });
    }

    function showEventPopup(e, eventId) {
        var evt = CS.events.find(function (x) { return x.id === eventId; });
        if (!evt) return;

        var popup = document.getElementById('event-popup');
        if (!popup) return;

        var titleEl = document.getElementById('popup-title');
        var timeEl = document.getElementById('popup-time');
        var locEl = document.getElementById('popup-location');
        var locRow = document.getElementById('popup-location-row');
        var descEl = document.getElementById('popup-description');
        var descRow = document.getElementById('popup-description-row');
        var calEl = document.getElementById('popup-calendar');
        var colorEl = document.getElementById('popup-color');

        if (titleEl) titleEl.textContent = evt.title || '';
        if (descEl) descEl.textContent = evt.description || 'No description';
        if (descRow) descRow.style.display = evt.description ? '' : 'none';
        if (locEl) locEl.textContent = evt.location || 'No location';
        if (locRow) locRow.style.display = evt.location ? '' : 'none';
        if (calEl) calEl.textContent = evt.calendar_id || 'Personal';
        if (colorEl) colorEl.style.background = evt.color || '#3b82f6';

        var start = new Date(evt.start_time);
        var end = new Date(evt.end_time);
        var fmtTime = function (d) {
            if (typeof window.gbFormatTime === 'function') return window.gbFormatTime(d);
            if (window.i18n && typeof window.i18n.formatDate === 'function') return window.i18n.formatDate(d, { hour: '2-digit', minute: '2-digit' });
            try {
                var locale = (window.i18n && typeof window.i18n.getLocale === 'function' && window.i18n.getLocale()) || undefined;
                return new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(d);
            } catch (_) { return ''; }
        };
        var fmtDate = function (d) {
            if (typeof window.gbFormatDate === 'function') return window.gbFormatDate(d);
            if (window.i18n && typeof window.i18n.formatDate === 'function') return window.i18n.formatDate(d);
            try {
                var locale = (window.i18n && typeof window.i18n.getLocale === 'function' && window.i18n.getLocale()) || undefined;
                return new Intl.DateTimeFormat(locale).format(d);
            } catch (_) { return ''; }
        };
        if (timeEl) timeEl.textContent = fmtDate(start) + ' ' + fmtTime(start) + ' - ' + fmtTime(end);

        popup.style.left = (e.clientX + 10) + 'px';
        popup.style.top = (e.clientY + 10) + 'px';
        popup.style.position = 'fixed';
        popup.classList.remove('hidden');
    }

    var base = window.CalendarModule || {};
    window.CalendarModule = Object.assign({}, base, {
        __drawEvents: drawEvents,
        showEventPopup: showEventPopup,
        __checkConflict: checkConflict
    });
})();
