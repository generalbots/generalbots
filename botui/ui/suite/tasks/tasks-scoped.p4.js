
function buildProgressTreeHTML(manifest, totalSteps) {
  // Detailed logging to debug children/items
  const totalChildren = manifest.sections?.reduce(
    (sum, s) => sum + (s.children?.length || 0),
    0,
  );
  const totalItems = manifest.sections?.reduce((sum, s) => {
    let count = (s.items?.length || 0) + (s.item_groups?.length || 0);
    for (const c of s.children || []) {
      count += (c.items?.length || 0) + (c.item_groups?.length || 0);
    }
    return sum + count;
  }, 0);

  console.warn(
    "[BUILD_TREE] *** buildProgressTreeHTML ***",
    "\n  sections:",
    manifest.sections?.length,
    "\n  totalChildren:",
    totalChildren,
    "\n  totalItems:",
    totalItems,
    "\n  totalSteps:",
    totalSteps,
  );

  let html = '<div class="taskmd-tree">';

  for (const section of manifest.sections) {
    // Normalize status - backend sends "Running", "Completed", etc.
    const rawStatus = section.status || "Pending";
    const statusClass = rawStatus.toLowerCase();
    // ALWAYS expand all sections by default for visibility
    const shouldExpand = true;
    const globalCurrent =
      section.progress?.global_current || section.progress?.current || 0;

    const sectionChildCount = section.children?.length || 0;
    const sectionItemCount =
      (section.items?.length || 0) + (section.item_groups?.length || 0);

    console.log(
      "[BUILD_TREE] Section:",
      section.name,
      "| status:",
      rawStatus,
      "| children:",
      sectionChildCount,
      "| items:",
      sectionItemCount,
    );

    html += `
      <div class="tree-section ${statusClass}${shouldExpand ? " expanded" : ""}" data-section-id="${section.id}">
        <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
          <span class="tree-name">${escapeHtml(section.name)}</span>
          <span class="tree-step-badge">Step ${globalCurrent}/${totalSteps}</span>
          <span class="tree-status ${statusClass}">${rawStatus}</span>
          <span class="tree-section-dot ${statusClass}"></span>
        </div>
        <div class="tree-children">`;

    // Children (e.g., "Database Schema Design" under "Database & Models")
    if (section.children && section.children.length > 0) {
      console.log(
        "[BUILD_TREE]   -> Adding",
        section.children.length,
        "children to section",
        section.name,
      );
      for (const child of section.children) {
        const childRawStatus = child.status || "Pending";
        const childStatus = childRawStatus.toLowerCase();
        // ALWAYS expand all children by default for visibility
        const childShouldExpand = true;

        const childItemCount =
          (child.item_groups?.length || 0) + (child.items?.length || 0);
        console.log(
          "[BUILD_TREE]     Child:",
          child.name,
          "| status:",
          childRawStatus,
          "| items:",
          childItemCount,
        );

        html += `
          <div class="tree-child ${childStatus}${childShouldExpand ? " expanded" : ""}" data-child-id="${child.id}">
            <div class="tree-row tree-level-1" onclick="this.parentElement.classList.toggle('expanded')">
              <span class="tree-item-dot ${childStatus}"></span>
              <span class="tree-name">${escapeHtml(child.name)}</span>
              <span class="tree-step-badge">Step ${child.progress?.current || 0}/${child.progress?.total || 1}</span>
              <span class="tree-status ${childStatus}">${childRawStatus}</span>
            </div>
            <div class="tree-items">`;

        // Items within child (e.g., "email, password_hash, email_verified")
        const childItems = [
          ...(child.item_groups || []),
          ...(child.items || []),
        ];
        if (childItems.length > 0) {
          console.log(
            "[BUILD_TREE]       -> Adding",
            childItems.length,
            "items to child",
            child.name,
          );
        }
        for (const item of childItems) {
          html += buildItemHTML(item);
        }

        html += `</div></div>`;
      }
    }

    // Section-level items (items directly under section, not in children)
    const sectionItems = [
      ...(section.item_groups || []),
      ...(section.items || []),
    ];
    for (const item of sectionItems) {
      html += buildItemHTML(item);
    }

    html += `</div></div>`;
  }

  html += "</div>";

  // Final verification
  const hasChildren = html.includes("tree-child");
  const hasItems = html.includes("tree-item");
  console.warn(
    "[BUILD_TREE] *** Tree HTML built ***",
    "\n  length:",
    html.length,
    "\n  hasChildren:",
    hasChildren,
    "\n  hasItems:",
    hasItems,
  );

  return html;
}

