"use strict";
/**
 * 11_icons.js - Phosphor icon system (regular weight, MIT).
 *
 * One coherent set for the whole shell. Icons are inline SVG paths stored
 * in a lookup table; no font, no CDN, no sprite file to fetch.
 *
 * Usage: GBIcon("paperclip")        -> 16px SVG string
 *        GBIcon("folder", 20)       -> 20px SVG string
 *        GBIcon("brain", 24, "cls") -> 24px with extra class
 *
 * The subset below is copied from @phosphor-icons/core 2.1.1 (MIT).
 * Full licence: assistant/icons/LICENSE-phosphor.txt
 */
(function () {
    /* Each entry is the <path d="..."/> content for a 256x256 viewBox. */
    var P = {
        /* -- navigation & chrome -- */
        "appearance": '<circle cx="128" cy="128" r="96" stroke="currentColor" stroke-width="16"/><path d="M128 32a96 96 0 0 1 0 192z" fill="currentColor"/>',
        "sidebar": '<rect x="32" y="40" width="192" height="176" rx="16" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><line x1="96" y1="40" x2="96" y2="216" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        "plus": '<line x1="40" y1="128" x2="216" y2="128" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="128" y1="40" x2="128" y2="216" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        "magnifying-glass": '<circle cx="112" cy="112" r="80" fill="none" stroke="currentColor" stroke-width="16"/><line x1="168.57" y1="168.57" x2="224" y2="224" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        "folder": '<path d="M216,72H131.31L104,44.69A15.86,15.86,0,0,0,92.69,40H40A16,16,0,0,0,24,56V200.62A15.4,15.4,0,0,0,39.38,216H216.89A15.13,15.13,0,0,0,232,200.89V88A16,16,0,0,0,216,72Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "gear": '<circle cx="128" cy="128" r="40" fill="none" stroke="currentColor" stroke-width="16"/><path d="M130.05,206.11c-1.34,0-2.69,0-4,0L117,183.89a62.08,62.08,0,0,1-20.67-12l-24.19,7.5c-2-3-3.92-6.22-5.67-9.48L85.29,150a62.28,62.28,0,0,1,0-23.92L66.47,106.11c1.75-3.26,3.65-6.43,5.67-9.48l24.19,7.5A62.08,62.08,0,0,1,117,92.11L126,69.89c1.34,0,2.69,0,4,0l9,22.22a62.08,62.08,0,0,1,20.67,12l24.19-7.5c2,3,3.92,6.22,5.67,9.48L170.71,126a62.28,62.28,0,0,1,0,23.92l18.82,19.9c-1.75,3.26-3.65,6.43-5.67,9.48l-24.19-7.5a62.08,62.08,0,0,1-20.67,12Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "sign-out": '<polyline points="174 86 216 128 174 170" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><line x1="104" y1="128" x2="216" y2="128" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><path d="M104,216H48a8,8,0,0,1-8-8V48a8,8,0,0,1,8-8h56" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "user": '<circle cx="128" cy="96" r="64" fill="none" stroke="currentColor" stroke-width="16"/><path d="M32,216c19.37-33.47,54.55-56,96-56s76.63,22.53,96,56" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "minus": '<line x1="40" y1="128" x2="216" y2="128" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        "square": '<rect x="48" y="48" width="160" height="160" rx="8" fill="none" stroke="currentColor" stroke-width="16" stroke-linejoin="round"/>',
        "x": '<line x1="200" y1="56" x2="56" y2="200" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="200" y1="200" x2="56" y2="56" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        /* -- composer & files -- */
        "paperclip": '<path d="M67.25,132.33l88.77-88.77a48,48,0,0,1,67.88,67.88L115.57,219.77a32,32,0,0,1-45.25-45.26L166.54,78.29a16,16,0,0,1,22.63,22.62L100.4,189.68" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "paper-plane-right": '<path d="M221.42,123.67,56.53,35A8,8,0,0,0,45.24,47.17L76.81,128,45.24,208.83A8,8,0,0,0,56.53,221L221.42,132.33A8,8,0,0,0,221.42,123.67Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><line x1="76.81" y1="128" x2="136" y2="128" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        "stop-circle": '<circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-width="16"/><rect x="96" y="96" width="64" height="64" rx="8" fill="none" stroke="currentColor" stroke-width="16"/>',
        "file": '<path d="M200,224H56a8,8,0,0,1-8-8V40a8,8,0,0,1,8-8h96l56,56V216A8,8,0,0,1,200,224Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><polyline points="152 32 152 88 208 88" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "image": '<rect x="32" y="48" width="192" height="160" rx="8" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><circle cx="100" cy="116" r="20" fill="none" stroke="currentColor" stroke-width="16"/><path d="M224,160.7l-46.34-46.35a8,8,0,0,0-11.32,0l-92,92" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "x-circle": '<circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-width="16"/><line x1="160" y1="96" x2="96" y2="160" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="96" y1="96" x2="160" y2="160" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        /* -- chat actions -- */
        "copy": '<polyline points="216 184 216 40 72 40" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><rect x="40" y="72" width="144" height="144" rx="8" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "arrows-clockwise": '<polyline points="176 40 224 40 224 88" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M65.77,65.77A88,88,0,0,1,192,60.71L224,40" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><polyline points="80 216 32 216 32 168" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M190.23,190.23A88,88,0,0,1,64,195.29L32,216" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "caret-down": '<polyline points="208 96 128 176 48 96" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "caret-right": '<polyline points="96 48 176 128 96 208" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "caret-left": '<polyline points="160 208 80 128 160 48" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        /* -- blocks -- */
        "brain": '<path d="M128,40V216" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><path d="M88,48a40,40,0,0,0-40,40v0a40,40,0,0,0,12.73,29.27" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M168,48a40,40,0,0,1,40,40v0a40,40,0,0,1-12.73,29.27" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M60.73,117.27A40,40,0,0,0,40,148v8a40,40,0,0,0,40,40h8" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M195.27,117.27A40,40,0,0,1,216,148v8a40,40,0,0,1-40,40h-8" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "shield-check": '<path d="M40,114.79V56a8,8,0,0,1,8-8H208a8,8,0,0,1,8,8v58.77c0,84.18-71.31,112.07-84.56,116.44a7.54,7.54,0,0,1-4.88,0C113.31,226.86,40,199,40,114.79Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><polyline points="88 136 112 160 168 104" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "warning-circle": '<circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-width="16"/><line x1="128" y1="80" x2="128" y2="136" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><circle cx="128" cy="172" r="12" fill="currentColor"/>',
        "info": '<circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-width="16"/><polyline points="112 176 128 176 128 120 112 120" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><circle cx="124" cy="84" r="12" fill="currentColor"/>',
        "check-circle": '<circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-width="16"/><polyline points="88 136 112 160 168 104" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "spinner": '<line x1="128" y1="32" x2="128" y2="64" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="195.88" y1="60.12" x2="173.25" y2="82.75" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="224" y1="128" x2="192" y2="128" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="195.88" y1="195.88" x2="173.25" y2="173.25" stroke="currentColor" stroke-width="16" stroke-linecap="round" opacity=".5"/><line x1="128" y1="224" x2="128" y2="192" stroke="currentColor" stroke-width="16" stroke-linecap="round" opacity=".4"/><line x1="60.12" y1="195.88" x2="82.75" y2="173.25" stroke="currentColor" stroke-width="16" stroke-linecap="round" opacity=".3"/><line x1="32" y1="128" x2="64" y2="128" stroke="currentColor" stroke-width="16" stroke-linecap="round" opacity=".2"/><line x1="60.12" y1="60.12" x2="82.75" y2="82.75" stroke="currentColor" stroke-width="16" stroke-linecap="round" opacity=".1"/>',
        "clock": '<circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-width="16"/><polyline points="128 72 128 128 184 128" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "terminal": '<polyline points="80 96 120 128 80 160" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><line x1="136" y1="160" x2="176" y2="160" stroke="currentColor" stroke-width="16" stroke-linecap="round"/>',
        "folder-open": '<path d="M228.24,144H69.76a8,8,0,0,1-7.64-5.63L32,40H200a8,8,0,0,1,7.64,5.63l28.6,92.74A8,8,0,0,1,228.24,144Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M32,40V208a8,8,0,0,0,8,8H208" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "shield": '<path d="M40,114.79V56a8,8,0,0,1,8-8H208a8,8,0,0,1,8,8v58.77c0,84.18-71.31,112.07-84.56,116.44a7.54,7.54,0,0,1-4.88,0C113.31,226.86,40,199,40,114.79Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "warning": '<path d="M114.15,39.87,26.17,181.6A16,16,0,0,0,40,204H216a16,16,0,0,0,13.85-22.4L141.85,39.87A16,16,0,0,0,114.15,39.87Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><line x1="128" y1="112" x2="128" y2="144" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><circle cx="128" cy="176" r="12" fill="currentColor"/>',
        "eye": '<path d="M128,56C48,56,16,128,16,128s32,72,112,72,112-72,112-72S208,56,128,56Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><circle cx="128" cy="128" r="40" fill="none" stroke="currentColor" stroke-width="16"/>',
        "download": '<line x1="128" y1="24" x2="128" y2="168" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><polyline points="80 120 128 168 176 120" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M224,176v32a8,8,0,0,1-8,8H40a8,8,0,0,1-8-8V176" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "trash": '<line x1="216" y1="56" x2="40" y2="56" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="104" y1="104" x2="104" y2="168" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><line x1="152" y1="104" x2="152" y2="168" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><path d="M200,56V208a8,8,0,0,1-8,8H64a8,8,0,0,1-8-8V56" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M168,56V40a16,16,0,0,0-16-16H104A16,16,0,0,0,88,40V56" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "arrow-square-out": '<polyline points="184 104 184 40 120 40" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><line x1="120" y1="136" x2="184" y2="40" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><path d="M160,136v72a8,8,0,0,1-8,8H48a8,8,0,0,1-8-8V104A8,8,0,0,1,48,96h72" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "chat-circle": '<path d="M128,24A104,104,0,0,0,36.18,176.88L24.83,210.93a8,8,0,0,0,10.24,10.24l34.05-11.35A104,104,0,1,0,128,24Z" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
        "pencil": '<line x1="96" y1="216" x2="216" y2="216" stroke="currentColor" stroke-width="16" stroke-linecap="round"/><path d="M188,56l12,12" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><path d="M68,188,192.3,63.7a16,16,0,0,1,22.6,0h0a16,16,0,0,1,0,22.6L90.58,210.62A16,16,0,0,1,79.26,216H48V184.74A16,16,0,0,1,52.69,173.4" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>',
    };

    /**
     * Render a Phosphor icon as an inline SVG string.
     * @param {string} name - Icon name from the table above
     * @param {number} [size=16] - px dimension
     * @param {string} [cls=""] - extra CSS class
     * @returns {string} SVG markup
     */
    window.GBIcon = function (name, size, cls) {
        size = size || 16;
        cls = cls ? "gb-icon " + cls : "gb-icon";
        var paths = P[name];
        if (!paths) { return '<span class="' + cls + '" style="width:' + size + 'px;height:' + size + 'px"></span>'; }
        return '<svg class="' + cls + '" width="' + size + '" height="' + size +
            '" viewBox="0 0 256 256" fill="none" aria-hidden="true">' + paths + '</svg>';
    };
})();
