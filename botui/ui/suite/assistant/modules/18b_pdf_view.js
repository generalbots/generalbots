"use strict";
/**
 * 18b_pdf_view.js - PDF pages drawn to canvas by the bundled pdf.js
 * (/suite/vendor/pdfjs, pdfjs-dist 6.5.438, Apache-2.0). The library, its
 * worker, character maps, standard fonts and image decoders (wasm) load from
 * the same origin on first use; the file goes in as bytes, not a blob: URL,
 * and no frame or plug-in viewer is involved.
 *
 * F.pdfThumb(box, blob, name) -> Promise: the first page into a card thumbnail.
 * F.pdfView(box, blob, name) -> Promise<{destroy}>: every page in a scrolling
 *   column with zoom (buttons, Ctrl + wheel, Ctrl +/-/0); pages draw as they
 *   come into view. The handle is also `box._pdf` from the first page on.
 */
(function () {
    var A = window.GBAssistant;
    var F = A.files = A.files || {};
    var BASE = "/suite/vendor/pdfjs/";
    var ZOOMS = [0.5, 0.67, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3];
    var MAX_PAGES = 500;
    var lib = null;

    function t(pt, en) { return A.locale && A.locale.indexOf("pt") === 0 ? pt : en; }
    function ratio() { return Math.min(window.devicePixelRatio || 1, 3); }

    function load() {
        if (!lib) {
            lib = import(BASE + "pdf.min.mjs").then(function (m) {
                m.GlobalWorkerOptions.workerSrc = BASE + "pdf.worker.min.mjs";
                return m;
            });
            lib.catch(function () { lib = null; });
        }
        return lib;
    }

    function openDoc(blob) {
        return Promise.all([load(), blob.arrayBuffer()]).then(function (r) {
            return r[0].getDocument({
                data: new Uint8Array(r[1]), enableXfa: false,
                cMapUrl: BASE + "cmaps/", cMapPacked: true,
                standardFontDataUrl: BASE + "standard_fonts/", wasmUrl: BASE + "wasm/"
            }).promise;
        });
    }

    /* Draws `page` `cssWidth` CSS pixels wide, sharp on the screen's pixels.
       Returns pdf.js's render task (`.promise`, `.cancel()`). */
    function draw(page, canvas, cssWidth) {
        var unit = page.getViewport({ scale: 1 });
        var dpr = ratio();
        var vp = page.getViewport({ scale: (cssWidth / unit.width) * dpr });
        canvas.width = Math.max(1, Math.floor(vp.width));
        canvas.height = Math.max(1, Math.floor(vp.height));
        canvas.style.width = Math.floor(vp.width / dpr) + "px";
        canvas.style.height = Math.floor(vp.height / dpr) + "px";
        return page.render({ canvas: canvas, viewport: vp, background: "#ffffff" });
    }

    F.pdfThumb = function (box, blob, name) {
        return openDoc(blob).then(function (doc) {
            return doc.getPage(1).then(function (page) {
                var canvas = document.createElement("canvas");
                canvas.className = "gb-pdf-thumb";
                canvas.setAttribute("role", "img");
                canvas.setAttribute("aria-label", t("Primeira página de ", "First page of ") + name);
                var width = box.clientWidth || 76;
                return draw(page, canvas, width).promise.then(function () {
                    box.textContent = "";
                    box.appendChild(canvas);
                    canvas.style.width = "100%";
                    canvas.style.height = "auto";
                });
            }).then(function () { doc.loadingTask.destroy(); }, function (err) { doc.loadingTask.destroy(); throw err; });
        });
    };

    var ZOOM_OUT = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M5 12h14"/></svg>';
    var ZOOM_IN = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M5 12h14M12 5v14"/></svg>';

    F.pdfView = function (box, blob, name) {
        return openDoc(blob).then(function (doc) {
            var total = Math.min(doc.numPages, MAX_PAGES);
            box.innerHTML =
                '<div class="gb-pdf-bar" role="toolbar" aria-label="' + t("Zoom do PDF", "PDF zoom") + '">' +
                    '<span class="gb-pdf-count"></span>' +
                    '<button type="button" class="gb-pdf-btn" data-zoom="out" aria-label="' + t("Diminuir", "Zoom out") + '">' + ZOOM_OUT + '</button>' +
                    '<button type="button" class="gb-pdf-btn gb-pdf-level" data-zoom="fit" title="' + t("Ajustar à largura", "Fit to width") + '"></button>' +
                    '<button type="button" class="gb-pdf-btn" data-zoom="in" aria-label="' + t("Aumentar", "Zoom in") + '">' + ZOOM_IN + '</button>' +
                '</div>' +
                '<div class="gb-pdf-pages" tabindex="0" role="document"></div>';
            var scroller = box.querySelector(".gb-pdf-pages");
            scroller.setAttribute("aria-label", name);
            var level = box.querySelector(".gb-pdf-level");
            box.querySelector(".gb-pdf-count").textContent = doc.numPages === 1 ? t("1 página", "1 page")
                : doc.numPages + " " + t("páginas", "pages");
            var zoom = 0;           /* 0: fit to width */
            var pages = [];         /* {page, canvas, holder, drawn, task} */
            var dead = false;

            function fitWidth() { return Math.max(120, scroller.clientWidth - 32); }
            function widthOf(p) { return zoom ? p.base * zoom : fitWidth(); }
            function paintLevel() {
                var shown = zoom || (pages[0] ? fitWidth() / pages[0].base : 1);
                level.textContent = Math.round(shown * 100) + "%";
                level.setAttribute("aria-label", t("Zoom ", "Zoom ") + Math.round(shown * 100) + "% · " + t("ajustar à largura", "fit to width"));
            }
            function size(p) {
                var w = widthOf(p);
                p.holder.style.width = Math.floor(w) + "px";
                p.holder.style.height = Math.floor(w * p.aspect) + "px";
            }
            function render(p) {
                if (dead || p.drawn) { return; }
                p.drawn = true;
                p.task = draw(p.page, p.canvas, widthOf(p));
                p.task.promise.catch(function () { p.drawn = false; });
            }
            var seen = new IntersectionObserver(function (entries) {
                entries.forEach(function (e) { if (e.isIntersecting) { render(pages[Number(e.target.dataset.i)]); } });
            }, { root: scroller, rootMargin: "600px 0px" });
            function rezoom(next) {
                var mid = scroller.scrollTop / Math.max(1, scroller.scrollHeight);
                zoom = next;
                pages.forEach(function (p) {
                    if (p.task) { p.task.cancel(); p.task = null; }
                    p.drawn = false;
                    size(p);
                });
                scroller.scrollTop = mid * scroller.scrollHeight;
                pages.forEach(function (p) { seen.unobserve(p.holder); seen.observe(p.holder); });
                paintLevel();
            }
            function step(dir) {
                var cur = zoom || (pages[0] ? fitWidth() / pages[0].base : 1);
                var next = dir > 0 ? ZOOMS.find(function (z) { return z > cur * 1.05; }) : ZOOMS.slice().reverse().find(function (z) { return z < cur / 1.05; });
                if (next) { rezoom(next); }
            }
            box.querySelector(".gb-pdf-bar").addEventListener("click", function (e) {
                var b = e.target.closest("[data-zoom]");
                if (!b) { return; }
                var z = b.getAttribute("data-zoom");
                if (z === "fit") { rezoom(0); } else { step(z === "in" ? 1 : -1); }
            });
            scroller.addEventListener("wheel", function (e) {
                if (!e.ctrlKey) { return; }
                e.preventDefault();
                step(e.deltaY < 0 ? 1 : -1);
            }, { passive: false });
            box.addEventListener("keydown", function (e) {
                if (!(e.ctrlKey || e.metaKey)) { return; }
                if (e.key === "+" || e.key === "=") { e.preventDefault(); step(1); }
                else if (e.key === "-") { e.preventDefault(); step(-1); }
                else if (e.key === "0") { e.preventDefault(); rezoom(0); }
            });
            var resizeTimer = null;
            var resized = new ResizeObserver(function () {
                if (zoom) { return; }
                clearTimeout(resizeTimer);
                resizeTimer = setTimeout(function () { if (!dead) { rezoom(0); } }, 120);
            });

            var handle = { destroy: function () {
                if (dead) { return; }
                dead = true;
                seen.disconnect();
                resized.disconnect();
                pages.forEach(function (p) { if (p.task) { p.task.cancel(); } });
                doc.loadingTask.destroy();
            } };
            box._pdf = handle;
            var chain = Promise.resolve();
            for (var i = 1; i <= total; i++) {
                (function (n) {
                    chain = chain.then(function () { return dead ? null : doc.getPage(n); }).then(function (page) {
                        if (!page) { return; }
                        var unit = page.getViewport({ scale: 1 });
                        var holder = document.createElement("div");
                        holder.className = "gb-pdf-page";
                        holder.dataset.i = String(pages.length);
                        holder.setAttribute("aria-label", t("Página ", "Page ") + n);
                        var canvas = document.createElement("canvas");
                        holder.appendChild(canvas);
                        var p = { page: page, canvas: canvas, holder: holder, drawn: false, base: unit.width * 96 / 72, aspect: unit.height / unit.width };
                        pages.push(p);
                        size(p);
                        scroller.appendChild(holder);
                        seen.observe(holder);
                        if (n === 1) { paintLevel(); resized.observe(scroller); }
                    });
                })(i);
            }
            return chain.then(function () { return handle; });
        });
    };
})();
