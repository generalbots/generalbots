// === util/int.js ===
(function() { // module util_int
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2020 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

function toUnsigned32bit(toConvert) {
    return toConvert >>> 0;
}

function toSigned32bit(toConvert) {
    return toConvert | 0;
}

}})();

// === util/logging.js ===
(function() { // module util_logging
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

/*
 * Logging/debug routines
 */

let _logLevel = 'warn';

let Debug = () => {};
let Info = () => {};
let Warn = () => {};
let Error = () => {};

function initLogging(level) {
    if (typeof level === 'undefined') {
        level = _logLevel;
    } else {
        _logLevel = level;
    }

    Debug = Info = Warn = Error = () => {};

    if (typeof window.console !== "undefined") {
        /* eslint-disable no-console, no-fallthrough */
        switch (level) {
            case 'debug':
                Debug = console.debug.bind(window.console);
            case 'info':
                Info  = console.info.bind(window.console);
            case 'warn':
                Warn  = console.warn.bind(window.console);
            case 'error':
                Error = console.error.bind(window.console);
            case 'none':
                break;
            default:
                throw new window.Error("invalid logging type '" + level + "'");
        }
        /* eslint-enable no-console, no-fallthrough */
    }
}

function getLogging() {
    return _logLevel;
}


// Initialize logging level
initLogging();

}})();

// === util/strings.js ===
(function() { // module util_strings
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

// Decode from UTF-8
function decodeUTF8(utf8string, allowLatin1=false) {
    try {
        return decodeURIComponent(escape(utf8string));
    } catch (e) {
        if (e instanceof URIError) {
            if (allowLatin1) {
                // If we allow Latin1 we can ignore any decoding fails
                // and in these cases return the original string
                return utf8string;
            }
        }
        throw e;
    }
}

// Encode to UTF-8
function encodeUTF8(DOMString) {
    return unescape(encodeURIComponent(DOMString));
}

}})();

// === util/browser.js ===
(function() { // module util_browser
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 * Browser feature support detection
 */


// Touch detection
let isTouchDevice = ('ontouchstart' in document.documentElement) ||
                                 // requried for Chrome debugger
                                 (document.ontouchstart !== undefined) ||
                                 // required for MS Surface
                                 (navigator.maxTouchPoints > 0) ||
                                 (navigator.msMaxTouchPoints > 0);
window.addEventListener('touchstart', function onFirstTouch() {
    isTouchDevice = true;
    window.removeEventListener('touchstart', onFirstTouch, false);
}, false);


// The goal is to find a certain physical width, the devicePixelRatio
// brings us a bit closer but is not optimal.
let dragThreshold = 10 * (window.devicePixelRatio || 1);

let _supportsCursorURIs = false;

try {
    const target = document.createElement('canvas');
    target.style.cursor = 'url("data:image/x-icon;base64,AAACAAEACAgAAAIAAgA4AQAAFgAAACgAAAAIAAAAEAAAAAEAIAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAD/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////AAAAAAAAAAAAAAAAAAAAAA==") 2 2, default';

    if (target.style.cursor.indexOf("url") === 0) {
        Log.Info("Data URI scheme cursor supported");
        _supportsCursorURIs = true;
    } else {
        Log.Warn("Data URI scheme cursor not supported");
    }
} catch (exc) {
    Log.Error("Data URI scheme cursor test exception: " + exc);
}

const supportsCursorURIs = _supportsCursorURIs;

let _hasScrollbarGutter = true;
try {
    // Create invisible container
    const container = document.createElement('div');
    container.style.visibility = 'hidden';
    container.style.overflow = 'scroll'; // forcing scrollbars
    document.body.appendChild(container);

    // Create a div and place it in the container
    const child = document.createElement('div');
    container.appendChild(child);

    // Calculate the difference between the container's full width
    // and the child's width - the difference is the scrollbars
    const scrollbarWidth = (container.offsetWidth - child.offsetWidth);

    // Clean up
    container.parentNode.removeChild(container);

    _hasScrollbarGutter = scrollbarWidth != 0;
} catch (exc) {
    Log.Error("Scrollbar test exception: " + exc);
}
const hasScrollbarGutter = _hasScrollbarGutter;

/*
 * The functions for detection of platforms and browsers below are exported
 * but the use of these should be minimized as much as possible.
 *
 * It's better to use feature detection than platform detection.
 */

/* OS */

function isMac() {
    return !!(/mac/i).exec(navigator.platform);
}

function isWindows() {
    return !!(/win/i).exec(navigator.platform);
}

function isIOS() {
    return (!!(/ipad/i).exec(navigator.platform) ||
            !!(/iphone/i).exec(navigator.platform) ||
            !!(/ipod/i).exec(navigator.platform));
}

function isAndroid() {
    /* Android sets navigator.platform to Linux :/ */
    return !!navigator.userAgent.match('Android ');
}

function isChromeOS() {
    /* ChromeOS sets navigator.platform to Linux :/ */
    return !!navigator.userAgent.match(' CrOS ');
}

/* Browser */

function isSafari() {
    return !!navigator.userAgent.match('Safari/...') &&
           !navigator.userAgent.match('Chrome/...') &&
           !navigator.userAgent.match('Chromium/...') &&
           !navigator.userAgent.match('Epiphany/...');
}

function isFirefox() {
    return !!navigator.userAgent.match('Firefox/...') &&
           !navigator.userAgent.match('Seamonkey/...');
}

function isChrome() {
    return !!navigator.userAgent.match('Chrome/...') &&
           !navigator.userAgent.match('Chromium/...') &&
           !navigator.userAgent.match('Edg/...') &&
           !navigator.userAgent.match('OPR/...');
}

function isChromium() {
    return !!navigator.userAgent.match('Chromium/...');
}

function isOpera() {
    return !!navigator.userAgent.match('OPR/...');
}

function isEdge() {
    return !!navigator.userAgent.match('Edg/...');
}

/* Engine */

function isGecko() {
    return !!navigator.userAgent.match('Gecko/...');
}

function isWebKit() {
    return !!navigator.userAgent.match('AppleWebKit/...') &&
           !navigator.userAgent.match('Chrome/...');
}

function isBlink() {
    return !!navigator.userAgent.match('Chrome/...');
}

}})();

// === util/element.js ===
(function() { // module util_element
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2020 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

/*
 * HTML element utility functions
 */

function clientToElement(x, y, elem) {
    const bounds = elem.getBoundingClientRect();
    let pos = { x: 0, y: 0 };
    // Clip to target bounds
    if (x < bounds.left) {
        pos.x = 0;
    } else if (x >= bounds.right) {
        pos.x = bounds.width - 1;
    } else {
        pos.x = x - bounds.left;
    }
    if (y < bounds.top) {
        pos.y = 0;
    } else if (y >= bounds.bottom) {
        pos.y = bounds.height - 1;
    } else {
        pos.y = y - bounds.top;
    }
    return pos;
}

}})();

// === util/events.js ===
(function() { // module util_events
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2018 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

/*
 * Cross-browser event and position routines
 */

function getPointerEvent(e) {
    return e.changedTouches ? e.changedTouches[0] : e.touches ? e.touches[0] : e;
}

function stopEvent(e) {
    e.stopPropagation();
    e.preventDefault();
}

// Emulate Element.setCapture() when not supported
let _captureRecursion = false;
let _elementForUnflushedEvents = null;
document.captureElement = null;
function _captureProxy(e) {
    // Recursion protection as we'll see our own event
    if (_captureRecursion) return;

    // Clone the event as we cannot dispatch an already dispatched event
    const newEv = new e.constructor(e.type, e);

    _captureRecursion = true;
    if (document.captureElement) {
        document.captureElement.dispatchEvent(newEv);
    } else {
        _elementForUnflushedEvents.dispatchEvent(newEv);
    }
    _captureRecursion = false;

    // Avoid double events
    e.stopPropagation();

    // Respect the wishes of the redirected event handlers
    if (newEv.defaultPrevented) {
        e.preventDefault();
    }

    // Implicitly release the capture on button release
    if (e.type === "mouseup") {
        releaseCapture();
    }
}

// Follow cursor style of target element
function _capturedElemChanged() {
    const proxyElem = document.getElementById("noVNC_mouse_capture_elem");
    proxyElem.style.cursor = window.getComputedStyle(document.captureElement).cursor;
}

const _captureObserver = new MutationObserver(_capturedElemChanged);

function setCapture(target) {
    if (target.setCapture) {

        target.setCapture();
        document.captureElement = target;
    } else {
        // Release any existing capture in case this method is
        // called multiple times without coordination
        releaseCapture();

        let proxyElem = document.getElementById("noVNC_mouse_capture_elem");

        if (proxyElem === null) {
            proxyElem = document.createElement("div");
            proxyElem.id = "noVNC_mouse_capture_elem";
            proxyElem.style.position = "fixed";
            proxyElem.style.top = "0px";
            proxyElem.style.left = "0px";
            proxyElem.style.width = "100%";
            proxyElem.style.height = "100%";
            proxyElem.style.zIndex = 10000;
            proxyElem.style.display = "none";
            document.body.appendChild(proxyElem);

            // This is to make sure callers don't get confused by having
            // our blocking element as the target
            proxyElem.addEventListener('contextmenu', _captureProxy);

            proxyElem.addEventListener('mousemove', _captureProxy);
            proxyElem.addEventListener('mouseup', _captureProxy);
        }

        document.captureElement = target;

        // Track cursor and get initial cursor
        _captureObserver.observe(target, {attributes: true});
        _capturedElemChanged();

        proxyElem.style.display = "";

        // We listen to events on window in order to keep tracking if it
        // happens to leave the viewport
        window.addEventListener('mousemove', _captureProxy);
        window.addEventListener('mouseup', _captureProxy);
    }
}

function releaseCapture() {
    if (document.releaseCapture) {

        document.releaseCapture();
        document.captureElement = null;

    } else {
        if (!document.captureElement) {
            return;
        }

        // There might be events already queued. The event proxy needs
        // access to the captured element for these queued events.
        // E.g. contextmenu (right-click) in Microsoft Edge
        //
        // Before removing the capturedElem pointer we save it to a
        // temporary variable that the unflushed events can use.
        _elementForUnflushedEvents = document.captureElement;
        document.captureElement = null;

        _captureObserver.disconnect();

        const proxyElem = document.getElementById("noVNC_mouse_capture_elem");
        proxyElem.style.display = "none";

        window.removeEventListener('mousemove', _captureProxy);
        window.removeEventListener('mouseup', _captureProxy);
    }
}

}})();

