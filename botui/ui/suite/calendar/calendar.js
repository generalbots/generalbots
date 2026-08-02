/* Calendar Module JavaScript - Real API Integration & Conflict Prevention */

(function() {
    'use strict';

    // State
    let currentDate = new Date();
    let currentView = 'week';
    let selectedDate = new Date();
    let events = [];

    // DOM Elements
    let miniCalDays, miniCalTitle, currentPeriod;
    let dayView, weekView, monthView;
    let eventModal, eventPopup;

    function escapeHtml(str) {
        if (!str) return '';
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;');
    }

    /**
     * Initialize calendar module
     */
    function init() {
        // Get DOM elements
        miniCalDays = document.getElementById('mini-cal-days');
        miniCalTitle = document.getElementById('mini-cal-title');
        currentPeriod = document.getElementById('current-period');
        dayView = document.getElementById('day-view');
        weekView = document.getElementById('week-view');
        monthView = document.getElementById('month-view');
        eventModal = document.getElementById('event-modal');
        eventPopup = document.getElementById('event-popup');

        if (!miniCalDays) return; // Not on calendar page

        generateTimeSlots();
        renderMiniCalendar();
        renderCurrentView();
        updateCurrentTimeIndicator();
        setInterval(updateCurrentTimeIndicator, 60000);
        bindEvents();
        loadEvents();
    }

    /**
     * Generate time slots for day/week views
     */
    function generateTimeSlots() {
        const dayTimeSlots = document.getElementById('day-time-slots');
        const weekTimeSlots = document.getElementById('week-time-slots');

        let html = '';
        for (let i = 0; i < 24; i++) {
            const hour = i === 0 ? '12 AM' : i < 12 ? `${i} AM` : i === 12 ? '12 PM' : `${i - 12} PM`;
            html += `<div class="time-slot">${hour}</div>`;
        }

        if (dayTimeSlots) dayTimeSlots.innerHTML = html;
        if (weekTimeSlots) weekTimeSlots.innerHTML = html;

        // Generate week grid columns
        const weekGrid = document.getElementById('week-grid');
        if (weekGrid) {
            let gridHtml = '';
            for (let d = 0; d < 7; d++) {
                gridHtml += '<div class="day-column">';
                for (let h = 0; h < 24; h++) {
                    gridHtml += '<div class="hour-row"></div>';
                }
                gridHtml += '</div>';
            }
            weekGrid.innerHTML = gridHtml;
        }
    }

    /**
     * Render mini calendar in sidebar
     */
    function renderMiniCalendar() {
        if (!miniCalDays || !miniCalTitle) return;

        const year = currentDate.getFullYear();
        const month = currentDate.getMonth();

        miniCalTitle.textContent = new Date(year, month).toLocaleDateString('en-US', {
            month: 'long',
            year: 'numeric'
        });

        const firstDay = new Date(year, month, 1);
        const lastDay = new Date(year, month + 1, 0);
        const startDay = firstDay.getDay();
        const daysInMonth = lastDay.getDate();

        let html = '';
        const today = new Date();

        // Previous month days
        const prevMonthLastDay = new Date(year, month, 0).getDate();
        for (let i = startDay - 1; i >= 0; i--) {
            html += `<button class="mini-day other-month" data-date="${year}-${month}-${prevMonthLastDay - i}">${prevMonthLastDay - i}</button>`;
        }

        // Current month days
        for (let d = 1; d <= daysInMonth; d++) {
            const isToday = today.getDate() === d && today.getMonth() === month && today.getFullYear() === year;
            const isSelected = selectedDate.getDate() === d && selectedDate.getMonth() === month && selectedDate.getFullYear() === year;
            const classes = ['mini-day'];
            if (isToday) classes.push('today');
            if (isSelected) classes.push('selected');
            html += `<button class="${classes.join(' ')}" data-date="${year}-${month + 1}-${d}">${d}</button>`;
        }

        // Next month days
        const remainingDays = 42 - (startDay + daysInMonth);
        for (let i = 1; i <= remainingDays; i++) {
            html += `<button class="mini-day other-month" data-date="${year}-${month + 2}-${i}">${i}</button>`;
        }

        miniCalDays.innerHTML = html;
    }

    /**
     * Render current view (day, week, or month)
     */
    function renderCurrentView() {
        switch (currentView) {
            case 'day':
                renderDayView();
                break;
            case 'week':
                renderWeekView();
                break;
            case 'month':
                renderMonthView();
                break;
        }
        updatePeriodTitle();
    }

    /**
     * Render day view
     */
    function renderDayView() {
        if (!dayView || !weekView || !monthView) return;

        dayView.classList.remove('hidden');
        weekView.classList.add('hidden');
        monthView.classList.add('hidden');

        const dayName = document.getElementById('day-view-name');
        const dayNumber = document.getElementById('day-view-number');

        if (dayName) dayName.textContent = selectedDate.toLocaleDateString('en-US', { weekday: 'long' });
        if (dayNumber) {
            dayNumber.textContent = selectedDate.getDate();
            const today = new Date();
            if (selectedDate.toDateString() === today.toDateString()) {
                dayNumber.classList.add('today');
            } else {
                dayNumber.classList.remove('today');
            }
        }
    }

    /**
     * Render week view
     */
    function renderWeekView() {
        if (!dayView || !weekView || !monthView) return;

        dayView.classList.add('hidden');
        weekView.classList.remove('hidden');
        monthView.classList.add('hidden');

        const weekDaysHeader = document.getElementById('week-days-header');
        if (!weekDaysHeader) return;

        const weekStart = getWeekStart(selectedDate);
        let html = '';
        const today = new Date();

        for (let i = 0; i < 7; i++) {
            const day = new Date(weekStart);
            day.setDate(weekStart.getDate() + i);
            const isToday = day.toDateString() === today.toDateString();

            html += `
                <div class="week-day-header">
                    <span class="day-name">${day.toLocaleDateString('en-US', { weekday: 'short' })}</span>
                    <span class="day-number ${isToday ? 'today' : ''}">${day.getDate()}</span>
                </div>
            `;
        }

        weekDaysHeader.innerHTML = html;
    }

    /**
     * Render month view
     */
    function renderMonthView() {
        if (!dayView || !weekView || !monthView) return;

        dayView.classList.add('hidden');
        weekView.classList.add('hidden');
        monthView.classList.remove('hidden');

        const monthGrid = document.getElementById('month-grid');
        if (!monthGrid) return;

        const year = currentDate.getFullYear();
        const month = currentDate.getMonth();

        const firstDay = new Date(year, month, 1);
        const lastDay = new Date(year, month + 1, 0);
        const startDay = firstDay.getDay();
        const daysInMonth = lastDay.getDate();

        let html = '';
        const today = new Date();

        // Previous month days
        const prevMonthLastDay = new Date(year, month, 0).getDate();
        for (let i = startDay - 1; i >= 0; i--) {
            html += `
                <div class="month-day other-month">
                    <span class="month-day-number">${prevMonthLastDay - i}</span>
                    <div class="month-day-events"></div>
                </div>
            `;
        }

        // Current month days
        for (let d = 1; d <= daysInMonth; d++) {
            const isToday = today.getDate() === d && today.getMonth() === month && today.getFullYear() === year;
            html += `
                <div class="month-day ${isToday ? 'today' : ''}" data-date="${year}-${month + 1}-${d}">
                    <span class="month-day-number">${d}</span>
                    <div class="month-day-events"></div>
                </div>
            `;
        }

        // Next month days
        const totalCells = Math.ceil((startDay + daysInMonth) / 7) * 7;
        const remainingDays = totalCells - (startDay + daysInMonth);
        for (let i = 1; i <= remainingDays; i++) {
            html += `
                <div class="month-day other-month">
                    <span class="month-day-number">${i}</span>
                    <div class="month-day-events"></div>
                </div>
            `;
        }

        monthGrid.innerHTML = html;
    }

    /**
     * Update period title based on current view
     */
    function updatePeriodTitle() {
        if (!currentPeriod) return;

        let title = '';
        switch (currentView) {
            case 'day':
                title = selectedDate.toLocaleDateString('en-US', {
                    weekday: 'long',
                    month: 'long',
                    day: 'numeric',
                    year: 'numeric'
                });
                break;
            case 'week':
                const weekStart = getWeekStart(selectedDate);
                const weekEnd = new Date(weekStart);
                weekEnd.setDate(weekStart.getDate() + 6);
                if (weekStart.getMonth() === weekEnd.getMonth()) {
                    title = `${weekStart.toLocaleDateString('en-US', { month: 'long' })} ${weekStart.getDate()} - ${weekEnd.getDate()}, ${weekStart.getFullYear()}`;
                } else {
                    title = `${weekStart.toLocaleDateString('en-US', { month: 'short' })} ${weekStart.getDate()} - ${weekEnd.toLocaleDateString('en-US', { month: 'short' })} ${weekEnd.getDate()}, ${weekEnd.getFullYear()}`;
                }
                break;
            case 'month':
                title = currentDate.toLocaleDateString('en-US', {
                    month: 'long',
                    year: 'numeric'
                });
                break;
        }
        currentPeriod.textContent = title;
    }

    /**
     * Get start of week (Sunday)
     */
    function getWeekStart(date) {
        const d = new Date(date);
        const day = d.getDay();
        d.setDate(d.getDate() - day);
        return d;
    }

    /**
     * Update current time indicator position
     */
    function updateCurrentTimeIndicator() {
        const indicator = document.getElementById('current-time-indicator');
        if (!indicator) return;

        const now = new Date();
        const minutes = now.getHours() * 60 + now.getMinutes();
        const top = (minutes / 60) * 48; // 48px per hour

        indicator.style.top = `${top + 52}px`; // Offset for header

        if (currentView === 'month') {
            indicator.style.display = 'none';
        } else {
            indicator.style.display = 'block';
        }
    }

    /**
     * Navigate by direction (-1 or 1)
     */
    function navigate(direction) {
        switch (currentView) {
            case 'day':
                selectedDate.setDate(selectedDate.getDate() + direction);
                break;
            case 'week':
                selectedDate.setDate(selectedDate.getDate() + (direction * 7));
                break;
            case 'month':
                currentDate.setMonth(currentDate.getMonth() + direction);
                break;
        }
        currentDate = new Date(selectedDate);
        renderMiniCalendar();
        renderCurrentView();
        loadEvents();
    }

    /**
     * Load events from backend API
     */
    function loadEvents() {
        let start, end;
        if (currentView === 'month') {
            const year = currentDate.getFullYear();
            const month = currentDate.getMonth();
            start = new Date(year, month - 1, 1).toISOString();
            end = new Date(year, month + 2, 0).toISOString();
        } else if (currentView === 'week') {
            const weekStart = getWeekStart(selectedDate);
            start = new Date(weekStart).toISOString();
            const weekEnd = new Date(weekStart);
            weekEnd.setDate(weekStart.getDate() + 7);
            end = weekEnd.toISOString();
        } else { // day
            const dayStart = new Date(selectedDate);
            dayStart.setHours(0,0,0,0);
            start = dayStart.toISOString();
            const dayEnd = new Date(selectedDate);
            dayEnd.setHours(23,59,59,999);
            end = dayEnd.toISOString();
        }

        fetch(`/api/calendar/events?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`)
            .then(r => r.json())
            .then(data => {
                events = data || [];
                drawEvents();
            })
            .catch(err => {
                console.error("Failed to load events:", err);
            });
    }

    /**
     * Draw events on grids
     */
    function drawEvents() {
        if (currentView === 'month') {
            drawEventsMonth();
        } else if (currentView === 'week') {
            drawEventsWeek();
        } else {
            drawEventsDay();
        }
    }

    function drawEventsMonth() {
        const cells = document.querySelectorAll('.month-day');
        cells.forEach(cell => {
            const dateStr = cell.dataset.date;
            if (!dateStr) return;
            const parts = dateStr.split('-');
            const cellYear = parseInt(parts[0], 10);
            const cellMonth = parseInt(parts[1], 10) - 1;
            const cellDay = parseInt(parts[2], 10);

            const dayEvents = events.filter(evt => {
                const d = new Date(evt.start_time);
                return d.getFullYear() === cellYear && d.getMonth() === cellMonth && d.getDate() === cellDay;
            });

            const container = cell.querySelector('.month-day-events');
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

        const weekStart = getWeekStart(selectedDate);
        events.forEach(evt => {
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
        const container = document.getElementById('day-events');
        if (!container) return;
        container.innerHTML = '';

        const cellYear = selectedDate.getFullYear();
        const cellMonth = selectedDate.getMonth();
        const cellDay = selectedDate.getDate();

        const dayEvents = events.filter(evt => {
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
        return events.some(evt => {
            const evtS = new Date(evt.start_time).getTime();
            const evtE = new Date(evt.end_time).getTime();
            return (s < evtE && e > evtS);
        });
    }

    function showEventPopup(e, eventId) {
        const evt = events.find(x => x.id === eventId);
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
                currentView = this.dataset.view;
                renderCurrentView();
                loadEvents();
            });
        });

        // Today button
        const todayBtn = document.getElementById('today-btn');
        if (todayBtn) {
            todayBtn.addEventListener('click', () => {
                currentDate = new Date();
                selectedDate = new Date();
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
            currentDate.setMonth(currentDate.getMonth() - 1);
            renderMiniCalendar();
        });
        if (nextMonth) nextMonth.addEventListener('click', () => {
            currentDate.setMonth(currentDate.getMonth() + 1);
            renderMiniCalendar();
        });

        // Mini calendar day click
        if (miniCalDays) {
            miniCalDays.addEventListener('click', (e) => {
                if (e.target.classList.contains('mini-day')) {
                    const dateParts = e.target.dataset.date.split('-');
                    selectedDate = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
                    currentDate = new Date(selectedDate);
                    renderMiniCalendar();
                    renderCurrentView();
                    loadEvents();
                }
            });
        }

        // New event button
        const newEventBtn = document.getElementById('new-event-btn');
        if (newEventBtn && eventModal) {
            newEventBtn.addEventListener('click', () => {
                eventModal.classList.remove('hidden');
            });
        }

        // Close modal
        document.querySelectorAll('.close-modal').forEach(btn => {
            btn.addEventListener('click', () => {
                if (eventModal) eventModal.classList.add('hidden');
            });
        });

        // Close modal on backdrop click
        if (eventModal) {
            eventModal.addEventListener('click', (e) => {
                if (e.target === eventModal) {
                    eventModal.classList.add('hidden');
                }
            });
        }

        // Close popup
        const closePopup = document.querySelector('.close-popup');
        if (closePopup && eventPopup) {
            closePopup.addEventListener('click', () => {
                eventPopup.classList.add('hidden');
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
        if (monthGrid && eventModal) {
            monthGrid.addEventListener('click', (e) => {
                const monthDay = e.target.closest('.month-day');
                if (monthDay && !monthDay.classList.contains('other-month')) {
                    const dateParts = monthDay.dataset.date?.split('-');
                    if (dateParts) {
                        selectedDate = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
                        eventModal.classList.remove('hidden');

                        const startInput = document.querySelector('input[name="start"]');
                        const endInput = document.querySelector('input[name="end"]');
                        if (startInput && endInput) {
                            const dateStr = selectedDate.toISOString().slice(0, 10);
                            startInput.value = `${dateStr}T09:00`;
                            endInput.value = `${dateStr}T10:00`;
                        }
                    }
                }
            });
        }

        // Event form submit
        const eventForm = document.getElementById('event-form');
        if (eventForm && eventModal) {
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
                    fetch('/api/calendar/events', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    })
                    .then(r => r.json())
                    .then(() => {
                        eventModal.classList.add('hidden');
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
                if (eventModal) eventModal.classList.add('hidden');
                if (eventPopup) eventPopup.classList.add('hidden');
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

    // Export functions for external use
    window.CalendarModule = {
        init,
        navigate,
        renderCurrentView,
        showEventPopup,
        setView: function(view) {
            currentView = view;
            renderCurrentView();
            loadEvents();
        }
    };

    // Auto-initialize when DOM is ready
    if (document.readyState === 'loading') {
        (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
    } else {
        init();
    }
})();
