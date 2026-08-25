/* Calendar Module JavaScript - Real API Integration & Conflict Prevention */

(function() {
    if (window.GBAppLifecycle) GBAppLifecycle.begin('calendar');
    'use strict';
        var CS = window.__calCS = {
            currentDate: new Date(), currentView: 'week', selectedDate: new Date(), events: [],
            miniCalDays: null, miniCalTitle: null, currentPeriod: null,
            dayView: null, weekView: null, monthView: null, eventModal: null, eventPopup: null
        };

    // State

    // DOM Elements

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
        CS.miniCalDays = document.getElementById('mini-cal-days');
        CS.miniCalTitle = document.getElementById('mini-cal-title');
        CS.currentPeriod = document.getElementById('current-period');
        CS.dayView = document.getElementById('day-view');
        CS.weekView = document.getElementById('week-view');
        CS.monthView = document.getElementById('month-view');
        CS.eventModal = document.getElementById('event-modal');
        CS.eventPopup = document.getElementById('event-popup');

        if (!CS.miniCalDays) return; // Not on calendar page

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
        if (!CS.miniCalDays || !CS.miniCalTitle) return;

        const year = CS.currentDate.getFullYear();
        const month = CS.currentDate.getMonth();

        CS.miniCalTitle.textContent = new Date(year, month).toLocaleDateString('en-US', {
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
            const isSelected = CS.selectedDate.getDate() === d && CS.selectedDate.getMonth() === month && CS.selectedDate.getFullYear() === year;
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

        CS.miniCalDays.innerHTML = html;
    }

    /**
     * Render current view (day, week, or month)
     */
    function renderCurrentView() {
        switch (CS.currentView) {
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
        if (!CS.dayView || !CS.weekView || !CS.monthView) return;

        CS.dayView.classList.remove('hidden');
        CS.weekView.classList.add('hidden');
        CS.monthView.classList.add('hidden');

        const dayName = document.getElementById('day-view-name');
        const dayNumber = document.getElementById('day-view-number');

        if (dayName) dayName.textContent = CS.selectedDate.toLocaleDateString('en-US', { weekday: 'long' });
        if (dayNumber) {
            dayNumber.textContent = CS.selectedDate.getDate();
            const today = new Date();
            if (CS.selectedDate.toDateString() === today.toDateString()) {
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
        if (!CS.dayView || !CS.weekView || !CS.monthView) return;

        CS.dayView.classList.add('hidden');
        CS.weekView.classList.remove('hidden');
        CS.monthView.classList.add('hidden');

        const weekDaysHeader = document.getElementById('week-days-header');
        if (!weekDaysHeader) return;

        const weekStart = getWeekStart(CS.selectedDate);
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
        if (!CS.dayView || !CS.weekView || !CS.monthView) return;

        CS.dayView.classList.add('hidden');
        CS.weekView.classList.add('hidden');
        CS.monthView.classList.remove('hidden');

        const monthGrid = document.getElementById('month-grid');
        if (!monthGrid) return;

        const year = CS.currentDate.getFullYear();
        const month = CS.currentDate.getMonth();

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
                    <div class="month-day-CS.events"></div>
                </div>
            `;
        }

        // Current month days
        for (let d = 1; d <= daysInMonth; d++) {
            const isToday = today.getDate() === d && today.getMonth() === month && today.getFullYear() === year;
            html += `
                <div class="month-day ${isToday ? 'today' : ''}" data-date="${year}-${month + 1}-${d}">
                    <span class="month-day-number">${d}</span>
                    <div class="month-day-CS.events"></div>
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
                    <div class="month-day-CS.events"></div>
                </div>
            `;
        }

        monthGrid.innerHTML = html;
    }

    /**
     * Update period title based on current view
     */
    function updatePeriodTitle() {
        if (!CS.currentPeriod) return;

        let title = '';
        switch (CS.currentView) {
            case 'day':
                title = CS.selectedDate.toLocaleDateString('en-US', {
                    weekday: 'long',
                    month: 'long',
                    day: 'numeric',
                    year: 'numeric'
                });
                break;
            case 'week':
                const weekStart = getWeekStart(CS.selectedDate);
                const weekEnd = new Date(weekStart);
                weekEnd.setDate(weekStart.getDate() + 6);
                if (weekStart.getMonth() === weekEnd.getMonth()) {
                    title = `${weekStart.toLocaleDateString('en-US', { month: 'long' })} ${weekStart.getDate()} - ${weekEnd.getDate()}, ${weekStart.getFullYear()}`;
                } else {
                    title = `${weekStart.toLocaleDateString('en-US', { month: 'short' })} ${weekStart.getDate()} - ${weekEnd.toLocaleDateString('en-US', { month: 'short' })} ${weekEnd.getDate()}, ${weekEnd.getFullYear()}`;
                }
                break;
            case 'month':
                title = CS.currentDate.toLocaleDateString('en-US', {
                    month: 'long',
                    year: 'numeric'
                });
                break;
        }
        CS.currentPeriod.textContent = title;
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

        if (CS.currentView === 'month') {
            indicator.style.display = 'none';
        } else {
            indicator.style.display = 'block';
        }
    }

    /**
     * Navigate by direction (-1 or 1)
     */
    function navigate(direction) {
        switch (CS.currentView) {
            case 'day':
                CS.selectedDate.setDate(CS.selectedDate.getDate() + direction);
                break;
            case 'week':
                CS.selectedDate.setDate(CS.selectedDate.getDate() + (direction * 7));
                break;
            case 'month':
                CS.currentDate.setMonth(CS.currentDate.getMonth() + direction);
                break;
        }
        CS.currentDate = new Date(CS.selectedDate);
        renderMiniCalendar();
        renderCurrentView();
        loadEvents();
    }

    /**
     * Load CS.events from backend API
     */
    function loadEvents() {
        let start, end;
        if (CS.currentView === 'month') {
            const year = CS.currentDate.getFullYear();
            const month = CS.currentDate.getMonth();
            start = new Date(year, month - 1, 1).toISOString();
            end = new Date(year, month + 2, 0).toISOString();
        } else if (CS.currentView === 'week') {
            const weekStart = getWeekStart(CS.selectedDate);
            start = new Date(weekStart).toISOString();
            const weekEnd = new Date(weekStart);
            weekEnd.setDate(weekStart.getDate() + 7);
            end = weekEnd.toISOString();
        } else { // day
            const dayStart = new Date(CS.selectedDate);
            dayStart.setHours(0,0,0,0);
            start = dayStart.toISOString();
            const dayEnd = new Date(CS.selectedDate);
            dayEnd.setHours(23,59,59,999);
            end = dayEnd.toISOString();
        }

        fetch(`/api/calendar/CS.events?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`)
            .then(r => r.json())
            .then(data => {
                CS.events = data || [];
                drawEvents();
            })
            .catch(err => {
                console.error("Failed to load CS.events:", err);
            });
    }

    /**
     * Draw CS.events on grids
     */
    function drawEvents() {
        if (CS.currentView === 'month') {
            drawEventsMonth();
        } else if (CS.currentView === 'week') {
            drawEventsWeek();
        } else {
            drawEventsDay();
        }
    }

    // Expose the core functions so calendar-more.js (loaded after) can
    // extend the module instead of re-declaring them from an empty scope.
    window.CalendarModule = {
        init: init,
        navigate: navigate,
        renderCurrentView: renderCurrentView,
        loadEvents: loadEvents,
        drawEvents: drawEvents,
        showEventPopup: showEventPopup
    };
})();
