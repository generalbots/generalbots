/* Split from 04_auth_fullscreen.js for #2305 (450-line limit): clock rendering. */
            // Taskbar clock. The date/time format comes from Settings ->
            // Language through the shared GBDateTime helper, and a preference
            // change re-renders immediately (#1339).
            (() => {
                var timeEl = null, dateEl = null, last = "";
                // #2933 — share the shell's idle scheduler with the other boot
                // chrome (widgets, status strip, header). Fall back safely when
                // this fragment is loaded without the desktop shell.
                var deferBoot = window.gbDeferIdle || function (fn) {
                    if (typeof window.requestIdleCallback === "function") return window.requestIdleCallback(fn, { timeout: 800 });
                    return window.setTimeout(fn, 1);
                };
                function paintClock(formatted) {
                    if (timeEl) timeEl.textContent = formatted.time;
                    if (dateEl) dateEl.textContent = formatted.date;
                }
                function renderClock() {
                    if (document.hidden) return;
                    // Read phase: resolve nodes, time and the change key before
                    // any write; the paint itself runs on the shared idle pass.
                    if (!timeEl) timeEl = document.getElementById("clock-time");
                    if (!dateEl) dateEl = document.getElementById("clock-date");
                    const now = new Date();
                    const formatted = window.GBDateTime
                        ? window.GBDateTime.now()
                        : {
                              time: gbFormatTime(now, { hour: "2-digit", minute: "2-digit" }),
                              date: gbFormatDate(now),
                          };
                    var key = formatted.time + "|" + formatted.date;
                    if (key === last) return;
                    last = key;
                    deferBoot(function () { paintClock(formatted); });
                }
                renderClock();
                setInterval(renderClock, 1000);
                window.addEventListener("storage", (e) => {
                    if (e.key === "gb-date-format" || e.key === "gb-time-format") { last = ""; renderClock(); }
                });
            })();
