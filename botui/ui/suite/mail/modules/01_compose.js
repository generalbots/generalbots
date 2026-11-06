/* Mail app — split from mail/mail.js for #1731 */

"use strict";

  if (window.GBAppLifecycle) GBAppLifecycle.begin("mail");

  var selectedEmails = new Set();
  var currentFolder = "inbox";

  function openCompose() {
    var modal = document.getElementById("compose-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeCompose() {
    var modal = document.getElementById("compose-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function minimizeCompose() {
    closeCompose();
  }

  function toggleCcBcc() {
    document.querySelectorAll(".cc-bcc").forEach(function (el) {
      el.style.display = el.style.display === "none" ? "flex" : "none";
    });
  }

  function toggleScheduleMenu() {
    var menu = document.getElementById("schedule-menu");
    if (menu) {
      menu.classList.toggle("show");
    }
  }

  function scheduleSend(option) {
    var date = new Date();
    switch (option) {
      case "tomorrow-morning":
        date.setDate(date.getDate() + 1);
        date.setHours(8, 0, 0, 0);
        break;
      case "tomorrow-afternoon":
        date.setDate(date.getDate() + 1);
        date.setHours(13, 0, 0, 0);
        break;
      case "monday":
        var daysUntilMonday = (8 - date.getDay()) % 7 || 7;
        date.setDate(date.getDate() + daysUntilMonday);
        date.setHours(8, 0, 0, 0);
        break;
    }
    confirmScheduleSend(date);
    toggleScheduleMenu();
  }

  function openCustomSchedule() {
    toggleScheduleMenu();
    var today = new Date().toISOString().split("T")[0];
    var dateInput = document.getElementById("schedule-date");
    if (dateInput) {
      dateInput.min = today;
      dateInput.value = today;
    }
    var modal = document.getElementById("schedule-modal");
    if (modal && modal.showModal) {
      modal.showModal();
    }
  }

  function closeScheduleModal() {
    var modal = document.getElementById("schedule-modal");
    if (modal && modal.close) {
      modal.close();
    }
  }

  function confirmSchedule() {
    var dateInput = document.getElementById("schedule-date");
    var timeInput = document.getElementById("schedule-time");
    if (dateInput && timeInput && dateInput.value && timeInput.value) {
      var scheduledDate = new Date(dateInput.value + "T" + timeInput.value);
      if (isNaN(scheduledDate.getTime())) {
        if (typeof window.showNotification === "function") {
          window.showNotification(
            "Please select a valid date and time",
            "error",
          );
        }
        return;
      }
      confirmScheduleSend(scheduledDate);
    } else {
      if (typeof window.showNotification === "function") {
        window.showNotification("Please select a date and time", "error");
      }
      return;
    }
    closeScheduleModal();
  }

  function confirmScheduleSend(date) {
    var form = document.getElementById("compose-form");
    if (form) {
      var input = document.createElement("input");
      input.type = "hidden";
      input.name = "scheduled_at";
      input.value = date.toISOString();
      form.appendChild(input);
      prepareSubmit();
      form.requestSubmit();
    }
  }

  function prepareSubmit() {
    var body = document.getElementById("compose-body");
    var hidden = document.getElementById("compose-body-hidden");
    if (body && hidden) {
      hidden.value = body.innerHTML;
    }
  }

  function formatText(command) {
    var selection = window.getSelection();
    if (selection.rangeCount) {
      var range = selection.getRangeAt(0);
      var tag = "";
      if (command === "bold") tag = "strong";
      else if (command === "italic") tag = "em";
      else if (command === "underline") tag = "u";

      if (tag) {
        var el = document.createElement(tag);
        try {
          range.surroundContents(el);
        } catch (e) {
          var contents = range.extractContents();
          el.appendChild(contents);
          range.insertNode(el);
        }
      }
    }
    var body = document.getElementById("compose-body");
    if (body) {
      body.focus();
    }
  }
