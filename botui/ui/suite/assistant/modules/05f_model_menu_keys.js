"use strict";
/**
 * 05f_model_menu_keys.js — Keyboard navigation for the model picker menu.
 *
 * Loaded after 05b_model_picker.js; hooks into A.onModelMenuOpen.
 * Adds arrow-key, Home/End, Enter/Space and collapse/expand behaviour.
 * ARIA roles (menu / menuitem / menuitemradio) are added here.
 */
(function () {
    var A = window.GBAssistant;

    /** Collect every visible, focusable item in document order.
     *  Headings (group-head) and non-disabled entries (menu-item) qualify. */
    function visibleItems(menu) {
        var all = menu.querySelectorAll(
            ".gb-model-group-head, .gb-model-menu-item, .gb-model-menu-action"
        );
        var out = [];
        for (var i = 0; i < all.length; i++) {
            var el = all[i];
            if (el.disabled) { continue; }
            if (el.classList.contains("gb-model-menu-disabled")) { continue; }
            /* Items inside a collapsed group are hidden. */
            var group = el.closest(".gb-model-group");
            if (group && !group.classList.contains("open") &&
                !el.classList.contains("gb-model-group-head")) { continue; }
            out.push(el);
        }
        return out;
    }

    function isHeading(el) { return el.classList.contains("gb-model-group-head"); }

    /** Apply ARIA roles to the freshly built menu. */
    function applyAria(menu) {
        menu.setAttribute("role", "menu");
        var items = menu.querySelectorAll(".gb-model-menu-item");
        for (var i = 0; i < items.length; i++) {
            var el = items[i];
            if (el.classList.contains("gb-model-menu-action")) {
                el.setAttribute("role", "menuitem");
            } else {
                el.setAttribute("role", "menuitemradio");
                el.setAttribute("aria-checked", el.classList.contains("active") ? "true" : "false");
            }
        }
        var actions = menu.querySelectorAll(".gb-model-menu-action");
        for (var j = 0; j < actions.length; j++) {
            actions[j].setAttribute("role", "menuitem");
        }
        var heads = menu.querySelectorAll(".gb-model-group-head");
        for (var k = 0; k < heads.length; k++) {
            heads[k].setAttribute("role", "menuitem");
        }
    }

    /** Set roving tabindex: only `target` is tabbable. */
    function rove(menu, target) {
        var all = menu.querySelectorAll(
            ".gb-model-group-head, .gb-model-menu-item, .gb-model-menu-action"
        );
        for (var i = 0; i < all.length; i++) {
            all[i].setAttribute("tabindex", all[i] === target ? "0" : "-1");
        }
        if (target) { target.focus(); }
    }

    function handleKeydown(menu, anchor, e) {
        var items = visibleItems(menu);
        if (!items.length) { return; }
        var idx = items.indexOf(document.activeElement);
        var handled = true;

        switch (e.key) {
            case "ArrowDown":
                idx = idx < 0 ? 0 : (idx + 1) % items.length;
                rove(menu, items[idx]);
                break;
            case "ArrowUp":
                idx = idx < 0 ? items.length - 1 : (idx - 1 + items.length) % items.length;
                rove(menu, items[idx]);
                break;
            case "Home":
                rove(menu, items[0]);
                break;
            case "End":
                rove(menu, items[items.length - 1]);
                break;
            case "ArrowRight":
                if (idx >= 0 && isHeading(items[idx])) {
                    var group = items[idx].closest(".gb-model-group");
                    if (group && !group.classList.contains("open")) {
                        items[idx].click();
                    }
                }
                break;
            case "ArrowLeft":
                if (idx >= 0) {
                    if (isHeading(items[idx])) {
                        var grp = items[idx].closest(".gb-model-group");
                        if (grp && grp.classList.contains("open")) {
                            items[idx].click();
                        }
                    } else {
                        var parent = items[idx].closest(".gb-model-group");
                        if (parent) {
                            var head = parent.querySelector(".gb-model-group-head");
                            if (head) { rove(menu, head); }
                        }
                    }
                }
                break;
            case "Enter":
            case " ":
                if (idx >= 0) { items[idx].click(); }
                break;
            case "Escape":
                /* Escape is already handled by closeModelMenuOnEscape in 05b;
                   let it bubble. */
                handled = false;
                break;
            default:
                handled = false;
                break;
        }
        if (handled) { e.preventDefault(); e.stopPropagation(); }
    }

    /** Called by 05b_model_picker.js each time a menu opens. */
    A.onModelMenuOpen = function (menu, anchor) {
        applyAria(menu);
        /* Focus the selected entry, or the first item. */
        var target = menu.querySelector(".gb-model-menu-item.active") ||
            visibleItems(menu)[0] || null;
        rove(menu, target);
        var handler = function (e) { handleKeydown(menu, anchor, e); };
        menu.addEventListener("keydown", handler);
    };
})();
