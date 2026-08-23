) {
  // Normalize task IDs for comparison (both to lowercase string)
  const normalizedTaskId = String(taskId).toLowerCase().trim();
  const normalizedSelectedId = TasksState.selectedTaskId
    ? String(TasksState.selectedTaskId).toLowerCase().trim()
    : null;

  console.warn(
    "[MANIFEST] *** renderManifestProgress ***",
    "\n  taskId:",
    taskId,
    "\n  selectedTaskId:",
    TasksState.selectedTaskId,
    "\n  normalized match:",
    normalizedTaskId === normalizedSelectedId,
    "\n  sections:",
    manifest?.sections?.length,
    "\n  section statuses:",
    manifest?.sections?.map((s) => `${s.name}:${s.status}`).join(", "),
    "\n  retryCount:",
    retryCount,
  );

  // Always store the manifest for this task (use normalized ID for consistent lookup)
  pendingManifestUpdates.set(normalizedTaskId, manifest);

  // Only render UI if this is the selected task (use normalized comparison)
  if (normalizedSelectedId !== normalizedTaskId) {
    console.log(
      "[Manifest] Storing manifest but not rendering - not selected task",
      "\n  taskId:",
      normalizedTaskId,
      "\n  selectedId:",
      normalizedSelectedId,
    );
    return;
  }

  // Try multiple selectors to find the progress log element
  let progressLog = document.getElementById(`progress-log-${taskId}`);
  console.log(
    "[MANIFEST] Looking for progress-log-" + taskId + ", found:",
    !!progressLog,
  );
  if (!progressLog) {
    progressLog = document.querySelector(".taskmd-progress-content");
    console.log(
      "[MANIFEST] Looking for .taskmd-progress-content, found:",
      !!progressLog,
    );
  }

  if (!progressLog) {
    console.warn(
      "[MANIFEST] No progress log element found, retry:",
      retryCount,
    );
    // If task is selected but element not yet loaded, retry after a delay
    if (retryCount < 5) {
      pendingManifestUpdates.set(normalizedTaskId, manifest);
      setTimeout(
        () => {
          const pending = pendingManifestUpdates.get(normalizedTaskId);
          const currentSelectedNormalized = TasksState.selectedTaskId
            ? String(TasksState.selectedTaskId).toLowerCase().trim()
            : null;
          if (pending && currentSelectedNormalized === normalizedTaskId) {
            renderManifestProgress(taskId, pending, retryCount + 1);
          }
        },
        150 * (retryCount + 1),
      );
    }
    return;
  }

  // Clear pending update (use normalized ID)
  pendingManifestUpdates.delete(normalizedTaskId);

  if (!manifest || !manifest.sections) {
    console.log("[Manifest] No sections in manifest, skipping render");
    return;
  }

  const totalSteps = manifest.progress?.total || 60;

  console.warn(
    "[MANIFEST] Rendering progress tree:",
    "\n  totalSteps:",
    totalSteps,
    "\n  sections:",
    manifest.sections.length,
    "\n  progressLog element:",
    progressLog?.id || progressLog?.className,
  );

  // Update STATUS section if exists
  updateStatusSection(manifest);

  // Check if tree exists - if not, create it; if yes, update incrementally
  // Clear any "progress-empty" placeholder first
  const emptyPlaceholder = progressLog.querySelector(".progress-empty");
  if (emptyPlaceholder) {
    console.log("[Manifest] Removing progress-empty placeholder");
    emptyPlaceholder.remove();
  }

  let tree = progressLog.querySelector(".taskmd-tree");
  console.log("[Manifest] Existing tree found:", !!tree);

  // Check if we need to rebuild the tree (structure changed significantly)
  let shouldRebuild = !tree;
  if (tree && manifest.sections) {
    const existingSections = tree.querySelectorAll(".tree-section");
    const existingChildren = tree.querySelectorAll(".tree-child");
    const existingItems = tree.querySelectorAll(".tree-item");
    const newChildCount = manifest.sections.reduce(
      (sum, s) => sum + (s.children?.length || 0),
      0,
    );
    const newItemCount = manifest.sections.reduce((sum, s) => {
      let count = (s.items?.length || 0) + (s.item_groups?.length || 0);
      for (const child of s.children || []) {
        count += (child.items?.length || 0) + (child.item_groups?.length || 0);
      }
      return sum + count;
    }, 0);

    // Check if section names match (IDs may change but names should be stable)
    const existingNames = new Set(
      Array.from(existingSections).map(
        (el) => el.querySelector(".tree-name")?.textContent,
      ),
    );
    const newNames = new Set(manifest.sections.map((s) => s.name));
    const namesMatch =
      existingNames.size === newNames.size &&
      [...existingNames].every((n) => newNames.has(n));

    // Rebuild if:
    // - section count changed
    // - children appeared where there were none
    // - items appeared where there were none
    // - section names don't match (structure completely different)
    if (
      existingSections.length !== manifest.sections.length ||
      (existingChildren.length === 0 && newChildCount > 0) ||
      (existingItems.length === 0 && newItemCount > 0) ||
      !namesMatch
    ) {
      console.log(
        "[Manifest] Structure changed significantly, REBUILDING tree",
        "\n  existing sections:",
        existingSections.length,
        "-> new:",
        manifest.sections.length,
        "\n  existing children:",
        existingChildren.length,
        "-> new:",
        newChildCount,
        "\n  existing items:",
        existingItems.length,
        "-> new:",
        newItemCount,
        "\n  names match:",
        namesMatch,
      );
      shouldRebuild = true;
    }
  }

  try {
    if (shouldRebuild) {
      // Full rebuild - create the tree structure from scratch
      console.log("[Manifest] === REBUILDING TREE FROM SCRATCH ===");
      const treeHTML = buildProgressTreeHTML(manifest, totalSteps);
      console.log(
        "[Manifest] Tree HTML length:",
        treeHTML.length,
        "sections:",
        manifest.sections.length,
        "with children:",
        manifest.sections.filter((s) => s.children?.length > 0).length,
      );
      progressLog.innerHTML = treeHTML;
      // Auto-expand all sections and children for visibility
      progressLog
        .querySelectorAll(".tree-section, .tree-child")
        .forEach((el) => {
          el.classList.add("expanded");
        });
      const expandedCount = progressLog.querySelectorAll(".expanded").length;
      const childCount = progressLog.querySelectorAll(".tree-child").length;
      const itemCount = progressLog.querySelectorAll(".tree-item").length;
      console.log(
        "[Manifest] Tree rebuilt:",
        expandedCount,
        "expanded,",
        childCount,
        "children,",
        itemCount,
        "items",
      );
      // Auto-scroll to running item
      scrollToRunningItem(progressLog);
    } else {
      // Incremental update - only update what changed (no flicker)
      console.log("[Manifest] Updating tree in place");
      updateProgressTreeInPlace(tree, manifest, totalSteps);
      console.log("[Manifest] Tree updated successfully");
      // Auto-scroll to running item
      scrollToRunningItem(progressLog);
    }
  } catch (treeError) {
    console.error(
      "[Manifest] Error building/updating tree:",
      treeError,
      "\n  stack:",
      treeError.stack,
    );
  }

  // Update terminal stats
  updateTerminalStats(taskId, manifest);
}