// === util/eventtarget.js ===
(function() { // module util_eventtarget
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

class EventTargetMixin {
    constructor() {
        this._listeners = new Map();
    }

    addEventListener(type, callback) {
        if (!this._listeners.has(type)) {
            this._listeners.set(type, new Set());
        }
        this._listeners.get(type).add(callback);
    }

    removeEventListener(type, callback) {
        if (this._listeners.has(type)) {
            this._listeners.get(type).delete(callback);
        }
    }

    dispatchEvent(event) {
        if (!this._listeners.has(event.type)) {
            return true;
        }
        this._listeners.get(event.type)
            .forEach(callback => callback.call(this, event));
        return !event.defaultPrevented;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === util/cursor.js ===
(function() { // module util_cursor
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 or any later version (see LICENSE.txt)
 */


const useFallback = !supportsCursorURIs || isTouchDevice;

class Cursor {
    constructor() {
        this._target = null;

        this._canvas = document.createElement('canvas');

        if (useFallback) {
            this._canvas.style.position = 'fixed';
            this._canvas.style.zIndex = '65535';
            this._canvas.style.pointerEvents = 'none';
            // Safari on iOS can select the cursor image
            // https://bugs.webkit.org/show_bug.cgi?id=249223
            this._canvas.style.userSelect = 'none';
            this._canvas.style.WebkitUserSelect = 'none';
            // Can't use "display" because of Firefox bug #1445997
            this._canvas.style.visibility = 'hidden';
        }

        this._position = { x: 0, y: 0 };
        this._hotSpot = { x: 0, y: 0 };

        this._eventHandlers = {
            'mouseover': this._handleMouseOver.bind(this),
            'mouseleave': this._handleMouseLeave.bind(this),
            'mousemove': this._handleMouseMove.bind(this),
            'mouseup': this._handleMouseUp.bind(this),
        };
    }

    attach(target) {
        if (this._target) {
            this.detach();
        }

        this._target = target;

        if (useFallback) {
            document.body.appendChild(this._canvas);

            const options = { capture: true, passive: true };
            this._target.addEventListener('mouseover', this._eventHandlers.mouseover, options);
            this._target.addEventListener('mouseleave', this._eventHandlers.mouseleave, options);
            this._target.addEventListener('mousemove', this._eventHandlers.mousemove, options);
            this._target.addEventListener('mouseup', this._eventHandlers.mouseup, options);
        }

        this.clear();
    }

    detach() {
        if (!this._target) {
            return;
        }

        if (useFallback) {
            const options = { capture: true, passive: true };
            this._target.removeEventListener('mouseover', this._eventHandlers.mouseover, options);
            this._target.removeEventListener('mouseleave', this._eventHandlers.mouseleave, options);
            this._target.removeEventListener('mousemove', this._eventHandlers.mousemove, options);
            this._target.removeEventListener('mouseup', this._eventHandlers.mouseup, options);

            if (document.contains(this._canvas)) {
                document.body.removeChild(this._canvas);
            }
        }

        this._target = null;
    }

    change(rgba, hotx, hoty, w, h) {
        if ((w === 0) || (h === 0)) {
            this.clear();
            return;
        }

        this._position.x = this._position.x + this._hotSpot.x - hotx;
        this._position.y = this._position.y + this._hotSpot.y - hoty;
        this._hotSpot.x = hotx;
        this._hotSpot.y = hoty;

        let ctx = this._canvas.getContext('2d');

        this._canvas.width = w;
        this._canvas.height = h;

        let img = new ImageData(new Uint8ClampedArray(rgba), w, h);
        ctx.clearRect(0, 0, w, h);
        ctx.putImageData(img, 0, 0);

        if (useFallback) {
            this._updatePosition();
        } else {
            let url = this._canvas.toDataURL();
            this._target.style.cursor = 'url(' + url + ')' + hotx + ' ' + hoty + ', default';
        }
    }

    clear() {
        this._target.style.cursor = 'none';
        this._canvas.width = 0;
        this._canvas.height = 0;
        this._position.x = this._position.x + this._hotSpot.x;
        this._position.y = this._position.y + this._hotSpot.y;
        this._hotSpot.x = 0;
        this._hotSpot.y = 0;
    }

    // Mouse events might be emulated, this allows
    // moving the cursor in such cases
    move(clientX, clientY) {
        if (!useFallback) {
            return;
        }
        // clientX/clientY are relative the _visual viewport_,
        // but our position is relative the _layout viewport_,
        // so try to compensate when we can
        if (window.visualViewport) {
            this._position.x = clientX + window.visualViewport.offsetLeft;
            this._position.y = clientY + window.visualViewport.offsetTop;
        } else {
            this._position.x = clientX;
            this._position.y = clientY;
        }
        this._updatePosition();
        let target = document.elementFromPoint(clientX, clientY);
        this._updateVisibility(target);
    }

    _handleMouseOver(event) {
        // This event could be because we're entering the target, or
        // moving around amongst its sub elements. Let the move handler
        // sort things out.
        this._handleMouseMove(event);
    }

    _handleMouseLeave(event) {
        // Check if we should show the cursor on the element we are leaving to
        this._updateVisibility(event.relatedTarget);
    }

    _handleMouseMove(event) {
        this._updateVisibility(event.target);

        this._position.x = event.clientX - this._hotSpot.x;
        this._position.y = event.clientY - this._hotSpot.y;

        this._updatePosition();
    }

    _handleMouseUp(event) {
        // We might get this event because of a drag operation that
        // moved outside of the target. Check what's under the cursor
        // now and adjust visibility based on that.
        let target = document.elementFromPoint(event.clientX, event.clientY);
        this._updateVisibility(target);

        // Captures end with a mouseup but we can't know the event order of
        // mouseup vs releaseCapture.
        //
        // In the cases when releaseCapture comes first, the code above is
        // enough.
        //
        // In the cases when the mouseup comes first, we need wait for the
        // browser to flush all events and then check again if the cursor
        // should be visible.
        if (this._captureIsActive()) {
            window.setTimeout(() => {
                // We might have detached at this point
                if (!this._target) {
                    return;
                }
                // Refresh the target from elementFromPoint since queued events
                // might have altered the DOM
                target = document.elementFromPoint(event.clientX,
                                                   event.clientY);
                this._updateVisibility(target);
            }, 0);
        }
    }

    _showCursor() {
        if (this._canvas.style.visibility === 'hidden') {
            this._canvas.style.visibility = '';
        }
    }

    _hideCursor() {
        if (this._canvas.style.visibility !== 'hidden') {
            this._canvas.style.visibility = 'hidden';
        }
    }

    // Should we currently display the cursor?
    // (i.e. are we over the target, or a child of the target without a
    // different cursor set)
    _shouldShowCursor(target) {
        if (!target) {
            return false;
        }
        // Easy case
        if (target === this._target) {
            return true;
        }
        // Other part of the DOM?
        if (!this._target.contains(target)) {
            return false;
        }
        // Has the child its own cursor?
        // FIXME: How can we tell that a sub element has an
        //        explicit "cursor: none;"?
        if (window.getComputedStyle(target).cursor !== 'none') {
            return false;
        }
        return true;
    }

    _updateVisibility(target) {
        // When the cursor target has capture we want to show the cursor.
        // So, if a capture is active - look at the captured element instead.
        if (this._captureIsActive()) {
            target = document.captureElement;
        }
        if (this._shouldShowCursor(target)) {
            this._showCursor();
        } else {
            this._hideCursor();
        }
    }

    _updatePosition() {
        this._canvas.style.left = this._position.x + "px";
        this._canvas.style.top = this._position.y + "px";
    }

    _captureIsActive() {
        return document.captureElement &&
            document.documentElement.contains(document.captureElement);
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === input/keysym.js ===
(function() { // module input_keysym
/* eslint-disable key-spacing */

{
    XK_VoidSymbol:                  0xffffff, /* Void symbol */

    XK_BackSpace:                   0xff08, /* Back space, back char */
    XK_Tab:                         0xff09,
    XK_Linefeed:                    0xff0a, /* Linefeed, LF */
    XK_Clear:                       0xff0b,
    XK_Return:                      0xff0d, /* Return, enter */
    XK_Pause:                       0xff13, /* Pause, hold */
    XK_Scroll_Lock:                 0xff14,
    XK_Sys_Req:                     0xff15,
    XK_Escape:                      0xff1b,
    XK_Delete:                      0xffff, /* Delete, rubout */

    /* International & multi-key character composition */

    XK_Multi_key:                   0xff20, /* Multi-key character compose */
    XK_Codeinput:                   0xff37,
    XK_SingleCandidate:             0xff3c,
    XK_MultipleCandidate:           0xff3d,
    XK_PreviousCandidate:           0xff3e,

    /* Japanese keyboard support */

    XK_Kanji:                       0xff21, /* Kanji, Kanji convert */
    XK_Muhenkan:                    0xff22, /* Cancel Conversion */
    XK_Henkan_Mode:                 0xff23, /* Start/Stop Conversion */
    XK_Henkan:                      0xff23, /* Alias for Henkan_Mode */
    XK_Romaji:                      0xff24, /* to Romaji */
    XK_Hiragana:                    0xff25, /* to Hiragana */
    XK_Katakana:                    0xff26, /* to Katakana */
    XK_Hiragana_Katakana:           0xff27, /* Hiragana/Katakana toggle */
    XK_Zenkaku:                     0xff28, /* to Zenkaku */
    XK_Hankaku:                     0xff29, /* to Hankaku */
    XK_Zenkaku_Hankaku:             0xff2a, /* Zenkaku/Hankaku toggle */
    XK_Touroku:                     0xff2b, /* Add to Dictionary */
    XK_Massyo:                      0xff2c, /* Delete from Dictionary */
    XK_Kana_Lock:                   0xff2d, /* Kana Lock */
    XK_Kana_Shift:                  0xff2e, /* Kana Shift */
    XK_Eisu_Shift:                  0xff2f, /* Alphanumeric Shift */
    XK_Eisu_toggle:                 0xff30, /* Alphanumeric toggle */
    XK_Kanji_Bangou:                0xff37, /* Codeinput */
    XK_Zen_Koho:                    0xff3d, /* Multiple/All Candidate(s) */
    XK_Mae_Koho:                    0xff3e, /* Previous Candidate */

    /* Cursor control & motion */

    XK_Home:                        0xff50,
    XK_Left:                        0xff51, /* Move left, left arrow */
    XK_Up:                          0xff52, /* Move up, up arrow */
    XK_Right:                       0xff53, /* Move right, right arrow */
    XK_Down:                        0xff54, /* Move down, down arrow */
    XK_Prior:                       0xff55, /* Prior, previous */
    XK_Page_Up:                     0xff55,
    XK_Next:                        0xff56, /* Next */
    XK_Page_Down:                   0xff56,
    XK_End:                         0xff57, /* EOL */
    XK_Begin:                       0xff58, /* BOL */


    /* Misc functions */

    XK_Select:                      0xff60, /* Select, mark */
    XK_Print:                       0xff61,
    XK_Execute:                     0xff62, /* Execute, run, do */
    XK_Insert:                      0xff63, /* Insert, insert here */
    XK_Undo:                        0xff65,
    XK_Redo:                        0xff66, /* Redo, again */
    XK_Menu:                        0xff67,
    XK_Find:                        0xff68, /* Find, search */
    XK_Cancel:                      0xff69, /* Cancel, stop, abort, exit */
    XK_Help:                        0xff6a, /* Help */
    XK_Break:                       0xff6b,
    XK_Mode_switch:                 0xff7e, /* Character set switch */
    XK_script_switch:               0xff7e, /* Alias for mode_switch */
    XK_Num_Lock:                    0xff7f,

    /* Keypad functions, keypad numbers cleverly chosen to map to ASCII */

    XK_KP_Space:                    0xff80, /* Space */
    XK_KP_Tab:                      0xff89,
    XK_KP_Enter:                    0xff8d, /* Enter */
    XK_KP_F1:                       0xff91, /* PF1, KP_A, ... */
    XK_KP_F2:                       0xff92,
    XK_KP_F3:                       0xff93,
    XK_KP_F4:                       0xff94,
    XK_KP_Home:                     0xff95,
    XK_KP_Left:                     0xff96,
    XK_KP_Up:                       0xff97,
    XK_KP_Right:                    0xff98,
    XK_KP_Down:                     0xff99,
    XK_KP_Prior:                    0xff9a,
    XK_KP_Page_Up:                  0xff9a,
    XK_KP_Next:                     0xff9b,
    XK_KP_Page_Down:                0xff9b,
    XK_KP_End:                      0xff9c,
    XK_KP_Begin:                    0xff9d,
    XK_KP_Insert:                   0xff9e,
    XK_KP_Delete:                   0xff9f,
    XK_KP_Equal:                    0xffbd, /* Equals */
    XK_KP_Multiply:                 0xffaa,
    XK_KP_Add:                      0xffab,
    XK_KP_Separator:                0xffac, /* Separator, often comma */
    XK_KP_Subtract:                 0xffad,
    XK_KP_Decimal:                  0xffae,
    XK_KP_Divide:                   0xffaf,

    XK_KP_0:                        0xffb0,
    XK_KP_1:                        0xffb1,
    XK_KP_2:                        0xffb2,
    XK_KP_3:                        0xffb3,
    XK_KP_4:                        0xffb4,
    XK_KP_5:                        0xffb5,
    XK_KP_6:                        0xffb6,
    XK_KP_7:                        0xffb7,
    XK_KP_8:                        0xffb8,
    XK_KP_9:                        0xffb9,

    /*
     * Auxiliary functions; note the duplicate definitions for left and right
     * function keys;  Sun keyboards and a few other manufacturers have such
     * function key groups on the left and/or right sides of the keyboard.
     * We've not found a keyboard with more than 35 function keys total.
     */

    XK_F1:                          0xffbe,
    XK_F2:                          0xffbf,
    XK_F3:                          0xffc0,
    XK_F4:                          0xffc1,
    XK_F5:                          0xffc2,
    XK_F6:                          0xffc3,
    XK_F7:                          0xffc4,
    XK_F8:                          0xffc5,
    XK_F9:                          0xffc6,
    XK_F10:                         0xffc7,
    XK_F11:                         0xffc8,
    XK_L1:                          0xffc8,
    XK_F12:                         0xffc9,
    XK_L2:                          0xffc9,
    XK_F13:                         0xffca,
    XK_L3:                          0xffca,
    XK_F14:                         0xffcb,
    XK_L4:                          0xffcb,
    XK_F15:                         0xffcc,
    XK_L5:                          0xffcc,
    XK_F16:                         0xffcd,
    XK_L6:                          0xffcd,
    XK_F17:                         0xffce,
    XK_L7:                          0xffce,
    XK_F18:                         0xffcf,
    XK_L8:                          0xffcf,
    XK_F19:                         0xffd0,
    XK_L9:                          0xffd0,
    XK_F20:                         0xffd1,
    XK_L10:                         0xffd1,
    XK_F21:                         0xffd2,
    XK_R1:                          0xffd2,
    XK_F22:                         0xffd3,
    XK_R2:                          0xffd3,
    XK_F23:                         0xffd4,
    XK_R3:                          0xffd4,
    XK_F24:                         0xffd5,
    XK_R4:                          0xffd5,
    XK_F25:                         0xffd6,
    XK_R5:                          0xffd6,
    XK_F26:                         0xffd7,
    XK_R6:                          0xffd7,
    XK_F27:                         0xffd8,
    XK_R7:                          0xffd8,
    XK_F28:                         0xffd9,
    XK_R8:                          0xffd9,
    XK_F29:                         0xffda,
    XK_R9:                          0xffda,
    XK_F30:                         0xffdb,
    XK_R10:                         0xffdb,
    XK_F31:                         0xffdc,
    XK_R11:                         0xffdc,
    XK_F32:                         0xffdd,
    XK_R12:                         0xffdd,
    XK_F33:                         0xffde,
    XK_R13:                         0xffde,
    XK_F34:                         0xffdf,
    XK_R14:                         0xffdf,
    XK_F35:                         0xffe0,
    XK_R15:                         0xffe0,

    /* Modifiers */

    XK_Shift_L:                     0xffe1, /* Left shift */
    XK_Shift_R:                     0xffe2, /* Right shift */
    XK_Control_L:                   0xffe3, /* Left control */
    XK_Control_R:                   0xffe4, /* Right control */
    XK_Caps_Lock:                   0xffe5, /* Caps lock */
    XK_Shift_Lock:                  0xffe6, /* Shift lock */

    XK_Meta_L:                      0xffe7, /* Left meta */
    XK_Meta_R:                      0xffe8, /* Right meta */
    XK_Alt_L:                       0xffe9, /* Left alt */
    XK_Alt_R:                       0xffea, /* Right alt */
    XK_Super_L:                     0xffeb, /* Left super */
    XK_Super_R:                     0xffec, /* Right super */
    XK_Hyper_L:                     0xffed, /* Left hyper */
    XK_Hyper_R:                     0xffee, /* Right hyper */

    /*
     * Keyboard (XKB) Extension function and modifier keys
     * (from Appendix C of "The X Keyboard Extension: Protocol Specification")
     * Byte 3 = 0xfe
     */

    XK_ISO_Level3_Shift:            0xfe03, /* AltGr */
    XK_ISO_Next_Group:              0xfe08,
    XK_ISO_Prev_Group:              0xfe0a,
    XK_ISO_First_Group:             0xfe0c,
    XK_ISO_Last_Group:              0xfe0e,

    /*
     * Latin 1
     * (ISO/IEC 8859-1: Unicode U+0020..U+00FF)
     * Byte 3: 0
     */

    XK_space:                       0x0020, /* U+0020 SPACE */
    XK_exclam:                      0x0021, /* U+0021 EXCLAMATION MARK */
    XK_quotedbl:                    0x0022, /* U+0022 QUOTATION MARK */
    XK_numbersign:                  0x0023, /* U+0023 NUMBER SIGN */
    XK_dollar:                      0x0024, /* U+0024 DOLLAR SIGN */
    XK_percent:                     0x0025, /* U+0025 PERCENT SIGN */
    XK_ampersand:                   0x0026, /* U+0026 AMPERSAND */
    XK_apostrophe:                  0x0027, /* U+0027 APOSTROPHE */
    XK_quoteright:                  0x0027, /* deprecated */
    XK_parenleft:                   0x0028, /* U+0028 LEFT PARENTHESIS */
    XK_parenright:                  0x0029, /* U+0029 RIGHT PARENTHESIS */
    XK_asterisk:                    0x002a, /* U+002A ASTERISK */
    XK_plus:                        0x002b, /* U+002B PLUS SIGN */
    XK_comma:                       0x002c, /* U+002C COMMA */
    XK_minus:                       0x002d, /* U+002D HYPHEN-MINUS */
    XK_period:                      0x002e, /* U+002E FULL STOP */
    XK_slash:                       0x002f, /* U+002F SOLIDUS */
    XK_0:                           0x0030, /* U+0030 DIGIT ZERO */
    XK_1:                           0x0031, /* U+0031 DIGIT ONE */
    XK_2:                           0x0032, /* U+0032 DIGIT TWO */
    XK_3:                           0x0033, /* U+0033 DIGIT THREE */
    XK_4:                           0x0034, /* U+0034 DIGIT FOUR */
    XK_5:                           0x0035, /* U+0035 DIGIT FIVE */
    XK_6:                           0x0036, /* U+0036 DIGIT SIX */
    XK_7:                           0x0037, /* U+0037 DIGIT SEVEN */
    XK_8:                           0x0038, /* U+0038 DIGIT EIGHT */
    XK_9:                           0x0039, /* U+0039 DIGIT NINE */
    XK_colon:                       0x003a, /* U+003A COLON */
    XK_semicolon:                   0x003b, /* U+003B SEMICOLON */
    XK_less:                        0x003c, /* U+003C LESS-THAN SIGN */
    XK_equal:                       0x003d, /* U+003D EQUALS SIGN */
    XK_greater:                     0x003e, /* U+003E GREATER-THAN SIGN */
    XK_question:                    0x003f, /* U+003F QUESTION MARK */
    XK_at:                          0x0040, /* U+0040 COMMERCIAL AT */
    XK_A:                           0x0041, /* U+0041 LATIN CAPITAL LETTER A */
    XK_B:                           0x0042, /* U+0042 LATIN CAPITAL LETTER B */
    XK_C:                           0x0043, /* U+0043 LATIN CAPITAL LETTER C */
    XK_D:                           0x0044, /* U+0044 LATIN CAPITAL LETTER D */
    XK_E:                           0x0045, /* U+0045 LATIN CAPITAL LETTER E */
    XK_F:                           0x0046, /* U+0046 LATIN CAPITAL LETTER F */
    XK_G:                           0x0047, /* U+0047 LATIN CAPITAL LETTER G */
    XK_H:                           0x0048, /* U+0048 LATIN CAPITAL LETTER H */
    XK_I:                           0x0049, /* U+0049 LATIN CAPITAL LETTER I */
    XK_J:                           0x004a, /* U+004A LATIN CAPITAL LETTER J */
    XK_K:                           0x004b, /* U+004B LATIN CAPITAL LETTER K */
    XK_L:                           0x004c, /* U+004C LATIN CAPITAL LETTER L */
    XK_M:                           0x004d, /* U+004D LATIN CAPITAL LETTER M */
    XK_N:                           0x004e, /* U+004E LATIN CAPITAL LETTER N */
    XK_O:                           0x004f, /* U+004F LATIN CAPITAL LETTER O */
    XK_P:                           0x0050, /* U+0050 LATIN CAPITAL LETTER P */
    XK_Q:                           0x0051, /* U+0051 LATIN CAPITAL LETTER Q */
    XK_R:                           0x0052, /* U+0052 LATIN CAPITAL LETTER R */
    XK_S:                           0x0053, /* U+0053 LATIN CAPITAL LETTER S */
    XK_T:                           0x0054, /* U+0054 LATIN CAPITAL LETTER T */
    XK_U:                           0x0055, /* U+0055 LATIN CAPITAL LETTER U */
    XK_V:                           0x0056, /* U+0056 LATIN CAPITAL LETTER V */
    XK_W:                           0x0057, /* U+0057 LATIN CAPITAL LETTER W */
    XK_X:                           0x0058, /* U+0058 LATIN CAPITAL LETTER X */
    XK_Y:                           0x0059, /* U+0059 LATIN CAPITAL LETTER Y */
    XK_Z:                           0x005a, /* U+005A LATIN CAPITAL LETTER Z */
    XK_bracketleft:                 0x005b, /* U+005B LEFT SQUARE BRACKET */
    XK_backslash:                   0x005c, /* U+005C REVERSE SOLIDUS */
    XK_bracketright:                0x005d, /* U+005D RIGHT SQUARE BRACKET */
    XK_asciicircum:                 0x005e, /* U+005E CIRCUMFLEX ACCENT */
    XK_underscore:                  0x005f, /* U+005F LOW LINE */
    XK_grave:                       0x0060, /* U+0060 GRAVE ACCENT */
    XK_quoteleft:                   0x0060, /* deprecated */
    XK_a:                           0x0061, /* U+0061 LATIN SMALL LETTER A */
    XK_b:                           0x0062, /* U+0062 LATIN SMALL LETTER B */
    XK_c:                           0x0063, /* U+0063 LATIN SMALL LETTER C */
    XK_d:                           0x0064, /* U+0064 LATIN SMALL LETTER D */
    XK_e:                           0x0065, /* U+0065 LATIN SMALL LETTER E */
    XK_f:                           0x0066, /* U+0066 LATIN SMALL LETTER F */
    XK_g:                           0x0067, /* U+0067 LATIN SMALL LETTER G */
    XK_h:                           0x0068, /* U+0068 LATIN SMALL LETTER H */
    XK_i:                           0x0069, /* U+0069 LATIN SMALL LETTER I */
    XK_j:                           0x006a, /* U+006A LATIN SMALL LETTER J */
    XK_k:                           0x006b, /* U+006B LATIN SMALL LETTER K */
    XK_l:                           0x006c, /* U+006C LATIN SMALL LETTER L */
    XK_m:                           0x006d, /* U+006D LATIN SMALL LETTER M */
    XK_n:                           0x006e, /* U+006E LATIN SMALL LETTER N */
    XK_o:                           0x006f, /* U+006F LATIN SMALL LETTER O */
    XK_p:                           0x0070, /* U+0070 LATIN SMALL LETTER P */
    XK_q:                           0x0071, /* U+0071 LATIN SMALL LETTER Q */
    XK_r:                           0x0072, /* U+0072 LATIN SMALL LETTER R */
    XK_s:                           0x0073, /* U+0073 LATIN SMALL LETTER S */
    XK_t:                           0x0074, /* U+0074 LATIN SMALL LETTER T */
    XK_u:                           0x0075, /* U+0075 LATIN SMALL LETTER U */
    XK_v:                           0x0076, /* U+0076 LATIN SMALL LETTER V */
    XK_w:                           0x0077, /* U+0077 LATIN SMALL LETTER W */
    XK_x:                           0x0078, /* U+0078 LATIN SMALL LETTER X */
    XK_y:                           0x0079, /* U+0079 LATIN SMALL LETTER Y */
    XK_z:                           0x007a, /* U+007A LATIN SMALL LETTER Z */
    XK_braceleft:                   0x007b, /* U+007B LEFT CURLY BRACKET */
    XK_bar:                         0x007c, /* U+007C VERTICAL LINE */
    XK_braceright:                  0x007d, /* U+007D RIGHT CURLY BRACKET */
    XK_asciitilde:                  0x007e, /* U+007E TILDE */

    XK_nobreakspace:                0x00a0, /* U+00A0 NO-BREAK SPACE */
    XK_exclamdown:                  0x00a1, /* U+00A1 INVERTED EXCLAMATION MARK */
    XK_cent:                        0x00a2, /* U+00A2 CENT SIGN */
    XK_sterling:                    0x00a3, /* U+00A3 POUND SIGN */
    XK_currency:                    0x00a4, /* U+00A4 CURRENCY SIGN */
    XK_yen:                         0x00a5, /* U+00A5 YEN SIGN */
    XK_brokenbar:                   0x00a6, /* U+00A6 BROKEN BAR */
    XK_section:                     0x00a7, /* U+00A7 SECTION SIGN */
    XK_diaeresis:                   0x00a8, /* U+00A8 DIAERESIS */
    XK_copyright:                   0x00a9, /* U+00A9 COPYRIGHT SIGN */
    XK_ordfeminine:                 0x00aa, /* U+00AA FEMININE ORDINAL INDICATOR */
    XK_guillemotleft:               0x00ab, /* U+00AB LEFT-POINTING DOUBLE ANGLE QUOTATION MARK */
    XK_notsign:                     0x00ac, /* U+00AC NOT SIGN */
    XK_hyphen:                      0x00ad, /* U+00AD SOFT HYPHEN */
    XK_registered:                  0x00ae, /* U+00AE REGISTERED SIGN */
    XK_macron:                      0x00af, /* U+00AF MACRON */
    XK_degree:                      0x00b0, /* U+00B0 DEGREE SIGN */
    XK_plusminus:                   0x00b1, /* U+00B1 PLUS-MINUS SIGN */
    XK_twosuperior:                 0x00b2, /* U+00B2 SUPERSCRIPT TWO */
    XK_threesuperior:               0x00b3, /* U+00B3 SUPERSCRIPT THREE */
    XK_acute:                       0x00b4, /* U+00B4 ACUTE ACCENT */
    XK_mu:                          0x00b5, /* U+00B5 MICRO SIGN */
    XK_paragraph:                   0x00b6, /* U+00B6 PILCROW SIGN */
    XK_periodcentered:              0x00b7, /* U+00B7 MIDDLE DOT */
    XK_cedilla:                     0x00b8, /* U+00B8 CEDILLA */
    XK_onesuperior:                 0x00b9, /* U+00B9 SUPERSCRIPT ONE */
    XK_masculine:                   0x00ba, /* U+00BA MASCULINE ORDINAL INDICATOR */
    XK_guillemotright:              0x00bb, /* U+00BB RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK */
    XK_onequarter:                  0x00bc, /* U+00BC VULGAR FRACTION ONE QUARTER */
    XK_onehalf:                     0x00bd, /* U+00BD VULGAR FRACTION ONE HALF */
    XK_threequarters:               0x00be, /* U+00BE VULGAR FRACTION THREE QUARTERS */
    XK_questiondown:                0x00bf, /* U+00BF INVERTED QUESTION MARK */
    XK_Agrave:                      0x00c0, /* U+00C0 LATIN CAPITAL LETTER A WITH GRAVE */
    XK_Aacute:                      0x00c1, /* U+00C1 LATIN CAPITAL LETTER A WITH ACUTE */
    XK_Acircumflex:                 0x00c2, /* U+00C2 LATIN CAPITAL LETTER A WITH CIRCUMFLEX */
    XK_Atilde:                      0x00c3, /* U+00C3 LATIN CAPITAL LETTER A WITH TILDE */
    XK_Adiaeresis:                  0x00c4, /* U+00C4 LATIN CAPITAL LETTER A WITH DIAERESIS */
    XK_Aring:                       0x00c5, /* U+00C5 LATIN CAPITAL LETTER A WITH RING ABOVE */
    XK_AE:                          0x00c6, /* U+00C6 LATIN CAPITAL LETTER AE */
    XK_Ccedilla:                    0x00c7, /* U+00C7 LATIN CAPITAL LETTER C WITH CEDILLA */
    XK_Egrave:                      0x00c8, /* U+00C8 LATIN CAPITAL LETTER E WITH GRAVE */
    XK_Eacute:                      0x00c9, /* U+00C9 LATIN CAPITAL LETTER E WITH ACUTE */
    XK_Ecircumflex:                 0x00ca, /* U+00CA LATIN CAPITAL LETTER E WITH CIRCUMFLEX */
    XK_Ediaeresis:                  0x00cb, /* U+00CB LATIN CAPITAL LETTER E WITH DIAERESIS */
    XK_Igrave:                      0x00cc, /* U+00CC LATIN CAPITAL LETTER I WITH GRAVE */
    XK_Iacute:                      0x00cd, /* U+00CD LATIN CAPITAL LETTER I WITH ACUTE */
    XK_Icircumflex:                 0x00ce, /* U+00CE LATIN CAPITAL LETTER I WITH CIRCUMFLEX */
    XK_Idiaeresis:                  0x00cf, /* U+00CF LATIN CAPITAL LETTER I WITH DIAERESIS */
    XK_ETH:                         0x00d0, /* U+00D0 LATIN CAPITAL LETTER ETH */
    XK_Eth:                         0x00d0, /* deprecated */
    XK_Ntilde:                      0x00d1, /* U+00D1 LATIN CAPITAL LETTER N WITH TILDE */
    XK_Ograve:                      0x00d2, /* U+00D2 LATIN CAPITAL LETTER O WITH GRAVE */
    XK_Oacute:                      0x00d3, /* U+00D3 LATIN CAPITAL LETTER O WITH ACUTE */
    XK_Ocircumflex:                 0x00d4, /* U+00D4 LATIN CAPITAL LETTER O WITH CIRCUMFLEX */
    XK_Otilde:                      0x00d5, /* U+00D5 LATIN CAPITAL LETTER O WITH TILDE */
    XK_Odiaeresis:                  0x00d6, /* U+00D6 LATIN CAPITAL LETTER O WITH DIAERESIS */
    XK_multiply:                    0x00d7, /* U+00D7 MULTIPLICATION SIGN */
    XK_Oslash:                      0x00d8, /* U+00D8 LATIN CAPITAL LETTER O WITH STROKE */
    XK_Ooblique:                    0x00d8, /* U+00D8 LATIN CAPITAL LETTER O WITH STROKE */
    XK_Ugrave:                      0x00d9, /* U+00D9 LATIN CAPITAL LETTER U WITH GRAVE */
    XK_Uacute:                      0x00da, /* U+00DA LATIN CAPITAL LETTER U WITH ACUTE */
    XK_Ucircumflex:                 0x00db, /* U+00DB LATIN CAPITAL LETTER U WITH CIRCUMFLEX */
    XK_Udiaeresis:                  0x00dc, /* U+00DC LATIN CAPITAL LETTER U WITH DIAERESIS */
    XK_Yacute:                      0x00dd, /* U+00DD LATIN CAPITAL LETTER Y WITH ACUTE */
    XK_THORN:                       0x00de, /* U+00DE LATIN CAPITAL LETTER THORN */
    XK_Thorn:                       0x00de, /* deprecated */
    XK_ssharp:                      0x00df, /* U+00DF LATIN SMALL LETTER SHARP S */
    XK_agrave:                      0x00e0, /* U+00E0 LATIN SMALL LETTER A WITH GRAVE */
    XK_aacute:                      0x00e1, /* U+00E1 LATIN SMALL LETTER A WITH ACUTE */
    XK_acircumflex:                 0x00e2, /* U+00E2 LATIN SMALL LETTER A WITH CIRCUMFLEX */
    XK_atilde:                      0x00e3, /* U+00E3 LATIN SMALL LETTER A WITH TILDE */
    XK_adiaeresis:                  0x00e4, /* U+00E4 LATIN SMALL LETTER A WITH DIAERESIS */
    XK_aring:                       0x00e5, /* U+00E5 LATIN SMALL LETTER A WITH RING ABOVE */
    XK_ae:                          0x00e6, /* U+00E6 LATIN SMALL LETTER AE */
    XK_ccedilla:                    0x00e7, /* U+00E7 LATIN SMALL LETTER C WITH CEDILLA */
    XK_egrave:                      0x00e8, /* U+00E8 LATIN SMALL LETTER E WITH GRAVE */
    XK_eacute:                      0x00e9, /* U+00E9 LATIN SMALL LETTER E WITH ACUTE */
    XK_ecircumflex:                 0x00ea, /* U+00EA LATIN SMALL LETTER E WITH CIRCUMFLEX */
    XK_ediaeresis:                  0x00eb, /* U+00EB LATIN SMALL LETTER E WITH DIAERESIS */
    XK_igrave:                      0x00ec, /* U+00EC LATIN SMALL LETTER I WITH GRAVE */
    XK_iacute:                      0x00ed, /* U+00ED LATIN SMALL LETTER I WITH ACUTE */
    XK_icircumflex:                 0x00ee, /* U+00EE LATIN SMALL LETTER I WITH CIRCUMFLEX */
    XK_idiaeresis:                  0x00ef, /* U+00EF LATIN SMALL LETTER I WITH DIAERESIS */
    XK_eth:                         0x00f0, /* U+00F0 LATIN SMALL LETTER ETH */
    XK_ntilde:                      0x00f1, /* U+00F1 LATIN SMALL LETTER N WITH TILDE */
    XK_ograve:                      0x00f2, /* U+00F2 LATIN SMALL LETTER O WITH GRAVE */
    XK_oacute:                      0x00f3, /* U+00F3 LATIN SMALL LETTER O WITH ACUTE */
    XK_ocircumflex:                 0x00f4, /* U+00F4 LATIN SMALL LETTER O WITH CIRCUMFLEX */
    XK_otilde:                      0x00f5, /* U+00F5 LATIN SMALL LETTER O WITH TILDE */
    XK_odiaeresis:                  0x00f6, /* U+00F6 LATIN SMALL LETTER O WITH DIAERESIS */
    XK_division:                    0x00f7, /* U+00F7 DIVISION SIGN */
    XK_oslash:                      0x00f8, /* U+00F8 LATIN SMALL LETTER O WITH STROKE */
    XK_ooblique:                    0x00f8, /* U+00F8 LATIN SMALL LETTER O WITH STROKE */
    XK_ugrave:                      0x00f9, /* U+00F9 LATIN SMALL LETTER U WITH GRAVE */
    XK_uacute:                      0x00fa, /* U+00FA LATIN SMALL LETTER U WITH ACUTE */
    XK_ucircumflex:                 0x00fb, /* U+00FB LATIN SMALL LETTER U WITH CIRCUMFLEX */
    XK_udiaeresis:                  0x00fc, /* U+00FC LATIN SMALL LETTER U WITH DIAERESIS */
    XK_yacute:                      0x00fd, /* U+00FD LATIN SMALL LETTER Y WITH ACUTE */
    XK_thorn:                       0x00fe, /* U+00FE LATIN SMALL LETTER THORN */
    XK_ydiaeresis:                  0x00ff, /* U+00FF LATIN SMALL LETTER Y WITH DIAERESIS */

    /*
     * Korean
     * Byte 3 = 0x0e
     */

    XK_Hangul:                      0xff31, /* Hangul start/stop(toggle) */
    XK_Hangul_Hanja:                0xff34, /* Start Hangul->Hanja Conversion */
    XK_Hangul_Jeonja:               0xff38, /* Jeonja mode */

    /*
     * XFree86 vendor specific keysyms.
     *
     * The XFree86 keysym range is 0x10080001 - 0x1008FFFF.
     */

    XF86XK_ModeLock:                0x1008FF01,
    XF86XK_MonBrightnessUp:         0x1008FF02,
    XF86XK_MonBrightnessDown:       0x1008FF03,
    XF86XK_KbdLightOnOff:           0x1008FF04,
    XF86XK_KbdBrightnessUp:         0x1008FF05,
    XF86XK_KbdBrightnessDown:       0x1008FF06,
    XF86XK_Standby:                 0x1008FF10,
    XF86XK_AudioLowerVolume:        0x1008FF11,
    XF86XK_AudioMute:               0x1008FF12,
    XF86XK_AudioRaiseVolume:        0x1008FF13,
    XF86XK_AudioPlay:               0x1008FF14,
    XF86XK_AudioStop:               0x1008FF15,
    XF86XK_AudioPrev:               0x1008FF16,
    XF86XK_AudioNext:               0x1008FF17,
    XF86XK_HomePage:                0x1008FF18,
    XF86XK_Mail:                    0x1008FF19,
    XF86XK_Start:                   0x1008FF1A,
    XF86XK_Search:                  0x1008FF1B,
    XF86XK_AudioRecord:             0x1008FF1C,
    XF86XK_Calculator:              0x1008FF1D,
    XF86XK_Memo:                    0x1008FF1E,
    XF86XK_ToDoList:                0x1008FF1F,
    XF86XK_Calendar:                0x1008FF20,
    XF86XK_PowerDown:               0x1008FF21,
    XF86XK_ContrastAdjust:          0x1008FF22,
    XF86XK_RockerUp:                0x1008FF23,
    XF86XK_RockerDown:              0x1008FF24,
    XF86XK_RockerEnter:             0x1008FF25,
    XF86XK_Back:                    0x1008FF26,
    XF86XK_Forward:                 0x1008FF27,
    XF86XK_Stop:                    0x1008FF28,
    XF86XK_Refresh:                 0x1008FF29,
    XF86XK_PowerOff:                0x1008FF2A,
    XF86XK_WakeUp:                  0x1008FF2B,
    XF86XK_Eject:                   0x1008FF2C,
    XF86XK_ScreenSaver:             0x1008FF2D,
    XF86XK_WWW:                     0x1008FF2E,
    XF86XK_Sleep:                   0x1008FF2F,
    XF86XK_Favorites:               0x1008FF30,
    XF86XK_AudioPause:              0x1008FF31,
    XF86XK_AudioMedia:              0x1008FF32,
    XF86XK_MyComputer:              0x1008FF33,
    XF86XK_VendorHome:              0x1008FF34,
    XF86XK_LightBulb:               0x1008FF35,
    XF86XK_Shop:                    0x1008FF36,
    XF86XK_History:                 0x1008FF37,
    XF86XK_OpenURL:                 0x1008FF38,
    XF86XK_AddFavorite:             0x1008FF39,
    XF86XK_HotLinks:                0x1008FF3A,
    XF86XK_BrightnessAdjust:        0x1008FF3B,
    XF86XK_Finance:                 0x1008FF3C,
    XF86XK_Community:               0x1008FF3D,
    XF86XK_AudioRewind:             0x1008FF3E,
    XF86XK_BackForward:             0x1008FF3F,
    XF86XK_Launch0:                 0x1008FF40,
    XF86XK_Launch1:                 0x1008FF41,
    XF86XK_Launch2:                 0x1008FF42,
    XF86XK_Launch3:                 0x1008FF43,
    XF86XK_Launch4:                 0x1008FF44,
    XF86XK_Launch5:                 0x1008FF45,
    XF86XK_Launch6:                 0x1008FF46,
    XF86XK_Launch7:                 0x1008FF47,
    XF86XK_Launch8:                 0x1008FF48,
    XF86XK_Launch9:                 0x1008FF49,
    XF86XK_LaunchA:                 0x1008FF4A,
    XF86XK_LaunchB:                 0x1008FF4B,
    XF86XK_LaunchC:                 0x1008FF4C,
    XF86XK_LaunchD:                 0x1008FF4D,
    XF86XK_LaunchE:                 0x1008FF4E,
    XF86XK_LaunchF:                 0x1008FF4F,
    XF86XK_ApplicationLeft:         0x1008FF50,
    XF86XK_ApplicationRight:        0x1008FF51,
    XF86XK_Book:                    0x1008FF52,
    XF86XK_CD:                      0x1008FF53,
    XF86XK_Calculater:              0x1008FF54,
    XF86XK_Clear:                   0x1008FF55,
    XF86XK_Close:                   0x1008FF56,
    XF86XK_Copy:                    0x1008FF57,
    XF86XK_Cut:                     0x1008FF58,
    XF86XK_Display:                 0x1008FF59,
    XF86XK_DOS:                     0x1008FF5A,
    XF86XK_Documents:               0x1008FF5B,
    XF86XK_Excel:                   0x1008FF5C,
    XF86XK_Explorer:                0x1008FF5D,
    XF86XK_Game:                    0x1008FF5E,
    XF86XK_Go:                      0x1008FF5F,
    XF86XK_iTouch:                  0x1008FF60,
    XF86XK_LogOff:                  0x1008FF61,
    XF86XK_Market:                  0x1008FF62,
    XF86XK_Meeting:                 0x1008FF63,
    XF86XK_MenuKB:                  0x1008FF65,
    XF86XK_MenuPB:                  0x1008FF66,
    XF86XK_MySites:                 0x1008FF67,
    XF86XK_New:                     0x1008FF68,
    XF86XK_News:                    0x1008FF69,
    XF86XK_OfficeHome:              0x1008FF6A,
    XF86XK_Open:                    0x1008FF6B,
    XF86XK_Option:                  0x1008FF6C,
    XF86XK_Paste:                   0x1008FF6D,
    XF86XK_Phone:                   0x1008FF6E,
    XF86XK_Q:                       0x1008FF70,
    XF86XK_Reply:                   0x1008FF72,
    XF86XK_Reload:                  0x1008FF73,
    XF86XK_RotateWindows:           0x1008FF74,
    XF86XK_RotationPB:              0x1008FF75,
    XF86XK_RotationKB:              0x1008FF76,
    XF86XK_Save:                    0x1008FF77,
    XF86XK_ScrollUp:                0x1008FF78,
    XF86XK_ScrollDown:              0x1008FF79,
    XF86XK_ScrollClick:             0x1008FF7A,
    XF86XK_Send:                    0x1008FF7B,
    XF86XK_Spell:                   0x1008FF7C,
    XF86XK_SplitScreen:             0x1008FF7D,
    XF86XK_Support:                 0x1008FF7E,
    XF86XK_TaskPane:                0x1008FF7F,
    XF86XK_Terminal:                0x1008FF80,
    XF86XK_Tools:                   0x1008FF81,
    XF86XK_Travel:                  0x1008FF82,
    XF86XK_UserPB:                  0x1008FF84,
    XF86XK_User1KB:                 0x1008FF85,
    XF86XK_User2KB:                 0x1008FF86,
    XF86XK_Video:                   0x1008FF87,
    XF86XK_WheelButton:             0x1008FF88,
    XF86XK_Word:                    0x1008FF89,
    XF86XK_Xfer:                    0x1008FF8A,
    XF86XK_ZoomIn:                  0x1008FF8B,
    XF86XK_ZoomOut:                 0x1008FF8C,
    XF86XK_Away:                    0x1008FF8D,
    XF86XK_Messenger:               0x1008FF8E,
    XF86XK_WebCam:                  0x1008FF8F,
    XF86XK_MailForward:             0x1008FF90,
    XF86XK_Pictures:                0x1008FF91,
    XF86XK_Music:                   0x1008FF92,
    XF86XK_Battery:                 0x1008FF93,
    XF86XK_Bluetooth:               0x1008FF94,
    XF86XK_WLAN:                    0x1008FF95,
    XF86XK_UWB:                     0x1008FF96,
    XF86XK_AudioForward:            0x1008FF97,
    XF86XK_AudioRepeat:             0x1008FF98,
    XF86XK_AudioRandomPlay:         0x1008FF99,
    XF86XK_Subtitle:                0x1008FF9A,
    XF86XK_AudioCycleTrack:         0x1008FF9B,
    XF86XK_CycleAngle:              0x1008FF9C,
    XF86XK_FrameBack:               0x1008FF9D,
    XF86XK_FrameForward:            0x1008FF9E,
    XF86XK_Time:                    0x1008FF9F,
    XF86XK_Select:                  0x1008FFA0,
    XF86XK_View:                    0x1008FFA1,
    XF86XK_TopMenu:                 0x1008FFA2,
    XF86XK_Red:                     0x1008FFA3,
    XF86XK_Green:                   0x1008FFA4,
    XF86XK_Yellow:                  0x1008FFA5,
    XF86XK_Blue:                    0x1008FFA6,
    XF86XK_Suspend:                 0x1008FFA7,
    XF86XK_Hibernate:               0x1008FFA8,
    XF86XK_TouchpadToggle:          0x1008FFA9,
    XF86XK_TouchpadOn:              0x1008FFB0,
    XF86XK_TouchpadOff:             0x1008FFB1,
    XF86XK_AudioMicMute:            0x1008FFB2,
    XF86XK_Switch_VT_1:             0x1008FE01,
    XF86XK_Switch_VT_2:             0x1008FE02,
    XF86XK_Switch_VT_3:             0x1008FE03,
    XF86XK_Switch_VT_4:             0x1008FE04,
    XF86XK_Switch_VT_5:             0x1008FE05,
    XF86XK_Switch_VT_6:             0x1008FE06,
    XF86XK_Switch_VT_7:             0x1008FE07,
    XF86XK_Switch_VT_8:             0x1008FE08,
    XF86XK_Switch_VT_9:             0x1008FE09,
    XF86XK_Switch_VT_10:            0x1008FE0A,
    XF86XK_Switch_VT_11:            0x1008FE0B,
    XF86XK_Switch_VT_12:            0x1008FE0C,
    XF86XK_Ungrab:                  0x1008FE20,
    XF86XK_ClearGrab:               0x1008FE21,
    XF86XK_Next_VMode:              0x1008FE22,
    XF86XK_Prev_VMode:              0x1008FE23,
    XF86XK_LogWindowTree:           0x1008FE24,
    XF86XK_LogGrabInfo:             0x1008FE25,
};

}})();

// === input/xtscancodes.js ===
(function() { // module input_xtscancodes
/*
 * This file is auto-generated from keymaps.csv
 * Database checksum sha256(76d68c10e97d37fe2ea459e210125ae41796253fb217e900bf2983ade13a7920)
 * To re-generate, run:
 *   keymap-gen code-map --lang=js keymaps.csv html atset1
*/
{
  "Again": 0xe005, /* html:Again (Again) -> linux:129 (KEY_AGAIN) -> atset1:57349 */
  "AltLeft": 0x38, /* html:AltLeft (AltLeft) -> linux:56 (KEY_LEFTALT) -> atset1:56 */
  "AltRight": 0xe038, /* html:AltRight (AltRight) -> linux:100 (KEY_RIGHTALT) -> atset1:57400 */
  "ArrowDown": 0xe050, /* html:ArrowDown (ArrowDown) -> linux:108 (KEY_DOWN) -> atset1:57424 */
  "ArrowLeft": 0xe04b, /* html:ArrowLeft (ArrowLeft) -> linux:105 (KEY_LEFT) -> atset1:57419 */
  "ArrowRight": 0xe04d, /* html:ArrowRight (ArrowRight) -> linux:106 (KEY_RIGHT) -> atset1:57421 */
  "ArrowUp": 0xe048, /* html:ArrowUp (ArrowUp) -> linux:103 (KEY_UP) -> atset1:57416 */
  "AudioVolumeDown": 0xe02e, /* html:AudioVolumeDown (AudioVolumeDown) -> linux:114 (KEY_VOLUMEDOWN) -> atset1:57390 */
  "AudioVolumeMute": 0xe020, /* html:AudioVolumeMute (AudioVolumeMute) -> linux:113 (KEY_MUTE) -> atset1:57376 */
  "AudioVolumeUp": 0xe030, /* html:AudioVolumeUp (AudioVolumeUp) -> linux:115 (KEY_VOLUMEUP) -> atset1:57392 */
  "Backquote": 0x29, /* html:Backquote (Backquote) -> linux:41 (KEY_GRAVE) -> atset1:41 */
  "Backslash": 0x2b, /* html:Backslash (Backslash) -> linux:43 (KEY_BACKSLASH) -> atset1:43 */
  "Backspace": 0xe, /* html:Backspace (Backspace) -> linux:14 (KEY_BACKSPACE) -> atset1:14 */
  "BracketLeft": 0x1a, /* html:BracketLeft (BracketLeft) -> linux:26 (KEY_LEFTBRACE) -> atset1:26 */
  "BracketRight": 0x1b, /* html:BracketRight (BracketRight) -> linux:27 (KEY_RIGHTBRACE) -> atset1:27 */
  "BrowserBack": 0xe06a, /* html:BrowserBack (BrowserBack) -> linux:158 (KEY_BACK) -> atset1:57450 */
  "BrowserFavorites": 0xe066, /* html:BrowserFavorites (BrowserFavorites) -> linux:156 (KEY_BOOKMARKS) -> atset1:57446 */
  "BrowserForward": 0xe069, /* html:BrowserForward (BrowserForward) -> linux:159 (KEY_FORWARD) -> atset1:57449 */
  "BrowserHome": 0xe032, /* html:BrowserHome (BrowserHome) -> linux:172 (KEY_HOMEPAGE) -> atset1:57394 */
  "BrowserRefresh": 0xe067, /* html:BrowserRefresh (BrowserRefresh) -> linux:173 (KEY_REFRESH) -> atset1:57447 */
  "BrowserSearch": 0xe065, /* html:BrowserSearch (BrowserSearch) -> linux:217 (KEY_SEARCH) -> atset1:57445 */
  "BrowserStop": 0xe068, /* html:BrowserStop (BrowserStop) -> linux:128 (KEY_STOP) -> atset1:57448 */
  "CapsLock": 0x3a, /* html:CapsLock (CapsLock) -> linux:58 (KEY_CAPSLOCK) -> atset1:58 */
  "Comma": 0x33, /* html:Comma (Comma) -> linux:51 (KEY_COMMA) -> atset1:51 */
  "ContextMenu": 0xe05d, /* html:ContextMenu (ContextMenu) -> linux:127 (KEY_COMPOSE) -> atset1:57437 */
  "ControlLeft": 0x1d, /* html:ControlLeft (ControlLeft) -> linux:29 (KEY_LEFTCTRL) -> atset1:29 */
  "ControlRight": 0xe01d, /* html:ControlRight (ControlRight) -> linux:97 (KEY_RIGHTCTRL) -> atset1:57373 */
  "Convert": 0x79, /* html:Convert (Convert) -> linux:92 (KEY_HENKAN) -> atset1:121 */
  "Copy": 0xe078, /* html:Copy (Copy) -> linux:133 (KEY_COPY) -> atset1:57464 */
  "Cut": 0xe03c, /* html:Cut (Cut) -> linux:137 (KEY_CUT) -> atset1:57404 */
  "Delete": 0xe053, /* html:Delete (Delete) -> linux:111 (KEY_DELETE) -> atset1:57427 */
  "Digit0": 0xb, /* html:Digit0 (Digit0) -> linux:11 (KEY_0) -> atset1:11 */
  "Digit1": 0x2, /* html:Digit1 (Digit1) -> linux:2 (KEY_1) -> atset1:2 */
  "Digit2": 0x3, /* html:Digit2 (Digit2) -> linux:3 (KEY_2) -> atset1:3 */
  "Digit3": 0x4, /* html:Digit3 (Digit3) -> linux:4 (KEY_3) -> atset1:4 */
  "Digit4": 0x5, /* html:Digit4 (Digit4) -> linux:5 (KEY_4) -> atset1:5 */
  "Digit5": 0x6, /* html:Digit5 (Digit5) -> linux:6 (KEY_5) -> atset1:6 */
  "Digit6": 0x7, /* html:Digit6 (Digit6) -> linux:7 (KEY_6) -> atset1:7 */
  "Digit7": 0x8, /* html:Digit7 (Digit7) -> linux:8 (KEY_7) -> atset1:8 */
  "Digit8": 0x9, /* html:Digit8 (Digit8) -> linux:9 (KEY_8) -> atset1:9 */
  "Digit9": 0xa, /* html:Digit9 (Digit9) -> linux:10 (KEY_9) -> atset1:10 */
  "Eject": 0xe07d, /* html:Eject (Eject) -> linux:162 (KEY_EJECTCLOSECD) -> atset1:57469 */
  "End": 0xe04f, /* html:End (End) -> linux:107 (KEY_END) -> atset1:57423 */
  "Enter": 0x1c, /* html:Enter (Enter) -> linux:28 (KEY_ENTER) -> atset1:28 */
  "Equal": 0xd, /* html:Equal (Equal) -> linux:13 (KEY_EQUAL) -> atset1:13 */
  "Escape": 0x1, /* html:Escape (Escape) -> linux:1 (KEY_ESC) -> atset1:1 */
  "F1": 0x3b, /* html:F1 (F1) -> linux:59 (KEY_F1) -> atset1:59 */
  "F10": 0x44, /* html:F10 (F10) -> linux:68 (KEY_F10) -> atset1:68 */
  "F11": 0x57, /* html:F11 (F11) -> linux:87 (KEY_F11) -> atset1:87 */
  "F12": 0x58, /* html:F12 (F12) -> linux:88 (KEY_F12) -> atset1:88 */
  "F13": 0x5d, /* html:F13 (F13) -> linux:183 (KEY_F13) -> atset1:93 */
  "F14": 0x5e, /* html:F14 (F14) -> linux:184 (KEY_F14) -> atset1:94 */
  "F15": 0x5f, /* html:F15 (F15) -> linux:185 (KEY_F15) -> atset1:95 */
  "F16": 0x55, /* html:F16 (F16) -> linux:186 (KEY_F16) -> atset1:85 */
  "F17": 0xe003, /* html:F17 (F17) -> linux:187 (KEY_F17) -> atset1:57347 */
  "F18": 0xe077, /* html:F18 (F18) -> linux:188 (KEY_F18) -> atset1:57463 */
  "F19": 0xe004, /* html:F19 (F19) -> linux:189 (KEY_F19) -> atset1:57348 */
  "F2": 0x3c, /* html:F2 (F2) -> linux:60 (KEY_F2) -> atset1:60 */
  "F20": 0x5a, /* html:F20 (F20) -> linux:190 (KEY_F20) -> atset1:90 */
  "F21": 0x74, /* html:F21 (F21) -> linux:191 (KEY_F21) -> atset1:116 */
  "F22": 0xe079, /* html:F22 (F22) -> linux:192 (KEY_F22) -> atset1:57465 */
  "F23": 0x6d, /* html:F23 (F23) -> linux:193 (KEY_F23) -> atset1:109 */
  "F24": 0x6f, /* html:F24 (F24) -> linux:194 (KEY_F24) -> atset1:111 */
  "F3": 0x3d, /* html:F3 (F3) -> linux:61 (KEY_F3) -> atset1:61 */
  "F4": 0x3e, /* html:F4 (F4) -> linux:62 (KEY_F4) -> atset1:62 */
  "F5": 0x3f, /* html:F5 (F5) -> linux:63 (KEY_F5) -> atset1:63 */
  "F6": 0x40, /* html:F6 (F6) -> linux:64 (KEY_F6) -> atset1:64 */
  "F7": 0x41, /* html:F7 (F7) -> linux:65 (KEY_F7) -> atset1:65 */
  "F8": 0x42, /* html:F8 (F8) -> linux:66 (KEY_F8) -> atset1:66 */
  "F9": 0x43, /* html:F9 (F9) -> linux:67 (KEY_F9) -> atset1:67 */
  "Find": 0xe041, /* html:Find (Find) -> linux:136 (KEY_FIND) -> atset1:57409 */
  "Help": 0xe075, /* html:Help (Help) -> linux:138 (KEY_HELP) -> atset1:57461 */
  "Hiragana": 0x77, /* html:Hiragana (Lang4) -> linux:91 (KEY_HIRAGANA) -> atset1:119 */
  "Home": 0xe047, /* html:Home (Home) -> linux:102 (KEY_HOME) -> atset1:57415 */
  "Insert": 0xe052, /* html:Insert (Insert) -> linux:110 (KEY_INSERT) -> atset1:57426 */
  "IntlBackslash": 0x56, /* html:IntlBackslash (IntlBackslash) -> linux:86 (KEY_102ND) -> atset1:86 */
  "IntlRo": 0x73, /* html:IntlRo (IntlRo) -> linux:89 (KEY_RO) -> atset1:115 */
  "IntlYen": 0x7d, /* html:IntlYen (IntlYen) -> linux:124 (KEY_YEN) -> atset1:125 */
  "KanaMode": 0x70, /* html:KanaMode (KanaMode) -> linux:93 (KEY_KATAKANAHIRAGANA) -> atset1:112 */
  "Katakana": 0x78, /* html:Katakana (Lang3) -> linux:90 (KEY_KATAKANA) -> atset1:120 */
  "KeyA": 0x1e, /* html:KeyA (KeyA) -> linux:30 (KEY_A) -> atset1:30 */
  "KeyB": 0x30, /* html:KeyB (KeyB) -> linux:48 (KEY_B) -> atset1:48 */
  "KeyC": 0x2e, /* html:KeyC (KeyC) -> linux:46 (KEY_C) -> atset1:46 */
  "KeyD": 0x20, /* html:KeyD (KeyD) -> linux:32 (KEY_D) -> atset1:32 */
  "KeyE": 0x12, /* html:KeyE (KeyE) -> linux:18 (KEY_E) -> atset1:18 */
  "KeyF": 0x21, /* html:KeyF (KeyF) -> linux:33 (KEY_F) -> atset1:33 */
  "KeyG": 0x22, /* html:KeyG (KeyG) -> linux:34 (KEY_G) -> atset1:34 */
  "KeyH": 0x23, /* html:KeyH (KeyH) -> linux:35 (KEY_H) -> atset1:35 */
  "KeyI": 0x17, /* html:KeyI (KeyI) -> linux:23 (KEY_I) -> atset1:23 */
  "KeyJ": 0x24, /* html:KeyJ (KeyJ) -> linux:36 (KEY_J) -> atset1:36 */
  "KeyK": 0x25, /* html:KeyK (KeyK) -> linux:37 (KEY_K) -> atset1:37 */
  "KeyL": 0x26, /* html:KeyL (KeyL) -> linux:38 (KEY_L) -> atset1:38 */
  "KeyM": 0x32, /* html:KeyM (KeyM) -> linux:50 (KEY_M) -> atset1:50 */
  "KeyN": 0x31, /* html:KeyN (KeyN) -> linux:49 (KEY_N) -> atset1:49 */
  "KeyO": 0x18, /* html:KeyO (KeyO) -> linux:24 (KEY_O) -> atset1:24 */
  "KeyP": 0x19, /* html:KeyP (KeyP) -> linux:25 (KEY_P) -> atset1:25 */
  "KeyQ": 0x10, /* html:KeyQ (KeyQ) -> linux:16 (KEY_Q) -> atset1:16 */
  "KeyR": 0x13, /* html:KeyR (KeyR) -> linux:19 (KEY_R) -> atset1:19 */
  "KeyS": 0x1f, /* html:KeyS (KeyS) -> linux:31 (KEY_S) -> atset1:31 */
  "KeyT": 0x14, /* html:KeyT (KeyT) -> linux:20 (KEY_T) -> atset1:20 */
  "KeyU": 0x16, /* html:KeyU (KeyU) -> linux:22 (KEY_U) -> atset1:22 */
  "KeyV": 0x2f, /* html:KeyV (KeyV) -> linux:47 (KEY_V) -> atset1:47 */
  "KeyW": 0x11, /* html:KeyW (KeyW) -> linux:17 (KEY_W) -> atset1:17 */
  "KeyX": 0x2d, /* html:KeyX (KeyX) -> linux:45 (KEY_X) -> atset1:45 */
  "KeyY": 0x15, /* html:KeyY (KeyY) -> linux:21 (KEY_Y) -> atset1:21 */
  "KeyZ": 0x2c, /* html:KeyZ (KeyZ) -> linux:44 (KEY_Z) -> atset1:44 */
  "Lang1": 0x72, /* html:Lang1 (Lang1) -> linux:122 (KEY_HANGEUL) -> atset1:114 */
  "Lang2": 0x71, /* html:Lang2 (Lang2) -> linux:123 (KEY_HANJA) -> atset1:113 */
  "Lang3": 0x78, /* html:Lang3 (Lang3) -> linux:90 (KEY_KATAKANA) -> atset1:120 */
  "Lang4": 0x77, /* html:Lang4 (Lang4) -> linux:91 (KEY_HIRAGANA) -> atset1:119 */
  "Lang5": 0x76, /* html:Lang5 (Lang5) -> linux:85 (KEY_ZENKAKUHANKAKU) -> atset1:118 */
  "LaunchApp1": 0xe06b, /* html:LaunchApp1 (LaunchApp1) -> linux:157 (KEY_COMPUTER) -> atset1:57451 */
  "LaunchApp2": 0xe021, /* html:LaunchApp2 (LaunchApp2) -> linux:140 (KEY_CALC) -> atset1:57377 */
  "LaunchMail": 0xe06c, /* html:LaunchMail (LaunchMail) -> linux:155 (KEY_MAIL) -> atset1:57452 */
  "MediaPlayPause": 0xe022, /* html:MediaPlayPause (MediaPlayPause) -> linux:164 (KEY_PLAYPAUSE) -> atset1:57378 */
  "MediaSelect": 0xe06d, /* html:MediaSelect (MediaSelect) -> linux:226 (KEY_MEDIA) -> atset1:57453 */
  "MediaStop": 0xe024, /* html:MediaStop (MediaStop) -> linux:166 (KEY_STOPCD) -> atset1:57380 */
  "MediaTrackNext": 0xe019, /* html:MediaTrackNext (MediaTrackNext) -> linux:163 (KEY_NEXTSONG) -> atset1:57369 */
  "MediaTrackPrevious": 0xe010, /* html:MediaTrackPrevious (MediaTrackPrevious) -> linux:165 (KEY_PREVIOUSSONG) -> atset1:57360 */
  "MetaLeft": 0xe05b, /* html:MetaLeft (MetaLeft) -> linux:125 (KEY_LEFTMETA) -> atset1:57435 */
  "MetaRight": 0xe05c, /* html:MetaRight (MetaRight) -> linux:126 (KEY_RIGHTMETA) -> atset1:57436 */
  "Minus": 0xc, /* html:Minus (Minus) -> linux:12 (KEY_MINUS) -> atset1:12 */
  "NonConvert": 0x7b, /* html:NonConvert (NonConvert) -> linux:94 (KEY_MUHENKAN) -> atset1:123 */
  "NumLock": 0x45, /* html:NumLock (NumLock) -> linux:69 (KEY_NUMLOCK) -> atset1:69 */
  "Numpad0": 0x52, /* html:Numpad0 (Numpad0) -> linux:82 (KEY_KP0) -> atset1:82 */
  "Numpad1": 0x4f, /* html:Numpad1 (Numpad1) -> linux:79 (KEY_KP1) -> atset1:79 */
  "Numpad2": 0x50, /* html:Numpad2 (Numpad2) -> linux:80 (KEY_KP2) -> atset1:80 */
  "Numpad3": 0x51, /* html:Numpad3 (Numpad3) -> linux:81 (KEY_KP3) -> atset1:81 */
  "Numpad4": 0x4b, /* html:Numpad4 (Numpad4) -> linux:75 (KEY_KP4) -> atset1:75 */
  "Numpad5": 0x4c, /* html:Numpad5 (Numpad5) -> linux:76 (KEY_KP5) -> atset1:76 */
  "Numpad6": 0x4d, /* html:Numpad6 (Numpad6) -> linux:77 (KEY_KP6) -> atset1:77 */
  "Numpad7": 0x47, /* html:Numpad7 (Numpad7) -> linux:71 (KEY_KP7) -> atset1:71 */
  "Numpad8": 0x48, /* html:Numpad8 (Numpad8) -> linux:72 (KEY_KP8) -> atset1:72 */
  "Numpad9": 0x49, /* html:Numpad9 (Numpad9) -> linux:73 (KEY_KP9) -> atset1:73 */
  "NumpadAdd": 0x4e, /* html:NumpadAdd (NumpadAdd) -> linux:78 (KEY_KPPLUS) -> atset1:78 */
  "NumpadComma": 0x7e, /* html:NumpadComma (NumpadComma) -> linux:121 (KEY_KPCOMMA) -> atset1:126 */
  "NumpadDecimal": 0x53, /* html:NumpadDecimal (NumpadDecimal) -> linux:83 (KEY_KPDOT) -> atset1:83 */
  "NumpadDivide": 0xe035, /* html:NumpadDivide (NumpadDivide) -> linux:98 (KEY_KPSLASH) -> atset1:57397 */
  "NumpadEnter": 0xe01c, /* html:NumpadEnter (NumpadEnter) -> linux:96 (KEY_KPENTER) -> atset1:57372 */
  "NumpadEqual": 0x59, /* html:NumpadEqual (NumpadEqual) -> linux:117 (KEY_KPEQUAL) -> atset1:89 */
  "NumpadMultiply": 0x37, /* html:NumpadMultiply (NumpadMultiply) -> linux:55 (KEY_KPASTERISK) -> atset1:55 */
  "NumpadParenLeft": 0xe076, /* html:NumpadParenLeft (NumpadParenLeft) -> linux:179 (KEY_KPLEFTPAREN) -> atset1:57462 */
  "NumpadParenRight": 0xe07b, /* html:NumpadParenRight (NumpadParenRight) -> linux:180 (KEY_KPRIGHTPAREN) -> atset1:57467 */
  "NumpadSubtract": 0x4a, /* html:NumpadSubtract (NumpadSubtract) -> linux:74 (KEY_KPMINUS) -> atset1:74 */
  "Open": 0x64, /* html:Open (Open) -> linux:134 (KEY_OPEN) -> atset1:100 */
  "PageDown": 0xe051, /* html:PageDown (PageDown) -> linux:109 (KEY_PAGEDOWN) -> atset1:57425 */
  "PageUp": 0xe049, /* html:PageUp (PageUp) -> linux:104 (KEY_PAGEUP) -> atset1:57417 */
  "Paste": 0x65, /* html:Paste (Paste) -> linux:135 (KEY_PASTE) -> atset1:101 */
  "Pause": 0xe046, /* html:Pause (Pause) -> linux:119 (KEY_PAUSE) -> atset1:57414 */
  "Period": 0x34, /* html:Period (Period) -> linux:52 (KEY_DOT) -> atset1:52 */
  "Power": 0xe05e, /* html:Power (Power) -> linux:116 (KEY_POWER) -> atset1:57438 */
  "PrintScreen": 0x54, /* html:PrintScreen (PrintScreen) -> linux:99 (KEY_SYSRQ) -> atset1:84 */
  "Props": 0xe006, /* html:Props (Props) -> linux:130 (KEY_PROPS) -> atset1:57350 */
  "Quote": 0x28, /* html:Quote (Quote) -> linux:40 (KEY_APOSTROPHE) -> atset1:40 */
  "ScrollLock": 0x46, /* html:ScrollLock (ScrollLock) -> linux:70 (KEY_SCROLLLOCK) -> atset1:70 */
  "Semicolon": 0x27, /* html:Semicolon (Semicolon) -> linux:39 (KEY_SEMICOLON) -> atset1:39 */
  "ShiftLeft": 0x2a, /* html:ShiftLeft (ShiftLeft) -> linux:42 (KEY_LEFTSHIFT) -> atset1:42 */
  "ShiftRight": 0x36, /* html:ShiftRight (ShiftRight) -> linux:54 (KEY_RIGHTSHIFT) -> atset1:54 */
  "Slash": 0x35, /* html:Slash (Slash) -> linux:53 (KEY_SLASH) -> atset1:53 */
  "Sleep": 0xe05f, /* html:Sleep (Sleep) -> linux:142 (KEY_SLEEP) -> atset1:57439 */
  "Space": 0x39, /* html:Space (Space) -> linux:57 (KEY_SPACE) -> atset1:57 */
  "Suspend": 0xe025, /* html:Suspend (Suspend) -> linux:205 (KEY_SUSPEND) -> atset1:57381 */
  "Tab": 0xf, /* html:Tab (Tab) -> linux:15 (KEY_TAB) -> atset1:15 */
  "Undo": 0xe007, /* html:Undo (Undo) -> linux:131 (KEY_UNDO) -> atset1:57351 */
  "WakeUp": 0xe063, /* html:WakeUp (WakeUp) -> linux:143 (KEY_WAKEUP) -> atset1:57443 */
};

}})();

// === input/keyboard.js ===
(function() { // module input_keyboard
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 or any later version (see LICENSE.txt)
 */


//
// Keyboard event handler
//

class Keyboard {
    constructor(target) {
        this._target = target || null;

        this._keyDownList = {};         // List of depressed keys
                                        // (even if they are happy)
        this._altGrArmed = false;       // Windows AltGr detection

        // keep these here so we can refer to them later
        this._eventHandlers = {
            'keyup': this._handleKeyUp.bind(this),
            'keydown': this._handleKeyDown.bind(this),
            'blur': this._allKeysUp.bind(this),
        };

        // ===== EVENT HANDLERS =====

        this.onkeyevent = () => {}; // Handler for key press/release
    }

    // ===== PRIVATE METHODS =====

    _sendKeyEvent(keysym, code, down, numlock = null, capslock = null) {
        if (down) {
            this._keyDownList[code] = keysym;
        } else {
            // Do we really think this key is down?
            if (!(code in this._keyDownList)) {
                return;
            }
            delete this._keyDownList[code];
        }

        Log.Debug("onkeyevent " + (down ? "down" : "up") +
                  ", keysym: " + keysym, ", code: " + code +
                  ", numlock: " + numlock + ", capslock: " + capslock);
        this.onkeyevent(keysym, code, down, numlock, capslock);
    }

    _getKeyCode(e) {
        const code = KeyboardUtil.getKeycode(e);
        if (code !== 'Unidentified') {
            return code;
        }

        // Unstable, but we don't have anything else to go on
        if (e.keyCode) {
            // 229 is used for composition events
            if (e.keyCode !== 229) {
                return 'Platform' + e.keyCode;
            }
        }

        // A precursor to the final DOM3 standard. Unfortunately it
        // is not layout independent, so it is as bad as using keyCode
        if (e.keyIdentifier) {
            // Non-character key?
            if (e.keyIdentifier.substr(0, 2) !== 'U+') {
                return e.keyIdentifier;
            }

            const codepoint = parseInt(e.keyIdentifier.substr(2), 16);
            const char = String.fromCharCode(codepoint).toUpperCase();

            return 'Platform' + char.charCodeAt();
        }

        return 'Unidentified';
    }

    _handleKeyDown(e) {
        const code = this._getKeyCode(e);
        let keysym = KeyboardUtil.getKeysym(e);
        let numlock = e.getModifierState('NumLock');
        let capslock = e.getModifierState('CapsLock');

        // getModifierState for NumLock is not supported on mac and ios and always returns false.
        // Set to null to indicate unknown/unsupported instead.
        if (browser.isMac() || browser.isIOS()) {
            numlock = null;
        }

        // Windows doesn't have a proper AltGr, but handles it using
        // fake Ctrl+Alt. However the remote end might not be Windows,
        // so we need to merge those in to a single AltGr event. We
        // detect this case by seeing the two key events directly after
        // each other with a very short time between them (<50ms).
        if (this._altGrArmed) {
            this._altGrArmed = false;
            clearTimeout(this._altGrTimeout);

            if ((code === "AltRight") &&
                ((e.timeStamp - this._altGrCtrlTime) < 50)) {
                // FIXME: We fail to detect this if either Ctrl key is
                //        first manually pressed as Windows then no
                //        longer sends the fake Ctrl down event. It
                //        does however happily send real Ctrl events
                //        even when AltGr is already down. Some
                //        browsers detect this for us though and set the
                //        key to "AltGraph".
                keysym = KeyTable.XK_ISO_Level3_Shift;
            } else {
                this._sendKeyEvent(KeyTable.XK_Control_L, "ControlLeft", true, numlock, capslock);
            }
        }

        // We cannot handle keys we cannot track, but we also need
        // to deal with virtual keyboards which omit key info
        if (code === 'Unidentified') {
            if (keysym) {
                // If it's a virtual keyboard then it should be
                // sufficient to just send press and release right
                // after each other
                this._sendKeyEvent(keysym, code, true, numlock, capslock);
                this._sendKeyEvent(keysym, code, false, numlock, capslock);
            }

            stopEvent(e);
            return;
        }

        // Alt behaves more like AltGraph on macOS, so shuffle the
        // keys around a bit to make things more sane for the remote
        // server. This method is used by RealVNC and TigerVNC (and
        // possibly others).
        if (browser.isMac() || browser.isIOS()) {
            switch (keysym) {
                case KeyTable.XK_Super_L:
                    keysym = KeyTable.XK_Alt_L;
                    break;
                case KeyTable.XK_Super_R:
                    keysym = KeyTable.XK_Super_L;
                    break;
                case KeyTable.XK_Alt_L:
                    keysym = KeyTable.XK_Mode_switch;
                    break;
                case KeyTable.XK_Alt_R:
                    keysym = KeyTable.XK_ISO_Level3_Shift;
                    break;
            }
        }

        // Is this key already pressed? If so, then we must use the
        // same keysym or we'll confuse the server
        if (code in this._keyDownList) {
            keysym = this._keyDownList[code];
        }

        // macOS doesn't send proper key releases if a key is pressed
        // while meta is held down
        if ((browser.isMac() || browser.isIOS()) &&
            (e.metaKey && code !== 'MetaLeft' && code !== 'MetaRight')) {
            this._sendKeyEvent(keysym, code, true, numlock, capslock);
            this._sendKeyEvent(keysym, code, false, numlock, capslock);
            stopEvent(e);
            return;
        }

        // macOS doesn't send proper key events for modifiers, only
        // state change events. That gets extra confusing for CapsLock
        // which toggles on each press, but not on release. So pretend
        // it was a quick press and release of the button.
        if ((browser.isMac() || browser.isIOS()) && (code === 'CapsLock')) {
            this._sendKeyEvent(KeyTable.XK_Caps_Lock, 'CapsLock', true, numlock, capslock);
            this._sendKeyEvent(KeyTable.XK_Caps_Lock, 'CapsLock', false, numlock, capslock);
            stopEvent(e);
            return;
        }

        // Windows doesn't send proper key releases for a bunch of
        // Japanese IM keys so we have to fake the release right away
        const jpBadKeys = [ KeyTable.XK_Zenkaku_Hankaku,
                            KeyTable.XK_Eisu_toggle,
                            KeyTable.XK_Katakana,
                            KeyTable.XK_Hiragana,
                            KeyTable.XK_Romaji ];
        if (browser.isWindows() && jpBadKeys.includes(keysym)) {
            this._sendKeyEvent(keysym, code, true, numlock, capslock);
            this._sendKeyEvent(keysym, code, false, numlock, capslock);
            stopEvent(e);
            return;
        }

        stopEvent(e);

        // Possible start of AltGr sequence? (see above)
        if ((code === "ControlLeft") && browser.isWindows() &&
            !("ControlLeft" in this._keyDownList)) {
            this._altGrArmed = true;
            this._altGrTimeout = setTimeout(this._handleAltGrTimeout.bind(this), 100);
            this._altGrCtrlTime = e.timeStamp;
            return;
        }

        this._sendKeyEvent(keysym, code, true, numlock, capslock);
    }

    _handleKeyUp(e) {
        stopEvent(e);

        const code = this._getKeyCode(e);

        // We can't get a release in the middle of an AltGr sequence, so
        // abort that detection
        if (this._altGrArmed) {
            this._altGrArmed = false;
            clearTimeout(this._altGrTimeout);
            this._sendKeyEvent(KeyTable.XK_Control_L, "ControlLeft", true);
        }

        // See comment in _handleKeyDown()
        if ((browser.isMac() || browser.isIOS()) && (code === 'CapsLock')) {
            this._sendKeyEvent(KeyTable.XK_Caps_Lock, 'CapsLock', true);
            this._sendKeyEvent(KeyTable.XK_Caps_Lock, 'CapsLock', false);
            return;
        }

        this._sendKeyEvent(this._keyDownList[code], code, false);

        // Windows has a rather nasty bug where it won't send key
        // release events for a Shift button if the other Shift is still
        // pressed
        if (browser.isWindows() && ((code === 'ShiftLeft') ||
                                    (code === 'ShiftRight'))) {
            if ('ShiftRight' in this._keyDownList) {
                this._sendKeyEvent(this._keyDownList['ShiftRight'],
                                   'ShiftRight', false);
            }
            if ('ShiftLeft' in this._keyDownList) {
                this._sendKeyEvent(this._keyDownList['ShiftLeft'],
                                   'ShiftLeft', false);
            }
        }
    }

    _handleAltGrTimeout() {
        this._altGrArmed = false;
        clearTimeout(this._altGrTimeout);
        this._sendKeyEvent(KeyTable.XK_Control_L, "ControlLeft", true);
    }

    _allKeysUp() {
        Log.Debug(">> Keyboard.allKeysUp");
        for (let code in this._keyDownList) {
            this._sendKeyEvent(this._keyDownList[code], code, false);
        }
        Log.Debug("<< Keyboard.allKeysUp");
    }

    // ===== PUBLIC METHODS =====

    grab() {
        //Log.Debug(">> Keyboard.grab");

        this._target.addEventListener('keydown', this._eventHandlers.keydown);
        this._target.addEventListener('keyup', this._eventHandlers.keyup);

        // Release (key up) if window loses focus
        window.addEventListener('blur', this._eventHandlers.blur);

        //Log.Debug("<< Keyboard.grab");
    }

    ungrab() {
        //Log.Debug(">> Keyboard.ungrab");

        this._target.removeEventListener('keydown', this._eventHandlers.keydown);
        this._target.removeEventListener('keyup', this._eventHandlers.keyup);
        window.removeEventListener('blur', this._eventHandlers.blur);

        // Release (key up) all keys that are in a down state
        this._allKeysUp();

        //Log.Debug(">> Keyboard.ungrab");
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === input/gesturehandler.js ===
(function() { // module input_gesturehandler
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2020 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */

const GH_NOGESTURE = 0;
const GH_ONETAP    = 1;
const GH_TWOTAP    = 2;
const GH_THREETAP  = 4;
const GH_DRAG      = 8;
const GH_LONGPRESS = 16;
const GH_TWODRAG   = 32;
const GH_PINCH     = 64;

const GH_INITSTATE = 127;

const GH_MOVE_THRESHOLD = 50;
const GH_ANGLE_THRESHOLD = 90; // Degrees

// Timeout when waiting for gestures (ms)
const GH_MULTITOUCH_TIMEOUT = 250;

// Maximum time between press and release for a tap (ms)
const GH_TAP_TIMEOUT = 1000;

// Timeout when waiting for longpress (ms)
const GH_LONGPRESS_TIMEOUT = 1000;

// Timeout when waiting to decide between PINCH and TWODRAG (ms)
const GH_TWOTOUCH_TIMEOUT = 50;

class GestureHandler {
    constructor() {
        this._target = null;

        this._state = GH_INITSTATE;

        this._tracked = [];
        this._ignored = [];

        this._waitingRelease = false;
        this._releaseStart = 0.0;

        this._longpressTimeoutId = null;
        this._twoTouchTimeoutId = null;

        this._boundEventHandler = this._eventHandler.bind(this);
    }

    attach(target) {
        this.detach();

        this._target = target;
        this._target.addEventListener('touchstart',
                                      this._boundEventHandler);
        this._target.addEventListener('touchmove',
                                      this._boundEventHandler);
        this._target.addEventListener('touchend',
                                      this._boundEventHandler);
        this._target.addEventListener('touchcancel',
                                      this._boundEventHandler);
    }

    detach() {
        if (!this._target) {
            return;
        }

        this._stopLongpressTimeout();
        this._stopTwoTouchTimeout();

        this._target.removeEventListener('touchstart',
                                         this._boundEventHandler);
        this._target.removeEventListener('touchmove',
                                         this._boundEventHandler);
        this._target.removeEventListener('touchend',
                                         this._boundEventHandler);
        this._target.removeEventListener('touchcancel',
                                         this._boundEventHandler);
        this._target = null;
    }

    _eventHandler(e) {
        let fn;

        e.stopPropagation();
        e.preventDefault();

        switch (e.type) {
            case 'touchstart':
                fn = this._touchStart;
                break;
            case 'touchmove':
                fn = this._touchMove;
                break;
            case 'touchend':
            case 'touchcancel':
                fn = this._touchEnd;
                break;
        }

        for (let i = 0; i < e.changedTouches.length; i++) {
            let touch = e.changedTouches[i];
            fn.call(this, touch.identifier, touch.clientX, touch.clientY);
        }
    }

    _touchStart(id, x, y) {
        // Ignore any new touches if there is already an active gesture,
        // or we're in a cleanup state
        if (this._hasDetectedGesture() || (this._state === GH_NOGESTURE)) {
            this._ignored.push(id);
            return;
        }

        // Did it take too long between touches that we should no longer
        // consider this a single gesture?
        if ((this._tracked.length > 0) &&
            ((Date.now() - this._tracked[0].started) > GH_MULTITOUCH_TIMEOUT)) {
            this._state = GH_NOGESTURE;
            this._ignored.push(id);
            return;
        }

        // If we're waiting for fingers to release then we should no longer
        // recognize new touches
        if (this._waitingRelease) {
            this._state = GH_NOGESTURE;
            this._ignored.push(id);
            return;
        }

        this._tracked.push({
            id: id,
            started: Date.now(),
            active: true,
            firstX: x,
            firstY: y,
            lastX: x,
            lastY: y,
            angle: 0
        });

        switch (this._tracked.length) {
            case 1:
                this._startLongpressTimeout();
                break;

            case 2:
                this._state &= ~(GH_ONETAP | GH_DRAG | GH_LONGPRESS);
                this._stopLongpressTimeout();
                break;

            case 3:
                this._state &= ~(GH_TWOTAP | GH_TWODRAG | GH_PINCH);
                break;

            default:
                this._state = GH_NOGESTURE;
        }
    }

    _touchMove(id, x, y) {
        let touch = this._tracked.find(t => t.id === id);

        // If this is an update for a touch we're not tracking, ignore it
        if (touch === undefined) {
            return;
        }

        // Update the touches last position with the event coordinates
        touch.lastX = x;
        touch.lastY = y;

        let deltaX = x - touch.firstX;
        let deltaY = y - touch.firstY;

        // Update angle when the touch has moved
        if ((touch.firstX !== touch.lastX) ||
            (touch.firstY !== touch.lastY)) {
            touch.angle = Math.atan2(deltaY, deltaX) * 180 / Math.PI;
        }

        if (!this._hasDetectedGesture()) {
            // Ignore moves smaller than the minimum threshold
            if (Math.hypot(deltaX, deltaY) < GH_MOVE_THRESHOLD) {
                return;
            }

            // Can't be a tap or long press as we've seen movement
            this._state &= ~(GH_ONETAP | GH_TWOTAP | GH_THREETAP | GH_LONGPRESS);
            this._stopLongpressTimeout();

            if (this._tracked.length !== 1) {
                this._state &= ~(GH_DRAG);
            }
            if (this._tracked.length !== 2) {
                this._state &= ~(GH_TWODRAG | GH_PINCH);
            }

            // We need to figure out which of our different two touch gestures
            // this might be
            if (this._tracked.length === 2) {

                // The other touch is the one where the id doesn't match
                let prevTouch = this._tracked.find(t => t.id !== id);

                // How far the previous touch point has moved since start
                let prevDeltaMove = Math.hypot(prevTouch.firstX - prevTouch.lastX,
                                               prevTouch.firstY - prevTouch.lastY);

                // We know that the current touch moved far enough,
                // but unless both touches moved further than their
                // threshold we don't want to disqualify any gestures
                if (prevDeltaMove > GH_MOVE_THRESHOLD) {

                    // The angle difference between the direction of the touch points
                    let deltaAngle = Math.abs(touch.angle - prevTouch.angle);
                    deltaAngle = Math.abs(((deltaAngle + 180) % 360) - 180);

                    // PINCH or TWODRAG can be eliminated depending on the angle
                    if (deltaAngle > GH_ANGLE_THRESHOLD) {
                        this._state &= ~GH_TWODRAG;
                    } else {
                        this._state &= ~GH_PINCH;
                    }

                    if (this._isTwoTouchTimeoutRunning()) {
                        this._stopTwoTouchTimeout();
                    }
                } else if (!this._isTwoTouchTimeoutRunning()) {
                    // We can't determine the gesture right now, let's
                    // wait and see if more events are on their way
                    this._startTwoTouchTimeout();
                }
            }

            if (!this._hasDetectedGesture()) {
                return;
            }

            this._pushEvent('gesturestart');
        }

        this._pushEvent('gesturemove');
    }

    _touchEnd(id, x, y) {
        // Check if this is an ignored touch
        if (this._ignored.indexOf(id) !== -1) {
            // Remove this touch from ignored
            this._ignored.splice(this._ignored.indexOf(id), 1);

            // And reset the state if there are no more touches
            if ((this._ignored.length === 0) &&
                (this._tracked.length === 0)) {
                this._state = GH_INITSTATE;
                this._waitingRelease = false;
            }
            return;
        }

        // We got a touchend before the timer triggered,
        // this cannot result in a gesture anymore.
        if (!this._hasDetectedGesture() &&
            this._isTwoTouchTimeoutRunning()) {
            this._stopTwoTouchTimeout();
            this._state = GH_NOGESTURE;
        }

        // Some gestures don't trigger until a touch is released
        if (!this._hasDetectedGesture()) {
            // Can't be a gesture that relies on movement
            this._state &= ~(GH_DRAG | GH_TWODRAG | GH_PINCH);
            // Or something that relies on more time
            this._state &= ~GH_LONGPRESS;
            this._stopLongpressTimeout();

            if (!this._waitingRelease) {
                this._releaseStart = Date.now();
                this._waitingRelease = true;

                // Can't be a tap that requires more touches than we current have
                switch (this._tracked.length) {
                    case 1:
                        this._state &= ~(GH_TWOTAP | GH_THREETAP);
                        break;

                    case 2:
                        this._state &= ~(GH_ONETAP | GH_THREETAP);
                        break;
                }
            }
        }

        // Waiting for all touches to release? (i.e. some tap)
        if (this._waitingRelease) {
            // Were all touches released at roughly the same time?
            if ((Date.now() - this._releaseStart) > GH_MULTITOUCH_TIMEOUT) {
                this._state = GH_NOGESTURE;
            }

            // Did too long time pass between press and release?
            if (this._tracked.some(t => (Date.now() - t.started) > GH_TAP_TIMEOUT)) {
                this._state = GH_NOGESTURE;
            }

            let touch = this._tracked.find(t => t.id === id);
            touch.active = false;

            // Are we still waiting for more releases?
            if (this._hasDetectedGesture()) {
                this._pushEvent('gesturestart');
            } else {
                // Have we reached a dead end?
                if (this._state !== GH_NOGESTURE) {
                    return;
                }
            }
        }

        if (this._hasDetectedGesture()) {
            this._pushEvent('gestureend');
        }

        // Ignore any remaining touches until they are ended
        for (let i = 0; i < this._tracked.length; i++) {
            if (this._tracked[i].active) {
                this._ignored.push(this._tracked[i].id);
            }
        }
        this._tracked = [];

        this._state = GH_NOGESTURE;

        // Remove this touch from ignored if it's in there
        if (this._ignored.indexOf(id) !== -1) {
            this._ignored.splice(this._ignored.indexOf(id), 1);
        }

        // We reset the state if ignored is empty
        if ((this._ignored.length === 0)) {
            this._state = GH_INITSTATE;
            this._waitingRelease = false;
        }
    }

    _hasDetectedGesture() {
        if (this._state === GH_NOGESTURE) {
            return false;
        }
        // Check to see if the bitmask value is a power of 2
        // (i.e. only one bit set). If it is, we have a state.
        if (this._state & (this._state - 1)) {
            return false;
        }

        // For taps we also need to have all touches released
        // before we've fully detected the gesture
        if (this._state & (GH_ONETAP | GH_TWOTAP | GH_THREETAP)) {
            if (this._tracked.some(t => t.active)) {
                return false;
            }
        }

        return true;
    }

    _startLongpressTimeout() {
        this._stopLongpressTimeout();
        this._longpressTimeoutId = setTimeout(() => this._longpressTimeout(),
                                              GH_LONGPRESS_TIMEOUT);
    }

    _stopLongpressTimeout() {
        clearTimeout(this._longpressTimeoutId);
        this._longpressTimeoutId = null;
    }

    _longpressTimeout() {
        if (this._hasDetectedGesture()) {
            throw new Error("A longpress gesture failed, conflict with a different gesture");
        }

        this._state = GH_LONGPRESS;
        this._pushEvent('gesturestart');
    }

    _startTwoTouchTimeout() {
        this._stopTwoTouchTimeout();
        this._twoTouchTimeoutId = setTimeout(() => this._twoTouchTimeout(),
                                             GH_TWOTOUCH_TIMEOUT);
    }

    _stopTwoTouchTimeout() {
        clearTimeout(this._twoTouchTimeoutId);
        this._twoTouchTimeoutId = null;
    }

    _isTwoTouchTimeoutRunning() {
        return this._twoTouchTimeoutId !== null;
    }

    _twoTouchTimeout() {
        if (this._tracked.length === 0) {
            throw new Error("A pinch or two drag gesture failed, no tracked touches");
        }

        // How far each touch point has moved since start
        let avgM = this._getAverageMovement();
        let avgMoveH = Math.abs(avgM.x);
        let avgMoveV = Math.abs(avgM.y);

        // The difference in the distance between where
        // the touch points started and where they are now
        let avgD = this._getAverageDistance();
        let deltaTouchDistance = Math.abs(Math.hypot(avgD.first.x, avgD.first.y) -
                                          Math.hypot(avgD.last.x, avgD.last.y));

        if ((avgMoveV < deltaTouchDistance) &&
            (avgMoveH < deltaTouchDistance)) {
            this._state = GH_PINCH;
        } else {
            this._state = GH_TWODRAG;
        }

        this._pushEvent('gesturestart');
        this._pushEvent('gesturemove');
    }

    _pushEvent(type) {
        let detail = { type: this._stateToGesture(this._state) };

        // For most gesture events the current (average) position is the
        // most useful
        let avg = this._getPosition();
        let pos = avg.last;

        // However we have a slight distance to detect gestures, so for the
        // first gesture event we want to use the first positions we saw
        if (type === 'gesturestart') {
            pos = avg.first;
        }

        // For these gestures, we always want the event coordinates
        // to be where the gesture began, not the current touch location.
        switch (this._state) {
            case GH_TWODRAG:
            case GH_PINCH:
                pos = avg.first;
                break;
        }

        detail['clientX'] = pos.x;
        detail['clientY'] = pos.y;

        // FIXME: other coordinates?

        // Some gestures also have a magnitude
        if (this._state === GH_PINCH) {
            let distance = this._getAverageDistance();
            if (type === 'gesturestart') {
                detail['magnitudeX'] = distance.first.x;
                detail['magnitudeY'] = distance.first.y;
            } else {
                detail['magnitudeX'] = distance.last.x;
                detail['magnitudeY'] = distance.last.y;
            }
        } else if (this._state === GH_TWODRAG) {
            if (type === 'gesturestart') {
                detail['magnitudeX'] = 0.0;
                detail['magnitudeY'] = 0.0;
            } else {
                let movement = this._getAverageMovement();
                detail['magnitudeX'] = movement.x;
                detail['magnitudeY'] = movement.y;
            }
        }

        let gev = new CustomEvent(type, { detail: detail });
        this._target.dispatchEvent(gev);
    }

    _stateToGesture(state) {
        switch (state) {
            case GH_ONETAP:
                return 'onetap';
            case GH_TWOTAP:
                return 'twotap';
            case GH_THREETAP:
                return 'threetap';
            case GH_DRAG:
                return 'drag';
            case GH_LONGPRESS:
                return 'longpress';
            case GH_TWODRAG:
                return 'twodrag';
            case GH_PINCH:
                return 'pinch';
        }

        throw new Error("Unknown gesture state: " + state);
    }

    _getPosition() {
        if (this._tracked.length === 0) {
            throw new Error("Failed to get gesture position, no tracked touches");
        }

        let size = this._tracked.length;
        let fx = 0, fy = 0, lx = 0, ly = 0;

        for (let i = 0; i < this._tracked.length; i++) {
            fx += this._tracked[i].firstX;
            fy += this._tracked[i].firstY;
            lx += this._tracked[i].lastX;
            ly += this._tracked[i].lastY;
        }

        return { first: { x: fx / size,
                          y: fy / size },
                 last: { x: lx / size,
                         y: ly / size } };
    }

    _getAverageMovement() {
        if (this._tracked.length === 0) {
            throw new Error("Failed to get gesture movement, no tracked touches");
        }

        let totalH, totalV;
        totalH = totalV = 0;
        let size = this._tracked.length;

        for (let i = 0; i < this._tracked.length; i++) {
            totalH += this._tracked[i].lastX - this._tracked[i].firstX;
            totalV += this._tracked[i].lastY - this._tracked[i].firstY;
        }

        return { x: totalH / size,
                 y: totalV / size };
    }

    _getAverageDistance() {
        if (this._tracked.length === 0) {
            throw new Error("Failed to get gesture distance, no tracked touches");
        }

        // Distance between the first and last tracked touches

        let first = this._tracked[0];
        let last = this._tracked[this._tracked.length - 1];

        let fdx = Math.abs(last.firstX - first.firstX);
        let fdy = Math.abs(last.firstY - first.firstY);

        let ldx = Math.abs(last.lastX - first.lastX);
        let ldy = Math.abs(last.lastY - first.lastY);

        return { first: { x: fdx, y: fdy },
                 last: { x: ldx, y: ldy } };
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === encodings.js ===
(function() { // module encodings
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */

const encodings = {
    encodingRaw: 0,
    encodingCopyRect: 1,
    encodingRRE: 2,
    encodingHextile: 5,
    encodingTight: 7,
    encodingZRLE: 16,
    encodingTightPNG: -260,
    encodingJPEG: 21,

    pseudoEncodingQualityLevel9: -23,
    pseudoEncodingQualityLevel0: -32,
    pseudoEncodingDesktopSize: -223,
    pseudoEncodingLastRect: -224,
    pseudoEncodingCursor: -239,
    pseudoEncodingQEMUExtendedKeyEvent: -258,
    pseudoEncodingQEMULedEvent: -261,
    pseudoEncodingDesktopName: -307,
    pseudoEncodingExtendedDesktopSize: -308,
    pseudoEncodingXvp: -309,
    pseudoEncodingFence: -312,
    pseudoEncodingContinuousUpdates: -313,
    pseudoEncodingCompressLevel9: -247,
    pseudoEncodingCompressLevel0: -256,
    pseudoEncodingVMwareCursor: 0x574d5664,
    pseudoEncodingExtendedClipboard: 0xc0a1e5ce
};

function encodingName(num) {
    switch (num) {
        case encodings.encodingRaw:      return "Raw";
        case encodings.encodingCopyRect: return "CopyRect";
        case encodings.encodingRRE:      return "RRE";
        case encodings.encodingHextile:  return "Hextile";
        case encodings.encodingTight:    return "Tight";
        case encodings.encodingZRLE:     return "ZRLE";
        case encodings.encodingTightPNG: return "TightPNG";
        case encodings.encodingJPEG:     return "JPEG";
        default:                         return "[unknown encoding " + num + "]";
    }
}

}})();

// === decoders/raw.js ===
(function() { // module decoders_raw
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */

class RawDecoder {
    constructor() {
        this._lines = 0;
    }

    decodeRect(x, y, width, height, sock, display, depth) {
        if ((width === 0) || (height === 0)) {
            return true;
        }

        if (this._lines === 0) {
            this._lines = height;
        }

        const pixelSize = depth == 8 ? 1 : 4;
        const bytesPerLine = width * pixelSize;

        while (this._lines > 0) {
            if (sock.rQwait("RAW", bytesPerLine)) {
                return false;
            }

            const curY = y + (height - this._lines);

            let data = sock.rQshiftBytes(bytesPerLine, false);

            // Convert data if needed
            if (depth == 8) {
                const newdata = new Uint8Array(width * 4);
                for (let i = 0; i < width; i++) {
                    newdata[i * 4 + 0] = ((data[i] >> 0) & 0x3) * 255 / 3;
                    newdata[i * 4 + 1] = ((data[i] >> 2) & 0x3) * 255 / 3;
                    newdata[i * 4 + 2] = ((data[i] >> 4) & 0x3) * 255 / 3;
                    newdata[i * 4 + 3] = 255;
                }
                data = newdata;
            }

            // Max sure the image is fully opaque
            for (let i = 0; i < width; i++) {
                data[i * 4 + 3] = 255;
            }

            display.blitImage(x, curY, width, 1, data, 0);
            this._lines--;
        }

        return true;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/copyrect.js ===
(function() { // module decoders_copyrect
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */

class CopyRectDecoder {
    decodeRect(x, y, width, height, sock, display, depth) {
        if (sock.rQwait("COPYRECT", 4)) {
            return false;
        }

        let deltaX = sock.rQshift16();
        let deltaY = sock.rQshift16();

        if ((width === 0) || (height === 0)) {
            return true;
        }

        display.copyImage(deltaX, deltaY, x, y, width, height);

        return true;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/rre.js ===
(function() { // module decoders_rre
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */

class RREDecoder {
    constructor() {
        this._subrects = 0;
    }

    decodeRect(x, y, width, height, sock, display, depth) {
        if (this._subrects === 0) {
            if (sock.rQwait("RRE", 4 + 4)) {
                return false;
            }

            this._subrects = sock.rQshift32();

            let color = sock.rQshiftBytes(4);  // Background
            display.fillRect(x, y, width, height, color);
        }

        while (this._subrects > 0) {
            if (sock.rQwait("RRE", 4 + 8)) {
                return false;
            }

            let color = sock.rQshiftBytes(4);
            let sx = sock.rQshift16();
            let sy = sock.rQshift16();
            let swidth = sock.rQshift16();
            let sheight = sock.rQshift16();
            display.fillRect(x + sx, y + sy, swidth, sheight, color);

            this._subrects--;
        }

        return true;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/hextile.js ===
(function() { // module decoders_hextile
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */


class HextileDecoder {
    constructor() {
        this._tiles = 0;
        this._lastsubencoding = 0;
        this._tileBuffer = new Uint8Array(16 * 16 * 4);
    }

    decodeRect(x, y, width, height, sock, display, depth) {
        if (this._tiles === 0) {
            this._tilesX = Math.ceil(width / 16);
            this._tilesY = Math.ceil(height / 16);
            this._totalTiles = this._tilesX * this._tilesY;
            this._tiles = this._totalTiles;
        }

        while (this._tiles > 0) {
            let bytes = 1;

            if (sock.rQwait("HEXTILE", bytes)) {
                return false;
            }

            let subencoding = sock.rQpeek8();
            if (subencoding > 30) {  // Raw
                throw new Error("Illegal hextile subencoding (subencoding: " +
                            subencoding + ")");
            }

            const currTile = this._totalTiles - this._tiles;
            const tileX = currTile % this._tilesX;
            const tileY = Math.floor(currTile / this._tilesX);
            const tx = x + tileX * 16;
            const ty = y + tileY * 16;
            const tw = Math.min(16, (x + width) - tx);
            const th = Math.min(16, (y + height) - ty);

            // Figure out how much we are expecting
            if (subencoding & 0x01) {  // Raw
                bytes += tw * th * 4;
            } else {
                if (subencoding & 0x02) {  // Background
                    bytes += 4;
                }
                if (subencoding & 0x04) {  // Foreground
                    bytes += 4;
                }
                if (subencoding & 0x08) {  // AnySubrects
                    bytes++;  // Since we aren't shifting it off

                    if (sock.rQwait("HEXTILE", bytes)) {
                        return false;
                    }

                    let subrects = sock.rQpeekBytes(bytes).at(-1);
                    if (subencoding & 0x10) {  // SubrectsColoured
                        bytes += subrects * (4 + 2);
                    } else {
                        bytes += subrects * 2;
                    }
                }
            }

            if (sock.rQwait("HEXTILE", bytes)) {
                return false;
            }

            // We know the encoding and have a whole tile
            sock.rQshift8();
            if (subencoding === 0) {
                if (this._lastsubencoding & 0x01) {
                    // Weird: ignore blanks are RAW
                    Log.Debug("     Ignoring blank after RAW");
                } else {
                    display.fillRect(tx, ty, tw, th, this._background);
                }
            } else if (subencoding & 0x01) {  // Raw
                let pixels = tw * th;
                let data = sock.rQshiftBytes(pixels * 4, false);
                // Max sure the image is fully opaque
                for (let i = 0;i <  pixels;i++) {
                    data[i * 4 + 3] = 255;
                }
                display.blitImage(tx, ty, tw, th, data, 0);
            } else {
                if (subencoding & 0x02) {  // Background
                    this._background = new Uint8Array(sock.rQshiftBytes(4));
                }
                if (subencoding & 0x04) {  // Foreground
                    this._foreground = new Uint8Array(sock.rQshiftBytes(4));
                }

                this._startTile(tx, ty, tw, th, this._background);
                if (subencoding & 0x08) {  // AnySubrects
                    let subrects = sock.rQshift8();

                    for (let s = 0; s < subrects; s++) {
                        let color;
                        if (subencoding & 0x10) {  // SubrectsColoured
                            color = sock.rQshiftBytes(4);
                        } else {
                            color = this._foreground;
                        }
                        const xy = sock.rQshift8();
                        const sx = (xy >> 4);
                        const sy = (xy & 0x0f);

                        const wh = sock.rQshift8();
                        const sw = (wh >> 4) + 1;
                        const sh = (wh & 0x0f) + 1;

                        this._subTile(sx, sy, sw, sh, color);
                    }
                }
                this._finishTile(display);
            }
            this._lastsubencoding = subencoding;
            this._tiles--;
        }

        return true;
    }

    // start updating a tile
    _startTile(x, y, width, height, color) {
        this._tileX = x;
        this._tileY = y;
        this._tileW = width;
        this._tileH = height;

        const red = color[0];
        const green = color[1];
        const blue = color[2];

        const data = this._tileBuffer;
        for (let i = 0; i < width * height * 4; i += 4) {
            data[i]     = red;
            data[i + 1] = green;
            data[i + 2] = blue;
            data[i + 3] = 255;
        }
    }

    // update sub-rectangle of the current tile
    _subTile(x, y, w, h, color) {
        const red = color[0];
        const green = color[1];
        const blue = color[2];
        const xend = x + w;
        const yend = y + h;

        const data = this._tileBuffer;
        const width = this._tileW;
        for (let j = y; j < yend; j++) {
            for (let i = x; i < xend; i++) {
                const p = (i + (j * width)) * 4;
                data[p]     = red;
                data[p + 1] = green;
                data[p + 2] = blue;
                data[p + 3] = 255;
            }
        }
    }

    // draw the current tile to the screen
    _finishTile(display) {
        display.blitImage(this._tileX, this._tileY,
                          this._tileW, this._tileH,
                          this._tileBuffer, 0);
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/tight.js ===
(function() { // module decoders_tight
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * (c) 2012 Michael Tinglof, Joe Balaz, Les Piech (Mercuri.ca)
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */


class TightDecoder {
    constructor() {
        this._ctl = null;
        this._filter = null;
        this._numColors = 0;
        this._palette = new Uint8Array(1024);  // 256 * 4 (max palette size * max bytes-per-pixel)
        this._len = 0;

        this._zlibs = [];
        for (let i = 0; i < 4; i++) {
            this._zlibs[i] = new Inflator();
        }
    }

    decodeRect(x, y, width, height, sock, display, depth) {
        if (this._ctl === null) {
            if (sock.rQwait("TIGHT compression-control", 1)) {
                return false;
            }

            this._ctl = sock.rQshift8();

            // Reset streams if the server requests it
            for (let i = 0; i < 4; i++) {
                if ((this._ctl >> i) & 1) {
                    this._zlibs[i].reset();
                    Log.Info("Reset zlib stream " + i);
                }
            }

            // Figure out filter
            this._ctl = this._ctl >> 4;
        }

        let ret;

        if (this._ctl === 0x08) {
            ret = this._fillRect(x, y, width, height,
                                 sock, display, depth);
        } else if (this._ctl === 0x09) {
            ret = this._jpegRect(x, y, width, height,
                                 sock, display, depth);
        } else if (this._ctl === 0x0A) {
            ret = this._pngRect(x, y, width, height,
                                sock, display, depth);
        } else if ((this._ctl & 0x08) == 0) {
            ret = this._basicRect(this._ctl, x, y, width, height,
                                  sock, display, depth);
        } else {
            throw new Error("Illegal tight compression received (ctl: " +
                                   this._ctl + ")");
        }

        if (ret) {
            this._ctl = null;
        }

        return ret;
    }

    _fillRect(x, y, width, height, sock, display, depth) {
        if (sock.rQwait("TIGHT", 3)) {
            return false;
        }

        let pixel = sock.rQshiftBytes(3);
        display.fillRect(x, y, width, height, pixel, false);

        return true;
    }

    _jpegRect(x, y, width, height, sock, display, depth) {
        let data = this._readData(sock);
        if (data === null) {
            return false;
        }

        display.imageRect(x, y, width, height, "image/jpeg", data);

        return true;
    }

    _pngRect(x, y, width, height, sock, display, depth) {
        throw new Error("PNG received in standard Tight rect");
    }

    _basicRect(ctl, x, y, width, height, sock, display, depth) {
        if (this._filter === null) {
            if (ctl & 0x4) {
                if (sock.rQwait("TIGHT", 1)) {
                    return false;
                }

                this._filter = sock.rQshift8();
            } else {
                // Implicit CopyFilter
                this._filter = 0;
            }
        }

        let streamId = ctl & 0x3;

        let ret;

        switch (this._filter) {
            case 0: // CopyFilter
                ret = this._copyFilter(streamId, x, y, width, height,
                                       sock, display, depth);
                break;
            case 1: // PaletteFilter
                ret = this._paletteFilter(streamId, x, y, width, height,
                                          sock, display, depth);
                break;
            case 2: // GradientFilter
                ret = this._gradientFilter(streamId, x, y, width, height,
                                           sock, display, depth);
                break;
            default:
                throw new Error("Illegal tight filter received (ctl: " +
                                       this._filter + ")");
        }

        if (ret) {
            this._filter = null;
        }

        return ret;
    }

    _copyFilter(streamId, x, y, width, height, sock, display, depth) {
        const uncompressedSize = width * height * 3;
        let data;

        if (uncompressedSize === 0) {
            return true;
        }

        if (uncompressedSize < 12) {
            if (sock.rQwait("TIGHT", uncompressedSize)) {
                return false;
            }

            data = sock.rQshiftBytes(uncompressedSize);
        } else {
            data = this._readData(sock);
            if (data === null) {
                return false;
            }

            this._zlibs[streamId].setInput(data);
            data = this._zlibs[streamId].inflate(uncompressedSize);
            this._zlibs[streamId].setInput(null);
        }

        let rgbx = new Uint8Array(width * height * 4);
        for (let i = 0, j = 0; i < width * height * 4; i += 4, j += 3) {
            rgbx[i]     = data[j];
            rgbx[i + 1] = data[j + 1];
            rgbx[i + 2] = data[j + 2];
            rgbx[i + 3] = 255;  // Alpha
        }

        display.blitImage(x, y, width, height, rgbx, 0, false);

        return true;
    }

    _paletteFilter(streamId, x, y, width, height, sock, display, depth) {
        if (this._numColors === 0) {
            if (sock.rQwait("TIGHT palette", 1)) {
                return false;
            }

            const numColors = sock.rQpeek8() + 1;
            const paletteSize = numColors * 3;

            if (sock.rQwait("TIGHT palette", 1 + paletteSize)) {
                return false;
            }

            this._numColors = numColors;
            sock.rQskipBytes(1);

            sock.rQshiftTo(this._palette, paletteSize);
        }

        const bpp = (this._numColors <= 2) ? 1 : 8;
        const rowSize = Math.floor((width * bpp + 7) / 8);
        const uncompressedSize = rowSize * height;

        let data;

        if (uncompressedSize === 0) {
            return true;
        }

        if (uncompressedSize < 12) {
            if (sock.rQwait("TIGHT", uncompressedSize)) {
                return false;
            }

            data = sock.rQshiftBytes(uncompressedSize);
        } else {
            data = this._readData(sock);
            if (data === null) {
                return false;
            }

            this._zlibs[streamId].setInput(data);
            data = this._zlibs[streamId].inflate(uncompressedSize);
            this._zlibs[streamId].setInput(null);
        }

        // Convert indexed (palette based) image data to RGB
        if (this._numColors == 2) {
            this._monoRect(x, y, width, height, data, this._palette, display);
        } else {
            this._paletteRect(x, y, width, height, data, this._palette, display);
        }

        this._numColors = 0;

        return true;
    }

    _monoRect(x, y, width, height, data, palette, display) {
        // Convert indexed (palette based) image data to RGB
        // TODO: reduce number of calculations inside loop
        const dest = this._getScratchBuffer(width * height * 4);
        const w = Math.floor((width + 7) / 8);
        const w1 = Math.floor(width / 8);

        for (let y = 0; y < height; y++) {
            let dp, sp, x;
            for (x = 0; x < w1; x++) {
                for (let b = 7; b >= 0; b--) {
                    dp = (y * width + x * 8 + 7 - b) * 4;
                    sp = (data[y * w + x] >> b & 1) * 3;
                    dest[dp]     = palette[sp];
                    dest[dp + 1] = palette[sp + 1];
                    dest[dp + 2] = palette[sp + 2];
                    dest[dp + 3] = 255;
                }
            }

            for (let b = 7; b >= 8 - width % 8; b--) {
                dp = (y * width + x * 8 + 7 - b) * 4;
                sp = (data[y * w + x] >> b & 1) * 3;
                dest[dp]     = palette[sp];
                dest[dp + 1] = palette[sp + 1];
                dest[dp + 2] = palette[sp + 2];
                dest[dp + 3] = 255;
            }
        }

        display.blitImage(x, y, width, height, dest, 0, false);
    }

    _paletteRect(x, y, width, height, data, palette, display) {
        // Convert indexed (palette based) image data to RGB
        const dest = this._getScratchBuffer(width * height * 4);
        const total = width * height * 4;
        for (let i = 0, j = 0; i < total; i += 4, j++) {
            const sp = data[j] * 3;
            dest[i]     = palette[sp];
            dest[i + 1] = palette[sp + 1];
            dest[i + 2] = palette[sp + 2];
            dest[i + 3] = 255;
        }

        display.blitImage(x, y, width, height, dest, 0, false);
    }

    _gradientFilter(streamId, x, y, width, height, sock, display, depth) {
        // assume the TPIXEL is 3 bytes long
        const uncompressedSize = width * height * 3;
        let data;

        if (uncompressedSize === 0) {
            return true;
        }

        if (uncompressedSize < 12) {
            if (sock.rQwait("TIGHT", uncompressedSize)) {
                return false;
            }

            data = sock.rQshiftBytes(uncompressedSize);
        } else {
            data = this._readData(sock);
            if (data === null) {
                return false;
            }

            this._zlibs[streamId].setInput(data);
            data = this._zlibs[streamId].inflate(uncompressedSize);
            this._zlibs[streamId].setInput(null);
        }

        let rgbx = new Uint8Array(4 * width * height);

        let rgbxIndex = 0, dataIndex = 0;
        let left = new Uint8Array(3);
        for (let x = 0; x < width; x++) {
            for (let c = 0; c < 3; c++) {
                const prediction = left[c];
                const value = data[dataIndex++] + prediction;
                rgbx[rgbxIndex++] = value;
                left[c] = value;
            }
            rgbx[rgbxIndex++] = 255;
        }

        let upperIndex = 0;
        let upper = new Uint8Array(3),
            upperleft = new Uint8Array(3);
        for (let y = 1; y < height; y++) {
            left.fill(0);
            upperleft.fill(0);
            for (let x = 0; x < width; x++) {
                for (let c = 0; c < 3; c++) {
                    upper[c] = rgbx[upperIndex++];
                    let prediction = left[c] + upper[c] - upperleft[c];
                    if (prediction < 0) {
                        prediction = 0;
                    } else if (prediction > 255) {
                        prediction = 255;
                    }
                    const value = data[dataIndex++] + prediction;
                    rgbx[rgbxIndex++] = value;
                    upperleft[c] = upper[c];
                    left[c] = value;
                }
                rgbx[rgbxIndex++] = 255;
                upperIndex++;
            }
        }

        display.blitImage(x, y, width, height, rgbx, 0, false);

        return true;
    }

    _readData(sock) {
        if (this._len === 0) {
            if (sock.rQwait("TIGHT", 3)) {
                return null;
            }

            let byte;

            byte = sock.rQshift8();
            this._len = byte & 0x7f;
            if (byte & 0x80) {
                byte = sock.rQshift8();
                this._len |= (byte & 0x7f) << 7;
                if (byte & 0x80) {
                    byte = sock.rQshift8();
                    this._len |= byte << 14;
                }
            }
        }

        if (sock.rQwait("TIGHT", this._len)) {
            return null;
        }

        let data = sock.rQshiftBytes(this._len, false);
        this._len = 0;

        return data;
    }

    _getScratchBuffer(size) {
        if (!this._scratchBuffer || (this._scratchBuffer.length < size)) {
            this._scratchBuffer = new Uint8Array(size);
        }
        return this._scratchBuffer;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/tightpng.js ===
(function() { // module decoders_tightpng
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */


class TightPNGDecoder extends TightDecoder {
    _pngRect(x, y, width, height, sock, display, depth) {
        let data = this._readData(sock);
        if (data === null) {
            return false;
        }

        display.imageRect(x, y, width, height, "image/png", data);

        return true;
    }

    _basicRect(ctl, x, y, width, height, sock, display, depth) {
        throw new Error("BasicCompression received in TightPNG rect");
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/zrle.js ===
(function() { // module decoders_zrle
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2021 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */


const ZRLE_TILE_WIDTH = 64;
const ZRLE_TILE_HEIGHT = 64;

class ZRLEDecoder {
    constructor() {
        this._length = 0;
        this._inflator = new Inflate();

        this._pixelBuffer = new Uint8Array(ZRLE_TILE_WIDTH * ZRLE_TILE_HEIGHT * 4);
        this._tileBuffer = new Uint8Array(ZRLE_TILE_WIDTH * ZRLE_TILE_HEIGHT * 4);
    }

    decodeRect(x, y, width, height, sock, display, depth) {
        if (this._length === 0) {
            if (sock.rQwait("ZLib data length", 4)) {
                return false;
            }
            this._length = sock.rQshift32();
        }
        if (sock.rQwait("Zlib data", this._length)) {
            return false;
        }

        const data = sock.rQshiftBytes(this._length, false);

        this._inflator.setInput(data);

        for (let ty = y; ty < y + height; ty += ZRLE_TILE_HEIGHT) {
            let th = Math.min(ZRLE_TILE_HEIGHT, y + height - ty);

            for (let tx = x; tx < x + width; tx += ZRLE_TILE_WIDTH) {
                let tw = Math.min(ZRLE_TILE_WIDTH, x + width - tx);

                const tileSize = tw * th;
                const subencoding = this._inflator.inflate(1)[0];
                if (subencoding === 0) {
                    // raw data
                    const data = this._readPixels(tileSize);
                    display.blitImage(tx, ty, tw, th, data, 0, false);
                } else if (subencoding === 1) {
                    // solid
                    const background = this._readPixels(1);
                    display.fillRect(tx, ty, tw, th, [background[0], background[1], background[2]]);
                } else if (subencoding >= 2 && subencoding <= 16) {
                    const data = this._decodePaletteTile(subencoding, tileSize, tw, th);
                    display.blitImage(tx, ty, tw, th, data, 0, false);
                } else if (subencoding === 128) {
                    const data = this._decodeRLETile(tileSize);
                    display.blitImage(tx, ty, tw, th, data, 0, false);
                } else if (subencoding >= 130 && subencoding <= 255) {
                    const data = this._decodeRLEPaletteTile(subencoding - 128, tileSize);
                    display.blitImage(tx, ty, tw, th, data, 0, false);
                } else {
                    throw new Error('Unknown subencoding: ' + subencoding);
                }
            }
        }
        this._length = 0;
        return true;
    }

    _getBitsPerPixelInPalette(paletteSize) {
        if (paletteSize <= 2) {
            return 1;
        } else if (paletteSize <= 4) {
            return 2;
        } else if (paletteSize <= 16) {
            return 4;
        }
    }

    _readPixels(pixels) {
        let data = this._pixelBuffer;
        const buffer = this._inflator.inflate(3*pixels);
        for (let i = 0, j = 0; i < pixels*4; i += 4, j += 3) {
            data[i]     = buffer[j];
            data[i + 1] = buffer[j + 1];
            data[i + 2] = buffer[j + 2];
            data[i + 3] = 255;  // Add the Alpha
        }
        return data;
    }

    _decodePaletteTile(paletteSize, tileSize, tilew, tileh) {
        const data = this._tileBuffer;
        const palette = this._readPixels(paletteSize);
        const bitsPerPixel = this._getBitsPerPixelInPalette(paletteSize);
        const mask = (1 << bitsPerPixel) - 1;

        let offset = 0;
        let encoded = this._inflator.inflate(1)[0];

        for (let y=0; y<tileh; y++) {
            let shift = 8-bitsPerPixel;
            for (let x=0; x<tilew; x++) {
                if (shift<0) {
                    shift=8-bitsPerPixel;
                    encoded = this._inflator.inflate(1)[0];
                }
                let indexInPalette = (encoded>>shift) & mask;

                data[offset] = palette[indexInPalette * 4];
                data[offset + 1] = palette[indexInPalette * 4 + 1];
                data[offset + 2] = palette[indexInPalette * 4 + 2];
                data[offset + 3] = palette[indexInPalette * 4 + 3];
                offset += 4;
                shift-=bitsPerPixel;
            }
            if (shift<8-bitsPerPixel && y<tileh-1) {
                encoded =  this._inflator.inflate(1)[0];
            }
        }
        return data;
    }

    _decodeRLETile(tileSize) {
        const data = this._tileBuffer;
        let i = 0;
        while (i < tileSize) {
            const pixel = this._readPixels(1);
            const length = this._readRLELength();
            for (let j = 0; j < length; j++) {
                data[i * 4] = pixel[0];
                data[i * 4 + 1] = pixel[1];
                data[i * 4 + 2] = pixel[2];
                data[i * 4 + 3] = pixel[3];
                i++;
            }
        }
        return data;
    }

    _decodeRLEPaletteTile(paletteSize, tileSize) {
        const data = this._tileBuffer;

        // palette
        const palette = this._readPixels(paletteSize);

        let offset = 0;
        while (offset < tileSize) {
            let indexInPalette = this._inflator.inflate(1)[0];
            let length = 1;
            if (indexInPalette >= 128) {
                indexInPalette -= 128;
                length = this._readRLELength();
            }
            if (indexInPalette > paletteSize) {
                throw new Error('Too big index in palette: ' + indexInPalette + ', palette size: ' + paletteSize);
            }
            if (offset + length > tileSize) {
                throw new Error('Too big rle length in palette mode: ' + length + ', allowed length is: ' + (tileSize - offset));
            }

            for (let j = 0; j < length; j++) {
                data[offset * 4] = palette[indexInPalette * 4];
                data[offset * 4 + 1] = palette[indexInPalette * 4 + 1];
                data[offset * 4 + 2] = palette[indexInPalette * 4 + 2];
                data[offset * 4 + 3] = palette[indexInPalette * 4 + 3];
                offset++;
            }
        }
        return data;
    }

    _readRLELength() {
        let length = 0;
        let current = 0;
        do {
            current = this._inflator.inflate(1)[0];
            length += current;
        } while (current === 255);
        return length + 1;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === decoders/jpeg.js ===
(function() { // module decoders_jpeg
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */

class JPEGDecoder {
    constructor() {
        // RealVNC will reuse the quantization tables
        // and Huffman tables, so we need to cache them.
        this._cachedQuantTables = [];
        this._cachedHuffmanTables = [];

        this._segments = [];
    }

    decodeRect(x, y, width, height, sock, display, depth) {
        // A rect of JPEG encodings is simply a JPEG file
        while (true) {
            let segment = this._readSegment(sock);
            if (segment === null) {
                return false;
            }
            this._segments.push(segment);
            // End of image?
            if (segment[1] === 0xD9) {
                break;
            }
        }

        let huffmanTables = [];
        let quantTables = [];
        for (let segment of this._segments) {
            let type = segment[1];
            if (type === 0xC4) {
                // Huffman tables
                huffmanTables.push(segment);
            } else if (type === 0xDB) {
                // Quantization tables
                quantTables.push(segment);
            }
        }

        const sofIndex = this._segments.findIndex(
            x => x[1] == 0xC0 || x[1] == 0xC2
        );
        if (sofIndex == -1) {
            throw new Error("Illegal JPEG image without SOF");
        }

        if (quantTables.length === 0) {
            this._segments.splice(sofIndex+1, 0,
                                  ...this._cachedQuantTables);
        }
        if (huffmanTables.length === 0) {
            this._segments.splice(sofIndex+1, 0,
                                  ...this._cachedHuffmanTables);
        }

        let length = 0;
        for (let segment of this._segments) {
            length += segment.length;
        }

        let data = new Uint8Array(length);
        length = 0;
        for (let segment of this._segments) {
            data.set(segment, length);
            length += segment.length;
        }

        display.imageRect(x, y, width, height, "image/jpeg", data);

        if (huffmanTables.length !== 0) {
            this._cachedHuffmanTables = huffmanTables;
        }
        if (quantTables.length !== 0) {
            this._cachedQuantTables = quantTables;
        }

        this._segments = [];

        return true;
    }

    _readSegment(sock) {
        if (sock.rQwait("JPEG", 2)) {
            return null;
        }

        let marker = sock.rQshift8();
        if (marker != 0xFF) {
            throw new Error("Illegal JPEG marker received (byte: " +
                               marker + ")");
        }
        let type = sock.rQshift8();
        if (type >= 0xD0 && type <= 0xD9 || type == 0x01) {
            // No length after marker
            return new Uint8Array([marker, type]);
        }

        if (sock.rQwait("JPEG", 2, 2)) {
            return null;
        }

        let length = sock.rQshift16();
        if (length < 2) {
            throw new Error("Illegal JPEG length received (length: " +
                               length + ")");
        }

        if (sock.rQwait("JPEG", length-2, 4)) {
            return null;
        }

        let extra = 0;
        if (type === 0xDA) {
            // start of scan
            extra += 2;
            while (true) {
                if (sock.rQwait("JPEG", length-2+extra, 4)) {
                    return null;
                }
                let data = sock.rQpeekBytes(length-2+extra, false);
                if (data.at(-2) === 0xFF && data.at(-1) !== 0x00 &&
                    !(data.at(-1) >= 0xD0 && data.at(-1) <= 0xD7)) {
                    extra -= 2;
                    break;
                }
                extra++;
            }
        }

        let segment = new Uint8Array(2 + length + extra);
        segment[0] = marker;
        segment[1] = type;
        segment[2] = length >> 8;
        segment[3] = length;
        segment.set(sock.rQshiftBytes(length-2+extra, false), 4);

        return segment;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === base64.js ===
(function() { // module base64
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// From: http://hg.mozilla.org/mozilla-central/raw-file/ec10630b1a54/js/src/devtools/jint/sunspider/string-base64.js


{
    /* Convert data (an array of integers) to a Base64 string. */
    toBase64Table: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='.split(''),
    base64Pad: '=',

    encode(data) {
        "use strict";
        let result = '';
        const length = data.length;
        const lengthpad = (length % 3);
        // Convert every three bytes to 4 ascii characters.

        for (let i = 0; i < (length - 2); i += 3) {
            result += this.toBase64Table[data[i] >> 2];
            result += this.toBase64Table[((data[i] & 0x03) << 4) + (data[i + 1] >> 4)];
            result += this.toBase64Table[((data[i + 1] & 0x0f) << 2) + (data[i + 2] >> 6)];
            result += this.toBase64Table[data[i + 2] & 0x3f];
        }

        // Convert the remaining 1 or 2 bytes, pad out to 4 characters.
        const j = length - lengthpad;
        if (lengthpad === 2) {
            result += this.toBase64Table[data[j] >> 2];
            result += this.toBase64Table[((data[j] & 0x03) << 4) + (data[j + 1] >> 4)];
            result += this.toBase64Table[(data[j + 1] & 0x0f) << 2];
            result += this.toBase64Table[64];
        } else if (lengthpad === 1) {
            result += this.toBase64Table[data[j] >> 2];
            result += this.toBase64Table[(data[j] & 0x03) << 4];
            result += this.toBase64Table[64];
            result += this.toBase64Table[64];
        }

        return result;
    },

    /* Convert Base64 data to a string */
    /* eslint-disable comma-spacing */
    toBinaryTable: [
        -1,-1,-1,-1, -1,-1,-1,-1, -1,-1,-1,-1, -1,-1,-1,-1,
        -1,-1,-1,-1, -1,-1,-1,-1, -1,-1,-1,-1, -1,-1,-1,-1,
        -1,-1,-1,-1, -1,-1,-1,-1, -1,-1,-1,62, -1,-1,-1,63,
        52,53,54,55, 56,57,58,59, 60,61,-1,-1, -1, 0,-1,-1,
        -1, 0, 1, 2,  3, 4, 5, 6,  7, 8, 9,10, 11,12,13,14,
        15,16,17,18, 19,20,21,22, 23,24,25,-1, -1,-1,-1,-1,
        -1,26,27,28, 29,30,31,32, 33,34,35,36, 37,38,39,40,
        41,42,43,44, 45,46,47,48, 49,50,51,-1, -1,-1,-1,-1
    ],
    /* eslint-enable comma-spacing */

    decode(data, offset = 0) {
        let dataLength = data.indexOf('=') - offset;
        if (dataLength < 0) { dataLength = data.length - offset; }

        /* Every four characters is 3 resulting numbers */
        const resultLength = (dataLength >> 2) * 3 + Math.floor((dataLength % 4) / 1.5);
        const result = new Array(resultLength);

        // Convert one by one.

        let leftbits = 0; // number of bits decoded, but yet to be appended
        let leftdata = 0; // bits decoded, but yet to be appended
        for (let idx = 0, i = offset; i < data.length; i++) {
            const c = this.toBinaryTable[data.charCodeAt(i) & 0x7f];
            const padding = (data.charAt(i) === this.base64Pad);
            // Skip illegal characters and whitespace
            if (c === -1) {
                Log.Error("Illegal character code " + data.charCodeAt(i) + " at position " + i);
                continue;
            }

            // Collect data into leftdata, update bitcount
            leftdata = (leftdata << 6) | c;
            leftbits += 6;

            // If we have 8 or more bits, append 8 bits to the result
            if (leftbits >= 8) {
                leftbits -= 8;
                // Append if not padding.
                if (!padding) {
                    result[idx++] = (leftdata >> leftbits) & 0xff;
                }
                leftdata &= (1 << leftbits) - 1;
            }
        }

        // If there are any bits left, the base64 string was corrupted
        if (leftbits) {
            const err = new Error('Corrupted base64 string');
            err.name = 'Base64-Error';
            throw err;
        }

        return result;
    }
}; /* End of Base64 namespace */

}})();

// === inflator.js ===
(function() { // module inflator
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2020 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */


class Inflate {
    constructor() {
        this.strm = new ZStream();
        this.chunkSize = 1024 * 10 * 10;
        this.strm.output = new Uint8Array(this.chunkSize);

        inflateInit(this.strm);
    }

    setInput(data) {
        if (!data) {
            //FIXME: flush remaining data.
            /* eslint-disable camelcase */
            this.strm.input = null;
            this.strm.avail_in = 0;
            this.strm.next_in = 0;
        } else {
            this.strm.input = data;
            this.strm.avail_in = this.strm.input.length;
            this.strm.next_in = 0;
            /* eslint-enable camelcase */
        }
    }

    inflate(expected) {
        // resize our output buffer if it's too small
        // (we could just use multiple chunks, but that would cause an extra
        // allocation each time to flatten the chunks)
        if (expected > this.chunkSize) {
            this.chunkSize = expected;
            this.strm.output = new Uint8Array(this.chunkSize);
        }

        /* eslint-disable camelcase */
        this.strm.next_out = 0;
        this.strm.avail_out = expected;
        /* eslint-enable camelcase */

        let ret = inflate(this.strm, 0); // Flush argument not used.
        if (ret < 0) {
            throw new Error("zlib inflate failed");
        }

        if (this.strm.next_out != expected) {
            throw new Error("Incomplete zlib block");
        }

        return new Uint8Array(this.strm.output.buffer, 0, this.strm.next_out);
    }

    reset() {
        inflateReset(this.strm);
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === deflator.js ===
(function() { // module deflator
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2020 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */


class Deflator {
    constructor() {
        this.strm = new ZStream();
        this.chunkSize = 1024 * 10 * 10;
        this.outputBuffer = new Uint8Array(this.chunkSize);

        deflateInit(this.strm, Z_DEFAULT_COMPRESSION);
    }

    deflate(inData) {
        /* eslint-disable camelcase */
        this.strm.input = inData;
        this.strm.avail_in = this.strm.input.length;
        this.strm.next_in = 0;
        this.strm.output = this.outputBuffer;
        this.strm.avail_out = this.chunkSize;
        this.strm.next_out = 0;
        /* eslint-enable camelcase */

        let lastRet = deflate(this.strm, Z_FULL_FLUSH);
        let outData = new Uint8Array(this.strm.output.buffer, 0, this.strm.next_out);

        if (lastRet < 0) {
            throw new Error("zlib deflate failed");
        }

        if (this.strm.avail_in > 0) {
            // Read chunks until done

            let chunks = [outData];
            let totalLen = outData.length;
            do {
                /* eslint-disable camelcase */
                this.strm.output = new Uint8Array(this.chunkSize);
                this.strm.next_out = 0;
                this.strm.avail_out = this.chunkSize;
                /* eslint-enable camelcase */

                lastRet = deflate(this.strm, Z_FULL_FLUSH);

                if (lastRet < 0) {
                    throw new Error("zlib deflate failed");
                }

                let chunk = new Uint8Array(this.strm.output.buffer, 0, this.strm.next_out);
                totalLen += chunk.length;
                chunks.push(chunk);
            } while (this.strm.avail_in > 0);

            // Combine chunks into a single data

            let newData = new Uint8Array(totalLen);
            let offset = 0;

            for (let i = 0; i < chunks.length; i++) {
                newData.set(chunks[i], offset);
                offset += chunks[i].length;
            }

            outData = newData;
        }

        /* eslint-disable camelcase */
        this.strm.input = null;
        this.strm.avail_in = 0;
        this.strm.next_in = 0;
        /* eslint-enable camelcase */

        return outData;
    }

}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === websock.js ===
(function() { // module websock
/*
 * Websock: high-performance buffering wrapper
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * Websock is similar to the standard WebSocket / RTCDataChannel object
 * but with extra buffer handling.
 *
 * Websock has built-in receive queue buffering; the message event
 * does not contain actual data but is simply a notification that
 * there is new data available. Several rQ* methods are available to
 * read binary data off of the receive queue.
 */


// this has performance issues in some versions Chromium, and
// doesn't gain a tremendous amount of performance increase in Firefox
// at the moment.  It may be valuable to turn it on in the future.
const MAX_RQ_GROW_SIZE = 40 * 1024 * 1024;  // 40 MiB

// Constants pulled from RTCDataChannelState enum
// https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel/readyState#RTCDataChannelState_enum
const DataChannel = {
    CONNECTING: "connecting",
    OPEN: "open",
    CLOSING: "closing",
    CLOSED: "closed"
};

const ReadyStates = {
    CONNECTING: [WebSocket.CONNECTING, DataChannel.CONNECTING],
    OPEN: [WebSocket.OPEN, DataChannel.OPEN],
    CLOSING: [WebSocket.CLOSING, DataChannel.CLOSING],
    CLOSED: [WebSocket.CLOSED, DataChannel.CLOSED],
};

// Properties a raw channel must have, WebSocket and RTCDataChannel are two examples
const rawChannelProps = [
    "send",
    "close",
    "binaryType",
    "onerror",
    "onmessage",
    "onopen",
    "protocol",
    "readyState",
];

class Websock {
    constructor() {
        this._websocket = null;  // WebSocket or RTCDataChannel object

        this._rQi = 0;           // Receive queue index
        this._rQlen = 0;         // Next write position in the receive queue
        this._rQbufferSize = 1024 * 1024 * 4; // Receive queue buffer size (4 MiB)
        // called in init: this._rQ = new Uint8Array(this._rQbufferSize);
        this._rQ = null; // Receive queue

        this._sQbufferSize = 1024 * 10;  // 10 KiB
        // called in init: this._sQ = new Uint8Array(this._sQbufferSize);
        this._sQlen = 0;
        this._sQ = null;  // Send queue

        this._eventHandlers = {
            message: () => {},
            open: () => {},
            close: () => {},
            error: () => {}
        };
    }

    // Getters and Setters

    get readyState() {
        let subState;

        if (this._websocket === null) {
            return "unused";
        }

        subState = this._websocket.readyState;

        if (ReadyStates.CONNECTING.includes(subState)) {
            return "connecting";
        } else if (ReadyStates.OPEN.includes(subState)) {
            return "open";
        } else if (ReadyStates.CLOSING.includes(subState)) {
            return "closing";
        } else if (ReadyStates.CLOSED.includes(subState)) {
            return "closed";
        }

        return "unknown";
    }

    // Receive Queue
    rQpeek8() {
        return this._rQ[this._rQi];
    }

    rQskipBytes(bytes) {
        this._rQi += bytes;
    }

    rQshift8() {
        return this._rQshift(1);
    }

    rQshift16() {
        return this._rQshift(2);
    }

    rQshift32() {
        return this._rQshift(4);
    }

    // TODO(directxman12): test performance with these vs a DataView
    _rQshift(bytes) {
        let res = 0;
        for (let byte = bytes - 1; byte >= 0; byte--) {
            res += this._rQ[this._rQi++] << (byte * 8);
        }
        return res >>> 0;
    }

    rQshiftStr(len) {
        let str = "";
        // Handle large arrays in steps to avoid long strings on the stack
        for (let i = 0; i < len; i += 4096) {
            let part = this.rQshiftBytes(Math.min(4096, len - i), false);
            str += String.fromCharCode.apply(null, part);
        }
        return str;
    }

    rQshiftBytes(len, copy=true) {
        this._rQi += len;
        if (copy) {
            return this._rQ.slice(this._rQi - len, this._rQi);
        } else {
            return this._rQ.subarray(this._rQi - len, this._rQi);
        }
    }

    rQshiftTo(target, len) {
        // TODO: make this just use set with views when using a ArrayBuffer to store the rQ
        target.set(new Uint8Array(this._rQ.buffer, this._rQi, len));
        this._rQi += len;
    }

    rQpeekBytes(len, copy=true) {
        if (copy) {
            return this._rQ.slice(this._rQi, this._rQi + len);
        } else {
            return this._rQ.subarray(this._rQi, this._rQi + len);
        }
    }

    // Check to see if we must wait for 'num' bytes (default to FBU.bytes)
    // to be available in the receive queue. Return true if we need to
    // wait (and possibly print a debug message), otherwise false.
    rQwait(msg, num, goback) {
        if (this._rQlen - this._rQi < num) {
            if (goback) {
                if (this._rQi < goback) {
                    throw new Error("rQwait cannot backup " + goback + " bytes");
                }
                this._rQi -= goback;
            }
            return true; // true means need more data
        }
        return false;
    }

    // Send Queue

    sQpush8(num) {
        this._sQensureSpace(1);
        this._sQ[this._sQlen++] = num;
    }

    sQpush16(num) {
        this._sQensureSpace(2);
        this._sQ[this._sQlen++] = (num >> 8) & 0xff;
        this._sQ[this._sQlen++] = (num >> 0) & 0xff;
    }

    sQpush32(num) {
        this._sQensureSpace(4);
        this._sQ[this._sQlen++] = (num >> 24) & 0xff;
        this._sQ[this._sQlen++] = (num >> 16) & 0xff;
        this._sQ[this._sQlen++] = (num >>  8) & 0xff;
        this._sQ[this._sQlen++] = (num >>  0) & 0xff;
    }

    sQpushString(str) {
        let bytes = str.split('').map(chr => chr.charCodeAt(0));
        this.sQpushBytes(new Uint8Array(bytes));
    }

    sQpushBytes(bytes) {
        for (let offset = 0;offset < bytes.length;) {
            this._sQensureSpace(1);

            let chunkSize = this._sQbufferSize - this._sQlen;
            if (chunkSize > bytes.length - offset) {
                chunkSize = bytes.length - offset;
            }

            this._sQ.set(bytes.subarray(offset, chunkSize), this._sQlen);
            this._sQlen += chunkSize;
            offset += chunkSize;
        }
    }

    flush() {
        if (this._sQlen > 0 && this.readyState === 'open') {
            this._websocket.send(new Uint8Array(this._sQ.buffer, 0, this._sQlen));
            this._sQlen = 0;
        }
    }

    _sQensureSpace(bytes) {
        if (this._sQbufferSize - this._sQlen < bytes) {
            this.flush();
        }
    }

    // Event Handlers
    off(evt) {
        this._eventHandlers[evt] = () => {};
    }

    on(evt, handler) {
        this._eventHandlers[evt] = handler;
    }

    _allocateBuffers() {
        this._rQ = new Uint8Array(this._rQbufferSize);
        this._sQ = new Uint8Array(this._sQbufferSize);
    }

    init() {
        this._allocateBuffers();
        this._rQi = 0;
        this._websocket = null;
    }

    open(uri, protocols) {
        this.attach(new WebSocket(uri, protocols));
    }

    attach(rawChannel) {
        this.init();

        // Must get object and class methods to be compatible with the tests.
        const channelProps = [...Object.keys(rawChannel), ...Object.getOwnPropertyNames(Object.getPrototypeOf(rawChannel))];
        for (let i = 0; i < rawChannelProps.length; i++) {
            const prop = rawChannelProps[i];
            if (channelProps.indexOf(prop) < 0) {
                throw new Error('Raw channel missing property: ' + prop);
            }
        }

        this._websocket = rawChannel;
        this._websocket.binaryType = "arraybuffer";
        this._websocket.onmessage = this._recvMessage.bind(this);

        this._websocket.onopen = () => {
            Log.Debug('>> WebSock.onopen');
            if (this._websocket.protocol) {
                Log.Info("Server choose sub-protocol: " + this._websocket.protocol);
            }

            this._eventHandlers.open();
            Log.Debug("<< WebSock.onopen");
        };

        this._websocket.onclose = (e) => {
            Log.Debug(">> WebSock.onclose");
            this._eventHandlers.close(e);
            Log.Debug("<< WebSock.onclose");
        };

        this._websocket.onerror = (e) => {
            Log.Debug(">> WebSock.onerror: " + e);
            this._eventHandlers.error(e);
            Log.Debug("<< WebSock.onerror: " + e);
        };
    }

    close() {
        if (this._websocket) {
            if (this.readyState === 'connecting' ||
                this.readyState === 'open') {
                Log.Info("Closing WebSocket connection");
                this._websocket.close();
            }

            this._websocket.onmessage = () => {};
        }
    }

    // private methods

    // We want to move all the unread data to the start of the queue,
    // e.g. compacting.
    // The function also expands the receive que if needed, and for
    // performance reasons we combine these two actions to avoid
    // unnecessary copying.
    _expandCompactRQ(minFit) {
        // if we're using less than 1/8th of the buffer even with the incoming bytes, compact in place
        // instead of resizing
        const requiredBufferSize =  (this._rQlen - this._rQi + minFit) * 8;
        const resizeNeeded = this._rQbufferSize < requiredBufferSize;

        if (resizeNeeded) {
            // Make sure we always *at least* double the buffer size, and have at least space for 8x
            // the current amount of data
            this._rQbufferSize = Math.max(this._rQbufferSize * 2, requiredBufferSize);
        }

        // we don't want to grow unboundedly
        if (this._rQbufferSize > MAX_RQ_GROW_SIZE) {
            this._rQbufferSize = MAX_RQ_GROW_SIZE;
            if (this._rQbufferSize - (this._rQlen - this._rQi) < minFit) {
                throw new Error("Receive Queue buffer exceeded " + MAX_RQ_GROW_SIZE + " bytes, and the new message could not fit");
            }
        }

        if (resizeNeeded) {
            const oldRQbuffer = this._rQ.buffer;
            this._rQ = new Uint8Array(this._rQbufferSize);
            this._rQ.set(new Uint8Array(oldRQbuffer, this._rQi, this._rQlen - this._rQi));
        } else {
            this._rQ.copyWithin(0, this._rQi, this._rQlen);
        }

        this._rQlen = this._rQlen - this._rQi;
        this._rQi = 0;
    }

    // push arraybuffer values onto the end of the receive que
    _recvMessage(e) {
        if (this._rQlen == this._rQi) {
            // All data has now been processed, this means we
            // can reset the receive queue.
            this._rQlen = 0;
            this._rQi = 0;
        }
        const u8 = new Uint8Array(e.data);
        if (u8.length > this._rQbufferSize - this._rQlen) {
            this._expandCompactRQ(u8.length);
        }
        this._rQ.set(u8, this._rQlen);
        this._rQlen += u8.length;

        if (this._rQlen - this._rQi > 0) {
            this._eventHandlers.message();
        } else {
            Log.Debug("Ignoring empty message");
        }
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === display.js ===
(function() { // module display
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2019 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 */


class Display {
    constructor(target) {
        this._drawCtx = null;

        this._renderQ = [];  // queue drawing actions for in-oder rendering
        this._flushPromise = null;

        // the full frame buffer (logical canvas) size
        this._fbWidth = 0;
        this._fbHeight = 0;

        this._prevDrawStyle = "";

        Log.Debug(">> Display.constructor");

        // The visible canvas
        this._target = target;

        if (!this._target) {
            throw new Error("Target must be set");
        }

        if (typeof this._target === 'string') {
            throw new Error('target must be a DOM element');
        }

        if (!this._target.getContext) {
            throw new Error("no getContext method");
        }

        this._targetCtx = this._target.getContext('2d');

        // the visible canvas viewport (i.e. what actually gets seen)
        this._viewportLoc = { 'x': 0, 'y': 0, 'w': this._target.width, 'h': this._target.height };

        // The hidden canvas, where we do the actual rendering
        this._backbuffer = document.createElement('canvas');
        this._drawCtx = this._backbuffer.getContext('2d');

        this._damageBounds = { left: 0, top: 0,
                               right: this._backbuffer.width,
                               bottom: this._backbuffer.height };

        Log.Debug("User Agent: " + navigator.userAgent);

        Log.Debug("<< Display.constructor");

        // ===== PROPERTIES =====

        this._scale = 1.0;
        this._clipViewport = false;
    }

    // ===== PROPERTIES =====

    get scale() { return this._scale; }
    set scale(scale) {
        this._rescale(scale);
    }

    get clipViewport() { return this._clipViewport; }
    set clipViewport(viewport) {
        this._clipViewport = viewport;
        // May need to readjust the viewport dimensions
        const vp = this._viewportLoc;
        this.viewportChangeSize(vp.w, vp.h);
        this.viewportChangePos(0, 0);
    }

    get width() {
        return this._fbWidth;
    }

    get height() {
        return this._fbHeight;
    }

    // ===== PUBLIC METHODS =====

    viewportChangePos(deltaX, deltaY) {
        const vp = this._viewportLoc;
        deltaX = Math.floor(deltaX);
        deltaY = Math.floor(deltaY);

        if (!this._clipViewport) {
            deltaX = -vp.w;  // clamped later of out of bounds
            deltaY = -vp.h;
        }

        const vx2 = vp.x + vp.w - 1;
        const vy2 = vp.y + vp.h - 1;

        // Position change

        if (deltaX < 0 && vp.x + deltaX < 0) {
            deltaX = -vp.x;
        }
        if (vx2 + deltaX >= this._fbWidth) {
            deltaX -= vx2 + deltaX - this._fbWidth + 1;
        }

        if (vp.y + deltaY < 0) {
            deltaY = -vp.y;
        }
        if (vy2 + deltaY >= this._fbHeight) {
            deltaY -= (vy2 + deltaY - this._fbHeight + 1);
        }

        if (deltaX === 0 && deltaY === 0) {
            return;
        }
        Log.Debug("viewportChange deltaX: " + deltaX + ", deltaY: " + deltaY);

        vp.x += deltaX;
        vp.y += deltaY;

        this._damage(vp.x, vp.y, vp.w, vp.h);

        this.flip();
    }

    viewportChangeSize(width, height) {

        if (!this._clipViewport ||
            typeof(width) === "undefined" ||
            typeof(height) === "undefined") {

            Log.Debug("Setting viewport to full display region");
            width = this._fbWidth;
            height = this._fbHeight;
        }

        width = Math.floor(width);
        height = Math.floor(height);

        if (width > this._fbWidth) {
            width = this._fbWidth;
        }
        if (height > this._fbHeight) {
            height = this._fbHeight;
        }

        const vp = this._viewportLoc;
        if (vp.w !== width || vp.h !== height) {
            vp.w = width;
            vp.h = height;

            const canvas = this._target;
            canvas.width = width;
            canvas.height = height;

            // The position might need to be updated if we've grown
            this.viewportChangePos(0, 0);

            this._damage(vp.x, vp.y, vp.w, vp.h);
            this.flip();

            // Update the visible size of the target canvas
            this._rescale(this._scale);
        }
    }

    absX(x) {
        if (this._scale === 0) {
            return 0;
        }
        return toSigned32bit(x / this._scale + this._viewportLoc.x);
    }

    absY(y) {
        if (this._scale === 0) {
            return 0;
        }
        return toSigned32bit(y / this._scale + this._viewportLoc.y);
    }

    resize(width, height) {
        this._prevDrawStyle = "";

        this._fbWidth = width;
        this._fbHeight = height;

        const canvas = this._backbuffer;
        if (canvas.width !== width || canvas.height !== height) {

            // We have to save the canvas data since changing the size will clear it
            let saveImg = null;
            if (canvas.width > 0 && canvas.height > 0) {
                saveImg = this._drawCtx.getImageData(0, 0, canvas.width, canvas.height);
            }

            if (canvas.width !== width) {
                canvas.width = width;
            }
            if (canvas.height !== height) {
                canvas.height = height;
            }

            if (saveImg) {
                this._drawCtx.putImageData(saveImg, 0, 0);
            }
        }

        // Readjust the viewport as it may be incorrectly sized
        // and positioned
        const vp = this._viewportLoc;
        this.viewportChangeSize(vp.w, vp.h);
        this.viewportChangePos(0, 0);
    }

    getImageData() {
        return this._drawCtx.getImageData(0, 0, this.width, this.height);
    }

    toDataURL(type, encoderOptions) {
        return this._backbuffer.toDataURL(type, encoderOptions);
    }

    toBlob(callback, type, quality) {
        return this._backbuffer.toBlob(callback, type, quality);
    }

    // Track what parts of the visible canvas that need updating
    _damage(x, y, w, h) {
        if (x < this._damageBounds.left) {
            this._damageBounds.left = x;
        }
        if (y < this._damageBounds.top) {
            this._damageBounds.top = y;
        }
        if ((x + w) > this._damageBounds.right) {
            this._damageBounds.right = x + w;
        }
        if ((y + h) > this._damageBounds.bottom) {
            this._damageBounds.bottom = y + h;
        }
    }

    // Update the visible canvas with the contents of the
    // rendering canvas
    flip(fromQueue) {
        if (this._renderQ.length !== 0 && !fromQueue) {
            this._renderQPush({
                'type': 'flip'
            });
        } else {
            let x = this._damageBounds.left;
            let y = this._damageBounds.top;
            let w = this._damageBounds.right - x;
            let h = this._damageBounds.bottom - y;

            let vx = x - this._viewportLoc.x;
            let vy = y - this._viewportLoc.y;

            if (vx < 0) {
                w += vx;
                x -= vx;
                vx = 0;
            }
            if (vy < 0) {
                h += vy;
                y -= vy;
                vy = 0;
            }

            if ((vx + w) > this._viewportLoc.w) {
                w = this._viewportLoc.w - vx;
            }
            if ((vy + h) > this._viewportLoc.h) {
                h = this._viewportLoc.h - vy;
            }

            if ((w > 0) && (h > 0)) {
                // FIXME: We may need to disable image smoothing here
                //        as well (see copyImage()), but we haven't
                //        noticed any problem yet.
                this._targetCtx.drawImage(this._backbuffer,
                                          x, y, w, h,
                                          vx, vy, w, h);
            }

            this._damageBounds.left = this._damageBounds.top = 65535;
            this._damageBounds.right = this._damageBounds.bottom = 0;
        }
    }

    pending() {
        return this._renderQ.length > 0;
    }

    flush() {
        if (this._renderQ.length === 0) {
            return Promise.resolve();
        } else {
            if (this._flushPromise === null) {
                this._flushPromise = new Promise((resolve) => {
                    this._flushResolve = resolve;
                });
            }
            return this._flushPromise;
        }
    }

    fillRect(x, y, width, height, color, fromQueue) {
        if (this._renderQ.length !== 0 && !fromQueue) {
            this._renderQPush({
                'type': 'fill',
                'x': x,
                'y': y,
                'width': width,
                'height': height,
                'color': color
            });
        } else {
            this._setFillColor(color);
            this._drawCtx.fillRect(x, y, width, height);
            this._damage(x, y, width, height);
        }
    }

    copyImage(oldX, oldY, newX, newY, w, h, fromQueue) {
        if (this._renderQ.length !== 0 && !fromQueue) {
            this._renderQPush({
                'type': 'copy',
                'oldX': oldX,
                'oldY': oldY,
                'x': newX,
                'y': newY,
                'width': w,
                'height': h,
            });
        } else {
            // Due to this bug among others [1] we need to disable the image-smoothing to
            // avoid getting a blur effect when copying data.
            //
            // 1. https://bugzilla.mozilla.org/show_bug.cgi?id=1194719
            //
            // We need to set these every time since all properties are reset
            // when the the size is changed
            this._drawCtx.mozImageSmoothingEnabled = false;
            this._drawCtx.webkitImageSmoothingEnabled = false;
            this._drawCtx.msImageSmoothingEnabled = false;
            this._drawCtx.imageSmoothingEnabled = false;

            this._drawCtx.drawImage(this._backbuffer,
                                    oldX, oldY, w, h,
                                    newX, newY, w, h);
            this._damage(newX, newY, w, h);
        }
    }

    imageRect(x, y, width, height, mime, arr) {
        /* The internal logic cannot handle empty images, so bail early */
        if ((width === 0) || (height === 0)) {
            return;
        }

        const img = new Image();
        img.src = "data: " + mime + ";base64," + Base64.encode(arr);

        this._renderQPush({
            'type': 'img',
            'img': img,
            'x': x,
            'y': y,
            'width': width,
            'height': height
        });
    }

    blitImage(x, y, width, height, arr, offset, fromQueue) {
        if (this._renderQ.length !== 0 && !fromQueue) {
            // NB(directxman12): it's technically more performant here to use preallocated arrays,
            // but it's a lot of extra work for not a lot of payoff -- if we're using the render queue,
            // this probably isn't getting called *nearly* as much
            const newArr = new Uint8Array(width * height * 4);
            newArr.set(new Uint8Array(arr.buffer, 0, newArr.length));
            this._renderQPush({
                'type': 'blit',
                'data': newArr,
                'x': x,
                'y': y,
                'width': width,
                'height': height,
            });
        } else {
            // NB(directxman12): arr must be an Type Array view
            let data = new Uint8ClampedArray(arr.buffer,
                                             arr.byteOffset + offset,
                                             width * height * 4);
            let img = new ImageData(data, width, height);
            this._drawCtx.putImageData(img, x, y);
            this._damage(x, y, width, height);
        }
    }

    drawImage(img, x, y) {
        this._drawCtx.drawImage(img, x, y);
        this._damage(x, y, img.width, img.height);
    }

    autoscale(containerWidth, containerHeight) {
        let scaleRatio;

        if (containerWidth === 0 || containerHeight === 0) {
            scaleRatio = 0;

        } else {

            const vp = this._viewportLoc;
            const targetAspectRatio = containerWidth / containerHeight;
            const fbAspectRatio = vp.w / vp.h;

            if (fbAspectRatio >= targetAspectRatio) {
                scaleRatio = containerWidth / vp.w;
            } else {
                scaleRatio = containerHeight / vp.h;
            }
        }

        this._rescale(scaleRatio);
    }

    // ===== PRIVATE METHODS =====

    _rescale(factor) {
        this._scale = factor;
        const vp = this._viewportLoc;

        // NB(directxman12): If you set the width directly, or set the
        //                   style width to a number, the canvas is cleared.
        //                   However, if you set the style width to a string
        //                   ('NNNpx'), the canvas is scaled without clearing.
        const width = factor * vp.w + 'px';
        const height = factor * vp.h + 'px';

        if ((this._target.style.width !== width) ||
            (this._target.style.height !== height)) {
            this._target.style.width = width;
            this._target.style.height = height;
        }
    }

    _setFillColor(color) {
        const newStyle = 'rgb(' + color[0] + ',' + color[1] + ',' + color[2] + ')';
        if (newStyle !== this._prevDrawStyle) {
            this._drawCtx.fillStyle = newStyle;
            this._prevDrawStyle = newStyle;
        }
    }

    _renderQPush(action) {
        this._renderQ.push(action);
        if (this._renderQ.length === 1) {
            // If this can be rendered immediately it will be, otherwise
            // the scanner will wait for the relevant event
            this._scanRenderQ();
        }
    }

    _resumeRenderQ() {
        // "this" is the object that is ready, not the
        // display object
        this.removeEventListener('load', this._noVNCDisplay._resumeRenderQ);
        this._noVNCDisplay._scanRenderQ();
    }

    _scanRenderQ() {
        let ready = true;
        while (ready && this._renderQ.length > 0) {
            const a = this._renderQ[0];
            switch (a.type) {
                case 'flip':
                    this.flip(true);
                    break;
                case 'copy':
                    this.copyImage(a.oldX, a.oldY, a.x, a.y, a.width, a.height, true);
                    break;
                case 'fill':
                    this.fillRect(a.x, a.y, a.width, a.height, a.color, true);
                    break;
                case 'blit':
                    this.blitImage(a.x, a.y, a.width, a.height, a.data, 0, true);
                    break;
                case 'img':
                    if (a.img.complete) {
                        if (a.img.width !== a.width || a.img.height !== a.height) {
                            Log.Error("Decoded image has incorrect dimensions. Got " +
                                      a.img.width + "x" + a.img.height + ". Expected " +
                                      a.width + "x" + a.height + ".");
                            return;
                        }
                        this.drawImage(a.img, a.x, a.y);
                    } else {
                        a.img._noVNCDisplay = this;
                        a.img.addEventListener('load', this._resumeRenderQ);
                        // We need to wait for this image to 'load'
                        // to keep things in-order
                        ready = false;
                    }
                    break;
            }

            if (ready) {
                this._renderQ.shift();
            }
        }

        if (this._renderQ.length === 0 &&
            this._flushPromise !== null) {
            this._flushResolve();
            this._flushPromise = null;
            this._flushResolve = null;
        }
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === ra2.js ===
(function() { // module ra2

class RA2Cipher {
    constructor() {
        this._cipher = null;
        this._counter = new Uint8Array(16);
    }

    async setKey(key) {
        this._cipher = await legacyCrypto.importKey(
            "raw", key, { name: "AES-EAX" }, false, ["encrypt, decrypt"]);
    }

    async makeMessage(message) {
        const ad = new Uint8Array([(message.length & 0xff00) >>> 8, message.length & 0xff]);
        const encrypted = await legacyCrypto.encrypt({
            name: "AES-EAX",
            iv: this._counter,
            additionalData: ad,
        }, this._cipher, message);
        for (let i = 0; i < 16 && this._counter[i]++ === 255; i++);
        const res = new Uint8Array(message.length + 2 + 16);
        res.set(ad);
        res.set(encrypted, 2);
        return res;
    }

    async receiveMessage(length, encrypted) {
        const ad = new Uint8Array([(length & 0xff00) >>> 8, length & 0xff]);
        const res = await legacyCrypto.decrypt({
            name: "AES-EAX",
            iv: this._counter,
            additionalData: ad,
        }, this._cipher, encrypted);
        for (let i = 0; i < 16 && this._counter[i]++ === 255; i++);
        return res;
    }
}

class RSAAESAuthenticationState extends EventTargetMixin {
    constructor(sock, getCredentials) {
        super();
        this._hasStarted = false;
        this._checkSock = null;
        this._checkCredentials = null;
        this._approveServerResolve = null;
        this._sockReject = null;
        this._credentialsReject = null;
        this._approveServerReject = null;
        this._sock = sock;
        this._getCredentials = getCredentials;
    }

    _waitSockAsync(len) {
        return new Promise((resolve, reject) => {
            const hasData = () => !this._sock.rQwait('RA2', len);
            if (hasData()) {
                resolve();
            } else {
                this._checkSock = () => {
                    if (hasData()) {
                        resolve();
                        this._checkSock = null;
                        this._sockReject = null;
                    }
                };
                this._sockReject = reject;
            }
        });
    }

    _waitApproveKeyAsync() {
        return new Promise((resolve, reject) => {
            this._approveServerResolve = resolve;
            this._approveServerReject = reject;
        });
    }

    _waitCredentialsAsync(subtype) {
        const hasCredentials = () => {
            if (subtype === 1 && this._getCredentials().username !== undefined &&
                this._getCredentials().password !== undefined) {
                return true;
            } else if (subtype === 2 && this._getCredentials().password !== undefined) {
                return true;
            }
            return false;
        };
        return new Promise((resolve, reject) => {
            if (hasCredentials()) {
                resolve();
            } else {
                this._checkCredentials = () => {
                    if (hasCredentials()) {
                        resolve();
                        this._checkCredentials = null;
                        this._credentialsReject = null;
                    }
                };
                this._credentialsReject = reject;
            }
        });
    }

    checkInternalEvents() {
        if (this._checkSock !== null) {
            this._checkSock();
        }
        if (this._checkCredentials !== null) {
            this._checkCredentials();
        }
    }

    approveServer() {
        if (this._approveServerResolve !== null) {
            this._approveServerResolve();
            this._approveServerResolve = null;
        }
    }

    disconnect() {
        if (this._sockReject !== null) {
            this._sockReject(new Error("disconnect normally"));
            this._sockReject = null;
        }
        if (this._credentialsReject !== null) {
            this._credentialsReject(new Error("disconnect normally"));
            this._credentialsReject = null;
        }
        if (this._approveServerReject !== null) {
            this._approveServerReject(new Error("disconnect normally"));
            this._approveServerReject = null;
        }
    }

    async negotiateRA2neAuthAsync() {
        this._hasStarted = true;
        // 1: Receive server public key
        await this._waitSockAsync(4);
        const serverKeyLengthBuffer = this._sock.rQpeekBytes(4);
        const serverKeyLength = this._sock.rQshift32();
        if (serverKeyLength < 1024) {
            throw new Error("RA2: server public key is too short: " + serverKeyLength);
        } else if (serverKeyLength > 8192) {
            throw new Error("RA2: server public key is too long: " + serverKeyLength);
        }
        const serverKeyBytes = Math.ceil(serverKeyLength / 8);
        await this._waitSockAsync(serverKeyBytes * 2);
        const serverN = this._sock.rQshiftBytes(serverKeyBytes);
        const serverE = this._sock.rQshiftBytes(serverKeyBytes);
        const serverRSACipher = await legacyCrypto.importKey(
            "raw", { n: serverN, e: serverE }, { name: "RSA-PKCS1-v1_5" }, false, ["encrypt"]);
        const serverPublickey = new Uint8Array(4 + serverKeyBytes * 2);
        serverPublickey.set(serverKeyLengthBuffer);
        serverPublickey.set(serverN, 4);
        serverPublickey.set(serverE, 4 + serverKeyBytes);

        // verify server public key
        let approveKey = this._waitApproveKeyAsync();
        this.dispatchEvent(new CustomEvent("serververification", {
            detail: { type: "RSA", publickey: serverPublickey }
        }));
        await approveKey;

        // 2: Send client public key
        const clientKeyLength = 2048;
        const clientKeyBytes = Math.ceil(clientKeyLength / 8);
        const clientRSACipher = (await legacyCrypto.generateKey({
            name: "RSA-PKCS1-v1_5",
            modulusLength: clientKeyLength,
            publicExponent: new Uint8Array([1, 0, 1]),
        }, true, ["encrypt"])).privateKey;
        const clientExportedRSAKey = await legacyCrypto.exportKey("raw", clientRSACipher);
        const clientN = clientExportedRSAKey.n;
        const clientE = clientExportedRSAKey.e;
        const clientPublicKey = new Uint8Array(4 + clientKeyBytes * 2);
        clientPublicKey[0] = (clientKeyLength & 0xff000000) >>> 24;
        clientPublicKey[1] = (clientKeyLength & 0xff0000) >>> 16;
        clientPublicKey[2] = (clientKeyLength & 0xff00) >>> 8;
        clientPublicKey[3] = clientKeyLength & 0xff;
        clientPublicKey.set(clientN, 4);
        clientPublicKey.set(clientE, 4 + clientKeyBytes);
        this._sock.sQpushBytes(clientPublicKey);
        this._sock.flush();

        // 3: Send client random
        const clientRandom = new Uint8Array(16);
        window.crypto.getRandomValues(clientRandom);
        const clientEncryptedRandom = await legacyCrypto.encrypt(
            { name: "RSA-PKCS1-v1_5" }, serverRSACipher, clientRandom);
        const clientRandomMessage = new Uint8Array(2 + serverKeyBytes);
        clientRandomMessage[0] = (serverKeyBytes & 0xff00) >>> 8;
        clientRandomMessage[1] = serverKeyBytes & 0xff;
        clientRandomMessage.set(clientEncryptedRandom, 2);
        this._sock.sQpushBytes(clientRandomMessage);
        this._sock.flush();

        // 4: Receive server random
        await this._waitSockAsync(2);
        if (this._sock.rQshift16() !== clientKeyBytes) {
            throw new Error("RA2: wrong encrypted message length");
        }
        const serverEncryptedRandom = this._sock.rQshiftBytes(clientKeyBytes);
        const serverRandom = await legacyCrypto.decrypt(
            { name: "RSA-PKCS1-v1_5" }, clientRSACipher, serverEncryptedRandom);
        if (serverRandom === null || serverRandom.length !== 16) {
            throw new Error("RA2: corrupted server encrypted random");
        }

        // 5: Compute session keys and set ciphers
        let clientSessionKey = new Uint8Array(32);
        let serverSessionKey = new Uint8Array(32);
        clientSessionKey.set(serverRandom);
        clientSessionKey.set(clientRandom, 16);
        serverSessionKey.set(clientRandom);
        serverSessionKey.set(serverRandom, 16);
        clientSessionKey = await window.crypto.subtle.digest("SHA-1", clientSessionKey);
        clientSessionKey = new Uint8Array(clientSessionKey).slice(0, 16);
        serverSessionKey = await window.crypto.subtle.digest("SHA-1", serverSessionKey);
        serverSessionKey = new Uint8Array(serverSessionKey).slice(0, 16);
        const clientCipher = new RA2Cipher();
        await clientCipher.setKey(clientSessionKey);
        const serverCipher = new RA2Cipher();
        await serverCipher.setKey(serverSessionKey);

        // 6: Compute and exchange hashes
        let serverHash = new Uint8Array(8 + serverKeyBytes * 2 + clientKeyBytes * 2);
        let clientHash = new Uint8Array(8 + serverKeyBytes * 2 + clientKeyBytes * 2);
        serverHash.set(serverPublickey);
        serverHash.set(clientPublicKey, 4 + serverKeyBytes * 2);
        clientHash.set(clientPublicKey);
        clientHash.set(serverPublickey, 4 + clientKeyBytes * 2);
        serverHash = await window.crypto.subtle.digest("SHA-1", serverHash);
        clientHash = await window.crypto.subtle.digest("SHA-1", clientHash);
        serverHash = new Uint8Array(serverHash);
        clientHash = new Uint8Array(clientHash);
        this._sock.sQpushBytes(await clientCipher.makeMessage(clientHash));
        this._sock.flush();
        await this._waitSockAsync(2 + 20 + 16);
        if (this._sock.rQshift16() !== 20) {
            throw new Error("RA2: wrong server hash");
        }
        const serverHashReceived = await serverCipher.receiveMessage(
            20, this._sock.rQshiftBytes(20 + 16));
        if (serverHashReceived === null) {
            throw new Error("RA2: failed to authenticate the message");
        }
        for (let i = 0; i < 20; i++) {
            if (serverHashReceived[i] !== serverHash[i]) {
                throw new Error("RA2: wrong server hash");
            }
        }

        // 7: Receive subtype
        await this._waitSockAsync(2 + 1 + 16);
        if (this._sock.rQshift16() !== 1) {
            throw new Error("RA2: wrong subtype");
        }
        let subtype = (await serverCipher.receiveMessage(
            1, this._sock.rQshiftBytes(1 + 16)));
        if (subtype === null) {
            throw new Error("RA2: failed to authenticate the message");
        }
        subtype = subtype[0];
        let waitCredentials = this._waitCredentialsAsync(subtype);
        if (subtype === 1) {
            if (this._getCredentials().username === undefined ||
                this._getCredentials().password === undefined) {
                this.dispatchEvent(new CustomEvent(
                    "credentialsrequired",
                    { detail: { types: ["username", "password"] } }));
            }
        } else if (subtype === 2) {
            if (this._getCredentials().password === undefined) {
                this.dispatchEvent(new CustomEvent(
                    "credentialsrequired",
                    { detail: { types: ["password"] } }));
            }
        } else {
            throw new Error("RA2: wrong subtype");
        }
        await waitCredentials;
        let username;
        if (subtype === 1) {
            username = encodeUTF8(this._getCredentials().username).slice(0, 255);
        } else {
            username = "";
        }
        const password = encodeUTF8(this._getCredentials().password).slice(0, 255);
        const credentials = new Uint8Array(username.length + password.length + 2);
        credentials[0] = username.length;
        credentials[username.length + 1] = password.length;
        for (let i = 0; i < username.length; i++) {
            credentials[i + 1] = username.charCodeAt(i);
        }
        for (let i = 0; i < password.length; i++) {
            credentials[username.length + 2 + i] = password.charCodeAt(i);
        }
        this._sock.sQpushBytes(await clientCipher.makeMessage(credentials));
        this._sock.flush();
    }

    get hasStarted() {
        return this._hasStarted;
    }

    set hasStarted(s) {
        this._hasStarted = s;
    }
}

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// === crypto/crypto.js ===
(function() { // module crypto_crypto

// A single interface for the cryptographic algorithms not supported by SubtleCrypto.
// Both synchronous and asynchronous implmentations are allowed.
class LegacyCrypto {
    constructor() {
        this._algorithms = {
            "AES-ECB": AESECBCipher,
            "AES-EAX": AESEAXCipher,
            "DES-ECB": DESECBCipher,
            "DES-CBC": DESCBCCipher,
            "RSA-PKCS1-v1_5": RSACipher,
            "DH": DHCipher,
            "MD5": MD5,
        };
    }

    encrypt(algorithm, key, data) {
        if (key.algorithm.name !== algorithm.name) {
            throw new Error("algorithm does not match");
        }
        if (typeof key.encrypt !== "function") {
            throw new Error("key does not support encryption");
        }
        return key.encrypt(algorithm, data);
    }

    decrypt(algorithm, key, data) {
        if (key.algorithm.name !== algorithm.name) {
            throw new Error("algorithm does not match");
        }
        if (typeof key.decrypt !== "function") {
            throw new Error("key does not support encryption");
        }
        return key.decrypt(algorithm, data);
    }

    importKey(format, keyData, algorithm, extractable, keyUsages) {
        if (format !== "raw") {
            throw new Error("key format is not supported");
        }
        const alg = this._algorithms[algorithm.name];
        if (typeof alg === "undefined" || typeof alg.importKey !== "function") {
            throw new Error("algorithm is not supported");
        }
        return alg.importKey(keyData, algorithm, extractable, keyUsages);
    }

    generateKey(algorithm, extractable, keyUsages) {
        const alg = this._algorithms[algorithm.name];
        if (typeof alg === "undefined" || typeof alg.generateKey !== "function") {
            throw new Error("algorithm is not supported");
        }
        return alg.generateKey(algorithm, extractable, keyUsages);
    }

    exportKey(format, key) {
        if (format !== "raw") {
            throw new Error("key format is not supported");
        }
        if (typeof key.exportKey !== "function") {
            throw new Error("key does not support exportKey");
        }
        return key.exportKey();
    }

    digest(algorithm, data) {
        const alg = this._algorithms[algorithm];
        if (typeof alg !== "function") {
            throw new Error("algorithm is not supported");
        }
        return alg(data);
    }

    deriveBits(algorithm, key, length) {
        if (key.algorithm.name !== algorithm.name) {
            throw new Error("algorithm does not match");
        }
        if (typeof key.deriveBits !== "function") {
            throw new Error("key does not support deriveBits");
        }
        return key.deriveBits(algorithm, length);
    }
}

new LegacyCrypto;

window['new'] = typeof new !== 'undefined' ? new : window['new'];
}})();

// === rfb.js ===
(function() { // module rfb
/*
 * noVNC: HTML5 VNC client
 * Copyright (C) 2020 The noVNC Authors
 * Licensed under MPL 2.0 (see LICENSE.txt)
 *
 * See README.md for usage and integration instructions.
 *
 */



// How many seconds to wait for a disconnect to finish
const DISCONNECT_TIMEOUT = 3;
const DEFAULT_BACKGROUND = 'rgb(40, 40, 40)';

// Minimum wait (ms) between two mouse moves
const MOUSE_MOVE_DELAY = 17;

// Wheel thresholds
const WHEEL_STEP = 50; // Pixels needed for one step
const WHEEL_LINE_HEIGHT = 19; // Assumed pixels for one line step

// Gesture thresholds
const GESTURE_ZOOMSENS = 75;
const GESTURE_SCRLSENS = 50;
const DOUBLE_TAP_TIMEOUT = 1000;
const DOUBLE_TAP_THRESHOLD = 50;

// Security types
const securityTypeNone              = 1;
const securityTypeVNCAuth           = 2;
const securityTypeRA2ne             = 6;
const securityTypeTight             = 16;
const securityTypeVeNCrypt          = 19;
const securityTypeXVP               = 22;
const securityTypeARD               = 30;
const securityTypeMSLogonII         = 113;

// Special Tight security types
const securityTypeUnixLogon         = 129;

// VeNCrypt security types
const securityTypePlain             = 256;

// Extended clipboard pseudo-encoding formats
const extendedClipboardFormatText   = 1;
/*eslint-disable no-unused-vars */
const extendedClipboardFormatRtf    = 1 << 1;
const extendedClipboardFormatHtml   = 1 << 2;
const extendedClipboardFormatDib    = 1 << 3;
const extendedClipboardFormatFiles  = 1 << 4;
/*eslint-enable */

// Extended clipboard pseudo-encoding actions
const extendedClipboardActionCaps    = 1 << 24;
const extendedClipboardActionRequest = 1 << 25;
const extendedClipboardActionPeek    = 1 << 26;
const extendedClipboardActionNotify  = 1 << 27;
const extendedClipboardActionProvide = 1 << 28;

class RFB extends EventTargetMixin {
    constructor(target, urlOrChannel, options) {
        if (!target) {
            throw new Error("Must specify target");
        }
        if (!urlOrChannel) {
            throw new Error("Must specify URL, WebSocket or RTCDataChannel");
        }

        // We rely on modern APIs which might not be available in an
        // insecure context
        if (!window.isSecureContext) {
            Log.Error("noVNC requires a secure context (TLS). Expect crashes!");
        }

        super();

        this._target = target;

        if (typeof urlOrChannel === "string") {
            this._url = urlOrChannel;
        } else {
            this._url = null;
            this._rawChannel = urlOrChannel;
        }

        // Connection details
        options = options || {};
        this._rfbCredentials = options.credentials || {};
        this._shared = 'shared' in options ? !!options.shared : true;
        this._repeaterID = options.repeaterID || '';
        this._wsProtocols = options.wsProtocols || [];

        // Internal state
        this._rfbConnectionState = '';
        this._rfbInitState = '';
        this._rfbAuthScheme = -1;
        this._rfbCleanDisconnect = true;
        this._rfbRSAAESAuthenticationState = null;

        // Server capabilities
        this._rfbVersion = 0;
        this._rfbMaxVersion = 3.8;
        this._rfbTightVNC = false;
        this._rfbVeNCryptState = 0;
        this._rfbXvpVer = 0;

        this._fbWidth = 0;
        this._fbHeight = 0;

        this._fbName = "";

        this._capabilities = { power: false };

        this._supportsFence = false;

        this._supportsContinuousUpdates = false;
        this._enabledContinuousUpdates = false;

        this._supportsSetDesktopSize = false;
        this._screenID = 0;
        this._screenFlags = 0;

        this._qemuExtKeyEventSupported = false;

        this._clipboardText = null;
        this._clipboardServerCapabilitiesActions = {};
        this._clipboardServerCapabilitiesFormats = {};

        // Internal objects
        this._sock = null;              // Websock object
        this._display = null;           // Display object
        this._flushing = false;         // Display flushing state
        this._keyboard = null;          // Keyboard input handler object
        this._gestures = null;          // Gesture input handler object
        this._resizeObserver = null;    // Resize observer object

        // Timers
        this._disconnTimer = null;      // disconnection timer
        this._resizeTimeout = null;     // resize rate limiting
        this._mouseMoveTimer = null;

        // Decoder states
        this._decoders = {};

        this._FBU = {
            rects: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            encoding: null,
        };

        // Mouse state
        this._mousePos = {};
        this._mouseButtonMask = 0;
        this._mouseLastMoveTime = 0;
        this._viewportDragging = false;
        this._viewportDragPos = {};
        this._viewportHasMoved = false;
        this._accumulatedWheelDeltaX = 0;
        this._accumulatedWheelDeltaY = 0;

        // Gesture state
        this._gestureLastTapTime = null;
        this._gestureFirstDoubleTapEv = null;
        this._gestureLastMagnitudeX = 0;
        this._gestureLastMagnitudeY = 0;

        // Bound event handlers
        this._eventHandlers = {
            focusCanvas: this._focusCanvas.bind(this),
            handleResize: this._handleResize.bind(this),
            handleMouse: this._handleMouse.bind(this),
            handleWheel: this._handleWheel.bind(this),
            handleGesture: this._handleGesture.bind(this),
            handleRSAAESCredentialsRequired: this._handleRSAAESCredentialsRequired.bind(this),
            handleRSAAESServerVerification: this._handleRSAAESServerVerification.bind(this),
        };

        // main setup
        Log.Debug(">> RFB.constructor");

        // Create DOM elements
        this._screen = document.createElement('div');
        this._screen.style.display = 'flex';
        this._screen.style.width = '100%';
        this._screen.style.height = '100%';
        this._screen.style.overflow = 'auto';
        this._screen.style.background = DEFAULT_BACKGROUND;
        this._canvas = document.createElement('canvas');
        this._canvas.style.margin = 'auto';
        // Some browsers add an outline on focus
        this._canvas.style.outline = 'none';
        this._canvas.width = 0;
        this._canvas.height = 0;
        this._canvas.tabIndex = -1;
        this._screen.appendChild(this._canvas);

        // Cursor
        this._cursor = new Cursor();

        // XXX: TightVNC 2.8.11 sends no cursor at all until Windows changes
        // it. Result: no cursor at all until a window border or an edit field
        // is hit blindly. But there are also VNC servers that draw the cursor
        // in the framebuffer and don't send the empty local cursor. There is
        // no way to satisfy both sides.
        //
        // The spec is unclear on this "initial cursor" issue. Many other
        // viewers (TigerVNC, RealVNC, Remmina) display an arrow as the
        // initial cursor instead.
        this._cursorImage = RFB.cursors.none;

        // populate decoder array with objects
        this._decoders[encodings.encodingRaw] = new RawDecoder();
        this._decoders[encodings.encodingCopyRect] = new CopyRectDecoder();
        this._decoders[encodings.encodingRRE] = new RREDecoder();
        this._decoders[encodings.encodingHextile] = new HextileDecoder();
        this._decoders[encodings.encodingTight] = new TightDecoder();
        this._decoders[encodings.encodingTightPNG] = new TightPNGDecoder();
        this._decoders[encodings.encodingZRLE] = new ZRLEDecoder();
        this._decoders[encodings.encodingJPEG] = new JPEGDecoder();

        // NB: nothing that needs explicit teardown should be done
        // before this point, since this can throw an exception
        try {
            this._display = new Display(this._canvas);
        } catch (exc) {
            Log.Error("Display exception: " + exc);
            throw exc;
        }

        this._keyboard = new Keyboard(this._canvas);
        this._keyboard.onkeyevent = this._handleKeyEvent.bind(this);
        this._remoteCapsLock = null; // Null indicates unknown or irrelevant
        this._remoteNumLock = null;

        this._gestures = new GestureHandler();

        this._sock = new Websock();
        this._sock.on('open', this._socketOpen.bind(this));
        this._sock.on('close', this._socketClose.bind(this));
        this._sock.on('message', this._handleMessage.bind(this));
        this._sock.on('error', this._socketError.bind(this));

        this._expectedClientWidth = null;
        this._expectedClientHeight = null;
        this._resizeObserver = new ResizeObserver(this._eventHandlers.handleResize);

        // All prepared, kick off the connection
        this._updateConnectionState('connecting');

        Log.Debug("<< RFB.constructor");

        // ===== PROPERTIES =====

        this.dragViewport = false;
        this.focusOnClick = true;

        this._viewOnly = false;
        this._clipViewport = false;
        this._clippingViewport = false;
        this._scaleViewport = false;
        this._resizeSession = false;

        this._showDotCursor = false;
        if (options.showDotCursor !== undefined) {
            Log.Warn("Specifying showDotCursor as a RFB constructor argument is deprecated");
            this._showDotCursor = options.showDotCursor;
        }

        this._qualityLevel = 6;
        this._compressionLevel = 2;
    }

    // ===== PROPERTIES =====

    get viewOnly() { return this._viewOnly; }
    set viewOnly(viewOnly) {
        this._viewOnly = viewOnly;

        if (this._rfbConnectionState === "connecting" ||
            this._rfbConnectionState === "connected") {
            if (viewOnly) {
                this._keyboard.ungrab();
            } else {
                this._keyboard.grab();
            }
        }
    }

    get capabilities() { return this._capabilities; }

    get clippingViewport() { return this._clippingViewport; }
    _setClippingViewport(on) {
        if (on === this._clippingViewport) {
            return;
        }
        this._clippingViewport = on;
        this.dispatchEvent(new CustomEvent("clippingviewport",
                                           { detail: this._clippingViewport }));
    }

    get touchButton() { return 0; }
    set touchButton(button) { Log.Warn("Using old API!"); }

    get clipViewport() { return this._clipViewport; }
    set clipViewport(viewport) {
        this._clipViewport = viewport;
        this._updateClip();
    }

    get scaleViewport() { return this._scaleViewport; }
    set scaleViewport(scale) {
        this._scaleViewport = scale;
        // Scaling trumps clipping, so we may need to adjust
        // clipping when enabling or disabling scaling
        if (scale && this._clipViewport) {
            this._updateClip();
        }
        this._updateScale();
        if (!scale && this._clipViewport) {
            this._updateClip();
        }
    }

    get resizeSession() { return this._resizeSession; }
    set resizeSession(resize) {
        this._resizeSession = resize;
        if (resize) {
            this._requestRemoteResize();
        }
    }

    get showDotCursor() { return this._showDotCursor; }
    set showDotCursor(show) {
        this._showDotCursor = show;
        this._refreshCursor();
    }

    get background() { return this._screen.style.background; }
    set background(cssValue) { this._screen.style.background = cssValue; }

    get qualityLevel() {
        return this._qualityLevel;
    }
    set qualityLevel(qualityLevel) {
        if (!Number.isInteger(qualityLevel) || qualityLevel < 0 || qualityLevel > 9) {
            Log.Error("qualityLevel must be an integer between 0 and 9");
            return;
        }

        if (this._qualityLevel === qualityLevel) {
            return;
        }

        this._qualityLevel = qualityLevel;

        if (this._rfbConnectionState === 'connected') {
            this._sendEncodings();
        }
    }

    get compressionLevel() {
        return this._compressionLevel;
    }
    set compressionLevel(compressionLevel) {
        if (!Number.isInteger(compressionLevel) || compressionLevel < 0 || compressionLevel > 9) {
            Log.Error("compressionLevel must be an integer between 0 and 9");
            return;
        }

        if (this._compressionLevel === compressionLevel) {
            return;
        }

        this._compressionLevel = compressionLevel;

        if (this._rfbConnectionState === 'connected') {
            this._sendEncodings();
        }
    }

    // ===== PUBLIC METHODS =====

    disconnect() {
        this._updateConnectionState('disconnecting');
        this._sock.off('error');
        this._sock.off('message');
        this._sock.off('open');
        if (this._rfbRSAAESAuthenticationState !== null) {
            this._rfbRSAAESAuthenticationState.disconnect();
        }
    }

    approveServer() {
        if (this._rfbRSAAESAuthenticationState !== null) {
            this._rfbRSAAESAuthenticationState.approveServer();
        }
    }

    sendCredentials(creds) {
        this._rfbCredentials = creds;
        this._resumeAuthentication();
    }

    sendCtrlAltDel() {
        if (this._rfbConnectionState !== 'connected' || this._viewOnly) { return; }
        Log.Info("Sending Ctrl-Alt-Del");

        this.sendKey(KeyTable.XK_Control_L, "ControlLeft", true);
        this.sendKey(KeyTable.XK_Alt_L, "AltLeft", true);
        this.sendKey(KeyTable.XK_Delete, "Delete", true);
        this.sendKey(KeyTable.XK_Delete, "Delete", false);
        this.sendKey(KeyTable.XK_Alt_L, "AltLeft", false);
        this.sendKey(KeyTable.XK_Control_L, "ControlLeft", false);
    }

    machineShutdown() {
        this._xvpOp(1, 2);
    }

    machineReboot() {
        this._xvpOp(1, 3);
    }

    machineReset() {
        this._xvpOp(1, 4);
    }

    // Send a key press. If 'down' is not specified then send a down key
    // followed by an up key.
    sendKey(keysym, code, down) {
        if (this._rfbConnectionState !== 'connected' || this._viewOnly) { return; }

        if (down === undefined) {
            this.sendKey(keysym, code, true);
            this.sendKey(keysym, code, false);
            return;
        }

        const scancode = XtScancode[code];

        if (this._qemuExtKeyEventSupported && scancode) {
            // 0 is NoSymbol
            keysym = keysym || 0;

            Log.Info("Sending key (" + (down ? "down" : "up") + "): keysym " + keysym + ", scancode " + scancode);

            RFB.messages.QEMUExtendedKeyEvent(this._sock, keysym, down, scancode);
        } else {
            if (!keysym) {
                return;
            }
            Log.Info("Sending keysym (" + (down ? "down" : "up") + "): " + keysym);
            RFB.messages.keyEvent(this._sock, keysym, down ? 1 : 0);
        }
    }

    focus(options) {
        this._canvas.focus(options);
    }

    blur() {
        this._canvas.blur();
    }

    clipboardPasteFrom(text) {
        if (this._rfbConnectionState !== 'connected' || this._viewOnly) { return; }

        if (this._clipboardServerCapabilitiesFormats[extendedClipboardFormatText] &&
            this._clipboardServerCapabilitiesActions[extendedClipboardActionNotify]) {

            this._clipboardText = text;
            RFB.messages.extendedClipboardNotify(this._sock, [extendedClipboardFormatText]);
        } else {
            let length, i;
            let data;

            length = 0;
            // eslint-disable-next-line no-unused-vars
            for (let codePoint of text) {
                length++;
            }

            data = new Uint8Array(length);

            i = 0;
            for (let codePoint of text) {
                let code = codePoint.codePointAt(0);

                /* Only ISO 8859-1 is supported */
                if (code > 0xff) {
                    code = 0x3f; // '?'
                }

                data[i++] = code;
            }

            RFB.messages.clientCutText(this._sock, data);
        }
    }

    getImageData() {
        return this._display.getImageData();
    }

    toDataURL(type, encoderOptions) {
        return this._display.toDataURL(type, encoderOptions);
    }

    toBlob(callback, type, quality) {
        return this._display.toBlob(callback, type, quality);
    }

    // ===== PRIVATE METHODS =====

    _connect() {
        Log.Debug(">> RFB.connect");

        if (this._url) {
            Log.Info(`connecting to ${this._url}`);
            this._sock.open(this._url, this._wsProtocols);
        } else {
            Log.Info(`attaching ${this._rawChannel} to Websock`);
            this._sock.attach(this._rawChannel);

            if (this._sock.readyState === 'closed') {
                throw Error("Cannot use already closed WebSocket/RTCDataChannel");
            }

            if (this._sock.readyState === 'open') {
                // FIXME: _socketOpen() can in theory call _fail(), which
                //        isn't allowed this early, but I'm not sure that can
                //        happen without a bug messing up our state variables
                this._socketOpen();
            }
        }

        // Make our elements part of the page
        this._target.appendChild(this._screen);

        this._gestures.attach(this._canvas);

        this._cursor.attach(this._canvas);
        this._refreshCursor();

        // Monitor size changes of the screen element
        this._resizeObserver.observe(this._screen);

        // Always grab focus on some kind of click event
        this._canvas.addEventListener("mousedown", this._eventHandlers.focusCanvas);
        this._canvas.addEventListener("touchstart", this._eventHandlers.focusCanvas);

        // Mouse events
        this._canvas.addEventListener('mousedown', this._eventHandlers.handleMouse);
        this._canvas.addEventListener('mouseup', this._eventHandlers.handleMouse);
        this._canvas.addEventListener('mousemove', this._eventHandlers.handleMouse);
        // Prevent middle-click pasting (see handler for why we bind to document)
        this._canvas.addEventListener('click', this._eventHandlers.handleMouse);
        // preventDefault() on mousedown doesn't stop this event for some
        // reason so we have to explicitly block it
        this._canvas.addEventListener('contextmenu', this._eventHandlers.handleMouse);

        // Wheel events
        this._canvas.addEventListener("wheel", this._eventHandlers.handleWheel);

        // Gesture events
        this._canvas.addEventListener("gesturestart", this._eventHandlers.handleGesture);
        this._canvas.addEventListener("gesturemove", this._eventHandlers.handleGesture);
        this._canvas.addEventListener("gestureend", this._eventHandlers.handleGesture);

        Log.Debug("<< RFB.connect");
    }

    _disconnect() {
        Log.Debug(">> RFB.disconnect");
        this._cursor.detach();
        this._canvas.removeEventListener("gesturestart", this._eventHandlers.handleGesture);
        this._canvas.removeEventListener("gesturemove", this._eventHandlers.handleGesture);
        this._canvas.removeEventListener("gestureend", this._eventHandlers.handleGesture);
        this._canvas.removeEventListener("wheel", this._eventHandlers.handleWheel);
        this._canvas.removeEventListener('mousedown', this._eventHandlers.handleMouse);
        this._canvas.removeEventListener('mouseup', this._eventHandlers.handleMouse);
        this._canvas.removeEventListener('mousemove', this._eventHandlers.handleMouse);
        this._canvas.removeEventListener('click', this._eventHandlers.handleMouse);
        this._canvas.removeEventListener('contextmenu', this._eventHandlers.handleMouse);
        this._canvas.removeEventListener("mousedown", this._eventHandlers.focusCanvas);
        this._canvas.removeEventListener("touchstart", this._eventHandlers.focusCanvas);
        this._resizeObserver.disconnect();
        this._keyboard.ungrab();
        this._gestures.detach();
        this._sock.close();
        try {
            this._target.removeChild(this._screen);
        } catch (e) {
            if (e.name === 'NotFoundError') {
                // Some cases where the initial connection fails
                // can disconnect before the _screen is created
            } else {
                throw e;
            }
        }
        clearTimeout(this._resizeTimeout);
        clearTimeout(this._mouseMoveTimer);
        Log.Debug("<< RFB.disconnect");
    }

    _socketOpen() {
        if ((this._rfbConnectionState === 'connecting') &&
            (this._rfbInitState === '')) {
            this._rfbInitState = 'ProtocolVersion';
            Log.Debug("Starting VNC handshake");
        } else {
            this._fail("Unexpected server connection while " +
                       this._rfbConnectionState);
        }
    }

    _socketClose(e) {
        Log.Debug("WebSocket on-close event");
        let msg = "";
        if (e.code) {
            msg = "(code: " + e.code;
            if (e.reason) {
                msg += ", reason: " + e.reason;
            }
            msg += ")";
        }
        switch (this._rfbConnectionState) {
            case 'connecting':
                this._fail("Connection closed " + msg);
                break;
            case 'connected':
                // Handle disconnects that were initiated server-side
                this._updateConnectionState('disconnecting');
                this._updateConnectionState('disconnected');
                break;
            case 'disconnecting':
                // Normal disconnection path
                this._updateConnectionState('disconnected');
                break;
            case 'disconnected':
                this._fail("Unexpected server disconnect " +
                           "when already disconnected " + msg);
                break;
            default:
                this._fail("Unexpected server disconnect before connecting " +
                           msg);
                break;
        }
        this._sock.off('close');
        // Delete reference to raw channel to allow cleanup.
        this._rawChannel = null;
    }

    _socketError(e) {
        Log.Warn("WebSocket on-error event");
    }

    _focusCanvas(event) {
        if (!this.focusOnClick) {
            return;
        }

        this.focus({ preventScroll: true });
    }

    _setDesktopName(name) {
        this._fbName = name;
        this.dispatchEvent(new CustomEvent(
            "desktopname",
            { detail: { name: this._fbName } }));
    }

    _saveExpectedClientSize() {
        this._expectedClientWidth = this._screen.clientWidth;
        this._expectedClientHeight = this._screen.clientHeight;
    }

    _currentClientSize() {
        return [this._screen.clientWidth, this._screen.clientHeight];
    }

    _clientHasExpectedSize() {
        const [currentWidth, currentHeight] = this._currentClientSize();
        return currentWidth == this._expectedClientWidth &&
            currentHeight == this._expectedClientHeight;
    }

    _handleResize() {
        // Don't change anything if the client size is already as expected
        if (this._clientHasExpectedSize()) {
            return;
        }
        // If the window resized then our screen element might have
        // as well. Update the viewport dimensions.
        window.requestAnimationFrame(() => {
            this._updateClip();
            this._updateScale();
        });

        if (this._resizeSession) {
            // Request changing the resolution of the remote display to
            // the size of the local browser viewport.

            // In order to not send multiple requests before the browser-resize
            // is finished we wait 0.5 seconds before sending the request.
            clearTimeout(this._resizeTimeout);
            this._resizeTimeout = setTimeout(this._requestRemoteResize.bind(this), 500);
        }
    }

    // Update state of clipping in Display object, and make sure the
    // configured viewport matches the current screen size
    _updateClip() {
        const curClip = this._display.clipViewport;
        let newClip = this._clipViewport;

        if (this._scaleViewport) {
            // Disable viewport clipping if we are scaling
            newClip = false;
        }

        if (curClip !== newClip) {
            this._display.clipViewport = newClip;
        }

        if (newClip) {
            // When clipping is enabled, the screen is limited to
            // the size of the container.
            const size = this._screenSize();
            this._display.viewportChangeSize(size.w, size.h);
            this._fixScrollbars();
            this._setClippingViewport(size.w < this._display.width ||
                                      size.h < this._display.height);
        } else {
            this._setClippingViewport(false);
        }

        // When changing clipping we might show or hide scrollbars.
        // This causes the expected client dimensions to change.
        if (curClip !== newClip) {
            this._saveExpectedClientSize();
        }
    }

    _updateScale() {
        if (!this._scaleViewport) {
            this._display.scale = 1.0;
        } else {
            const size = this._screenSize();
            this._display.autoscale(size.w, size.h);
        }
        this._fixScrollbars();
    }

    // Requests a change of remote desktop size. This message is an extension
    // and may only be sent if we have received an ExtendedDesktopSize message
    _requestRemoteResize() {
        clearTimeout(this._resizeTimeout);
        this._resizeTimeout = null;

        if (!this._resizeSession || this._viewOnly ||
            !this._supportsSetDesktopSize) {
            return;
        }

        const size = this._screenSize();

        RFB.messages.setDesktopSize(this._sock,
                                    Math.floor(size.w), Math.floor(size.h),
                                    this._screenID, this._screenFlags);

        Log.Debug('Requested new desktop size: ' +
                   size.w + 'x' + size.h);
    }

    // Gets the the size of the available screen
    _screenSize() {
        let r = this._screen.getBoundingClientRect();
        return { w: r.width, h: r.height };
    }

    _fixScrollbars() {
        // This is a hack because Safari on macOS screws up the calculation
        // for when scrollbars are needed. We get scrollbars when making the
        // browser smaller, despite remote resize being enabled. So to fix it
        // we temporarily toggle them off and on.
        const orig = this._screen.style.overflow;
        this._screen.style.overflow = 'hidden';
        // Force Safari to recalculate the layout by asking for
        // an element's dimensions
        this._screen.getBoundingClientRect();
        this._screen.style.overflow = orig;
    }

    /*
     * Connection states:
     *   connecting
     *   connected
     *   disconnecting
     *   disconnected - permanent state
     */
    _updateConnectionState(state) {
        const oldstate = this._rfbConnectionState;

        if (state === oldstate) {
            Log.Debug("Already in state '" + state + "', ignoring");
            return;
        }

        // The 'disconnected' state is permanent for each RFB object
        if (oldstate === 'disconnected') {
            Log.Error("Tried changing state of a disconnected RFB object");
            return;
        }

        // Ensure proper transitions before doing anything
        switch (state) {
            case 'connected':
                if (oldstate !== 'connecting') {
                    Log.Error("Bad transition to connected state, " +
                               "previous connection state: " + oldstate);
                    return;
                }
                break;

            case 'disconnected':
                if (oldstate !== 'disconnecting') {
                    Log.Error("Bad transition to disconnected state, " +
                               "previous connection state: " + oldstate);
                    return;
                }
                break;

            case 'connecting':
                if (oldstate !== '') {
                    Log.Error("Bad transition to connecting state, " +
                               "previous connection state: " + oldstate);
                    return;
                }
                break;

            case 'disconnecting':
                if (oldstate !== 'connected' && oldstate !== 'connecting') {
                    Log.Error("Bad transition to disconnecting state, " +
                               "previous connection state: " + oldstate);
                    return;
                }
                break;

            default:
                Log.Error("Unknown connection state: " + state);
                return;
        }

        // State change actions

        this._rfbConnectionState = state;

        Log.Debug("New state '" + state + "', was '" + oldstate + "'.");

        if (this._disconnTimer && state !== 'disconnecting') {
            Log.Debug("Clearing disconnect timer");
            clearTimeout(this._disconnTimer);
            this._disconnTimer = null;

            // make sure we don't get a double event
            this._sock.off('close');
        }

        switch (state) {
            case 'connecting':
                this._connect();
                break;

            case 'connected':
                this.dispatchEvent(new CustomEvent("connect", { detail: {} }));
                break;

            case 'disconnecting':
                this._disconnect();

                this._disconnTimer = setTimeout(() => {
                    Log.Error("Disconnection timed out.");
                    this._updateConnectionState('disconnected');
                }, DISCONNECT_TIMEOUT * 1000);
                break;

            case 'disconnected':
                this.dispatchEvent(new CustomEvent(
                    "disconnect", { detail:
                                    { clean: this._rfbCleanDisconnect } }));
                break;
        }
    }

    /* Print errors and disconnect
     *
     * The parameter 'details' is used for information that
     * should be logged but not sent to the user interface.
     */
    _fail(details) {
        switch (this._rfbConnectionState) {
            case 'disconnecting':
                Log.Error("Failed when disconnecting: " + details);
                break;
            case 'connected':
                Log.Error("Failed while connected: " + details);
                break;
            case 'connecting':
                Log.Error("Failed when connecting: " + details);
                break;
            default:
                Log.Error("RFB failure: " + details);
                break;
        }
        this._rfbCleanDisconnect = false; //This is sent to the UI

        // Transition to disconnected without waiting for socket to close
        this._updateConnectionState('disconnecting');
        this._updateConnectionState('disconnected');

        return false;
    }

    _setCapability(cap, val) {
        this._capabilities[cap] = val;
        this.dispatchEvent(new CustomEvent("capabilities",
                                           { detail: { capabilities: this._capabilities } }));
    }

    _handleMessage() {
        if (this._sock.rQwait("message", 1)) {
            Log.Warn("handleMessage called on an empty receive queue");
            return;
        }

        switch (this._rfbConnectionState) {
            case 'disconnected':
                Log.Error("Got data while disconnected");
                break;
            case 'connected':
                while (true) {
                    if (this._flushing) {
                        break;
                    }
                    if (!this._normalMsg()) {
                        break;
                    }
                    if (this._sock.rQwait("message", 1)) {
                        break;
                    }
                }
                break;
            case 'connecting':
                while (this._rfbConnectionState === 'connecting') {
                    if (!this._initMsg()) {
                        break;
                    }
                }
                break;
            default:
                Log.Error("Got data while in an invalid state");
                break;
        }
    }

    _handleKeyEvent(keysym, code, down, numlock, capslock) {
        // If remote state of capslock is known, and it doesn't match the local led state of
        // the keyboard, we send a capslock keypress first to bring it into sync.
        // If we just pressed CapsLock, or we toggled it remotely due to it being out of sync
        // we clear the remote state so that we don't send duplicate or spurious fixes,
        // since it may take some time to receive the new remote CapsLock state.
        if (code == 'CapsLock' && down) {
            this._remoteCapsLock = null;
        }
        if (this._remoteCapsLock !== null && capslock !== null && this._remoteCapsLock !== capslock && down) {
            Log.Debug("Fixing remote caps lock");

            this.sendKey(KeyTable.XK_Caps_Lock, 'CapsLock', true);
            this.sendKey(KeyTable.XK_Caps_Lock, 'CapsLock', false);
            // We clear the remote capsLock state when we do this to prevent issues with doing this twice
            // before we receive an update of the the remote state.
            this._remoteCapsLock = null;
        }

        // Logic for numlock is exactly the same.
        if (code == 'NumLock' && down) {
            this._remoteNumLock = null;
        }
        if (this._remoteNumLock !== null && numlock !== null && this._remoteNumLock !== numlock && down) {
            Log.Debug("Fixing remote num lock");
            this.sendKey(KeyTable.XK_Num_Lock, 'NumLock', true);
            this.sendKey(KeyTable.XK_Num_Lock, 'NumLock', false);
            this._remoteNumLock = null;
        }
        this.sendKey(keysym, code, down);
    }

    _handleMouse(ev) {
        /*
         * We don't check connection status or viewOnly here as the
         * mouse events might be used to control the viewport
         */

        if (ev.type === 'click') {
            /*
             * Note: This is only needed for the 'click' event as it fails
             *       to fire properly for the target element so we have
             *       to listen on the document element instead.
             */
            if (ev.target !== this._canvas) {
                return;
            }
        }

        // FIXME: if we're in view-only and not dragging,
        //        should we stop events?
        ev.stopPropagation();
        ev.preventDefault();

        if ((ev.type === 'click') || (ev.type === 'contextmenu')) {
            return;
        }

        let pos = clientToElement(ev.clientX, ev.clientY,
                                  this._canvas);

        switch (ev.type) {
            case 'mousedown':
                setCapture(this._canvas);
                this._handleMouseButton(pos.x, pos.y,
                                        true, 1 << ev.button);
                break;
            case 'mouseup':
                this._handleMouseButton(pos.x, pos.y,
                                        false, 1 << ev.button);
                break;
            case 'mousemove':
                this._handleMouseMove(pos.x, pos.y);
                break;
        }
    }

    _handleMouseButton(x, y, down, bmask) {
        if (this.dragViewport) {
            if (down && !this._viewportDragging) {
                this._viewportDragging = true;
                this._viewportDragPos = {'x': x, 'y': y};
                this._viewportHasMoved = false;

                // Skip sending mouse events
                return;
            } else {
                this._viewportDragging = false;

                // If we actually performed a drag then we are done
                // here and should not send any mouse events
                if (this._viewportHasMoved) {
                    return;
                }

                // Otherwise we treat this as a mouse click event.
                // Send the button down event here, as the button up
                // event is sent at the end of this function.
                this._sendMouse(x, y, bmask);
            }
        }

        // Flush waiting move event first
        if (this._mouseMoveTimer !== null) {
            clearTimeout(this._mouseMoveTimer);
            this._mouseMoveTimer = null;
            this._sendMouse(x, y, this._mouseButtonMask);
        }

        if (down) {
            this._mouseButtonMask |= bmask;
        } else {
            this._mouseButtonMask &= ~bmask;
        }

        this._sendMouse(x, y, this._mouseButtonMask);
    }

    _handleMouseMove(x, y) {
        if (this._viewportDragging) {
            const deltaX = this._viewportDragPos.x - x;
            const deltaY = this._viewportDragPos.y - y;

            if (this._viewportHasMoved || (Math.abs(deltaX) > dragThreshold ||
                                           Math.abs(deltaY) > dragThreshold)) {
                this._viewportHasMoved = true;

                this._viewportDragPos = {'x': x, 'y': y};
                this._display.viewportChangePos(deltaX, deltaY);
            }

            // Skip sending mouse events
            return;
        }

        this._mousePos = { 'x': x, 'y': y };

        // Limit many mouse move events to one every MOUSE_MOVE_DELAY ms
        if (this._mouseMoveTimer == null) {

            const timeSinceLastMove = Date.now() - this._mouseLastMoveTime;
            if (timeSinceLastMove > MOUSE_MOVE_DELAY) {
                this._sendMouse(x, y, this._mouseButtonMask);
                this._mouseLastMoveTime = Date.now();
            } else {
                // Too soon since the latest move, wait the remaining time
                this._mouseMoveTimer = setTimeout(() => {
                    this._handleDelayedMouseMove();
                }, MOUSE_MOVE_DELAY - timeSinceLastMove);
            }
        }
    }

    _handleDelayedMouseMove() {
        this._mouseMoveTimer = null;
        this._sendMouse(this._mousePos.x, this._mousePos.y,
                        this._mouseButtonMask);
        this._mouseLastMoveTime = Date.now();
    }

    _sendMouse(x, y, mask) {
        if (this._rfbConnectionState !== 'connected') { return; }
        if (this._viewOnly) { return; } // View only, skip mouse events

        RFB.messages.pointerEvent(this._sock, this._display.absX(x),
                                  this._display.absY(y), mask);
    }

    _handleWheel(ev) {
        if (this._rfbConnectionState !== 'connected') { return; }
        if (this._viewOnly) { return; } // View only, skip mouse events

        ev.stopPropagation();
        ev.preventDefault();

        let pos = clientToElement(ev.clientX, ev.clientY,
                                  this._canvas);

        let dX = ev.deltaX;
        let dY = ev.deltaY;

        // Pixel units unless it's non-zero.
        // Note that if deltamode is line or page won't matter since we aren't
        // sending the mouse wheel delta to the server anyway.
        // The difference between pixel and line can be important however since
        // we have a threshold that can be smaller than the line height.
        if (ev.deltaMode !== 0) {
            dX *= WHEEL_LINE_HEIGHT;
            dY *= WHEEL_LINE_HEIGHT;
        }

        // Mouse wheel events are sent in steps over VNC. This means that the VNC
        // protocol can't handle a wheel event with specific distance or speed.
        // Therefor, if we get a lot of small mouse wheel events we combine them.
        this._accumulatedWheelDeltaX += dX;
        this._accumulatedWheelDeltaY += dY;

        // Generate a mouse wheel step event when the accumulated delta
        // for one of the axes is large enough.
        if (Math.abs(this._accumulatedWheelDeltaX) >= WHEEL_STEP) {
            if (this._accumulatedWheelDeltaX < 0) {
                this._handleMouseButton(pos.x, pos.y, true, 1 << 5);
                this._handleMouseButton(pos.x, pos.y, false, 1 << 5);
            } else if (this._accumulatedWheelDeltaX > 0) {
                this._handleMouseButton(pos.x, pos.y, true, 1 << 6);
                this._handleMouseButton(pos.x, pos.y, false, 1 << 6);
            }

            this._accumulatedWheelDeltaX = 0;
        }
        if (Math.abs(this._accumulatedWheelDeltaY) >= WHEEL_STEP) {
            if (this._accumulatedWheelDeltaY < 0) {
                this._handleMouseButton(pos.x, pos.y, true, 1 << 3);
                this._handleMouseButton(pos.x, pos.y, false, 1 << 3);
            } else if (this._accumulatedWheelDeltaY > 0) {
                this._handleMouseButton(pos.x, pos.y, true, 1 << 4);
                this._handleMouseButton(pos.x, pos.y, false, 1 << 4);
            }

            this._accumulatedWheelDeltaY = 0;
        }
    }

    _fakeMouseMove(ev, elementX, elementY) {
        this._handleMouseMove(elementX, elementY);
        this._cursor.move(ev.detail.clientX, ev.detail.clientY);
    }

    _handleTapEvent(ev, bmask) {
        let pos = clientToElement(ev.detail.clientX, ev.detail.clientY,
                                  this._canvas);

        // If the user quickly taps multiple times we assume they meant to
        // hit the same spot, so slightly adjust coordinates

        if ((this._gestureLastTapTime !== null) &&
            ((Date.now() - this._gestureLastTapTime) < DOUBLE_TAP_TIMEOUT) &&
            (this._gestureFirstDoubleTapEv.detail.type === ev.detail.type)) {
            let dx = this._gestureFirstDoubleTapEv.detail.clientX - ev.detail.clientX;
            let dy = this._gestureFirstDoubleTapEv.detail.clientY - ev.detail.clientY;
            let distance = Math.hypot(dx, dy);

            if (distance < DOUBLE_TAP_THRESHOLD) {
                pos = clientToElement(this._gestureFirstDoubleTapEv.detail.clientX,
                                      this._gestureFirstDoubleTapEv.detail.clientY,
                                      this._canvas);
            } else {
                this._gestureFirstDoubleTapEv = ev;
            }
        } else {
            this._gestureFirstDoubleTapEv = ev;
        }
        this._gestureLastTapTime = Date.now();

        this._fakeMouseMove(this._gestureFirstDoubleTapEv, pos.x, pos.y);
        this._handleMouseButton(pos.x, pos.y, true, bmask);
        this._handleMouseButton(pos.x, pos.y, false, bmask);
    }

    _handleGesture(ev) {
        let magnitude;

        let pos = clientToElement(ev.detail.clientX, ev.detail.clientY,
                                  this._canvas);
        switch (ev.type) {
            case 'gesturestart':
                switch (ev.detail.type) {
                    case 'onetap':
                        this._handleTapEvent(ev, 0x1);
                        break;
                    case 'twotap':
                        this._handleTapEvent(ev, 0x4);
                        break;
                    case 'threetap':
                        this._handleTapEvent(ev, 0x2);
                        break;
                    case 'drag':
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        this._handleMouseButton(pos.x, pos.y, true, 0x1);
                        break;
                    case 'longpress':
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        this._handleMouseButton(pos.x, pos.y, true, 0x4);
                        break;

                    case 'twodrag':
                        this._gestureLastMagnitudeX = ev.detail.magnitudeX;
                        this._gestureLastMagnitudeY = ev.detail.magnitudeY;
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        break;
                    case 'pinch':
                        this._gestureLastMagnitudeX = Math.hypot(ev.detail.magnitudeX,
                                                                 ev.detail.magnitudeY);
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        break;
                }
                break;

            case 'gesturemove':
                switch (ev.detail.type) {
                    case 'onetap':
                    case 'twotap':
                    case 'threetap':
                        break;
                    case 'drag':
                    case 'longpress':
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        break;
                    case 'twodrag':
                        // Always scroll in the same position.
                        // We don't know if the mouse was moved so we need to move it
                        // every update.
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        while ((ev.detail.magnitudeY - this._gestureLastMagnitudeY) > GESTURE_SCRLSENS) {
                            this._handleMouseButton(pos.x, pos.y, true, 0x8);
                            this._handleMouseButton(pos.x, pos.y, false, 0x8);
                            this._gestureLastMagnitudeY += GESTURE_SCRLSENS;
                        }
                        while ((ev.detail.magnitudeY - this._gestureLastMagnitudeY) < -GESTURE_SCRLSENS) {
                            this._handleMouseButton(pos.x, pos.y, true, 0x10);
                            this._handleMouseButton(pos.x, pos.y, false, 0x10);
                            this._gestureLastMagnitudeY -= GESTURE_SCRLSENS;
                        }
                        while ((ev.detail.magnitudeX - this._gestureLastMagnitudeX) > GESTURE_SCRLSENS) {
                            this._handleMouseButton(pos.x, pos.y, true, 0x20);
                            this._handleMouseButton(pos.x, pos.y, false, 0x20);
                            this._gestureLastMagnitudeX += GESTURE_SCRLSENS;
                        }
                        while ((ev.detail.magnitudeX - this._gestureLastMagnitudeX) < -GESTURE_SCRLSENS) {
                            this._handleMouseButton(pos.x, pos.y, true, 0x40);
                            this._handleMouseButton(pos.x, pos.y, false, 0x40);
                            this._gestureLastMagnitudeX -= GESTURE_SCRLSENS;
                        }
                        break;
                    case 'pinch':
                        // Always scroll in the same position.
                        // We don't know if the mouse was moved so we need to move it
                        // every update.
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        magnitude = Math.hypot(ev.detail.magnitudeX, ev.detail.magnitudeY);
                        if (Math.abs(magnitude - this._gestureLastMagnitudeX) > GESTURE_ZOOMSENS) {
                            this._handleKeyEvent(KeyTable.XK_Control_L, "ControlLeft", true);
                            while ((magnitude - this._gestureLastMagnitudeX) > GESTURE_ZOOMSENS) {
                                this._handleMouseButton(pos.x, pos.y, true, 0x8);
                                this._handleMouseButton(pos.x, pos.y, false, 0x8);
                                this._gestureLastMagnitudeX += GESTURE_ZOOMSENS;
                            }
                            while ((magnitude -  this._gestureLastMagnitudeX) < -GESTURE_ZOOMSENS) {
                                this._handleMouseButton(pos.x, pos.y, true, 0x10);
                                this._handleMouseButton(pos.x, pos.y, false, 0x10);
                                this._gestureLastMagnitudeX -= GESTURE_ZOOMSENS;
                            }
                        }
                        this._handleKeyEvent(KeyTable.XK_Control_L, "ControlLeft", false);
                        break;
                }
                break;

            case 'gestureend':
                switch (ev.detail.type) {
                    case 'onetap':
                    case 'twotap':
                    case 'threetap':
                    case 'pinch':
                    case 'twodrag':
                        break;
                    case 'drag':
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        this._handleMouseButton(pos.x, pos.y, false, 0x1);
                        break;
                    case 'longpress':
                        this._fakeMouseMove(ev, pos.x, pos.y);
                        this._handleMouseButton(pos.x, pos.y, false, 0x4);
                        break;
                }
                break;
        }
    }

    // Message Handlers

    _negotiateProtocolVersion() {
        if (this._sock.rQwait("version", 12)) {
            return false;
        }

        const sversion = this._sock.rQshiftStr(12).substr(4, 7);
        Log.Info("Server ProtocolVersion: " + sversion);
        let isRepeater = 0;
        switch (sversion) {
            case "000.000":  // UltraVNC repeater
                isRepeater = 1;
                break;
            case "003.003":
            case "003.006":  // UltraVNC
                this._rfbVersion = 3.3;
                break;
            case "003.007":
                this._rfbVersion = 3.7;
                break;
            case "003.008":
            case "003.889":  // Apple Remote Desktop
            case "004.000":  // Intel AMT KVM
            case "004.001":  // RealVNC 4.6
            case "005.000":  // RealVNC 5.3
                this._rfbVersion = 3.8;
                break;
            default:
                return this._fail("Invalid server version " + sversion);
        }

        if (isRepeater) {
            let repeaterID = "ID:" + this._repeaterID;
            while (repeaterID.length < 250) {
                repeaterID += "\0";
            }
            this._sock.sQpushString(repeaterID);
            this._sock.flush();
            return true;
        }

        if (this._rfbVersion > this._rfbMaxVersion) {
            this._rfbVersion = this._rfbMaxVersion;
        }

        const cversion = "00" + parseInt(this._rfbVersion, 10) +
                       ".00" + ((this._rfbVersion * 10) % 10);
        this._sock.sQpushString("RFB " + cversion + "\n");
        this._sock.flush();
        Log.Debug('Sent ProtocolVersion: ' + cversion);

        this._rfbInitState = 'Security';
    }

    _isSupportedSecurityType(type) {
        const clientTypes = [
            securityTypeNone,
            securityTypeVNCAuth,
            securityTypeRA2ne,
            securityTypeTight,
            securityTypeVeNCrypt,
            securityTypeXVP,
            securityTypeARD,
            securityTypeMSLogonII,
            securityTypePlain,
        ];

        return clientTypes.includes(type);
    }

    _negotiateSecurity() {
        if (this._rfbVersion >= 3.7) {
            // Server sends supported list, client decides
            const numTypes = this._sock.rQshift8();
            if (this._sock.rQwait("security type", numTypes, 1)) { return false; }

            if (numTypes === 0) {
                this._rfbInitState = "SecurityReason";
                this._securityContext = "no security types";
                this._securityStatus = 1;
                return true;
            }

            const types = this._sock.rQshiftBytes(numTypes);
            Log.Debug("Server security types: " + types);

            // Look for a matching security type in the order that the
            // server prefers
            this._rfbAuthScheme = -1;
            for (let type of types) {
                if (this._isSupportedSecurityType(type)) {
                    this._rfbAuthScheme = type;
                    break;
                }
            }

            if (this._rfbAuthScheme === -1) {
                return this._fail("Unsupported security types (types: " + types + ")");
            }

            this._sock.sQpush8(this._rfbAuthScheme);
            this._sock.flush();
        } else {
            // Server decides
            if (this._sock.rQwait("security scheme", 4)) { return false; }
            this._rfbAuthScheme = this._sock.rQshift32();

            if (this._rfbAuthScheme == 0) {
                this._rfbInitState = "SecurityReason";
                this._securityContext = "authentication scheme";
                this._securityStatus = 1;
                return true;
            }
        }

        this._rfbInitState = 'Authentication';
        Log.Debug('Authenticating using scheme: ' + this._rfbAuthScheme);

        return true;
    }

    _handleSecurityReason() {
        if (this._sock.rQwait("reason length", 4)) {
            return false;
        }
        const strlen = this._sock.rQshift32();
        let reason = "";

        if (strlen > 0) {
            if (this._sock.rQwait("reason", strlen, 4)) { return false; }
            reason = this._sock.rQshiftStr(strlen);
        }

        if (reason !== "") {
            this.dispatchEvent(new CustomEvent(
                "securityfailure",
                { detail: { status: this._securityStatus,
                            reason: reason } }));

            return this._fail("Security negotiation failed on " +
                              this._securityContext +
                              " (reason: " + reason + ")");
        } else {
            this.dispatchEvent(new CustomEvent(
                "securityfailure",
                { detail: { status: this._securityStatus } }));

            return this._fail("Security negotiation failed on " +
                              this._securityContext);
        }
    }

    // authentication
    _negotiateXvpAuth() {
        if (this._rfbCredentials.username === undefined ||
            this._rfbCredentials.password === undefined ||
            this._rfbCredentials.target === undefined) {
            this.dispatchEvent(new CustomEvent(
                "credentialsrequired",
                { detail: { types: ["username", "password", "target"] } }));
            return false;
        }

        this._sock.sQpush8(this._rfbCredentials.username.length);
        this._sock.sQpush8(this._rfbCredentials.target.length);
        this._sock.sQpushString(this._rfbCredentials.username);
        this._sock.sQpushString(this._rfbCredentials.target);

        this._sock.flush();

        this._rfbAuthScheme = securityTypeVNCAuth;

        return this._negotiateAuthentication();
    }

    // VeNCrypt authentication, currently only supports version 0.2 and only Plain subtype
    _negotiateVeNCryptAuth() {

        // waiting for VeNCrypt version
        if (this._rfbVeNCryptState == 0) {
            if (this._sock.rQwait("vencrypt version", 2)) { return false; }

            const major = this._sock.rQshift8();
            const minor = this._sock.rQshift8();

            if (!(major == 0 && minor == 2)) {
                return this._fail("Unsupported VeNCrypt version " + major + "." + minor);
            }

            this._sock.sQpush8(0);
            this._sock.sQpush8(2);
            this._sock.flush();
            this._rfbVeNCryptState = 1;
        }

        // waiting for ACK
        if (this._rfbVeNCryptState == 1) {
            if (this._sock.rQwait("vencrypt ack", 1)) { return false; }

            const res = this._sock.rQshift8();

            if (res != 0) {
                return this._fail("VeNCrypt failure " + res);
            }

            this._rfbVeNCryptState = 2;
        }
        // must fall through here (i.e. no "else if"), beacause we may have already received
        // the subtypes length and won't be called again

        if (this._rfbVeNCryptState == 2) { // waiting for subtypes length
            if (this._sock.rQwait("vencrypt subtypes length", 1)) { return false; }

            const subtypesLength = this._sock.rQshift8();
            if (subtypesLength < 1) {
                return this._fail("VeNCrypt subtypes empty");
            }

            this._rfbVeNCryptSubtypesLength = subtypesLength;
            this._rfbVeNCryptState = 3;
        }

        // waiting for subtypes list
        if (this._rfbVeNCryptState == 3) {
            if (this._sock.rQwait("vencrypt subtypes", 4 * this._rfbVeNCryptSubtypesLength)) { return false; }

            const subtypes = [];
            for (let i = 0; i < this._rfbVeNCryptSubtypesLength; i++) {
                subtypes.push(this._sock.rQshift32());
            }

            // Look for a matching security type in the order that the
            // server prefers
            this._rfbAuthScheme = -1;
            for (let type of subtypes) {
                // Avoid getting in to a loop
                if (type === securityTypeVeNCrypt) {
                    continue;
                }

                if (this._isSupportedSecurityType(type)) {
                    this._rfbAuthScheme = type;
                    break;
                }
            }

            if (this._rfbAuthScheme === -1) {
                return this._fail("Unsupported security types (types: " + subtypes + ")");
            }

            this._sock.sQpush32(this._rfbAuthScheme);
            this._sock.flush();

            this._rfbVeNCryptState = 4;
            return true;
        }
    }

    _negotiatePlainAuth() {
        if (this._rfbCredentials.username === undefined ||
            this._rfbCredentials.password === undefined) {
            this.dispatchEvent(new CustomEvent(
                "credentialsrequired",
                { detail: { types: ["username", "password"] } }));
            return false;
        }

        const user = encodeUTF8(this._rfbCredentials.username);
        const pass = encodeUTF8(this._rfbCredentials.password);

        this._sock.sQpush32(user.length);
        this._sock.sQpush32(pass.length);
        this._sock.sQpushString(user);
        this._sock.sQpushString(pass);
        this._sock.flush();

        this._rfbInitState = "SecurityResult";
        return true;
    }

    _negotiateStdVNCAuth() {
        if (this._sock.rQwait("auth challenge", 16)) { return false; }

        if (this._rfbCredentials.password === undefined) {
            this.dispatchEvent(new CustomEvent(
                "credentialsrequired",
                { detail: { types: ["password"] } }));
            return false;
        }

        // TODO(directxman12): make genDES not require an Array
        const challenge = Array.prototype.slice.call(this._sock.rQshiftBytes(16));
        const response = RFB.genDES(this._rfbCredentials.password, challenge);
        this._sock.sQpushBytes(response);
        this._sock.flush();
        this._rfbInitState = "SecurityResult";
        return true;
    }

    _negotiateARDAuth() {

        if (this._rfbCredentials.username === undefined ||
            this._rfbCredentials.password === undefined) {
            this.dispatchEvent(new CustomEvent(
                "credentialsrequired",
                { detail: { types: ["username", "password"] } }));
            return false;
        }

        if (this._rfbCredentials.ardPublicKey != undefined &&
            this._rfbCredentials.ardCredentials != undefined) {
            // if the async web crypto is done return the results
            this._sock.sQpushBytes(this._rfbCredentials.ardCredentials);
            this._sock.sQpushBytes(this._rfbCredentials.ardPublicKey);
            this._sock.flush();
            this._rfbCredentials.ardCredentials = null;
            this._rfbCredentials.ardPublicKey = null;
            this._rfbInitState = "SecurityResult";
            return true;
        }

        if (this._sock.rQwait("read ard", 4)) { return false; }

        let generator = this._sock.rQshiftBytes(2);   // DH base generator value

        let keyLength = this._sock.rQshift16();

        if (this._sock.rQwait("read ard keylength", keyLength*2, 4)) { return false; }

        // read the server values
        let prime = this._sock.rQshiftBytes(keyLength);  // predetermined prime modulus
        let serverPublicKey = this._sock.rQshiftBytes(keyLength); // other party's public key

        let clientKey = legacyCrypto.generateKey(
            { name: "DH", g: generator, p: prime }, false, ["deriveBits"]);
        this._negotiateARDAuthAsync(keyLength, serverPublicKey, clientKey);

        return false;
    }

    async _negotiateARDAuthAsync(keyLength, serverPublicKey, clientKey) {
        const clientPublicKey = legacyCrypto.exportKey("raw", clientKey.publicKey);
        const sharedKey = legacyCrypto.deriveBits(
            { name: "DH", public: serverPublicKey }, clientKey.privateKey, keyLength * 8);

        const username = encodeUTF8(this._rfbCredentials.username).substring(0, 63);
        const password = encodeUTF8(this._rfbCredentials.password).substring(0, 63);

        const credentials = window.crypto.getRandomValues(new Uint8Array(128));
        for (let i = 0; i < username.length; i++) {
            credentials[i] = username.charCodeAt(i);
        }
        credentials[username.length] = 0;
        for (let i = 0; i < password.length; i++) {
            credentials[64 + i] = password.charCodeAt(i);
        }
        credentials[64 + password.length] = 0;

        const key = await legacyCrypto.digest("MD5", sharedKey);
        const cipher = await legacyCrypto.importKey(
            "raw", key, { name: "AES-ECB" }, false, ["encrypt"]);
        const encrypted = await legacyCrypto.encrypt({ name: "AES-ECB" }, cipher, credentials);

        this._rfbCredentials.ardCredentials = encrypted;
        this._rfbCredentials.ardPublicKey = clientPublicKey;

        this._resumeAuthentication();
    }

    _negotiateTightUnixAuth() {
        if (this._rfbCredentials.username === undefined ||
            this._rfbCredentials.password === undefined) {
            this.dispatchEvent(new CustomEvent(
                "credentialsrequired",
                { detail: { types: ["username", "password"] } }));
            return false;
        }

        this._sock.sQpush32(this._rfbCredentials.username.length);
        this._sock.sQpush32(this._rfbCredentials.password.length);
        this._sock.sQpushString(this._rfbCredentials.username);
        this._sock.sQpushString(this._rfbCredentials.password);
        this._sock.flush();

        this._rfbInitState = "SecurityResult";
        return true;
    }

    _negotiateTightTunnels(numTunnels) {
        const clientSupportedTunnelTypes = {
            0: { vendor: 'TGHT', signature: 'NOTUNNEL' }
        };
        const serverSupportedTunnelTypes = {};
        // receive tunnel capabilities
        for (let i = 0; i < numTunnels; i++) {
            const capCode = this._sock.rQshift32();
            const capVendor = this._sock.rQshiftStr(4);
            const capSignature = this._sock.rQshiftStr(8);
            serverSupportedTunnelTypes[capCode] = { vendor: capVendor, signature: capSignature };
        }

        Log.Debug("Server Tight tunnel types: " + serverSupportedTunnelTypes);

        // Siemens touch panels have a VNC server that supports NOTUNNEL,
        // but forgets to advertise it. Try to detect such servers by
        // looking for their custom tunnel type.
        if (serverSupportedTunnelTypes[1] &&
            (serverSupportedTunnelTypes[1].vendor === "SICR") &&
            (serverSupportedTunnelTypes[1].signature === "SCHANNEL")) {
            Log.Debug("Detected Siemens server. Assuming NOTUNNEL support.");
            serverSupportedTunnelTypes[0] = { vendor: 'TGHT', signature: 'NOTUNNEL' };
        }

        // choose the notunnel type
        if (serverSupportedTunnelTypes[0]) {
            if (serverSupportedTunnelTypes[0].vendor != clientSupportedTunnelTypes[0].vendor ||
                serverSupportedTunnelTypes[0].signature != clientSupportedTunnelTypes[0].signature) {
                return this._fail("Client's tunnel type had the incorrect " +
                                  "vendor or signature");
            }
            Log.Debug("Selected tunnel type: " + clientSupportedTunnelTypes[0]);
            this._sock.sQpush32(0); // use NOTUNNEL
            this._sock.flush();
            return false; // wait until we receive the sub auth count to continue
        } else {
            return this._fail("Server wanted tunnels, but doesn't support " +
                              "the notunnel type");
        }
    }

    _negotiateTightAuth() {
        if (!this._rfbTightVNC) {  // first pass, do the tunnel negotiation
            if (this._sock.rQwait("num tunnels", 4)) { return false; }
            const numTunnels = this._sock.rQshift32();
            if (numTunnels > 0 && this._sock.rQwait("tunnel capabilities", 16 * numTunnels, 4)) { return false; }

            this._rfbTightVNC = true;

            if (numTunnels > 0) {
                this._negotiateTightTunnels(numTunnels);
                return false;  // wait until we receive the sub auth to continue
            }
        }

        // second pass, do the sub-auth negotiation
        if (this._sock.rQwait("sub auth count", 4)) { return false; }
        const subAuthCount = this._sock.rQshift32();
        if (subAuthCount === 0) {  // empty sub-auth list received means 'no auth' subtype selected
            this._rfbInitState = 'SecurityResult';
            return true;
        }

        if (this._sock.rQwait("sub auth capabilities", 16 * subAuthCount, 4)) { return false; }

        const clientSupportedTypes = {
            'STDVNOAUTH__': 1,
            'STDVVNCAUTH_': 2,
            'TGHTULGNAUTH': 129
        };

        const serverSupportedTypes = [];

        for (let i = 0; i < subAuthCount; i++) {
            this._sock.rQshift32(); // capNum
            const capabilities = this._sock.rQshiftStr(12);
            serverSupportedTypes.push(capabilities);
        }

        Log.Debug("Server Tight authentication types: " + serverSupportedTypes);

        for (let authType in clientSupportedTypes) {
            if (serverSupportedTypes.indexOf(authType) != -1) {
                this._sock.sQpush32(clientSupportedTypes[authType]);
                this._sock.flush();
                Log.Debug("Selected authentication type: " + authType);

                switch (authType) {
                    case 'STDVNOAUTH__':  // no auth
                        this._rfbInitState = 'SecurityResult';
                        return true;
                    case 'STDVVNCAUTH_':
                        this._rfbAuthScheme = securityTypeVNCAuth;
                        return true;
                    case 'TGHTULGNAUTH':
                        this._rfbAuthScheme = securityTypeUnixLogon;
                        return true;
                    default:
                        return this._fail("Unsupported tiny auth scheme " +
                                          "(scheme: " + authType + ")");
                }
            }
        }

        return this._fail("No supported sub-auth types!");
    }

    _handleRSAAESCredentialsRequired(event) {
        this.dispatchEvent(event);
    }

    _handleRSAAESServerVerification(event) {
        this.dispatchEvent(event);
    }

    _negotiateRA2neAuth() {
        if (this._rfbRSAAESAuthenticationState === null) {
            this._rfbRSAAESAuthenticationState = new RSAAESAuthenticationState(this._sock, () => this._rfbCredentials);
            this._rfbRSAAESAuthenticationState.addEventListener(
                "serververification", this._eventHandlers.handleRSAAESServerVerification);
            this._rfbRSAAESAuthenticationState.addEventListener(
                "credentialsrequired", this._eventHandlers.handleRSAAESCredentialsRequired);
        }
        this._rfbRSAAESAuthenticationState.checkInternalEvents();
        if (!this._rfbRSAAESAuthenticationState.hasStarted) {
            this._rfbRSAAESAuthenticationState.negotiateRA2neAuthAsync()
                .catch((e) => {
                    if (e.message !== "disconnect normally") {
                        this._fail(e.message);
                    }
                })
                .then(() => {
                    this._rfbInitState = "SecurityResult";
                    return true;
                }).finally(() => {
                    this._rfbRSAAESAuthenticationState.removeEventListener(
                        "serververification", this._eventHandlers.handleRSAAESServerVerification);
                    this._rfbRSAAESAuthenticationState.removeEventListener(
                        "credentialsrequired", this._eventHandlers.handleRSAAESCredentialsRequired);
                    this._rfbRSAAESAuthenticationState = null;
                });
        }
        return false;
    }

    _negotiateMSLogonIIAuth() {
        if (this._sock.rQwait("mslogonii dh param", 24)) { return false; }

        if (this._rfbCredentials.username === undefined ||
            this._rfbCredentials.password === undefined) {
            this.dispatchEvent(new CustomEvent(
                "credentialsrequired",
                { detail: { types: ["username", "password"] } }));
            return false;
        }

        const g = this._sock.rQshiftBytes(8);
        const p = this._sock.rQshiftBytes(8);
        const A = this._sock.rQshiftBytes(8);
        const dhKey = legacyCrypto.generateKey({ name: "DH", g: g, p: p }, true, ["deriveBits"]);
        const B = legacyCrypto.exportKey("raw", dhKey.publicKey);
        const secret = legacyCrypto.deriveBits({ name: "DH", public: A }, dhKey.privateKey, 64);

        const key = legacyCrypto.importKey("raw", secret, { name: "DES-CBC" }, false, ["encrypt"]);
        const username = encodeUTF8(this._rfbCredentials.username).substring(0, 255);
        const password = encodeUTF8(this._rfbCredentials.password).substring(0, 63);
        let usernameBytes = new Uint8Array(256);
        let passwordBytes = new Uint8Array(64);
        window.crypto.getRandomValues(usernameBytes);
        window.crypto.getRandomValues(passwordBytes);
        for (let i = 0; i < username.length; i++) {
            usernameBytes[i] = username.charCodeAt(i);
        }
        usernameBytes[username.length] = 0;
        for (let i = 0; i < password.length; i++) {
            passwordBytes[i] = password.charCodeAt(i);
        }
        passwordBytes[password.length] = 0;
        usernameBytes = legacyCrypto.encrypt({ name: "DES-CBC", iv: secret }, key, usernameBytes);
        passwordBytes = legacyCrypto.encrypt({ name: "DES-CBC", iv: secret }, key, passwordBytes);
        this._sock.sQpushBytes(B);
        this._sock.sQpushBytes(usernameBytes);
        this._sock.sQpushBytes(passwordBytes);
        this._sock.flush();
        this._rfbInitState = "SecurityResult";
        return true;
    }

    _negotiateAuthentication() {
        switch (this._rfbAuthScheme) {
            case securityTypeNone:
                if (this._rfbVersion >= 3.8) {
                    this._rfbInitState = 'SecurityResult';
                } else {
                    this._rfbInitState = 'ClientInitialisation';
                }
                return true;

            case securityTypeXVP:
                return this._negotiateXvpAuth();

            case securityTypeARD:
                return this._negotiateARDAuth();

            case securityTypeVNCAuth:
                return this._negotiateStdVNCAuth();

            case securityTypeTight:
                return this._negotiateTightAuth();

            case securityTypeVeNCrypt:
                return this._negotiateVeNCryptAuth();

            case securityTypePlain:
                return this._negotiatePlainAuth();

            case securityTypeUnixLogon:
                return this._negotiateTightUnixAuth();

            case securityTypeRA2ne:
                return this._negotiateRA2neAuth();

            case securityTypeMSLogonII:
                return this._negotiateMSLogonIIAuth();

            default:
                return this._fail("Unsupported auth scheme (scheme: " +
                                  this._rfbAuthScheme + ")");
        }
    }

    _handleSecurityResult() {
        if (this._sock.rQwait('VNC auth response ', 4)) { return false; }

        const status = this._sock.rQshift32();

        if (status === 0) { // OK
            this._rfbInitState = 'ClientInitialisation';
            Log.Debug('Authentication OK');
            return true;
        } else {
            if (this._rfbVersion >= 3.8) {
                this._rfbInitState = "SecurityReason";
                this._securityContext = "security result";
                this._securityStatus = status;
                return true;
            } else {
                this.dispatchEvent(new CustomEvent(
                    "securityfailure",
                    { detail: { status: status } }));

                return this._fail("Security handshake failed");
            }
        }
    }

    _negotiateServerInit() {
        if (this._sock.rQwait("server initialization", 24)) { return false; }

        /* Screen size */
        const width = this._sock.rQshift16();
        const height = this._sock.rQshift16();

        /* PIXEL_FORMAT */
        const bpp         = this._sock.rQshift8();
        const depth       = this._sock.rQshift8();
        const bigEndian  = this._sock.rQshift8();
        const trueColor  = this._sock.rQshift8();

        const redMax     = this._sock.rQshift16();
        const greenMax   = this._sock.rQshift16();
        const blueMax    = this._sock.rQshift16();
        const redShift   = this._sock.rQshift8();
        const greenShift = this._sock.rQshift8();
        const blueShift  = this._sock.rQshift8();
        this._sock.rQskipBytes(3);  // padding

        // NB(directxman12): we don't want to call any callbacks or print messages until
        //                   *after* we're past the point where we could backtrack

        /* Connection name/title */
        const nameLength = this._sock.rQshift32();
        if (this._sock.rQwait('server init name', nameLength, 24)) { return false; }
        let name = this._sock.rQshiftStr(nameLength);
        name = decodeUTF8(name, true);

        if (this._rfbTightVNC) {
            if (this._sock.rQwait('TightVNC extended server init header', 8, 24 + nameLength)) { return false; }
            // In TightVNC mode, ServerInit message is extended
            const numServerMessages = this._sock.rQshift16();
            const numClientMessages = this._sock.rQshift16();
            const numEncodings = this._sock.rQshift16();
            this._sock.rQskipBytes(2);  // padding

            const totalMessagesLength = (numServerMessages + numClientMessages + numEncodings) * 16;
            if (this._sock.rQwait('TightVNC extended server init header', totalMessagesLength, 32 + nameLength)) { return false; }

            // we don't actually do anything with the capability information that TIGHT sends,
            // so we just skip the all of this.

            // TIGHT server message capabilities
            this._sock.rQskipBytes(16 * numServerMessages);

            // TIGHT client message capabilities
            this._sock.rQskipBytes(16 * numClientMessages);

            // TIGHT encoding capabilities
            this._sock.rQskipBytes(16 * numEncodings);
        }

        // NB(directxman12): these are down here so that we don't run them multiple times
        //                   if we backtrack
        Log.Info("Screen: " + width + "x" + height +
                  ", bpp: " + bpp + ", depth: " + depth +
                  ", bigEndian: " + bigEndian +
                  ", trueColor: " + trueColor +
                  ", redMax: " + redMax +
                  ", greenMax: " + greenMax +
                  ", blueMax: " + blueMax +
                  ", redShift: " + redShift +
                  ", greenShift: " + greenShift +
                  ", blueShift: " + blueShift);

        // we're past the point where we could backtrack, so it's safe to call this
        this._setDesktopName(name);
        this._resize(width, height);

        if (!this._viewOnly) { this._keyboard.grab(); }

        this._fbDepth = 24;

        if (this._fbName === "Intel(r) AMT KVM") {
            Log.Warn("Intel AMT KVM only supports 8/16 bit depths. Using low color mode.");
            this._fbDepth = 8;
        }

        RFB.messages.pixelFormat(this._sock, this._fbDepth, true);
        this._sendEncodings();
        RFB.messages.fbUpdateRequest(this._sock, false, 0, 0, this._fbWidth, this._fbHeight);

        this._updateConnectionState('connected');
        return true;
    }

    _sendEncodings() {
        const encs = [];

        // In preference order
        encs.push(encodings.encodingCopyRect);
        // Only supported with full depth support
        if (this._fbDepth == 24) {
            encs.push(encodings.encodingTight);
            encs.push(encodings.encodingTightPNG);
            encs.push(encodings.encodingZRLE);
            encs.push(encodings.encodingJPEG);
            encs.push(encodings.encodingHextile);
            encs.push(encodings.encodingRRE);
        }
        encs.push(encodings.encodingRaw);

        // Psuedo-encoding settings
        encs.push(encodings.pseudoEncodingQualityLevel0 + this._qualityLevel);
        encs.push(encodings.pseudoEncodingCompressLevel0 + this._compressionLevel);

        encs.push(encodings.pseudoEncodingDesktopSize);
        encs.push(encodings.pseudoEncodingLastRect);
        encs.push(encodings.pseudoEncodingQEMUExtendedKeyEvent);
        encs.push(encodings.pseudoEncodingQEMULedEvent);
        encs.push(encodings.pseudoEncodingExtendedDesktopSize);
        encs.push(encodings.pseudoEncodingXvp);
        encs.push(encodings.pseudoEncodingFence);
        encs.push(encodings.pseudoEncodingContinuousUpdates);
        encs.push(encodings.pseudoEncodingDesktopName);
        encs.push(encodings.pseudoEncodingExtendedClipboard);

        if (this._fbDepth == 24) {
            encs.push(encodings.pseudoEncodingVMwareCursor);
            encs.push(encodings.pseudoEncodingCursor);
        }

        RFB.messages.clientEncodings(this._sock, encs);
    }

    /* RFB protocol initialization states:
     *   ProtocolVersion
     *   Security
     *   Authentication
     *   SecurityResult
     *   ClientInitialization - not triggered by server message
     *   ServerInitialization
     */
    _initMsg() {
        switch (this._rfbInitState) {
            case 'ProtocolVersion':
                return this._negotiateProtocolVersion();

            case 'Security':
                return this._negotiateSecurity();

            case 'Authentication':
                return this._negotiateAuthentication();

            case 'SecurityResult':
                return this._handleSecurityResult();

            case 'SecurityReason':
                return this._handleSecurityReason();

            case 'ClientInitialisation':
                this._sock.sQpush8(this._shared ? 1 : 0); // ClientInitialisation
                this._sock.flush();
                this._rfbInitState = 'ServerInitialisation';
                return true;

            case 'ServerInitialisation':
                return this._negotiateServerInit();

            default:
                return this._fail("Unknown init state (state: " +
                                  this._rfbInitState + ")");
        }
    }

    // Resume authentication handshake after it was paused for some
    // reason, e.g. waiting for a password from the user
    _resumeAuthentication() {
        // We use setTimeout() so it's run in its own context, just like
        // it originally did via the WebSocket's event handler
        setTimeout(this._initMsg.bind(this), 0);
    }

    _handleSetColourMapMsg() {
        Log.Debug("SetColorMapEntries");

        return this._fail("Unexpected SetColorMapEntries message");
    }

    _handleServerCutText() {
        Log.Debug("ServerCutText");

        if (this._sock.rQwait("ServerCutText header", 7, 1)) { return false; }

        this._sock.rQskipBytes(3);  // Padding

        let length = this._sock.rQshift32();
        length = toSigned32bit(length);

        if (this._sock.rQwait("ServerCutText content", Math.abs(length), 8)) { return false; }

        if (length >= 0) {
            //Standard msg
            const text = this._sock.rQshiftStr(length);
            if (this._viewOnly) {
                return true;
            }

            this.dispatchEvent(new CustomEvent(
                "clipboard",
                { detail: { text: text } }));

        } else {
            //Extended msg.
            length = Math.abs(length);
            const flags = this._sock.rQshift32();
            let formats = flags & 0x0000FFFF;
            let actions = flags & 0xFF000000;

            let isCaps = (!!(actions & extendedClipboardActionCaps));
            if (isCaps) {
                this._clipboardServerCapabilitiesFormats = {};
                this._clipboardServerCapabilitiesActions = {};

                // Update our server capabilities for Formats
                for (let i = 0; i <= 15; i++) {
                    let index = 1 << i;

                    // Check if format flag is set.
                    if ((formats & index)) {
                        this._clipboardServerCapabilitiesFormats[index] = true;
                        // We don't send unsolicited clipboard, so we
                        // ignore the size
                        this._sock.rQshift32();
                    }
                }

                // Update our server capabilities for Actions
                for (let i = 24; i <= 31; i++) {
                    let index = 1 << i;
                    this._clipboardServerCapabilitiesActions[index] = !!(actions & index);
                }

                /*  Caps handling done, send caps with the clients
                    capabilities set as a response */
                let clientActions = [
                    extendedClipboardActionCaps,
                    extendedClipboardActionRequest,
                    extendedClipboardActionPeek,
                    extendedClipboardActionNotify,
                    extendedClipboardActionProvide
                ];
                RFB.messages.extendedClipboardCaps(this._sock, clientActions, {extendedClipboardFormatText: 0});

            } else if (actions === extendedClipboardActionRequest) {
                if (this._viewOnly) {
                    return true;
                }

                // Check if server has told us it can handle Provide and there is clipboard data to send.
                if (this._clipboardText != null &&
                    this._clipboardServerCapabilitiesActions[extendedClipboardActionProvide]) {

                    if (formats & extendedClipboardFormatText) {
                        RFB.messages.extendedClipboardProvide(this._sock, [extendedClipboardFormatText], [this._clipboardText]);
                    }
                }

            } else if (actions === extendedClipboardActionPeek) {
                if (this._viewOnly) {
                    return true;
                }

                if (this._clipboardServerCapabilitiesActions[extendedClipboardActionNotify]) {

                    if (this._clipboardText != null) {
                        RFB.messages.extendedClipboardNotify(this._sock, [extendedClipboardFormatText]);
                    } else {
                        RFB.messages.extendedClipboardNotify(this._sock, []);
                    }
                }

            } else if (actions === extendedClipboardActionNotify) {
                if (this._viewOnly) {
                    return true;
                }

                if (this._clipboardServerCapabilitiesActions[extendedClipboardActionRequest]) {

                    if (formats & extendedClipboardFormatText) {
                        RFB.messages.extendedClipboardRequest(this._sock, [extendedClipboardFormatText]);
                    }
                }

            } else if (actions === extendedClipboardActionProvide) {
                if (this._viewOnly) {
                    return true;
                }

                if (!(formats & extendedClipboardFormatText)) {
                    return true;
                }
                // Ignore what we had in our clipboard client side.
                this._clipboardText = null;

                // FIXME: Should probably verify that this data was actually requested
                let zlibStream = this._sock.rQshiftBytes(length - 4);
                let streamInflator = new Inflator();
                let textData = null;

                streamInflator.setInput(zlibStream);
                for (let i = 0; i <= 15; i++) {
                    let format = 1 << i;

                    if (formats & format) {

                        let size = 0x00;
                        let sizeArray = streamInflator.inflate(4);

                        size |= (sizeArray[0] << 24);
                        size |= (sizeArray[1] << 16);
                        size |= (sizeArray[2] << 8);
                        size |= (sizeArray[3]);
                        let chunk = streamInflator.inflate(size);

                        if (format === extendedClipboardFormatText) {
                            textData = chunk;
                        }
                    }
                }
                streamInflator.setInput(null);

                if (textData !== null) {
                    let tmpText = "";
                    for (let i = 0; i < textData.length; i++) {
                        tmpText += String.fromCharCode(textData[i]);
                    }
                    textData = tmpText;

                    textData = decodeUTF8(textData);
                    if ((textData.length > 0) && "\0" === textData.charAt(textData.length - 1)) {
                        textData = textData.slice(0, -1);
                    }

                    textData = textData.replaceAll("\r\n", "\n");

                    this.dispatchEvent(new CustomEvent(
                        "clipboard",
                        { detail: { text: textData } }));
                }
            } else {
                return this._fail("Unexpected action in extended clipboard message: " + actions);
            }
        }
        return true;
    }

    _handleServerFenceMsg() {
        if (this._sock.rQwait("ServerFence header", 8, 1)) { return false; }
        this._sock.rQskipBytes(3); // Padding
        let flags = this._sock.rQshift32();
        let length = this._sock.rQshift8();

        if (this._sock.rQwait("ServerFence payload", length, 9)) { return false; }

        if (length > 64) {
            Log.Warn("Bad payload length (" + length + ") in fence response");
            length = 64;
        }

        const payload = this._sock.rQshiftStr(length);

        this._supportsFence = true;

        /*
         * Fence flags
         *
         *  (1<<0)  - BlockBefore
         *  (1<<1)  - BlockAfter
         *  (1<<2)  - SyncNext
         *  (1<<31) - Request
         */

        if (!(flags & (1<<31))) {
            return this._fail("Unexpected fence response");
        }

        // Filter out unsupported flags
        // FIXME: support syncNext
        flags &= (1<<0) | (1<<1);

        // BlockBefore and BlockAfter are automatically handled by
        // the fact that we process each incoming message
        // synchronuosly.
        RFB.messages.clientFence(this._sock, flags, payload);

        return true;
    }

    _handleXvpMsg() {
        if (this._sock.rQwait("XVP version and message", 3, 1)) { return false; }
        this._sock.rQskipBytes(1);  // Padding
        const xvpVer = this._sock.rQshift8();
        const xvpMsg = this._sock.rQshift8();

        switch (xvpMsg) {
            case 0:  // XVP_FAIL
                Log.Error("XVP Operation Failed");
                break;
            case 1:  // XVP_INIT
                this._rfbXvpVer = xvpVer;
                Log.Info("XVP extensions enabled (version " + this._rfbXvpVer + ")");
                this._setCapability("power", true);
                break;
            default:
                this._fail("Illegal server XVP message (msg: " + xvpMsg + ")");
                break;
        }

        return true;
    }

    _normalMsg() {
        let msgType;
        if (this._FBU.rects > 0) {
            msgType = 0;
        } else {
            msgType = this._sock.rQshift8();
        }

        let first, ret;
        switch (msgType) {
            case 0:  // FramebufferUpdate
                ret = this._framebufferUpdate();
                if (ret && !this._enabledContinuousUpdates) {
                    RFB.messages.fbUpdateRequest(this._sock, true, 0, 0,
                                                 this._fbWidth, this._fbHeight);
                }
                return ret;

            case 1:  // SetColorMapEntries
                return this._handleSetColourMapMsg();

            case 2:  // Bell
                Log.Debug("Bell");
                this.dispatchEvent(new CustomEvent(
                    "bell",
                    { detail: {} }));
                return true;

            case 3:  // ServerCutText
                return this._handleServerCutText();

            case 150: // EndOfContinuousUpdates
                first = !this._supportsContinuousUpdates;
                this._supportsContinuousUpdates = true;
                this._enabledContinuousUpdates = false;
                if (first) {
                    this._enabledContinuousUpdates = true;
                    this._updateContinuousUpdates();
                    Log.Info("Enabling continuous updates.");
                } else {
                    // FIXME: We need to send a framebufferupdaterequest here
                    // if we add support for turning off continuous updates
                }
                return true;

            case 248: // ServerFence
                return this._handleServerFenceMsg();

            case 250:  // XVP
                return this._handleXvpMsg();

            default:
                this._fail("Unexpected server message (type " + msgType + ")");
                Log.Debug("sock.rQpeekBytes(30): " + this._sock.rQpeekBytes(30));
                return true;
        }
    }

    _framebufferUpdate() {
        if (this._FBU.rects === 0) {
            if (this._sock.rQwait("FBU header", 3, 1)) { return false; }
            this._sock.rQskipBytes(1);  // Padding
            this._FBU.rects = this._sock.rQshift16();

            // Make sure the previous frame is fully rendered first
            // to avoid building up an excessive queue
            if (this._display.pending()) {
                this._flushing = true;
                this._display.flush()
                    .then(() => {
                        this._flushing = false;
                        // Resume processing
                        if (!this._sock.rQwait("message", 1)) {
                            this._handleMessage();
                        }
                    });
                return false;
            }
        }

        while (this._FBU.rects > 0) {
            if (this._FBU.encoding === null) {
                if (this._sock.rQwait("rect header", 12)) { return false; }
                /* New FramebufferUpdate */

                this._FBU.x = this._sock.rQshift16();
                this._FBU.y = this._sock.rQshift16();
                this._FBU.width = this._sock.rQshift16();
                this._FBU.height = this._sock.rQshift16();
                this._FBU.encoding = this._sock.rQshift32();
                /* Encodings are signed */
                this._FBU.encoding >>= 0;
            }

            if (!this._handleRect()) {
                return false;
            }

            this._FBU.rects--;
            this._FBU.encoding = null;
        }

        this._display.flip();

        return true;  // We finished this FBU
    }

    _handleRect() {
        switch (this._FBU.encoding) {
            case encodings.pseudoEncodingLastRect:
                this._FBU.rects = 1; // Will be decreased when we return
                return true;

            case encodings.pseudoEncodingVMwareCursor:
                return this._handleVMwareCursor();

            case encodings.pseudoEncodingCursor:
                return this._handleCursor();

            case encodings.pseudoEncodingQEMUExtendedKeyEvent:
                this._qemuExtKeyEventSupported = true;
                return true;

            case encodings.pseudoEncodingDesktopName:
                return this._handleDesktopName();

            case encodings.pseudoEncodingDesktopSize:
                this._resize(this._FBU.width, this._FBU.height);
                return true;

            case encodings.pseudoEncodingExtendedDesktopSize:
                return this._handleExtendedDesktopSize();

            case encodings.pseudoEncodingQEMULedEvent:
                return this._handleLedEvent();

            default:
                return this._handleDataRect();
        }
    }

    _handleVMwareCursor() {
        const hotx = this._FBU.x;  // hotspot-x
        const hoty = this._FBU.y;  // hotspot-y
        const w = this._FBU.width;
        const h = this._FBU.height;
        if (this._sock.rQwait("VMware cursor encoding", 1)) {
            return false;
        }

        const cursorType = this._sock.rQshift8();

        this._sock.rQshift8(); //Padding

        let rgba;
        const bytesPerPixel = 4;

        //Classic cursor
        if (cursorType == 0) {
            //Used to filter away unimportant bits.
            //OR is used for correct conversion in js.
            const PIXEL_MASK = 0xffffff00 | 0;
            rgba = new Array(w * h * bytesPerPixel);

            if (this._sock.rQwait("VMware cursor classic encoding",
                                  (w * h * bytesPerPixel) * 2, 2)) {
                return false;
            }

            let andMask = new Array(w * h);
            for (let pixel = 0; pixel < (w * h); pixel++) {
                andMask[pixel] = this._sock.rQshift32();
            }

            let xorMask = new Array(w * h);
            for (let pixel = 0; pixel < (w * h); pixel++) {
                xorMask[pixel] = this._sock.rQshift32();
            }

            for (let pixel = 0; pixel < (w * h); pixel++) {
                if (andMask[pixel] == 0) {
                    //Fully opaque pixel
                    let bgr = xorMask[pixel];
                    let r   = bgr >> 8  & 0xff;
                    let g   = bgr >> 16 & 0xff;
                    let b   = bgr >> 24 & 0xff;

                    rgba[(pixel * bytesPerPixel)     ] = r;    //r
                    rgba[(pixel * bytesPerPixel) + 1 ] = g;    //g
                    rgba[(pixel * bytesPerPixel) + 2 ] = b;    //b
                    rgba[(pixel * bytesPerPixel) + 3 ] = 0xff; //a

                } else if ((andMask[pixel] & PIXEL_MASK) ==
                           PIXEL_MASK) {
                    //Only screen value matters, no mouse colouring
                    if (xorMask[pixel] == 0) {
                        //Transparent pixel
                        rgba[(pixel * bytesPerPixel)     ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 1 ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 2 ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 3 ] = 0x00;

                    } else if ((xorMask[pixel] & PIXEL_MASK) ==
                               PIXEL_MASK) {
                        //Inverted pixel, not supported in browsers.
                        //Fully opaque instead.
                        rgba[(pixel * bytesPerPixel)     ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 1 ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 2 ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 3 ] = 0xff;

                    } else {
                        //Unhandled xorMask
                        rgba[(pixel * bytesPerPixel)     ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 1 ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 2 ] = 0x00;
                        rgba[(pixel * bytesPerPixel) + 3 ] = 0xff;
                    }

                } else {
                    //Unhandled andMask
                    rgba[(pixel * bytesPerPixel)     ] = 0x00;
                    rgba[(pixel * bytesPerPixel) + 1 ] = 0x00;
                    rgba[(pixel * bytesPerPixel) + 2 ] = 0x00;
                    rgba[(pixel * bytesPerPixel) + 3 ] = 0xff;
                }
            }

        //Alpha cursor.
        } else if (cursorType == 1) {
            if (this._sock.rQwait("VMware cursor alpha encoding",
                                  (w * h * 4), 2)) {
                return false;
            }

            rgba = new Array(w * h * bytesPerPixel);

            for (let pixel = 0; pixel < (w * h); pixel++) {
                let data = this._sock.rQshift32();

                rgba[(pixel * 4)     ] = data >> 24 & 0xff; //r
                rgba[(pixel * 4) + 1 ] = data >> 16 & 0xff; //g
                rgba[(pixel * 4) + 2 ] = data >> 8 & 0xff;  //b
                rgba[(pixel * 4) + 3 ] = data & 0xff;       //a
            }

        } else {
            Log.Warn("The given cursor type is not supported: "
                      + cursorType + " given.");
            return false;
        }

        this._updateCursor(rgba, hotx, hoty, w, h);

        return true;
    }

    _handleCursor() {
        const hotx = this._FBU.x;  // hotspot-x
        const hoty = this._FBU.y;  // hotspot-y
        const w = this._FBU.width;
        const h = this._FBU.height;

        const pixelslength = w * h * 4;
        const masklength = Math.ceil(w / 8) * h;

        let bytes = pixelslength + masklength;
        if (this._sock.rQwait("cursor encoding", bytes)) {
            return false;
        }

        // Decode from BGRX pixels + bit mask to RGBA
        const pixels = this._sock.rQshiftBytes(pixelslength);
        const mask = this._sock.rQshiftBytes(masklength);
        let rgba = new Uint8Array(w * h * 4);

        let pixIdx = 0;
        for (let y = 0; y < h; y++) {
            for (let x = 0; x < w; x++) {
                let maskIdx = y * Math.ceil(w / 8) + Math.floor(x / 8);
                let alpha = (mask[maskIdx] << (x % 8)) & 0x80 ? 255 : 0;
                rgba[pixIdx    ] = pixels[pixIdx + 2];
                rgba[pixIdx + 1] = pixels[pixIdx + 1];
                rgba[pixIdx + 2] = pixels[pixIdx];
                rgba[pixIdx + 3] = alpha;
                pixIdx += 4;
            }
        }

        this._updateCursor(rgba, hotx, hoty, w, h);

        return true;
    }

    _handleDesktopName() {
        if (this._sock.rQwait("DesktopName", 4)) {
            return false;
        }

        let length = this._sock.rQshift32();

        if (this._sock.rQwait("DesktopName", length, 4)) {
            return false;
        }

        let name = this._sock.rQshiftStr(length);
        name = decodeUTF8(name, true);

        this._setDesktopName(name);

        return true;
    }

    _handleLedEvent() {
        if (this._sock.rQwait("LED Status", 1)) {
            return false;
        }

        let data = this._sock.rQshift8();
        // ScrollLock state can be retrieved with data & 1. This is currently not needed.
        let numLock = data & 2 ? true : false;
        let capsLock = data & 4 ? true : false;
        this._remoteCapsLock = capsLock;
        this._remoteNumLock = numLock;

        return true;
    }

    _handleExtendedDesktopSize() {
        if (this._sock.rQwait("ExtendedDesktopSize", 4)) {
            return false;
        }

        const numberOfScreens = this._sock.rQpeek8();

        let bytes = 4 + (numberOfScreens * 16);
        if (this._sock.rQwait("ExtendedDesktopSize", bytes)) {
            return false;
        }

        const firstUpdate = !this._supportsSetDesktopSize;
        this._supportsSetDesktopSize = true;

        this._sock.rQskipBytes(1);  // number-of-screens
        this._sock.rQskipBytes(3);  // padding

        for (let i = 0; i < numberOfScreens; i += 1) {
            // Save the id and flags of the first screen
            if (i === 0) {
                this._screenID = this._sock.rQshift32();    // id
                this._sock.rQskipBytes(2);                  // x-position
                this._sock.rQskipBytes(2);                  // y-position
                this._sock.rQskipBytes(2);                  // width
                this._sock.rQskipBytes(2);                  // height
                this._screenFlags = this._sock.rQshift32(); // flags
            } else {
                this._sock.rQskipBytes(16);
            }
        }

        /*
         * The x-position indicates the reason for the change:
         *
         *  0 - server resized on its own
         *  1 - this client requested the resize
         *  2 - another client requested the resize
         */

        // We need to handle errors when we requested the resize.
        if (this._FBU.x === 1 && this._FBU.y !== 0) {
            let msg = "";
            // The y-position indicates the status code from the server
            switch (this._FBU.y) {
                case 1:
                    msg = "Resize is administratively prohibited";
                    break;
                case 2:
                    msg = "Out of resources";
                    break;
                case 3:
                    msg = "Invalid screen layout";
                    break;
                default:
                    msg = "Unknown reason";
                    break;
            }
            Log.Warn("Server did not accept the resize request: "
                     + msg);
        } else {
            this._resize(this._FBU.width, this._FBU.height);
        }

        // Normally we only apply the current resize mode after a
        // window resize event. However there is no such trigger on the
        // initial connect. And we don't know if the server supports
        // resizing until we've gotten here.
        if (firstUpdate) {
            this._requestRemoteResize();
        }

        return true;
    }

    _handleDataRect() {
        let decoder = this._decoders[this._FBU.encoding];
        if (!decoder) {
            this._fail("Unsupported encoding (encoding: " +
                       this._FBU.encoding + ")");
            return false;
        }

        try {
            return decoder.decodeRect(this._FBU.x, this._FBU.y,
                                      this._FBU.width, this._FBU.height,
                                      this._sock, this._display,
                                      this._fbDepth);
        } catch (err) {
            this._fail("Error decoding rect: " + err);
            return false;
        }
    }

    _updateContinuousUpdates() {
        if (!this._enabledContinuousUpdates) { return; }

        RFB.messages.enableContinuousUpdates(this._sock, true, 0, 0,
                                             this._fbWidth, this._fbHeight);
    }

    _resize(width, height) {
        this._fbWidth = width;
        this._fbHeight = height;

        this._display.resize(this._fbWidth, this._fbHeight);

        // Adjust the visible viewport based on the new dimensions
        this._updateClip();
        this._updateScale();

        this._updateContinuousUpdates();

        // Keep this size until browser client size changes
        this._saveExpectedClientSize();
    }

    _xvpOp(ver, op) {
        if (this._rfbXvpVer < ver) { return; }
        Log.Info("Sending XVP operation " + op + " (version " + ver + ")");
        RFB.messages.xvpOp(this._sock, ver, op);
    }

    _updateCursor(rgba, hotx, hoty, w, h) {
        this._cursorImage = {
            rgbaPixels: rgba,
            hotx: hotx, hoty: hoty, w: w, h: h,
        };
        this._refreshCursor();
    }

    _shouldShowDotCursor() {
        // Called when this._cursorImage is updated
        if (!this._showDotCursor) {
            // User does not want to see the dot, so...
            return false;
        }

        // The dot should not be shown if the cursor is already visible,
        // i.e. contains at least one not-fully-transparent pixel.
        // So iterate through all alpha bytes in rgba and stop at the
        // first non-zero.
        for (let i = 3; i < this._cursorImage.rgbaPixels.length; i += 4) {
            if (this._cursorImage.rgbaPixels[i]) {
                return false;
            }
        }

        // At this point, we know that the cursor is fully transparent, and
        // the user wants to see the dot instead of this.
        return true;
    }

    _refreshCursor() {
        if (this._rfbConnectionState !== "connecting" &&
            this._rfbConnectionState !== "connected") {
            return;
        }
        const image = this._shouldShowDotCursor() ? RFB.cursors.dot : this._cursorImage;
        this._cursor.change(image.rgbaPixels,
                            image.hotx, image.hoty,
                            image.w, image.h
        );
    }

    static genDES(password, challenge) {
        const passwordChars = password.split('').map(c => c.charCodeAt(0));
        const key = legacyCrypto.importKey(
            "raw", passwordChars, { name: "DES-ECB" }, false, ["encrypt"]);
        return legacyCrypto.encrypt({ name: "DES-ECB" }, key, challenge);
    }
}

// Class Methods
RFB.messages = {
    keyEvent(sock, keysym, down) {
        sock.sQpush8(4); // msg-type
        sock.sQpush8(down);

        sock.sQpush16(0);

        sock.sQpush32(keysym);

        sock.flush();
    },

    QEMUExtendedKeyEvent(sock, keysym, down, keycode) {
        function getRFBkeycode(xtScanCode) {
            const upperByte = (keycode >> 8);
            const lowerByte = (keycode & 0x00ff);
            if (upperByte === 0xe0 && lowerByte < 0x7f) {
                return lowerByte | 0x80;
            }
            return xtScanCode;
        }

        sock.sQpush8(255); // msg-type
        sock.sQpush8(0); // sub msg-type

        sock.sQpush16(down);

        sock.sQpush32(keysym);

        const RFBkeycode = getRFBkeycode(keycode);

        sock.sQpush32(RFBkeycode);

        sock.flush();
    },

    pointerEvent(sock, x, y, mask) {
        sock.sQpush8(5); // msg-type

        sock.sQpush8(mask);

        sock.sQpush16(x);
        sock.sQpush16(y);

        sock.flush();
    },

    // Used to build Notify and Request data.
    _buildExtendedClipboardFlags(actions, formats) {
        let data = new Uint8Array(4);
        let formatFlag = 0x00000000;
        let actionFlag = 0x00000000;

        for (let i = 0; i < actions.length; i++) {
            actionFlag |= actions[i];
        }

        for (let i = 0; i < formats.length; i++) {
            formatFlag |= formats[i];
        }

        data[0] = actionFlag >> 24; // Actions
        data[1] = 0x00;             // Reserved
        data[2] = 0x00;             // Reserved
        data[3] = formatFlag;       // Formats

        return data;
    },

    extendedClipboardProvide(sock, formats, inData) {
        // Deflate incomming data and their sizes
        let deflator = new Deflator();
        let dataToDeflate = [];

        for (let i = 0; i < formats.length; i++) {
            // We only support the format Text at this time
            if (formats[i] != extendedClipboardFormatText) {
                throw new Error("Unsupported extended clipboard format for Provide message.");
            }

            // Change lone \r or \n into \r\n as defined in rfbproto
            inData[i] = inData[i].replace(/\r\n|\r|\n/gm, "\r\n");

            // Check if it already has \0
            let text = encodeUTF8(inData[i] + "\0");

            dataToDeflate.push( (text.length >> 24) & 0xFF,
                                (text.length >> 16) & 0xFF,
                                (text.length >>  8) & 0xFF,
                                (text.length & 0xFF));

            for (let j = 0; j < text.length; j++) {
                dataToDeflate.push(text.charCodeAt(j));
            }
        }

        let deflatedData = deflator.deflate(new Uint8Array(dataToDeflate));

        // Build data  to send
        let data = new Uint8Array(4 + deflatedData.length);
        data.set(RFB.messages._buildExtendedClipboardFlags([extendedClipboardActionProvide],
                                                           formats));
        data.set(deflatedData, 4);

        RFB.messages.clientCutText(sock, data, true);
    },

    extendedClipboardNotify(sock, formats) {
        let flags = RFB.messages._buildExtendedClipboardFlags([extendedClipboardActionNotify],
                                                              formats);
        RFB.messages.clientCutText(sock, flags, true);
    },

    extendedClipboardRequest(sock, formats) {
        let flags = RFB.messages._buildExtendedClipboardFlags([extendedClipboardActionRequest],
                                                              formats);
        RFB.messages.clientCutText(sock, flags, true);
    },

    extendedClipboardCaps(sock, actions, formats) {
        let formatKeys = Object.keys(formats);
        let data  = new Uint8Array(4 + (4 * formatKeys.length));

        formatKeys.map(x => parseInt(x));
        formatKeys.sort((a, b) =>  a - b);

        data.set(RFB.messages._buildExtendedClipboardFlags(actions, []));

        let loopOffset = 4;
        for (let i = 0; i < formatKeys.length; i++) {
            data[loopOffset]     = formats[formatKeys[i]] >> 24;
            data[loopOffset + 1] = formats[formatKeys[i]] >> 16;
            data[loopOffset + 2] = formats[formatKeys[i]] >> 8;
            data[loopOffset + 3] = formats[formatKeys[i]] >> 0;

            loopOffset += 4;
            data[3] |= (1 << formatKeys[i]); // Update our format flags
        }

        RFB.messages.clientCutText(sock, data, true);
    },

    clientCutText(sock, data, extended = false) {
        sock.sQpush8(6); // msg-type

        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding

        let length;
        if (extended) {
            length = toUnsigned32bit(-data.length);
        } else {
            length = data.length;
        }

        sock.sQpush32(length);
        sock.sQpushBytes(data);
        sock.flush();
    },

    setDesktopSize(sock, width, height, id, flags) {
        sock.sQpush8(251); // msg-type

        sock.sQpush8(0); // padding

        sock.sQpush16(width);
        sock.sQpush16(height);

        sock.sQpush8(1); // number-of-screens

        sock.sQpush8(0); // padding

        // screen array
        sock.sQpush32(id);
        sock.sQpush16(0); // x-position
        sock.sQpush16(0); // y-position
        sock.sQpush16(width);
        sock.sQpush16(height);
        sock.sQpush32(flags);

        sock.flush();
    },

    clientFence(sock, flags, payload) {
        sock.sQpush8(248); // msg-type

        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding

        sock.sQpush32(flags);

        sock.sQpush8(payload.length);
        sock.sQpushString(payload);

        sock.flush();
    },

    enableContinuousUpdates(sock, enable, x, y, width, height) {
        sock.sQpush8(150); // msg-type

        sock.sQpush8(enable);

        sock.sQpush16(x);
        sock.sQpush16(y);
        sock.sQpush16(width);
        sock.sQpush16(height);

        sock.flush();
    },

    pixelFormat(sock, depth, trueColor) {
        let bpp;

        if (depth > 16) {
            bpp = 32;
        } else if (depth > 8) {
            bpp = 16;
        } else {
            bpp = 8;
        }

        const bits = Math.floor(depth/3);

        sock.sQpush8(0); // msg-type

        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding

        sock.sQpush8(bpp);
        sock.sQpush8(depth);
        sock.sQpush8(0); // little-endian
        sock.sQpush8(trueColor ? 1 : 0);

        sock.sQpush16((1 << bits) - 1); // red-max
        sock.sQpush16((1 << bits) - 1); // green-max
        sock.sQpush16((1 << bits) - 1); // blue-max

        sock.sQpush8(bits * 0); // red-shift
        sock.sQpush8(bits * 1); // green-shift
        sock.sQpush8(bits * 2); // blue-shift

        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding
        sock.sQpush8(0); // padding

        sock.flush();
    },

    clientEncodings(sock, encodings) {
        sock.sQpush8(2); // msg-type

        sock.sQpush8(0); // padding

        sock.sQpush16(encodings.length);
        for (let i = 0; i < encodings.length; i++) {
            sock.sQpush32(encodings[i]);
        }

        sock.flush();
    },

    fbUpdateRequest(sock, incremental, x, y, w, h) {
        if (typeof(x) === "undefined") { x = 0; }
        if (typeof(y) === "undefined") { y = 0; }

        sock.sQpush8(3); // msg-type

        sock.sQpush8(incremental ? 1 : 0);

        sock.sQpush16(x);
        sock.sQpush16(y);
        sock.sQpush16(w);
        sock.sQpush16(h);

        sock.flush();
    },

    xvpOp(sock, ver, op) {
        sock.sQpush8(250); // msg-type

        sock.sQpush8(0); // padding

        sock.sQpush8(ver);
        sock.sQpush8(op);

        sock.flush();
    }
};

RFB.cursors = {
    none: {
        rgbaPixels: new Uint8Array(),
        w: 0, h: 0,
        hotx: 0, hoty: 0,
    },

    dot: {
        /* eslint-disable indent */
        rgbaPixels: new Uint8Array([
            255, 255, 255, 255,   0,   0,   0, 255, 255, 255, 255, 255,
              0,   0,   0, 255,   0,   0,   0,   0,   0,   0,  0,  255,
            255, 255, 255, 255,   0,   0,   0, 255, 255, 255, 255, 255,
        ]),
        /* eslint-enable indent */
        w: 3, h: 3,
        hotx: 1, hoty: 1,
    }
};

window['class'] = typeof class !== 'undefined' ? class : window['class'];
}})();

// noVNC namespace for RFB class access
if (typeof window.noVNC === 'undefined') {
    window.noVNC = {};
}
if (typeof RFB !== 'undefined') {
    window.noVNC.RFB = RFB;
}
if (typeof Display !== 'undefined') {
    window.noVNC.Display = Display;
}
if (typeof Websock !== 'undefined') {
    window.noVNC.Websock = Websock;
}
