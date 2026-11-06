    function switchReportTab(tab) {
        document.querySelectorAll(".report-tab").forEach(function (btn) {
            btn.classList.toggle(
                "active",
                btn.getAttribute("data-tab") === tab,
            );
        });
        document.querySelectorAll(".report-panel").forEach(function (panel) {
            panel.classList.toggle("active", panel.id === tab + "-reports");
        });
    }
