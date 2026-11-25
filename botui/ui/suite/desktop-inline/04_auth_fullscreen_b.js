/* Split from 04_auth_fullscreen.js for #2305 (450-line limit): clock rendering. */
            // Taskbar clock. The date/time format comes from Settings ->
            // Language through the shared GBDateTime helper, and a preference
            // change re-renders immediately (#1339).
            (() => {
                var timeEl = null, dateEl = null, last = "";
                function renderClock() {
                    if (document.hidden) return;
                    if (!timeEl) timeEl = document.getElementById("clock-time");
                    if (!dateEl) dateEl = document.getElementById("clock-date");
                    const now = new Date();
                    const formatted = window.GBDateTime
                        ? window.GBDateTime.now()
                        : {
                              time: gbFormatTime(now, { hour: "2-digit", minute: "2-digit" }),
                              date: gbFormatDate(now),
                          };
                    // #2933 — ticks once a second, changes once a minute. Skip
                    // the redundant write (and its style invalidation).
                    var key = formatted.time + "|" + formatted.date;
                    if (key === last) return;
                    last = key;
                    if (timeEl) timeEl.textContent = formatted.time;
                    if (dateEl) dateEl.textContent = formatted.date;
                }
                renderClock();
                setInterval(renderClock, 1000);
                window.addEventListener("storage", (e) => {
                    if (e.key === "gb-date-format" || e.key === "gb-time-format") { last = ""; renderClock(); }
                });
            })();
        
