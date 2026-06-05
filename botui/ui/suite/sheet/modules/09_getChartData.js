// sheet/modules/09_getChartData.js
"use strict";

// Functions: getChartData, renderBarChart, renderLineChart, renderPieChart, renderChartLegend, selectChart, deleteChart, startDragChart, renderImages, selectImage, startDragImage, startResizeImage

  function getChartData(dataRange) {
    if (!dataRange) return { labels: [], values: [] };

    const ws = state.worksheets[state.activeWorksheet];
    const rangeParts = dataRange.split(":");
    if (rangeParts.length !== 2) return { labels: [], values: [] };

    const startRef = parseCellRef(rangeParts[0]);
    const endRef = parseCellRef(rangeParts[1]);
    if (!startRef || !endRef) return { labels: [], values: [] };

    const labels = [];
    const values = [];

    if (startRef.col === endRef.col) {
      for (let r = startRef.row; r <= endRef.row; r++) {
        const key = `${r},${startRef.col}`;
        const cellData = ws.data[key];
        const val = parseFloat(cellData?.value) || 0;
        values.push(val);
        labels.push(`Row ${r + 1}`);
      }
    } else {
      for (let c = startRef.col; c <= endRef.col; c++) {
        const key = `${startRef.row},${c}`;
        const cellData = ws.data[key];
        const val = parseFloat(cellData?.value) || 0;
        values.push(val);
        labels.push(getColName(c));
      }
    }

    return { labels, values };
  }

  function renderBarChart(data, maxHeight) {
    if (!data.values.length) return '<div class="chart-empty">No data</div>';

    const maxVal = Math.max(...data.values, 1);
    const bars = data.values
      .map((val, i) => {
        const height = (val / maxVal) * maxHeight;
        return `<div class="chart-bar" style="height:${height}px;" title="${data.labels[i]}: ${val}"></div>`;
      })
      .join("");

    return `<div class="chart-bar-container" style="height:${maxHeight}px;">${bars}</div>`;
  }

  function renderLineChart(data, width, height) {
    if (!data.values.length) return '<div class="chart-empty">No data</div>';

    const maxVal = Math.max(...data.values, 1);
    const padding = 20;
    const chartWidth = width - padding * 2;
    const chartHeight = height - padding * 2;

    const points = data.values.map((val, i) => {
      const x = padding + (i / (data.values.length - 1 || 1)) * chartWidth;
      const y = padding + chartHeight - (val / maxVal) * chartHeight;
      return `${x},${y}`;
    });

    const circles = data.values
      .map((val, i) => {
        const x = padding + (i / (data.values.length - 1 || 1)) * chartWidth;
        const y = padding + chartHeight - (val / maxVal) * chartHeight;
        return `<circle class="chart-line-point" cx="${x}" cy="${y}" r="4"/>`;
      })
      .join("");

    return `
      <svg class="chart-canvas" viewBox="0 0 ${width} ${height}">
        <polyline class="chart-line" points="${points.join(" ")}"/>
        ${circles}
      </svg>
    `;
  }

  function renderPieChart(data, size) {
    if (!data.values.length) return '<div class="chart-empty">No data</div>';

    const total = data.values.reduce((a, b) => a + b, 0) || 1;
    const colors = [
      "#4285f4",
      "#34a853",
      "#fbbc04",
      "#ea4335",
      "#9c27b0",
      "#00bcd4",
      "#ff5722",
    ];
    const cx = size / 2;
    const cy = size / 2;
    const r = size / 2 - 10;

    let startAngle = 0;
    const slices = data.values
      .map((val, i) => {
        const angle = (val / total) * 360;
        const endAngle = startAngle + angle;
        const largeArc = angle > 180 ? 1 : 0;

        const x1 = cx + r * Math.cos((startAngle * Math.PI) / 180);
        const y1 = cy + r * Math.sin((startAngle * Math.PI) / 180);
        const x2 = cx + r * Math.cos((endAngle * Math.PI) / 180);
        const y2 = cy + r * Math.sin((endAngle * Math.PI) / 180);

        const path = `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} Z`;
        startAngle = endAngle;

        return `<path d="${path}" fill="${colors[i % colors.length]}" stroke="white" stroke-width="2"/>`;
      })
      .join("");

    return `
      <div class="chart-pie-container">
        <svg class="chart-canvas" viewBox="0 0 ${size} ${size}" width="${size}" height="${size}">
          ${slices}
        </svg>
      </div>
    `;
  }

  function renderChartLegend(data) {
    const colors = [
      "#4285f4",
      "#34a853",
      "#fbbc04",
      "#ea4335",
      "#9c27b0",
      "#00bcd4",
      "#ff5722",
    ];
    const items = data.labels
      .map(
        (label, i) =>
          `<div class="legend-item"><span class="legend-color" style="background:${colors[i % colors.length]}"></span>${escapeHtml(label)}</div>`,
      )
      .join("");

    return `<div class="chart-legend">${items}</div>`;
  }

  function selectChart(chartId) {
    document.querySelectorAll(".chart-wrapper").forEach((el) => {
      el.classList.toggle("selected", el.dataset.chartId === chartId);
    });
  }

  function deleteChart(chartId) {
    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.charts) return;

    ws.charts = ws.charts.filter((c) => c.id !== chartId);
    renderCharts();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function startDragChart(e, chartId) {
    const wrapper = document.querySelector(`[data-chart-id="${chartId}"]`);
    if (!wrapper) return;

    const startX = e.clientX;
    const startY = e.clientY;
    const startLeft = parseInt(wrapper.style.left) || 0;
    const startTop = parseInt(wrapper.style.top) || 0;

    const onMouseMove = (moveEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      wrapper.style.left = `${startLeft + dx}px`;
      wrapper.style.top = `${startTop + dy}px`;
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const ws = state.worksheets[state.activeWorksheet];
      const chart = ws.charts?.find((c) => c.id === chartId);
      if (chart) {
        chart.position.col = Math.round(
          parseInt(wrapper.style.left) / CONFIG.COL_WIDTH,
        );
        chart.position.row = Math.round(
          parseInt(wrapper.style.top) / CONFIG.ROW_HEIGHT,
        );
        state.isDirty = true;
        scheduleAutoSave();
      }
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }

  function renderImages() {
    const imagesContainer = document.getElementById("imagesContainer");
    if (!imagesContainer) return;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.images || ws.images.length === 0) {
      imagesContainer.innerHTML = "";
      return;
    }

    imagesContainer.innerHTML = ws.images
      .map((img) => {
        const left = img.position?.col
          ? img.position.col * CONFIG.COL_WIDTH
          : 100;
        const top = img.position?.row
          ? img.position.row * CONFIG.ROW_HEIGHT
          : 100;
        const width = img.position?.width || 200;
        const height = img.position?.height || 150;

        return `
          <div class="image-wrapper" data-image-id="${img.id}" style="left:${left}px;top:${top}px;width:${width}px;height:${height}px;">
            <img src="${escapeHtml(img.url)}" alt="Embedded image" />
            <div class="image-resize-handle"></div>
          </div>
        `;
      })
      .join("");

    imagesContainer.querySelectorAll(".image-wrapper").forEach((wrapper) => {
      const imageId = wrapper.dataset.imageId;
      wrapper.addEventListener("click", () => selectImage(imageId));
      wrapper.addEventListener("mousedown", (e) => {
        if (e.target.classList.contains("image-resize-handle")) {
          startResizeImage(e, imageId);
        } else {
          startDragImage(e, imageId);
        }
      });
    });
  }

  function selectImage(imageId) {
    document.querySelectorAll(".image-wrapper").forEach((el) => {
      el.classList.toggle("selected", el.dataset.imageId === imageId);
    });
  }

  function startDragImage(e, imageId) {
    const wrapper = document.querySelector(`[data-image-id="${imageId}"]`);
    if (!wrapper) return;

    const startX = e.clientX;
    const startY = e.clientY;
    const startLeft = parseInt(wrapper.style.left) || 0;
    const startTop = parseInt(wrapper.style.top) || 0;

    const onMouseMove = (moveEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      wrapper.style.left = `${startLeft + dx}px`;
      wrapper.style.top = `${startTop + dy}px`;
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const ws = state.worksheets[state.activeWorksheet];
      const img = ws.images?.find((i) => i.id === imageId);
      if (img) {
        img.position.col = Math.round(
          parseInt(wrapper.style.left) / CONFIG.COL_WIDTH,
        );
        img.position.row = Math.round(
          parseInt(wrapper.style.top) / CONFIG.ROW_HEIGHT,
        );
        state.isDirty = true;
        scheduleAutoSave();
      }
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    e.preventDefault();
  }

  function startResizeImage(e, imageId) {
    const wrapper = document.querySelector(`[data-image-id="${imageId}"]`);
    if (!wrapper) return;

    const startX = e.clientX;
    const startY = e.clientY;
    const startWidth = parseInt(wrapper.style.width) || 200;
    const startHeight = parseInt(wrapper.style.height) || 150;
    const aspectRatio = startWidth / startHeight;

    const onMouseMove = (moveEvent) => {
      const dx = moveEvent.clientX - startX;
      const newWidth = Math.max(50, startWidth + dx);
      const newHeight = newWidth / aspectRatio;
      wrapper.style.width = `${newWidth}px`;
      wrapper.style.height = `${newHeight}px`;
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const ws = state.worksheets[state.activeWorksheet];
      const img = ws.images?.find((i) => i.id === imageId);
      if (img) {
        img.position.width = parseInt(wrapper.style.width);
        img.position.height = parseInt(wrapper.style.height);
        state.isDirty = true;
        scheduleAutoSave();
      }
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    e.preventDefault();
    e.stopPropagation();
  }

