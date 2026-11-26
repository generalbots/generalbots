"use strict";

(function () {
    var A = window.GBAssistant;

    A.addMessageHeading = function (el, user, timestamp) {
        if (el.querySelector(".gb-msg-heading")) { return; }
        var head = document.createElement("div");
        head.className = "gb-msg-heading";
        var account = A.accountProfile || {};
        var profile = A.assistantProfile || {};
        var name = user ? (account.display_name || A.userName || ((A.locale || "").startsWith("pt") ? "Você" : "You")) : (profile.name || "General Bots");
        var badge = document.createElement("span");
        badge.className = "gb-msg-avatar";
        badge.setAttribute("aria-hidden", "true");
        badge.textContent = name.trim().split(/\s+/).slice(0, 2).map(function (word) {
            return word.charAt(0).toUpperCase();
        }).join("");
        if (user && /^#[0-9a-f]{6}$/i.test(account.avatar_color || "")) {
            badge.style.setProperty("--gb-msg-avatar-color", account.avatar_color);
            var color = account.avatar_color.slice(1).match(/../g).map(function (part) {
                var value = parseInt(part, 16) / 255;
                return value <= 0.04045 ? value / 12.92 : Math.pow((value + 0.055) / 1.055, 2.4);
            });
            var luminance = color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722;
            badge.classList.add(luminance > 0.179 ? "gb-msg-avatar--ink" : "gb-msg-avatar--paper");
        }
        if (!user && /^data:image\/(png|jpeg|webp);base64,[a-z0-9+/=]+$/i.test(profile.photo || "")) {
            var photo = document.createElement("img");
            photo.src = profile.photo;
            photo.alt = "";
            badge.replaceChildren(photo);
        } else if (!user && profile.name && A.assistantProfileIcon) {
            badge.innerHTML = A.assistantProfileIcon(14);
        }
        var label = document.createElement("strong");
        label.textContent = name;
        head.append(badge, label);
        var date = timestamp ? new Date(timestamp) : null;
        if (date && Number.isFinite(date.getTime())) {
            var time = document.createElement("time");
            time.dateTime = date.toISOString();
            time.textContent = date.toLocaleTimeString(A.locale || undefined, { hour: "2-digit", minute: "2-digit" });
            head.appendChild(time);
        }
        el.prepend(head);
    };

    function refreshHeadings() {
        document.querySelectorAll("#gbMessages > :is(.gb-msg-user, .gb-msg-assistant)").forEach(function (turn) {
            var heading = turn.querySelector(".gb-msg-heading");
            if (!heading) { return; }
            var time = heading.querySelector("time");
            heading.remove();
            A.addMessageHeading(turn, turn.classList.contains("gb-msg-user"), time && time.dateTime);
        });
    }

    document.addEventListener("gb-assistant-profile", refreshHeadings);
    document.addEventListener("gb-account-profile", refreshHeadings);
})();
