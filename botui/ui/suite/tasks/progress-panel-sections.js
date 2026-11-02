
function toggleLogSection(header) {
  const section = header.closest(".log-section");
  if (section) {
    section.classList.toggle("expanded");
  }
}

function toggleLogChild(header) {
  const child = header.closest(".log-child");
  if (child) {
    child.classList.toggle("expanded");
  }
}

function viewSectionDetails(sectionId) {
  ProgressPanel.viewDetails(sectionId);
}

function viewChildDetails(childId) {
  ProgressPanel.viewChildDetails(childId);
}

window.ProgressPanel = ProgressPanel;
