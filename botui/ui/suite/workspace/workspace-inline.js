    function toggleMembersPanel() {
        const panel = document.getElementById("members-panel");
        if (panel) {
            panel.classList.toggle("collapsed");
        }
    }

    document.addEventListener("DOMContentLoaded", function () {
        const blocksContainer = document.getElementById("blocks-container");
        if (blocksContainer) {
            blocksContainer.addEventListener("keydown", function (e) {
                if (e.key === "/") {
                    const slashMenu = document.getElementById("slash-menu");
                    if (slashMenu) {
                        slashMenu.classList.remove("hidden");
                    }
                }
                if (e.key === "Escape") {
                    const slashMenu = document.getElementById("slash-menu");
                    if (slashMenu) {
                        slashMenu.classList.add("hidden");
                    }
                }
            });
        }
    });
