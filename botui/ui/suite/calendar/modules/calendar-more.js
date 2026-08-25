(function () {
'use strict';
    var CS = window.__calCS;
    function drawEventsMonth() {
        const cells = document.querySelectorAll('.month-day');
        cells.forEach(cell => {
            const dateStr = cell.dataset.date;
            if (!dateStr) return;
            const parts = dateStr.split('-');
            const cellYear = parseInt(parts[0], 10);
            const cellMonth = parseInt(parts[1], 10) - 1;
            const cellDay = parseInt(parts[2], 10);

            const dayEvents = CS.events.filter(evt => {
                const d = new Date(evt.start_time);
                return d.getFullYear() === cellYear && d.getMonth() === cellMonth && d.getDate() === cellDay;
            });

            const container = cell.querySelector('.month-day-CS.events');
            if (container) {
                container.innerHTML = dayEvents.map(evt => {
                    const color = evt.color || '#3b82f6';
                    return `<div class="event-chip" data-id="${evt.id}" style="background:${color}20; border-left:3px solid ${color}; color:${color}; font-size:10px; padding:2px 4px; margin-top:2px; border-radius:3px; cursor:pointer;" onclick="event.stopPropagation(); window.CalendarModule.showEventPopup(event, '${evt.id}')">${escapeHtml(evt.title)}</div>`;
                }).join('');
            }
        });
    }

    function drawEventsWeek() {
        const columns = document.querySelectorAll('#week-grid .day-column');
        columns.forEach(col => {
            col.querySelectorAll('.event-chip-week').forEach(chip => chip.remove());
        });

        const weekStart = getWeekStart(CS.selectedDate);
        CS.events.forEach(evt => {
            const start = new Date(evt.start_time);
            const end = new Date(evt.end_time);
            
            const diffTime = start.getTime() - weekStart.getTime();
            const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
            if (diffDays >= 0 && diffDays < 7) {
                const col = columns[diffDays];
                if (col) {
                    const startMinutes = start.getHours() * 60 + start.getMinutes();
                    const durationMinutes = (end.getTime() - start.getTime()) / (1000 * 60);
                    const top = (startMinutes / 60) * 48;
                    const height = (durationMinutes / 60) * 48;
                    const color = evt.color || '#3b82f6';

                    const chip = document.createElement('div');
                    chip.className = 'event-chip-week';
                    chip.style.cssText = `position:absolute; top:${top}px; left:4px; right:4px; height:${height}px; background:${color}20; border-left:3px solid ${color}; color:${color}; font-size:11px; padding:4px; border-radius:4px; cursor:pointer; overflow:hidden; z-index:2;`;
                    chip.innerHTML = `<b>${escapeHtml(evt.title)}</b>`;
                    chip.onclick = (e) => { e.stopPropagation(); showEventPopup(e, evt.id); };
                    col.appendChild(chip);
                }
            }
        });
    }

    function drawEventsDay() {
        const container = document.getElementById('day-CS.events');
        if (!container) return;
        container.innerHTML = '';

        const cellYear = CS.selectedDate.getFullYear();
        const cellMonth = CS.selectedDate.getMonth();
        const cellDay = CS.selectedDate.getDate();

        const dayEvents = CS.events.filter(evt => {
            const d = new Date(evt.start_time);
            return d.getFullYear() === cellYear && d.getMonth() === cellMonth && d.getDate() === cellDay;
        });

        dayEvents.forEach(evt => {
            const start = new Date(evt.start_time);
            const end = new Date(evt.end_time);
            const startMinutes = start.getHours() * 60 + start.getMinutes();
            const durationMinutes = (end.getTime() - start.getTime()) / (1000 * 60);
            const top = (startMinutes / 60) * 48;
            const height = (durationMinutes / 60) * 48;
            const color = evt.color || '#3b82f6';

            const chip = document.createElement('div');
            chip.className = 'event-chip-day';
            chip.style.cssText = `position:absolute; top:${top}px; left:4px; right:4px; height:${height}px; background:${color}20; border-left:3px solid ${color}; color:${color}; font-size:12px; padding:6px; border-radius:4px; cursor:pointer; overflow:hidden; z-index:2;`;
            chip.innerHTML = `<b>${escapeHtml(evt.title)}</b><br><span style="font-size:10px;">${escapeHtml(evt.location || '')}</span>`;
            chip.onclick = (e) => { e.stopPropagation(); showEventPopup(e, evt.id); };
            container.appendChild(chip);
        });
    }

    function checkConflict(start, end) {
        const s = new Date(start).getTime();
        const e = new Date(end).getTime();
        return CS.events.some(evt => {
            const evtS = new Date(evt.start_time).getTime();
            const evtE = new Date(evt.end_time).getTime();
            return (s < evtE && e > evtS);
        });
    }

    function showEventPopup(e, eventId) {
        const evt = CS.events.find(x => x.id === eventId);
        if (!evt) return;

        const popup = document.getElementById('event-popup');
        if (!popup) return;

        popup.querySelector('.popup-title').textContent = evt.title;
        popup.querySelector('.popup-desc').textContent = evt.description || "No description";
        popup.querySelector('.popup-loc').textContent = evt.location || "No location";
        
        const start = new Date(evt.start_time);
        const end = new Date(evt.end_time);
        popup.querySelector('.popup-time').textContent = `${start.toLocaleDateString()} ${start.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})} - ${end.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}`;

        popup.style.left = `${e.clientX + 10}px`;
        popup.style.top = `${e.clientY + 10}px`;
        popup.style.position = 'fixed';
        popup.classList.remove('hidden');
    }

    /**
     * Bind event listeners
     */
    function bindEvents() {
        // View selector
        document.querySelectorAll('.view-btn').forEach(btn => {
            btn.addEventListener('click', function() {
                document.querySelectorAll('.view-btn').forEach(b => b.classList.remove('active'));
                this.classList.add('active');
                CS.currentView = this.dataset.view;
                renderCurrentView();
                loadEvents();
            });
        });

        // Today button
        const todayBtn = document.getElementById('today-btn');
        if (todayBtn) {
            todayBtn.addEventListener('click', () => {
                CS.currentDate = new Date();
                CS.selectedDate = new Date();
                renderMiniCalendar();
                renderCurrentView();
                loadEvents();
            });
        }

        // Navigation
        const prevPeriod = document.getElementById('prev-period');
        const nextPeriod = document.getElementById('next-period');
        const prevMonth = document.getElementById('prev-month');
        const nextMonth = document.getElementById('next-month');

        if (prevPeriod) prevPeriod.addEventListener('click', () => navigate(-1));
        if (nextPeriod) nextPeriod.addEventListener('click', () => navigate(1));
        if (prevMonth) prevMonth.addEventListener('click', () => {
            CS.currentDate.setMonth(CS.currentDate.getMonth() - 1);
            renderMiniCalendar();
        });
        if (nextMonth) nextMonth.addEventListener('click', () => {
            CS.currentDate.setMonth(CS.currentDate.getMonth() + 1);
            renderMiniCalendar();
        });

        // Mini calendar day click
        if (CS.miniCalDays) {
            CS.miniCalDays.addEventListener('click', (e) => {
                if (e.target.classList.contains('mini-day')) {
                    const dateParts = e.target.dataset.date.split('-');
                    CS.selectedDate = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
                    CS.currentDate = new Date(CS.selectedDate);
                    renderMiniCalendar();
                    renderCurrentView();
                    loadEvents();
                }
            });
        }

        // New event button
        const newEventBtn = document.getElementById('new-event-btn');
        if (newEventBtn && CS.eventModal) {
            newEventBtn.addEventListener('click', () => {
                CS.eventModal.classList.remove('hidden');
            });
        }

        // Close modal
        document.querySelectorAll('.close-modal').forEach(btn => {
            btn.addEventListener('click', () => {
                if (CS.eventModal) CS.eventModal.classList.add('hidden');
            });
        });

        // Close modal on backdrop click
        if (CS.eventModal) {
            CS.eventModal.addEventListener('click', (e) => {
                if (e.target === CS.eventModal) {
                    CS.eventModal.classList.add('hidden');
                }
            });
        }

        // Close popup
        const closePopup = document.querySelector('.close-popup');
        if (closePopup && CS.eventPopup) {
            closePopup.addEventListener('click', () => {
                CS.eventPopup.classList.add('hidden');
            });
        }

        // Toggle sidebar
        const toggleSidebarBtn = document.getElementById('toggle-cal-sidebar');
        const calendarSidebar = document.getElementById('calendar-sidebar');
        if (toggleSidebarBtn && calendarSidebar) {
            toggleSidebarBtn.addEventListener('click', () => {
                calendarSidebar.classList.toggle('collapsed');
            });
        }

        // Month day click (create event)
        const monthGrid = document.getElementById('month-grid');
        if (monthGrid && CS.eventModal) {
            monthGrid.addEventListener('click', (e) => {
                const monthDay = e.target.closest('.month-day');
                if (monthDay && !monthDay.classList.contains('other-month')) {
                    const dateParts = monthDay.dataset.date?.split('-');
                    if (dateParts) {
                        CS.selectedDate = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
                        CS.eventModal.classList.remove('hidden');

                        const startInput = document.querySelector('input[name="start"]');
                        const endInput = document.querySelector('input[name="end"]');
                        if (startInput && endInput) {
                            const dateStr = CS.selectedDate.toISOString().slice(0, 10);
                            startInput.value = `${dateStr}T09:00`;
                            endInput.value = `${dateStr}T10:00`;
                        }
                    }
                }
            });
        }

        // Event form submit
        const eventForm = document.getElementById('event-form');
        if (eventForm && CS.eventModal) {
            eventForm.addEventListener('submit', (e) => {
                e.preventDefault();

                const title = eventForm.querySelector('input[name="title"]').value;
                const startVal = eventForm.querySelector('input[name="start"]').value;
                const endVal = eventForm.querySelector('input[name="end"]').value;
                const allDay = eventForm.querySelector('input[name="all_day"]').checked;
                const location = eventForm.querySelector('input[name="location"]').value;
                const description = eventForm.querySelector('textarea[name="description"]').value;

                if (checkConflict(startVal, endVal)) {
                    if (!confirm("This slot overlaps with an existing event. Do you want to book it anyway?")) {
                        return;
                    }
                }

                const calendarId = (eventForm.querySelector('input[name="calendar_id"]') || {}).value
                    || (window.CalendarModule && window.CalendarModule.currentCalendarId)
                    || "00000000-0000-0000-0000-000000000000";
                const body = {
                    title: title,
                    start_time: new Date(startVal).toISOString(),
                    end_time: new Date(endVal).toISOString(),
                    all_day: allDay,
                    location: location,
                    description: description,
                    organizer: "user",
                    calendar_id: null
                };
                const conflictBody = {
                    calendar_id: calendarId,
                    start_time: body.start_time,
                    end_time: body.end_time
                };

                function sendEventSave() {
                    fetch('/api/calendar/CS.events', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    })
                    .then(r => r.json())
                    .then(() => {
                        CS.eventModal.classList.add('hidden');
                        eventForm.reset();
                        loadEvents();
                    })
                    .catch(err => {
                        console.error("Failed to save event:", err);
                    });
                }

                fetch('/api/calendar/conflicts', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(conflictBody)
                })
                .then(r => r.ok ? r.json() : null)
                .then(conflictRes => {
                    if (conflictRes && conflictRes.has_conflicts) {
                        const names = conflictRes.conflicts.map(c => c.title).slice(0, 3).join(", ");
                        if (!confirm("Server detected conflicts with: " + names + ". Save anyway?")) {
                            return;
                        }
                    }
                    sendEventSave();
                })
                .catch(() => sendEventSave());
            });
        }

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                if (CS.eventModal) CS.eventModal.classList.add('hidden');
                if (CS.eventPopup) CS.eventPopup.classList.add('hidden');
            }

            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

            switch (e.key) {
                case 't':
                    document.getElementById('today-btn')?.click();
                    break;
                case 'd':
                    document.querySelector('[data-view="day"]')?.click();
                    break;
                case 'w':
                    document.querySelector('[data-view="week"]')?.click();
                    break;
                case 'm':
                    document.querySelector('[data-view="month"]')?.click();
                    break;
                case 'ArrowLeft':
                    navigate(-1);
                    break;
                case 'ArrowRight':
                    navigate(1);
                    break;
                case 'n':
                    document.getElementById('new-event-btn')?.click();
                    break;
            }
        });
    }

    // Export functions for external use — merge over the core module
    // exposed by calendar.js so init/navigate/renderCurrentView/loadEvents
    // resolve from there instead of this module's empty scope.
    var base = window.CalendarModule || {};
    window.CalendarModule = {
        init: base.init || init,
        navigate: base.navigate || navigate,
        renderCurrentView: base.renderCurrentView || renderCurrentView,
        loadEvents: base.loadEvents || loadEvents,
        drawEvents: base.drawEvents || drawEvents,
        showEventPopup: showEventPopup,
        setView: function(view) {
            CS.currentView = view;
            renderCurrentView();
            loadEvents();
        }
    };

    // Auto-initialize when DOM is ready
    var bootInit = window.CalendarModule.init;
    if (document.readyState === 'loading') {
        (function(){ var __cb = bootInit; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
    } else {
        bootInit();
    }
})();
