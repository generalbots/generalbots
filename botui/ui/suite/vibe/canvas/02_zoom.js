/* Vibe Canvas — split from vibe/vibe-canvas.js for #1731 */

var vibeZoomLevel = 100;

// Top-level twin of the module's visEl: prefer the canvas in the standalone
// app window, else the visible inline surface (both carry the same ids).
function vibeVis(id) {
    var all = document.querySelectorAll("#" + id);
    if (all.length <= 1) return all[0] || null;
    var appWin = document.getElementById("window-vibe-canvas");
    if (appWin) {
        var inApp = appWin.querySelector("#" + id);
        if (inApp && inApp.offsetParent !== null) return inApp;
    }
    for (var i = 0; i < all.length; i++) { if (all[i].offsetParent !== null) return all[i]; }
    return all[all.length - 1];
}

function vibeSetZoom(delta) {
    // Same visible-copy resolution as the other surfaces: getElementById
    // returns the first match, which is the parked hidden copy inside the
    // Vibe window, not the visible canvas — zooming it was a no-op.
    var steps = vibeVis("vibeSteps");
    var design = vibeVis("vibeDesignSurface");
    var generated = vibeVis("vibeGeneratedLayer");
    vibeZoomLevel = Math.min(200, Math.max(40, vibeZoomLevel + (delta * 10)));
    var scale = vibeZoomLevel / 100;
    if (steps) {
        steps.style.transformOrigin = "0 0";
        steps.style.transform = "scale(" + scale + ")";
    }
    // Scale the drawable surface AND the generated architecture behind it
    // from the top-left corner so +/- zoom visibly works (was a no-op).
    if (design) {
        design.style.transformOrigin = "0 0";
        design.style.transform = "scale(" + scale + ")";
    }
    if (generated) {
        generated.style.transformOrigin = "0 0";
        generated.style.transform = "scale(" + scale + ")";
    }
    var labels = document.querySelectorAll("[data-vibe-zoom-label]");
    labels.forEach(function (label) {
        label.textContent = vibeZoomLevel + "%";
    });
}

document.addEventListener("click", function (e) {
    var btn = e.target.closest("[data-vibe-zoom]");
    if (!btn) return;
    vibeSetZoom(parseInt(btn.getAttribute("data-vibe-zoom"), 10) || 0);
});