// Auto-scroll progress log to show the currently running item
function scrollToRunningItem(progressLog) {
  if (!progressLog) return;

  // Find the running item (highest priority) or running section
  const runningItem = progressLog.querySelector(".tree-item.running");
  const runningSection = progressLog.querySelector(".tree-section.running");
  const runningChild = progressLog.querySelector(".tree-child.running");

  const targetElement = runningItem || runningChild || runningSection;

  if (targetElement) {
    // Scroll the target into view within the progress log container
    targetElement.scrollIntoView({
      behavior: "smooth",
      block: "center",
      inline: "nearest",
    });
  }
}

// Update STATUS section from task_progress messages
function updateStatusFromProgress(message, step) {
  const statusContent = document.querySelector(".taskmd-status-content");
  if (!statusContent) return;

  // Update current action text
  const actionText = statusContent.querySelector(
    ".status-current .status-text",
  );
  if (actionText && message) {
    actionText.textContent = message;
  }

  // Update the status dot to show activity
  const statusDot = statusContent.querySelector(".status-dot");
  if (statusDot) {
    statusDot.classList.add("active");
  }
}

function updateStatusSection(manifest) {
  const statusContent = document.querySelector(".taskmd-status-content");
  if (!statusContent) return;

  // Update current action text only if changed
  const actionText = statusContent.querySelector(
    ".status-current .status-text",
  );
  const currentAction =
    manifest.status?.current_action ||
    manifest.current_status?.current_action ||
    "Processing...";
  if (actionText && actionText.textContent !== currentAction) {
    actionText.textContent = currentAction;
  }

  // Update runtime text only
  const runtimeEl = statusContent.querySelector(".status-main .status-time");
  const runtime =
    manifest.status?.runtime_display || manifest.runtime || "Not started";
  if (runtimeEl) {
    // Only update text content, preserve indicator
    const indicator = runtimeEl.querySelector(".status-indicator");
    if (!indicator) {
      runtimeEl.innerHTML = `Runtime: ${runtime} <span class="status-indicator"></span>`;
    } else {
      runtimeEl.firstChild.textContent = `Runtime: ${runtime} `;
    }
  }

  // Update estimated text only
  const estimatedEl = statusContent.querySelector(
    ".status-current .status-time",
  );
  const estimated =
    manifest.status?.estimated_display ||
    (manifest.estimated_seconds
      ? `${manifest.estimated_seconds} sec`
      : "calculating...");
  if (estimatedEl) {
    const gear = estimatedEl.querySelector(".status-gear");
    if (!gear) {
      estimatedEl.innerHTML = `Estimated: ${estimated} <span class="status-gear">⚙</span>`;
    } else {
      estimatedEl.firstChild.textContent = `Estimated: ${estimated} `;
    }
  }
}