function buildItemHTML(item) {
  const status = item.status?.toLowerCase() || "pending";
  const checkIcon = status === "completed" ? "✓" : "";
  const duration = item.duration_seconds
    ? item.duration_seconds >= 60
      ? `Duration: ${Math.floor(item.duration_seconds / 60)} min`
      : `Duration: ${item.duration_seconds} sec`
    : "";
  const name = item.name || item.display_name || "";

  return `
    <div class="tree-item ${status}" data-item-id="${item.id || name}">
      <span class="tree-item-dot ${status}"></span>
      <span class="tree-item-name">${escapeHtml(name)}</span>
      <span class="tree-item-duration">${duration}</span>
      <span class="tree-item-check ${status}">${checkIcon}</span>
    </div>`;
}

// Incremental update - only change what's different (prevents flicker)
function updateProgressTreeInPlace(tree, manifest, totalSteps) {
  for (const section of manifest.sections) {
    let sectionEl = tree.querySelector(`[data-section-id="${section.id}"]`);

    // If section not found by ID, try to find by name (backend may regenerate IDs)
    if (!sectionEl) {
      const allSections = tree.querySelectorAll(".tree-section");
      for (const el of allSections) {
        const nameEl = el.querySelector(":scope > .tree-row .tree-name");
        if (nameEl && nameEl.textContent === section.name) {
          sectionEl = el;
          // Update the data-section-id to the new ID for future lookups
          sectionEl.setAttribute("data-section-id", section.id);
          console.log(
            "[Manifest] Found section by name, updated ID:",
            section.name,
            "->",
            section.id,
          );
          break;
        }
      }
    }

    // If section still doesn't exist, create it dynamically (new section arrived!)
    if (!sectionEl) {
      console.log("[Manifest] Creating new section:", section.name);
      const rawStatus = section.status || "Pending";
      const statusClass = rawStatus.toLowerCase();
      const globalCurrent =
        section.progress?.global_current || section.progress?.current || 0;

      const sectionHtml = `
        <div class="tree-section ${statusClass} expanded" data-section-id="${section.id}">
          <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
            <span class="tree-name">${escapeHtml(section.name)}</span>
            <span class="tree-step-badge">Step ${globalCurrent}/${totalSteps}</span>
            <span class="tree-status ${statusClass}">${rawStatus}</span>
            <span class="tree-section-dot ${statusClass}"></span>
          </div>
          <div class="tree-children"></div>
        </div>`;

      tree.insertAdjacentHTML("beforeend", sectionHtml);
      sectionEl = tree.querySelector(`[data-section-id="${section.id}"]`);
    }

    const rawStatus = section.status || "Pending";
    const statusClass = rawStatus.toLowerCase();
    const globalCurrent =
      section.progress?.global_current || section.progress?.current || 0;
    const isExpanded = sectionEl.classList.contains("expanded");

    // ALWAYS keep sections expanded for visibility
    const shouldExpand = true;
    const newClasses = `tree-section ${statusClass}${shouldExpand ? " expanded" : ""}`;
    if (sectionEl.className !== newClasses) {
      sectionEl.className = newClasses;
    }

    // Update step badge text only if changed
    const stepBadge = sectionEl.querySelector(
      ":scope > .tree-row .tree-step-badge",
    );
    const stepText = `Step ${globalCurrent}/${totalSteps}`;
    if (stepBadge && stepBadge.textContent !== stepText) {
      stepBadge.textContent = stepText;
    }

    // Update status text and class only if changed
    const statusEl = sectionEl.querySelector(":scope > .tree-row .tree-status");
    if (statusEl) {
      if (statusEl.textContent !== rawStatus) {
        statusEl.textContent = rawStatus;
      }
      const statusClasses = `tree-status ${statusClass}`;
      if (statusEl.className !== statusClasses) {
        statusEl.className = statusClasses;
      }
    }

    // Update section dot
    const sectionDot = sectionEl.querySelector(
      ":scope > .tree-row .tree-section-dot",
    );
    if (sectionDot) {
      const dotClasses = `tree-section-dot ${statusClass}`;
      if (sectionDot.className !== dotClasses) {
        sectionDot.className = dotClasses;
      }
    }

    // Update children
    if (section.children) {
      for (const child of section.children) {
        updateChildInPlace(sectionEl, child);
      }
    }

    // Update section-level items
    const childrenContainer = sectionEl.querySelector(".tree-children");
    if (childrenContainer) {
      updateItemsInPlace(childrenContainer, [
        ...(section.item_groups || []),
        ...(section.items || []),
      ]);
    }
  }
}

