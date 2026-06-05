
"use strict";

  function handleDvOperatorChange() {
    const operator = document.getElementById("dvOperator")?.value;
    const value2Row = document.getElementById("dvValue2Row");
    const value1Label = document.getElementById("dvValue1Label");

    if (value2Row) {
      value2Row.style.display =
        operator === "between" || operator === "not_between" ? "block" : "none";
    }

    if (value1Label) {
      if (operator === "between" || operator === "not_between") {
        value1Label.textContent = "Minimum:";
      } else {
        value1Label.textContent = "Value:";
      }
    }
  }

  function applyDataValidation() {
    const rangeStr = document.getElementById("dvRange")?.value;
    if (!rangeStr) {
      alert("Please specify a range.");
      return;
    }

    const dvType = document.getElementById("dvType")?.value;
    const operator = document.getElementById("dvOperator")?.value;
    const value1 = document.getElementById("dvValue1")?.value;
    const value2 = document.getElementById("dvValue2")?.value;
    const listSource = document.getElementById("dvListSource")?.value;
    const showInput = document.getElementById("dvShowInput")?.checked;
    const inputTitle = document.getElementById("dvInputTitle")?.value;
    const inputMessage = document.getElementById("dvInputMessage")?.value;
    const showError = document.getElementById("dvShowError")?.checked;
    const errorStyle = document.getElementById("dvErrorStyle")?.value;
    const errorTitle = document.getElementById("dvErrorTitle")?.value;
    const errorMessage = document.getElementById("dvErrorMessage")?.value;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.validations) ws.validations = {};

    const validation = {
      type: dvType,
      operator,
      value1,
      value2,
      listValues: listSource ? listSource.split(",").map((s) => s.trim()) : [],
      showInput,
      inputTitle,
      inputMessage,
      showError,
      errorStyle,
      errorTitle,
      errorMessage,
    };

    const rangeParts = rangeStr.split(":");
    const startRef = parseCellRef(rangeParts[0]);
    const endRef =
      rangeParts.length > 1 ? parseCellRef(rangeParts[1]) : startRef;

    if (startRef && endRef) {
      for (let r = startRef.row; r <= endRef.row; r++) {
        for (let c = startRef.col; c <= endRef.col; c++) {
          ws.validations[`${r},${c}`] = validation;
        }
      }
    }

    hideModal("dataValidationModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Data validation applied!");
  }

  function clearDataValidation() {
    const rangeStr = document.getElementById("dvRange")?.value;
    if (!rangeStr) return;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.validations) return;

    const rangeParts = rangeStr.split(":");
    const startRef = parseCellRef(rangeParts[0]);
    const endRef =
      rangeParts.length > 1 ? parseCellRef(rangeParts[1]) : startRef;

    if (startRef && endRef) {
      for (let r = startRef.row; r <= endRef.row; r++) {
        for (let c = startRef.col; c <= endRef.col; c++) {
          delete ws.validations[`${r},${c}`];
        }
      }
    }

    hideModal("dataValidationModal");
    state.isDirty = true;
    scheduleAutoSave();
  }

  function showPrintPreview() {
    showModal("printPreviewModal");
    updatePrintPreview();
  }

  function updatePrintPreview() {
    const orientation =
      document.getElementById("printOrientation")?.value || "portrait";
    const showGridlines = document.getElementById("printGridlines")?.checked;
    const showHeaders = document.getElementById("printHeaders")?.checked;
    const printPage = document.getElementById("printPage");
    const printContent = document.getElementById("printContent");

    if (printPage) {
      printPage.className = `print-page ${orientation}`;
    }

    if (!printContent) return;

    const ws = state.worksheets[state.activeWorksheet];
    let html = "<table>";

    if (showHeaders) {
      html += "<thead><tr><th></th>";
      for (let c = 0; c < CONFIG.COLS; c++) {
        html += `<th>${getColName(c)}</th>`;
      }
      html += "</tr></thead>";
    }

    html += "<tbody>";
    let hasData = false;
    let maxRow = 0;
    let maxCol = 0;

    for (const key in ws.data) {
      if (ws.data[key]?.value) {
        hasData = true;
        const [r, c] = key.split(",").map(Number);
        maxRow = Math.max(maxRow, r);
        maxCol = Math.max(maxCol, c);
      }
    }

    if (!hasData) {
      maxRow = 10;
      maxCol = 5;
    }

    for (let r = 0; r <= maxRow; r++) {
      html += "<tr>";
      if (showHeaders) {
        html += `<th>${r + 1}</th>`;
      }
      for (let c = 0; c <= maxCol; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        const value = cellData?.value || "";
        const style = cellData?.style || {};
        let styleStr = "";

        if (style.fontWeight) styleStr += `font-weight:${style.fontWeight};`;
        if (style.fontStyle) styleStr += `font-style:${style.fontStyle};`;
        if (style.textAlign) styleStr += `text-align:${style.textAlign};`;
        if (style.color) styleStr += `color:${style.color};`;
        if (style.background) styleStr += `background:${style.background};`;

        const borderStyle = showGridlines ? "" : "border:none;";
        html += `<td style="${styleStr}${borderStyle}">${escapeHtml(value)}</td>`;
      }
      html += "</tr>";
    }

    html += "</tbody></table>";
    printContent.innerHTML = html;
  }

  function printSheet() {
    const printContent = document.getElementById("printContent")?.innerHTML;
    if (!printContent) return;

    const orientation =
      document.getElementById("printOrientation")?.value || "portrait";
    const printWindow = window.open("", "_blank");

    printWindow.document.write(`
      <!DOCTYPE html>
      <html>
      <head>
        <title>${state.sheetName}</title>
        <style>
          @page { size: ${orientation}; margin: 0.5in; }
          body { font-family: Arial, sans-serif; font-size: 10pt; }
          table { width: 100%; border-collapse: collapse; }
          td, th { border: 1px solid #ccc; padding: 4px 8px; text-align: left; }
          th { background: #f5f5f5; font-weight: 600; }
        </style>
      </head>
      <body>
        ${printContent}
      </body>
      </html>
    `);

    printWindow.document.close();
    printWindow.focus();
    setTimeout(() => {
      printWindow.print();
      printWindow.close();
    }, 250);

    hideModal("printPreviewModal");
  }

  function insertChart() {
    const chartType =
      document.querySelector(".chart-type-btn.active")?.dataset.type || "bar";
    const dataRange = document.getElementById("chartDataRange")?.value;
    const chartTitle = document.getElementById("chartTitle")?.value || "Chart";

    if (!dataRange) {
      alert("Please specify a data range.");
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.charts) ws.charts = [];

    const chart = {
      id: `chart_${Date.now()}`,
      type: chartType,
      title: chartTitle,
      dataRange,
      position: {
        row: state.activeCell.row,
        col: state.activeCell.col,
        width: 400,
        height: 300,
      },
    };

    ws.charts.push(chart);
    hideModal("chartModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage(
      "assistant",
      `${chartType.charAt(0).toUpperCase() + chartType.slice(1)} chart created!`,
    );
  }

  function showInsertImageModal() {
    showModal("insertImageModal");
  }

  function switchImgTab(tabName) {
    document.querySelectorAll(".img-tab").forEach((tab) => {
      tab.classList.toggle("active", tab.dataset.tab === tabName);
    });
    document.querySelectorAll(".img-tab-content").forEach((content) => {
      const contentId = content.id
        .replace("img", "")
        .replace("Tab", "")
        .toLowerCase();
      content.classList.toggle("active", contentId === tabName);
    });
  }

  function insertImage() {
    const urlTab = document.getElementById("imgUrlTab");
    const isUrlTab = urlTab?.classList.contains("active");
    let imageUrl;

    if (isUrlTab) {
      imageUrl = document.getElementById("imgUrl")?.value;
    } else {
      const fileInput = document.getElementById("imgFile");
      if (fileInput?.files?.[0]) {
        addChatMessage(
          "assistant",
          "Image upload coming soon! Please use a URL for now.",
        );
        return;
      }
    }

    if (!imageUrl) {
      alert("Please enter an image URL.");
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.images) ws.images = [];

    const image = {
      id: `img_${Date.now()}`,
      url: imageUrl,
      position: {
        row: state.activeCell.row,
        col: state.activeCell.col,
        width: 200,
        height: 150,
      },
    };

    ws.images.push(image);
    hideModal("insertImageModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Image inserted!");
  }

  function toggleFilter() {
    const ws = state.worksheets[state.activeWorksheet];
    ws.filterEnabled = !ws.filterEnabled;
    addChatMessage(
      "assistant",
      ws.filterEnabled
        ? "Filter enabled. Click column headers to filter."
        : "Filter disabled.",
    );
  }

  function selectCustomFormat(formatCode) {
    document.querySelectorAll(".cnf-format-item").forEach((item) => {
      item.classList.toggle("selected", item.dataset.format === formatCode);
    });
    const formatInput = document.getElementById("cnfFormatCode");
    if (formatInput) {
      formatInput.value = formatCode;
    }
    updateCnfPreview();
  }

  function updateCnfPreview() {
    const formatCode =
      document.getElementById("cnfFormatCode")?.value || "#,##0.00";
    const previewEl = document.getElementById("cnfPreview");
    if (!previewEl) return;

    const sampleValue = 1234.5678;
    let formatted;

    if (formatCode.includes("$")) {
      formatted = sampleValue.toLocaleString("en-US", {
        style: "currency",
        currency: "USD",
      });
    } else if (formatCode.includes("%")) {
      formatted = (sampleValue * 100).toFixed(2) + "%";
    } else if (formatCode.includes("E")) {
      formatted = sampleValue.toExponential(2);
    } else if (formatCode.includes("MM") || formatCode.includes("DD")) {
      formatted = new Date().toLocaleDateString();
    } else if (formatCode.includes("HH")) {
      formatted = new Date().toLocaleTimeString();
    } else {
      const decimals = (formatCode.match(/0+$/)?.[0] || "").length;
      formatted = sampleValue.toLocaleString("en-US", {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
      });
    }

    previewEl.textContent = formatted;
  }

  function applyCustomNumberFormat() {
    const formatCode = document.getElementById("cnfFormatCode")?.value;
    if (!formatCode) return;
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (!ws.data[key]) ws.data[key] = { value: "" };
        ws.data[key].customFormat = formatCode;
        renderCell(r, c);
      }
    }
    hideModal("customNumberFormatModal");
    state.isDirty = true;
    scheduleAutoSave();
  }
  function renderCharts() {
    const chartsContainer = document.getElementById("chartsContainer");
    if (!chartsContainer) return;
    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.charts || ws.charts.length === 0) {
      chartsContainer.innerHTML = "";
      return;
    }
    chartsContainer.innerHTML = ws.charts
      .map((chart) => renderChartHTML(chart))
      .join("");
    chartsContainer.querySelectorAll(".chart-wrapper").forEach((wrapper) => {
      const chartId = wrapper.dataset.chartId;
      wrapper.addEventListener("click", () => selectChart(chartId));
      wrapper.querySelector(".chart-delete")?.addEventListener("click", (e) => {
        e.stopPropagation();
        deleteChart(chartId);
      });
      wrapper
        .querySelector(".chart-header")
        ?.addEventListener("mousedown", (e) => {
          startDragChart(e, chartId);
        });
    });
  }
  function renderChartHTML(chart) {
    const { id, type, title, position, dataRange } = chart;
    const left = position?.col ? position.col * CONFIG.COL_WIDTH : 100;
    const top = position?.row ? position.row * CONFIG.ROW_HEIGHT : 100;
    const width = position?.width || 400;
    const height = position?.height || 300;
    const data = getChartData(dataRange);
    let chartContent = "";
    switch (type) {
      case "bar":
        chartContent = renderBarChart(data, height - 80);
        break;
      case "line":
        chartContent = renderLineChart(data, width - 32, height - 80);
        break;
      case "pie":
        chartContent = renderPieChart(data, Math.min(width, height) - 100);
        break;
      default:
        chartContent = renderBarChart(data, height - 80);
    }
    return `
      <div class="chart-wrapper" data-chart-id="${id}" style="left:${left}px;top:${top}px;width:${width}px;height:${height}px;">
        <div class="chart-header">
          <h4 class="chart-title">${escapeHtml(title || "Chart")}</h4>
          <div class="chart-actions">
            <button class="chart-delete" title="Delete">×</button>
          </div>
        </div>
        <div class="chart-content">
          ${chartContent}
        </div>
        ${renderChartLegend(data)}
      </div>
    `;
  }
