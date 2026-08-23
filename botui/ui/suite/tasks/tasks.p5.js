
function updateItemsInPlace(container, items) {
  if (!container || !items) return;

  for (const item of items) {
    const itemId = item.id || item.name || item.display_name;
    const itemName = item.name || item.display_name;
    let itemEl = container.querySelector(`[data-item-id="${itemId}"]`);

    // If item not found by ID, try to find by name (backend may regenerate IDs)
    if (!itemEl && itemName) {
      const allItems = container.querySelectorAll(".tree-item");
      for (const el of allItems) {
        const nameEl = el.querySelector(".tree-item-name");
        if (nameEl && nameEl.textContent === itemName) {
          itemEl = el;
          // Update the data-item-id to the new ID for future lookups
          itemEl.setAttribute("data-item-id", itemId);
          break;
        }
      }
    }

    if (!itemEl) {
      // New item - append it
      container.insertAdjacentHTML("beforeend", buildItemHTML(item));
      continue;
    }

    const rawStatus = item.status || "Pending";
    const status = rawStatus.toLowerCase();

    // Update item class
    const newClasses = `tree-item ${status}`;
    if (itemEl.className !== newClasses) {
      itemEl.className = newClasses;
    }

    // Update dot
    const dot = itemEl.querySelector(".tree-item-dot");
    if (dot) {
      const dotClasses = `tree-item-dot ${status}`;
      if (dot.className !== dotClasses) {
        dot.className = dotClasses;
      }
    }

    // Update check
    const check = itemEl.querySelector(".tree-item-check");
    if (check) {
      const checkClasses = `tree-item-check ${status}`;
      if (check.className !== checkClasses) {
        check.className = checkClasses;
      }
      const checkText = status === "completed" ? "✓" : "";
      if (check.textContent !== checkText) {
        check.textContent = checkText;
      }
    }

    // Update duration
    const durationEl = itemEl.querySelector(".tree-item-duration");
    if (durationEl && item.duration_seconds) {
      const durationText =
        item.duration_seconds >= 60
          ? `Duration: ${Math.floor(item.duration_seconds / 60)} min`
          : `Duration: ${item.duration_seconds} sec`;
      if (durationEl.textContent !== durationText) {
        durationEl.textContent = durationText;
      }
    }
  }
}

function updateTerminalStats(taskId, manifest) {
  const processedEl = document.getElementById(`terminal-processed-${taskId}`);
  if (processedEl && manifest.terminal?.stats?.processed) {
    processedEl.textContent = manifest.terminal.stats.processed;
  }

  const etaEl = document.getElementById(`terminal-eta-${taskId}`);
  if (etaEl && manifest.terminal?.stats?.eta) {
    etaEl.textContent = manifest.terminal.stats.eta;
  }
}