function updateChildInPlace(sectionEl, child) {
  let childEl = sectionEl.querySelector(`[data-child-id="${child.id}"]`);

  // If child not found by ID, try to find by name (backend may regenerate IDs)
  if (!childEl) {
    const allChildren = sectionEl.querySelectorAll(".tree-child");
    for (const el of allChildren) {
      const nameEl = el.querySelector(":scope > .tree-row .tree-name");
      if (nameEl && nameEl.textContent === child.name) {
        childEl = el;
        // Update the data-child-id to the new ID for future lookups
        childEl.setAttribute("data-child-id", child.id);
        console.log(
          "[Manifest] Found child by name, updated ID:",
          child.name,
          "->",
          child.id,
        );
        break;
      }
    }
  }

  // If child still doesn't exist in DOM, create it (and auto-expand new children!)
  if (!childEl) {
    const childrenContainer = sectionEl.querySelector(".tree-children");
    if (!childrenContainer) return;

    const rawStatus = child.status || "Pending";
    const statusClass = rawStatus.toLowerCase();
    // NEW: Always expand newly created children so they're visible immediately
    const childHasItems =
      (child.item_groups?.length || 0) + (child.items?.length || 0) > 0;
    const shouldExpand = true; // Always expand new children for visibility

    console.log(
      "[Manifest] Creating new child:",
      child.name,
      "status:",
      rawStatus,
      "expanded:",
      shouldExpand,
    );

    const childHtml = `
      <div class="tree-child ${statusClass}${shouldExpand ? " expanded" : ""}" data-child-id="${child.id}">
        <div class="tree-row tree-level-1" onclick="this.parentElement.classList.toggle('expanded')">
          <span class="tree-item-dot ${statusClass}"></span>
          <span class="tree-name">${escapeHtml(child.name)}</span>
          <span class="tree-step-badge">Step ${child.progress?.current || 0}/${child.progress?.total || 1}</span>
          <span class="tree-status ${statusClass}">${rawStatus}</span>
        </div>
        <div class="tree-items"></div>
      </div>`;

    childrenContainer.insertAdjacentHTML("beforeend", childHtml);
    childEl = sectionEl.querySelector(`[data-child-id="${child.id}"]`);

    // Add items to the newly created child
    const itemsContainer = childEl.querySelector(".tree-items");
    if (itemsContainer) {
      const allItems = [...(child.item_groups || []), ...(child.items || [])];
      for (const item of allItems) {
        itemsContainer.insertAdjacentHTML("beforeend", buildItemHTML(item));
      }
    }
    return;
  }

  const rawStatus = child.status || "Pending";
  const statusClass = rawStatus.toLowerCase();
  const isExpanded = childEl.classList.contains("expanded");

  // ALWAYS keep children expanded for visibility
  const shouldExpand = true;
  const newClasses = `tree-child ${statusClass}${shouldExpand ? " expanded" : ""}`;
  if (childEl.className !== newClasses) {
    childEl.className = newClasses;
  }

  // Update step badge
  const stepBadge = childEl.querySelector(
    ":scope > .tree-row .tree-step-badge",
  );
  const stepText = `Step ${child.progress?.current || 0}/${child.progress?.total || 1}`;
  if (stepBadge && stepBadge.textContent !== stepText) {
    stepBadge.textContent = stepText;
  }

  // Update status
  const statusEl = childEl.querySelector(":scope > .tree-row .tree-status");
  if (statusEl) {
    if (statusEl.textContent !== rawStatus) {
      statusEl.textContent = rawStatus;
    }
    const statusClasses = `tree-status ${statusClass}`;
    if (statusEl.className !== statusClasses) {
      statusEl.className = statusClasses;
    }
  }

  // Update child dot
  const childDot = childEl.querySelector(":scope > .tree-row .tree-item-dot");
  if (childDot) {
    const dotClasses = `tree-item-dot ${statusClass}`;
    if (childDot.className !== dotClasses) {
      childDot.className = dotClasses;
    }
  }

  // Update items within child
  const itemsContainer = childEl.querySelector(".tree-items");
  if (itemsContainer) {
    updateItemsInPlace(itemsContainer, [
      ...(child.item_groups || []),
      ...(child.items || []),
    ]);
  }
}
