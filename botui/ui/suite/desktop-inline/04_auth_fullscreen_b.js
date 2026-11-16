/* Split from 04_auth_fullscreen.js for #2305 (450-line limit): clock rendering. */
            // Taskbar clock. The date/time format comes from Settings ->
            // Language through the shared GBDateTime helper, and a preference
            // change re-renders immediately (#1339).
            (() => {
                function renderClock() {
                    const now = new Date();
                    const timeEl = document.getElementById("clock-time");
                    const dateEl = document.getElementById("clock-date");
                    const formatted = window.GBDateTime
                        ? window.GBDateTime.now()
                        : {
                              time: gbFormatTime(now, { hour: "2-digit", minute: "2-digit" }),
                              date: gbFormatDate(now),
                          };
                    if (timeEl) timeEl.textContent = formatted.time;
                    if (dateEl) dateEl.textContent = formatted.date;
                }
                renderClock();
                setInterval(renderClock, 1000);
                window.addEventListener("storage", (e) => {
                    if (e.key === "gb-date-format" || e.key === "gb-time-format") renderClock();
                });
            })();
        
