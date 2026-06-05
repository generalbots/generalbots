
"use strict";

  function toFraction(decimal) {
    const tolerance = 1e-6;
    let h1 = 1,
      h2 = 0,
      k1 = 0,
      k2 = 1;
    let b = decimal;
    do {
      const a = Math.floor(b);
      let aux = h1;
      h1 = a * h1 + h2;
      h2 = aux;
      aux = k1;
      k1 = a * k1 + k2;
      k2 = aux;
      b = 1 / (b - a);
    } while (Math.abs(decimal - h1 / k1) > decimal * tolerance);

    if (k1 === 1) return String(h1);
    const whole = Math.floor(h1 / k1);
    const remainder = h1 % k1;
    if (whole === 0) return `${remainder}/${k1}`;
    return `${whole} ${remainder}/${k1}`;
  }

  function decreaseDecimal() {
    if (state.decimalPlaces > 0) {
      state.decimalPlaces--;
      reapplyFormats();
    }
  }

  function increaseDecimal() {
    if (state.decimalPlaces < 10) {
      state.decimalPlaces++;
      reapplyFormats();
    }
  }

  function reapplyFormats() {
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        if (cellData?.format && cellData?.rawValue) {
          cellData.value = formatValue(cellData.rawValue, cellData.format);
          renderCell(r, c);
        }
      }
    }
  }

  function showFindReplaceModal() {
    showModal("findReplaceModal");
    document.getElementById("findInput")?.focus();
    state.findMatches = [];
    state.findMatchIndex = -1;
  }

  function performFind() {
    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const wholeCell = document.getElementById("findWholeCell")?.checked;
    const useRegex = document.getElementById("findRegex")?.checked;

    state.findMatches = [];
    state.findMatchIndex = -1;

    if (!searchText) {
      updateFindResults();
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    let pattern;

    if (useRegex) {
      try {
        pattern = new RegExp(searchText, matchCase ? "" : "i");
      } catch (e) {
        updateFindResults();
        return;
      }
    }

    for (let r = 0; r < CONFIG.ROWS; r++) {
      for (let c = 0; c < CONFIG.COLS; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        const cellValue = cellData?.value || "";

        if (!cellValue) continue;

        let matches = false;
        const compareValue = matchCase ? cellValue : cellValue.toLowerCase();
        const compareSearch = matchCase ? searchText : searchText.toLowerCase();

        if (useRegex) {
          matches = pattern.test(cellValue);
        } else if (wholeCell) {
          matches = compareValue === compareSearch;
        } else {
          matches = compareValue.includes(compareSearch);
        }

        if (matches) {
          state.findMatches.push({ row: r, col: c });
        }
      }
    }

    updateFindResults();
    if (state.findMatches.length > 0) {
      state.findMatchIndex = 0;
      highlightFindMatch();
    }
  }

  function updateFindResults() {
    const resultsEl = document.getElementById("findResults");
    if (resultsEl) {
      const count = state.findMatches.length;
      resultsEl.querySelector("span").textContent =
        count === 0
          ? "0 matches found"
          : `${state.findMatchIndex + 1} of ${count} matches`;
    }
  }

  function highlightFindMatch() {
    if (state.findMatches.length === 0) return;
    const match = state.findMatches[state.findMatchIndex];
    selectCell(match.row, match.col);
    updateFindResults();
  }

  function findNext() {
    if (state.findMatches.length === 0) return;
    state.findMatchIndex =
      (state.findMatchIndex + 1) % state.findMatches.length;
    highlightFindMatch();
  }

  function findPrev() {
    if (state.findMatches.length === 0) return;
    state.findMatchIndex =
      (state.findMatchIndex - 1 + state.findMatches.length) %
      state.findMatches.length;
    highlightFindMatch();
  }

  function replaceOne() {
    if (state.findMatches.length === 0 || state.findMatchIndex < 0) return;

    const replaceText = document.getElementById("replaceInput")?.value || "";
    const match = state.findMatches[state.findMatchIndex];
    const ws = state.worksheets[state.activeWorksheet];
    const key = `${match.row},${match.col}`;

    saveToHistory();

    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const useRegex = document.getElementById("findRegex")?.checked;
    const cellValue = ws.data[key]?.value || "";

    let newValue;
    if (useRegex) {
      const pattern = new RegExp(searchText, matchCase ? "g" : "gi");
      newValue = cellValue.replace(pattern, replaceText);
    } else {
      const flags = matchCase ? "g" : "gi";
      const escapedSearch = searchText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      newValue = cellValue.replace(
        new RegExp(escapedSearch, flags),
        replaceText,
      );
    }

    if (!ws.data[key]) ws.data[key] = {};
    ws.data[key].value = newValue;
    renderCell(match.row, match.col);

    state.findMatches.splice(state.findMatchIndex, 1);
    if (state.findMatches.length > 0) {
      state.findMatchIndex = state.findMatchIndex % state.findMatches.length;
      highlightFindMatch();
    } else {
      state.findMatchIndex = -1;
      updateFindResults();
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function replaceAll() {
    if (state.findMatches.length === 0) return;

    const replaceText = document.getElementById("replaceInput")?.value || "";
    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const useRegex = document.getElementById("findRegex")?.checked;
    const ws = state.worksheets[state.activeWorksheet];

    saveToHistory();

    let count = 0;
    for (const match of state.findMatches) {
      const key = `${match.row},${match.col}`;
      const cellValue = ws.data[key]?.value || "";

      let newValue;
      if (useRegex) {
        const pattern = new RegExp(searchText, matchCase ? "g" : "gi");
        newValue = cellValue.replace(pattern, replaceText);
      } else {
        const flags = matchCase ? "g" : "gi";
        const escapedSearch = searchText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        newValue = cellValue.replace(
          new RegExp(escapedSearch, flags),
          replaceText,
        );
      }

      if (!ws.data[key]) ws.data[key] = {};
      ws.data[key].value = newValue;
      renderCell(match.row, match.col);
      count++;
    }

    state.findMatches = [];
    state.findMatchIndex = -1;
    updateFindResults();

    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", `Replaced ${count} occurrences.`);
  }

  function showConditionalFormatModal() {
    const { start, end } = state.selection;
    const range = `${getColName(start.col)}${start.row + 1}:${getColName(end.col)}${end.row + 1}`;
    const rangeInput = document.getElementById("cfRange");
    if (rangeInput) rangeInput.value = range;
    showModal("conditionalFormatModal");
    handleCfRuleTypeChange();
    updateCfPreview();
  }

  function handleCfRuleTypeChange() {
    const ruleType = document.getElementById("cfRuleType")?.value;
    const value2 = document.getElementById("cfValue2");
    const valuesSection = document.getElementById("cfValuesSection");

    if (value2) {
      if (ruleType === "between") {
        value2.classList.remove("hidden");
        value2.placeholder = "and";
      } else {
        value2.classList.add("hidden");
      }
    }

    const noValueTypes = [
      "duplicate",
      "unique",
      "blank",
      "not_blank",
      "above_average",
      "below_average",
      "color_scale",
      "data_bar",
      "icon_set",
    ];
    if (valuesSection) {
      if (noValueTypes.includes(ruleType)) {
        valuesSection.style.display = "none";
      } else {
        valuesSection.style.display = "flex";
      }
    }
  }

  function updateCfPreview() {
    const bgColor = document.getElementById("cfBgColor")?.value || "#ffeb3b";
    const textColor =
      document.getElementById("cfTextColor")?.value || "#000000";
    const bold = document.getElementById("cfBold")?.checked;
    const italic = document.getElementById("cfItalic")?.checked;

    const previewCell = document.getElementById("cfPreviewCell");
    if (previewCell) {
      previewCell.style.background = bgColor;
      previewCell.style.color = textColor;
      previewCell.style.fontWeight = bold ? "bold" : "normal";
      previewCell.style.fontStyle = italic ? "italic" : "normal";
    }
  }

  function applyConditionalFormat() {
    const rangeStr = document.getElementById("cfRange")?.value;
    if (!rangeStr) {
      alert("Please specify a range.");
      return;
    }

    const ruleType = document.getElementById("cfRuleType")?.value;
    const value1 = document.getElementById("cfValue1")?.value;
    const value2 = document.getElementById("cfValue2")?.value;
    const bgColor = document.getElementById("cfBgColor")?.value;
    const textColor = document.getElementById("cfTextColor")?.value;
    const bold = document.getElementById("cfBold")?.checked;
    const italic = document.getElementById("cfItalic")?.checked;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.conditionalFormats) ws.conditionalFormats = [];

    const rule = {
      id: `cf_${Date.now()}`,
      range: rangeStr,
      ruleType,
      value1,
      value2,
      style: {
        background: bgColor,
        color: textColor,
        fontWeight: bold ? "bold" : "normal",
        fontStyle: italic ? "italic" : "normal",
      },
    };

    ws.conditionalFormats.push(rule);
    applyConditionalFormatsToRange(rule);

    hideModal("conditionalFormatModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Conditional formatting applied!");
  }

  function applyConditionalFormatsToRange(rule) {
    const ws = state.worksheets[state.activeWorksheet];
    const rangeParts = rule.range.split(":");
    if (rangeParts.length !== 2) return;

    const startRef = parseCellRef(rangeParts[0]);
    const endRef = parseCellRef(rangeParts[1]);
    if (!startRef || !endRef) return;

    for (let r = startRef.row; r <= endRef.row; r++) {
      for (let c = startRef.col; c <= endRef.col; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        const cellValue = parseFloat(cellData?.value) || 0;

        let conditionMet = false;
        switch (rule.ruleType) {
          case "greater_than":
            conditionMet = cellValue > parseFloat(rule.value1);
            break;
          case "less_than":
            conditionMet = cellValue < parseFloat(rule.value1);
            break;
          case "equal_to":
            conditionMet = cellValue === parseFloat(rule.value1);
            break;
          case "between":
            conditionMet =
              cellValue >= parseFloat(rule.value1) &&
              cellValue <= parseFloat(rule.value2);
            break;
          case "text_contains":
            conditionMet = (cellData?.value || "")
              .toLowerCase()
              .includes(rule.value1.toLowerCase());
            break;
          case "blank":
            conditionMet = !cellData?.value;
            break;
          case "not_blank":
            conditionMet = !!cellData?.value;
            break;
          default:
            conditionMet = false;
        }

        if (conditionMet && cellData) {
          if (!cellData.style) cellData.style = {};
          Object.assign(cellData.style, rule.style);
          renderCell(r, c);
        }
      }
    }
  }

  function showDataValidationModal() {
    const { start, end } = state.selection;
    const range = `${getColName(start.col)}${start.row + 1}:${getColName(end.col)}${end.row + 1}`;
    const rangeInput = document.getElementById("dvRange");
    if (rangeInput) rangeInput.value = range;
    showModal("dataValidationModal");
    handleDvTypeChange();
  }

  function switchDvTab(tabName) {
    document.querySelectorAll(".dv-tab").forEach((tab) => {
      tab.classList.toggle("active", tab.dataset.tab === tabName);
    });
    document.querySelectorAll(".dv-tab-content").forEach((content) => {
      const contentId = content.id
        .replace("dv", "")
        .replace("Tab", "")
        .toLowerCase();
      content.classList.toggle("active", contentId === tabName);
    });
  }

  function handleDvTypeChange() {
    const dvType = document.getElementById("dvType")?.value;
    const criteriaSection = document.getElementById("dvCriteriaSection");
    const valuesSection = document.getElementById("dvValuesSection");
    const listSection = document.getElementById("dvListSection");
    const value2Row = document.getElementById("dvValue2Row");
    const value1Label = document.getElementById("dvValue1Label");

    if (criteriaSection) {
      criteriaSection.style.display =
        dvType === "any" || dvType === "list" || dvType === "custom"
          ? "none"
          : "block";
    }

    if (valuesSection) {
      valuesSection.style.display =
        dvType === "any" || dvType === "list" ? "none" : "block";
    }
    if (listSection) {
      listSection.classList.toggle("hidden", dvType !== "list");
    }
    if (value1Label) {
      value1Label.textContent = dvType === "custom" ? "Formula:" : "Minimum:";
    }
  }
