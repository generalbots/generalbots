/* Calendar Module JavaScript */

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
            html += `<button class="mini-day other-month" data-date="${year}-${month - 1}-${prevMonthLastDay - i}">${prevMonthLastDay - i}</button>`;
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
     * @param {Date} date - Date to get week start for
     * @returns {Date} - Start of week
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

        // Only show in day/week view
        if (currentView === 'month') {
            indicator.style.display = 'none';
        } else {
            indicator.style.display = 'block';
        }
    }

    /**
     * Navigate by direction (-1 or 1)
     * @param {number} direction - Direction to navigate
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

                        // Pre-fill date in form
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
                // Form is handled by HTMX, but we can add validation here
                eventModal.classList.add('hidden');
            });
        }

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                if (eventModal) eventModal.classList.add('hidden');
                if (eventPopup) eventPopup.classList.add('hidden');
            }

            // Only handle if not in input
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
        setView: function(view) {
            currentView = view;
            renderCurrentView();
        }
    };

    // Auto-initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