function escapeHtml(text) {
  if (!text) return "";
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function updateActivityMetrics(activity) {
  if (!activity) return;

  const metricsEl = document.getElementById("floating-activity-metrics");
  if (!metricsEl) return;

  let html = "";

  if (activity.phase) {
    html += `<div class="metric-row"><span class="metric-label">Phase:</span> <span class="metric-value phase-${activity.phase}">${activity.phase.toUpperCase()}</span></div>`;
  }

  if (activity.items_processed !== undefined) {
    const total = activity.items_total ? `/${activity.items_total}` : "";
    html += `<div class="metric-row"><span class="metric-label">Processed:</span> <span class="metric-value">${activity.items_processed}${total} items</span></div>`;
  }

  if (activity.speed_per_min) {
    html += `<div class="metric-row"><span class="metric-label">Speed:</span> <span class="metric-value">~${activity.speed_per_min.toFixed(1)} items/min</span></div>`;
  }

  if (activity.eta_seconds) {
    const mins = Math.floor(activity.eta_seconds / 60);
    const secs = activity.eta_seconds % 60;
    const eta = mins > 0 ? `${mins}m ${secs}s` : `${secs}s`;
    html += `<div class="metric-row"><span class="metric-label">ETA:</span> <span class="metric-value">${eta}</span></div>`;
  }

  if (activity.bytes_processed) {
    const kb = (activity.bytes_processed / 1024).toFixed(1);
    html += `<div class="metric-row"><span class="metric-label">Generated:</span> <span class="metric-value">${kb} KB</span></div>`;
  }

  if (activity.tokens_used) {
    html += `<div class="metric-row"><span class="metric-label">Tokens:</span> <span class="metric-value">${activity.tokens_used.toLocaleString()}</span></div>`;
  }

  if (activity.files_created && activity.files_created.length > 0) {
    html += `<div class="metric-row"><span class="metric-label">Files:</span> <span class="metric-value">${activity.files_created.length} created</span></div>`;
  }

  if (activity.tables_created && activity.tables_created.length > 0) {
    html += `<div class="metric-row"><span class="metric-label">Tables:</span> <span class="metric-value">${activity.tables_created.length} synced</span></div>`;
  }

  if (activity.current_item) {
    html += `<div class="metric-row current-item"><span class="metric-label">Current:</span> <span class="metric-value">${activity.current_item}</span></div>`;
  }

  metricsEl.innerHTML = html;
}

function logFinalStats(activity) {
  if (!activity) return;

  addAgentLog("info", "─────────────────────────────────");
  addAgentLog("info", "GENERATION COMPLETE");

  if (activity.files_created && activity.files_created.length > 0) {
    addAgentLog("success", `Files created: ${activity.files_created.length}`);
    activity.files_created.forEach((f) => addAgentLog("info", `  • ${f}`));
  }

  if (activity.tables_created && activity.tables_created.length > 0) {
    addAgentLog("success", `Tables synced: ${activity.tables_created.length}`);
    activity.tables_created.forEach((t) => addAgentLog("info", `  • ${t}`));
  }

  if (activity.bytes_processed) {
    const kb = (activity.bytes_processed / 1024).toFixed(1);
    addAgentLog("info", `Total size: ${kb} KB`);
  }

  addAgentLog("info", "─────────────────────────────────");
}

// =============================================================================
// FLOATING PROGRESS PANEL
// =============================================================================

// Update terminal in the detail panel with real-time data
function updateDetailTerminal(taskId, message, step, activity) {
  // Try multiple selectors to find the terminal
  let terminalOutput = document.getElementById(`terminal-output-${taskId}`);

  if (!terminalOutput) {
    // Try the currently visible terminal output (any task)
    terminalOutput = document.querySelector(".taskmd-terminal-output");
  }

  if (!terminalOutput) {
    // Try generic terminal output
    terminalOutput = document.querySelector(".terminal-output-rich");
  }

  if (!terminalOutput) {
    console.log("[Terminal] No terminal element found for task:", taskId);
    return;
  }

  // Ensure message is a string
  const messageStr =
    typeof message === "string" ? message : JSON.stringify(message) || "";
  console.log(
    "[Terminal] Adding line to terminal:",
    messageStr.substring(0, 50),
  );
  addTerminalLine(terminalOutput, messageStr, step, activity);
}

// Simple markdown parser for terminal/LLM output
function parseMarkdown(text) {
  if (!text) return "";

  let html = text;

  // Escape HTML first
  html = html
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  // Code blocks (```language\ncode```) - must be before other replacements
  html = html.replace(/```(\w*)\n?([\s\S]*?)```/g, (match, lang, code) => {
    const langClass = lang ? ` data-lang="${lang}"` : "";
    return `<pre class="terminal-code"${langClass}><code>${code.trim()}</code></pre>`;
  });

  // Headers (# ## ###)
  html = html.replace(/^### (.+)$/gm, '<h3 class="terminal-h3">$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2 class="terminal-h2">$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1 class="terminal-h1">$1</h1>');

  // Bold (**text** or __text__)
  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/__(.+?)__/g, "<strong>$1</strong>");

  // Italic (*text* or _text_)
  html = html.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  html = html.replace(/_([^_]+)_/g, "<em>$1</em>");

  // Inline code (`code`)
  html = html.replace(
    /`([^`]+)`/g,
    '<code class="terminal-inline-code">$1</code>',
  );

  // Links [text](url)
  html = html.replace(
    /\[([^\]]+)\]\(([^)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener">$1</a>',
  );

  // Unordered lists (- item or * item)
  html = html.replace(/^[-*]\s+(.+)$/gm, '<li class="terminal-li">$1</li>');
  // Wrap consecutive li elements in ul
  html = html.replace(
    /(<li class="terminal-li">.*<\/li>\n?)+/g,
    (match) => `<ul class="terminal-ul">${match}</ul>`,
  );

  // Ordered lists (1. item)
  html = html.replace(/^\d+\.\s+(.+)$/gm, '<li class="terminal-oli">$1</li>');
  html = html.replace(
    /(<li class="terminal-oli">.*<\/li>\n?)+/g,
    (match) => `<ol class="terminal-ol">${match}</ol>`,
  );

  // Blockquotes (> text)
  html = html.replace(
    /^>\s+(.+)$/gm,
    '<blockquote class="terminal-quote">$1</blockquote>',
  );

  // Horizontal rule (--- or ***)
  html = html.replace(/^[-*]{3,}$/gm, '<hr class="terminal-hr">');

  // Checkmarks
  html = html.replace(/^✓\s*/gm, '<span class="check-mark">✓</span> ');
  html = html.replace(/^\[x\]/gim, '<span class="check-mark">✓</span>');
  html = html.replace(/^\[ \]/g, '<span class="check-empty">○</span>');

  // Line breaks - convert double newlines to paragraphs
  html = html.replace(/\n\n+/g, '</p><p class="terminal-p">');
  if (!html.startsWith("<")) {
    html = '<p class="terminal-p">' + html + "</p>";
  }

  return html;
}

// Format markdown-like text for terminal display (simple version for status messages)
function formatTerminalMarkdown(text) {
  if (!text) return "";

  // Headers (## Header)
  text = text.replace(
    /^##\s+(.+)$/gm,
    '<strong class="terminal-header">$1</strong>',
  );

  // Bold (**text**)
  text = text.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");

  // Inline code (`code`)
  text = text.replace(/`([^`]+)`/g, "<code>$1</code>");

  // Code blocks (```code```)
  text = text.replace(
    /```([\s\S]*?)```/g,
    '<pre class="terminal-code"><code>$1</code></pre>',
  );

  // List items (- item)
  text = text.replace(/^-\s+(.+)$/gm, "  • $1");

  // Checkmarks
  text = text.replace(/^✓\s*/gm, '<span class="check-mark">✓</span> ');

  return text;
}

// Render full markdown content (for LLM output)
function renderMarkdownContent(container, markdown) {
  if (!container || !markdown) return;

  const content = document.createElement("div");
  content.className = "markdown-content";
  content.innerHTML = parseMarkdown(markdown);

  // Clear previous content and add new
  container.innerHTML = "";
  container.appendChild(content);
  container.scrollTop = container.scrollHeight;
}

// Update terminal with LLM markdown content
function updateTerminalWithMarkdown(taskId, markdown) {
  const terminalOutput = document.getElementById(`terminal-output-${taskId}`);
  if (terminalOutput) {
    renderMarkdownContent(terminalOutput, markdown);
  } else {
    const genericTerminal = document.querySelector(".taskmd-terminal-output");
    if (genericTerminal) {
      renderMarkdownContent(genericTerminal, markdown);
    }
  }
}

function addTerminalLine(terminal, message, step, activity) {
  const timestamp = new Date().toLocaleTimeString("en-US", { hour12: false });
  const isLlmStream = step === "llm_stream";
  const isLlmOutput =
    step === "llm_output" || (message && message.length > 200);

  // For large LLM output, render as full markdown
  if (isLlmOutput && message && message.length > 200) {
    renderMarkdownContent(terminal, message);
    return;
  }

  // Determine line type based on content
  const isHeader = message && message.startsWith("##");
  const isSuccess = message && message.startsWith("✓");
  const isError = step === "error";
  const isComplete = step === "complete";

  const stepClass = isError
    ? "error"
    : isComplete || isSuccess
      ? "success"
      : isHeader
        ? "progress"
        : isLlmStream
          ? "llm-stream"
          : "info";

  // Format the message with markdown
  const formattedMessage = formatTerminalMarkdown(message);

  const line = document.createElement("div");
  line.className = `terminal-line ${stepClass} current`;

  if (isLlmStream) {
    line.innerHTML = `<span class="llm-text">${formattedMessage}</span>`;
  } else if (isHeader) {
    line.innerHTML = formattedMessage;
  } else {
    line.innerHTML = `<span class="terminal-timestamp">${timestamp}</span>${formattedMessage}`;
  }

  // Remove 'current' class from previous lines
  terminal.querySelectorAll(".terminal-line.current").forEach((el) => {
    el.classList.remove("current");
  });

  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;

  // Keep only last 50 lines
  while (terminal.children.length > 50) {
    terminal.removeChild(terminal.firstChild);
  }
}

// Update progress bar in detail panel
function updateDetailProgress(taskId, current, total, percent) {
  const progressFill = document.querySelector(".progress-fill-rich");
  const progressLabel = document.querySelector(".progress-label-rich");
  const stepInfo = document.querySelector(".meta-estimated");

  const pct = percent || (total > 0 ? Math.round((current / total) * 100) : 0);

  if (progressFill) {
    progressFill.style.width = `${pct}%`;
  }
  if (progressLabel) {
    progressLabel.textContent = `Progress: ${pct}%`;
  }
  if (stepInfo) {
    stepInfo.textContent = `Step ${current}/${total}`;
  }
}

// Legacy functions kept for compatibility but now do nothing
function showFloatingProgress(taskName) {
  // Progress now shown in detail panel terminal
  console.log("[Tasks] Progress:", taskName);
}

function updateFloatingProgressBar(
  current,
  total,
  message,
  step,
  details,
  activity,
) {
  // Progress now shown in detail panel
  updateDetailProgress(null, current, total);
  if (message) {
    updateDetailTerminal(null, message, step, activity);
  }
}

function completeFloatingProgress(message, activity, appUrl) {
  // Completion now shown in detail panel
  console.log("[Tasks] Complete:", message);
}

function closeFloatingProgress() {
  // No floating panel to close
}

function minimizeFloatingProgress() {
  // No floating panel to minimize
}
