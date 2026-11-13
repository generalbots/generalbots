(function () {
    'use strict';
    var CS = window.__calCS;
    // calendar.js keeps its helpers in its own closure. Call them through the
    // module it exposes.
    var BASE = window.CalendarModule || {};
    function renderMiniCalendar() { return window.CalendarModule.renderMiniCalendar(); }
    function renderCurrentView() { return window.CalendarModule.renderCurrentView(); }
    function loadEvents() { return window.CalendarModule.loadEvents(); }
    function navigate(direction) { return window.CalendarModule.navigate(direction); }

    var drawImpl = (window.CalendarModule && window.CalendarModule.__drawEvents) || function () {};

    function checkConflict(start, end) {
        if (window.CalendarModule && window.CalendarModule.__checkConflict) {
            return window.CalendarModule.__checkConflict(start, end);
        }
        return false;
    }

    function bindEvents() {
        // View selector
        document.querySelectorAll('.view-btn').forEach(function (btn) {
            btn.addEventListener('click', function () {
                document.querySelectorAll('.view-btn').forEach(function (b) { b.classList.remove('active'); });
                this.classList.add('active');
                CS.currentView = this.dataset.view;
                renderCurrentView();
                loadEvents();
            });
        });

        var todayBtn = document.getElementById('today-btn');
        if (todayBtn) {
            todayBtn.addEventListener('click', function () {
                CS.currentDate = new Date();
                CS.selectedDate = new Date();
                renderMiniCalendar();
                renderCurrentView();
                loadEvents();
            });
        }

        var prevPeriod = document.getElementById('prev-period');
        var nextPeriod = document.getElementById('next-period');
        var prevMonth = document.getElementById('prev-month');
        var nextMonth = document.getElementById('next-month');

        if (prevPeriod) prevPeriod.addEventListener('click', function () { navigate(-1); });
        if (nextPeriod) nextPeriod.addEventListener('click', function () { navigate(1); });
        if (prevMonth) prevMonth.addEventListener('click', function () {
            CS.currentDate.setMonth(CS.currentDate.getMonth() - 1);
            renderMiniCalendar();
        });
        if (nextMonth) nextMonth.addEventListener('click', function () {
            CS.currentDate.setMonth(CS.currentDate.getMonth() + 1);
            renderMiniCalendar();
        });

        if (CS.miniCalDays) {
            CS.miniCalDays.addEventListener('click', function (e) {
                if (e.target.classList.contains('mini-day')) {
                    var dateParts = e.target.dataset.date.split('-');
                    CS.selectedDate = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
                    CS.currentDate = new Date(CS.selectedDate);
                    renderMiniCalendar();
                    renderCurrentView();
                    loadEvents();
                }
            });
        }

        function openEventModal() {
            if (window.CalendarModule && window.CalendarModule.refreshElements) {
                window.CalendarModule.refreshElements();
            }
            var modal = document.getElementById('event-modal');
            CS.eventModal = modal;
            if (!modal) return;
            modal.classList.remove('hidden');
            var first = modal.querySelector('input[name="title"]');
            if (first) first.focus();
        }

        var newEventBtn = document.getElementById('new-event-btn');
        if (newEventBtn) {
            newEventBtn.addEventListener('click', function (e) {
                e.preventDefault();
                openEventModal();
            });
        }

        document.querySelectorAll('.close-modal').forEach(function (btn) {
            btn.addEventListener('click', function (e) {
                var modal = e.target.closest('.modal') || document.getElementById('event-modal');
                if (modal) modal.classList.add('hidden');
            });
        });

        var eventModal = document.getElementById('event-modal');
        if (eventModal) {
            eventModal.addEventListener('click', function (e) {
                if (e.target === eventModal) eventModal.classList.add('hidden');
            });
        }

        var closePopup = document.querySelector('.close-popup');
        var eventPopup = document.getElementById('event-popup');
        if (closePopup && eventPopup) {
            closePopup.addEventListener('click', function () {
                eventPopup.classList.add('hidden');
            });
        }

        var toggleSidebarBtn = document.getElementById('toggle-cal-sidebar');
        var calendarSidebar = document.getElementById('calendar-sidebar');
        if (toggleSidebarBtn && calendarSidebar) {
            toggleSidebarBtn.addEventListener('click', function () {
                calendarSidebar.classList.toggle('collapsed');
            });
        }

        var monthGrid = document.getElementById('month-grid');
        if (monthGrid) {
            monthGrid.addEventListener('click', function (e) {
                var monthDay = e.target.closest('.month-day');
                if (monthDay && !monthDay.classList.contains('other-month')) {
                    var dateParts = monthDay.dataset.date ? monthDay.dataset.date.split('-') : null;
                    if (dateParts) {
                        CS.selectedDate = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
                        var modal = document.getElementById('event-modal');
                        if (modal) {
                            modal.classList.remove('hidden');
                            var startInput = modal.querySelector('input[name="start"]');
                            var endInput = modal.querySelector('input[name="end"]');
                            if (startInput && endInput) {
                                var dateStr = CS.selectedDate.toISOString().slice(0, 10);
                                startInput.value = dateStr + 'T09:00';
                                endInput.value = dateStr + 'T10:00';
                            }
                            var first = modal.querySelector('input[name="title"]');
                            if (first) first.focus();
                        }
                    }
                }
            });
        }

        var eventForm = document.getElementById('event-form');
        if (eventForm) {
            eventForm.addEventListener('submit', function (e) {
                e.preventDefault();

                var title = eventForm.querySelector('input[name="title"]').value;
                var startVal = eventForm.querySelector('input[name="start"]').value;
                var endVal = eventForm.querySelector('input[name="end"]').value;
                var allDay = eventForm.querySelector('input[name="all_day"]').checked;
                var location = eventForm.querySelector('input[name="location"]').value;
                var description = eventForm.querySelector('textarea[name="description"]').value;

                var calendarId = (eventForm.querySelector('input[name="calendar_id"]') || {}).value ||
                    (window.CalendarModule && window.CalendarModule.currentCalendarId) ||
                    '00000000-0000-0000-0000-000000000000';
                var body = {
                    title: title,
                    start_time: new Date(startVal).toISOString(),
                    end_time: new Date(endVal).toISOString(),
                    all_day: allDay,
                    location: location,
                    description: description,
                    organizer: 'user',
                    calendar_id: null
                };
                var conflictBody = {
                    calendar_id: calendarId,
                    start_time: body.start_time,
                    end_time: body.end_time
                };

                function calConfirm(message, onYes, onNo) {
                    if (window.GBModal && typeof window.GBModal.confirm === 'function') {
                        window.GBModal.confirm(message, 'Confirm', onYes, onNo || function () {});
                    } else if (window.GBToasts && typeof window.GBToasts.show === 'function') {
                        window.GBToasts.show('Calendar', message + ' Action cancelled (dialog unavailable).', 'warning');
                        if (typeof onNo === 'function') onNo();
                    } else {
                        console.warn('Confirm unavailable, cancelling:', message);
                        if (typeof onNo === 'function') onNo();
                    }
                }

                function sendEventSave() {
                    fetch('/api/calendar/events', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    })
                        .then(function (r) { return r.json(); })
                        .then(function () {
                            var modal = document.getElementById('event-modal');
                            if (modal) modal.classList.add('hidden');
                            eventForm.reset();
                            loadEvents();
                        })
                        .catch(function (err) {
                            console.error('Failed to save event:', err);
                        });
                }

                function proceedAfterLocalCheck() {
                    fetch('/api/calendar/conflicts', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(conflictBody)
                    })
                        .then(function (r) { return r.ok ? r.json() : null; })
                        .then(function (conflictRes) {
                            if (conflictRes && conflictRes.has_conflicts) {
                                var names = conflictRes.conflicts.map(function (c) { return c.title; }).slice(0, 3).join(', ');
                                calConfirm('Server detected conflicts with: ' + names + '. Save anyway?', function () {
                                    sendEventSave();
                                });
                                return;
                            }
                            sendEventSave();
                        })
                        .catch(function (err) {
                            console.error("Calendar conflict check failed, saving event anyway:", err);
                            sendEventSave();
                        });
                }

                if (checkConflict(startVal, endVal)) {
                    calConfirm('This slot overlaps with an existing event. Do you want to book it anyway?', function () {
                        proceedAfterLocalCheck();
                    });
                    return;
                }

                proceedAfterLocalCheck();
            });
        }

        document.addEventListener('keydown', function (e) {
            if (e.key === 'Escape') {
                var modal = document.getElementById('event-modal');
                var popup = document.getElementById('event-popup');
                if (modal) modal.classList.add('hidden');
                if (popup) popup.classList.add('hidden');
            }
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
            switch (e.key) {
                case 't': document.getElementById('today-btn') && document.getElementById('today-btn').click(); break;
                case 'd': var db = document.querySelector('[data-view="day"]'); if (db) db.click(); break;
                case 'w': var wb = document.querySelector('[data-view="week"]'); if (wb) wb.click(); break;
                case 'm': var mb = document.querySelector('[data-view="month"]'); if (mb) mb.click(); break;
                case 'ArrowLeft': navigate(-1); break;
                case 'ArrowRight': navigate(1); break;
                case 'n': var nb = document.getElementById('new-event-btn'); if (nb) nb.click(); break;
            }
        });
    }

    var base = window.CalendarModule || {};
    window.CalendarModule = Object.assign({}, base, {
        __bindEvents: bindEvents,
        __drawEvents: drawImpl
    });

    var bootInit = window.CalendarModule.init;
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () { if (bootInit) bootInit(); });
    } else if (bootInit) {
        bootInit();
    }
})();
