function renderSwitcherChips() {
  var container = document.getElementById("switcherChips");
  if (!container) return;
  container.innerHTML = ChatState.switcherDefinitions.map(function (sw) {
    var isActive = ChatState.activeSwitchers.has(sw.id);
    return '<div class="switcher-chip' + (isActive ? ' active' : '') + '" ' +
      'data-switch-id="' + sw.id + '" ' +
      'style="--switcher-color: ' + (sw.color || '#666') + '; ' +
      (isActive ? 'color: white; background: ' + (sw.color || '#666') + '; ' : '') + '">' +
      '<span class="switcher-chip-icon">' + (sw.icon || '\u{1F500}') + '</span>' +
      '<span>' + sw.label + '</span></div>';
  }).join('');
}

function toggleSwitcher(switcherId) {
  if (ChatState.activeSwitchers.has(switcherId)) {
    ChatState.activeSwitchers.delete(switcherId);
  } else {
    ChatState.activeSwitchers.add(switcherId);
  }
  renderSwitcherChips();
}

function renderBotSwitchers(switchers) {
  if (!switchers || switchers.length === 0) return;
  var existingIds = {};
  ChatState.switcherDefinitions.forEach(function (sw) { existingIds[sw.id] = true; });
  switchers.forEach(function (sw) {
    if (!existingIds[sw.id]) {
      ChatState.switcherDefinitions.push({
        id: sw.id,
        label: sw.label || sw.id,
        icon: sw.icon || SWITCHER_ICONS[sw.id] || "\u{1F500}",
        color: sw.color || "#666",
      });
      existingIds[sw.id] = true;
    }
  });
  renderSwitcherChips();
  var container = document.getElementById("switchers");
  if (container && ChatState.switcherDefinitions.length > 0) {
    container.style.display = '';
  }
  var chipsContainer = document.getElementById("switcherChips");
  if (chipsContainer && !chipsContainer.dataset.hasClickHandler) {
    chipsContainer.addEventListener("click", function (e) {
      var chip = e.target.closest(".switcher-chip");
      if (chip) {
        var switcherId = chip.getAttribute("data-switch-id");
        if (switcherId) toggleSwitcher(switcherId);
      }
    });
    chipsContainer.dataset.hasClickHandler = "true";
  }
}
