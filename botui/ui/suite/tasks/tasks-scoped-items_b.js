/* Split from tasks-scoped-items.js for #2305 (450-line limit). */
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
    const genericTerminal = document.getElementById("window-tasks").querySelector(".taskmd-terminal-output");
    if (genericTerminal) {
      renderMarkdownContent(genericTerminal, markdown);
    }
  }
}

function addTerminalLine(terminal, message, step, activity) {
  const now = new Date();
  const timestamp = (typeof window.gbFormatTime === "function")
    ? window.gbFormatTime(now)
    : (window.i18n && typeof window.i18n.formatDate === "function")
      ? window.i18n.formatDate(now, { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false })
      : gbFormatTime(now, { locale: (window.i18n && typeof window.i18n.getLocale === "function" && window.i18n.getLocale()) || undefined, hour12: false });
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
  }, 100);

  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;

  // Keep only last 50 lines
  while (terminal.children.length > 50) {
    terminal.removeChild(terminal.firstChild);
  }
}

// Update progress bar in detail panel
function updateDetailProgress(taskId, current, total, percent) {
  const progressFill = document.getElementById("window-tasks").querySelector(".progress-fill-rich");
  const progressLabel = document.getElementById("window-tasks").querySelector(".progress-label-rich");
  const stepInfo = document.getElementById("window-tasks").querySelector(".meta-estimated");

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
  gbLog.debug("[Tasks] Progress:", taskName);
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
  gbLog.debug("[Tasks] Complete:", message);
}

function closeFloatingProgress() {
  // No floating panel to close
}

function minimizeFloatingProgress() {
  // No floating panel to minimize
}

// #2839 self-test for the terminal render sink. Call from the browser console
// or the webtest: window.TasksTerminalSelfTest.run(). IDs are machine
// identifiers (not user-facing text), so no locale catalog entry is needed.
// guardClean() parses the rendered HTML and rejects real elements/attributes;
// escaped text such as "&lt;img onerror=alert(1)&gt;" is inert and passes.
(function () {
  const VECTORS = [
    {
      id: "terminal-img-onerror",
      input: 'before <img src=x onerror="alert(1)"> after',
      render: (value) => formatTerminalMarkdown(value),
    },
    {
      id: "terminal-svg-onload",
      input: '<svg onload="alert(1)">x</svg>',
      render: (value) => formatTerminalMarkdown(value),
    },
    {
      id: "markdown-link-attribute-breakout",
      input: '[x](" onmouseover="alert(1))',
      render: (value) => parseMarkdown(value),
    },
    {
      id: "markdown-link-javascript-scheme",
      input: "[x](javascript:alert(1))",
      render: (value) => parseMarkdown(value),
    },
    {
      id: "markdown-link-obfuscated-scheme",
      input: "[x](java\tscript:alert(1))",
      render: (value) => parseMarkdown(value),
    },
    {
      id: "markdown-raw-img",
      input: '<img src=x onerror="alert(1)">',
      render: (value) => parseMarkdown(value),
    },
  ];

  function guardClean(html) {
    const tpl = document.createElement("template");
    tpl.innerHTML = String(html == null ? "" : html);
    const DROP = ["img", "svg", "script", "iframe", "object", "embed", "form"];
    const nodes = tpl.content.querySelectorAll("*");
    for (const el of nodes) {
      const tag = (el.tagName || "").toLowerCase();
      if (DROP.includes(tag)) return false;
      for (const attr of el.attributes) {
        const name = attr.name.toLowerCase();
        if (name.startsWith("on")) return false;
        if (["href", "src", "xlink:href"].includes(name)) {
          const uri = attr.value.replace(/[\u0000-\u0020\u00A0]/g, "").toLowerCase();
          if (/^(javascript|vbscript|livescript):/.test(uri)) return false;
        }
      }
    }
    return true;
  }

  function run() {
    return VECTORS.map((vector) => {
      const html = vector.render(vector.input);
      return { id: vector.id, input: vector.input, html, pass: guardClean(html) };
    });
  }

  window.TasksTerminalSelfTest = { run, guardClean, VECTORS };
})();
