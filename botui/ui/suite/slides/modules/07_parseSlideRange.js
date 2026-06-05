// slides/modules/07_parseSlideRange.js
"use strict";

// Functions: parseSlideRange, getLayoutSlidesPerPage, renderSlideContentForExport, showMasterSlideModal, setColorInput, setSelectValue, selectMasterLayout, updateMasterLayoutSelection, updateMasterPreview, applyMasterSlide, resetMasterSlide

          .slide-4 { width: 45%; aspect-ratio: 16/9; }
          .slide-6 { width: 30%; aspect-ratio: 16/9; }
          .slide-content { padding: 20px; height: 100%; box-sizing: border-box; }
          .slide-number { text-align: center; font-size: 12px; color: #666; margin-top: 8px; }
          .notes-section { padding: 10px; font-size: 11px; border-top: 1px solid #ccc; }
        </style>
      </head>
      <body>
    `;

    let slideCount = 0;
    slidesToExport.forEach((slideIndex, i) => {
      const slide = state.slides[slideIndex];
      if (!slide) return;

      if (slideCount > 0 && slideCount % slidesPerPage === 0) {
        htmlContent += '<div class="page-break"></div>';
      }

      if (slideCount % slidesPerPage === 0) {
        htmlContent += '<div class="slide-container">';
      }

      const slideClass =
        slidesPerPage === 1
          ? "slide-full"
          : slidesPerPage === 2
            ? "slide-2"
            : slidesPerPage === 4
              ? "slide-4"
              : "slide-6";
      const bgColor = slide.background?.color || "#ffffff";

      htmlContent += `
        <div class="slide ${slideClass}" style="background:${bgColor};">
          <div class="slide-content">
            ${renderSlideContentForExport(slide)}
          </div>
          <div class="slide-number">Slide ${slideIndex + 1}</div>
          ${layout === "notes" && slide.notes ? `<div class="notes-section">${escapeHtml(slide.notes)}</div>` : ""}
        </div>
      `;

      slideCount++;
      if (slideCount % slidesPerPage === 0 || i === slidesToExport.length - 1) {
        htmlContent += "</div>";
      }
    });

    htmlContent += "</body></html>";

    printWindow.document.write(htmlContent);
    printWindow.document.close();
    printWindow.focus();

    setTimeout(() => {
      printWindow.print();
    }, 500);

    hideModal("exportPdfModal");
    addChatMessage(
      "assistant",
      `Exporting ${slidesToExport.length} slide(s) to PDF...`,
    );
  }

  function parseSlideRange(rangeStr) {
    const slides = [];
    const parts = rangeStr.split(",");

    parts.forEach((part) => {
      part = part.trim();
      if (part.includes("-")) {
        const [start, end] = part.split("-").map((n) => parseInt(n.trim()) - 1);
        for (
          let i = Math.max(0, start);
          i <= Math.min(state.slides.length - 1, end);
          i++
        ) {
          if (!slides.includes(i)) slides.push(i);
        }
      } else {
        const num = parseInt(part) - 1;
        if (num >= 0 && num < state.slides.length && !slides.includes(num)) {
          slides.push(num);
        }
      }
    });

    return slides.sort((a, b) => a - b);
  }

  function getLayoutSlidesPerPage(layout) {
    switch (layout) {
      case "full":
      case "notes":
        return 1;
      case "handout-2":
        return 2;
      case "handout-4":
        return 4;
      case "handout-6":
        return 6;
      default:
        return 1;
    }
  }

  function renderSlideContentForExport(slide) {
    let html = "";
    if (slide.elements) {
      slide.elements.forEach((el) => {
        if (el.element_type === "text" && el.content?.text) {
          const fontSize = el.style?.fontSize || 16;
          const fontWeight = el.style?.fontWeight || "normal";
          const color = el.style?.color || "#000";
          html += `<div style="font-size:${fontSize}px;font-weight:${fontWeight};color:${color};margin-bottom:8px;">${escapeHtml(el.content.text)}</div>`;
        }
      });
    }
    return html || "<p>Empty slide</p>";
  }

  let selectedMasterLayout = "title";

  function showMasterSlideModal() {
    showModal("masterSlideModal");
    selectedMasterLayout = "title";

    if (state.theme) {
      const colors = state.theme.colors || {};
      const fonts = state.theme.fonts || {};

      setColorInput("masterPrimaryColor", colors.primary || "#4285f4");
      setColorInput("masterSecondaryColor", colors.secondary || "#34a853");
      setColorInput("masterAccentColor", colors.accent || "#fbbc04");
      setColorInput("masterBgColor", colors.background || "#ffffff");
      setColorInput("masterTextColor", colors.text || "#212121");
      setColorInput("masterTextLightColor", colors.text_light || "#666666");

      setSelectValue("masterHeadingFont", fonts.heading || "Arial");
      setSelectValue("masterBodyFont", fonts.body || "Arial");
    }

    updateMasterPreview();
    updateMasterLayoutSelection();
  }

  function setColorInput(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
  }

  function setSelectValue(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
  }

  function selectMasterLayout(layout) {
    selectedMasterLayout = layout;
    updateMasterLayoutSelection();
  }

  function updateMasterLayoutSelection() {
    document.querySelectorAll(".master-layout-item").forEach((item) => {
      item.classList.toggle(
        "active",
        item.dataset.layout === selectedMasterLayout,
      );
    });
  }

  function updateMasterPreview() {
    const bgColor =
      document.getElementById("masterBgColor")?.value || "#ffffff";
    const textColor =
      document.getElementById("masterTextColor")?.value || "#212121";
    const textLightColor =
      document.getElementById("masterTextLightColor")?.value || "#666666";
    const headingFont =
      document.getElementById("masterHeadingFont")?.value || "Arial";
    const bodyFont =
      document.getElementById("masterBodyFont")?.value || "Arial";

    const previewSlide = document.querySelector(".preview-slide");
    const previewHeading = document.getElementById("previewHeading");
    const previewBody = document.getElementById("previewBody");

    if (previewSlide) {
      previewSlide.style.background = bgColor;
    }
    if (previewHeading) {
      previewHeading.style.color = textColor;
      previewHeading.style.fontFamily = headingFont;
    }
    if (previewBody) {
      previewBody.style.color = textLightColor;
      previewBody.style.fontFamily = bodyFont;
    }
  }

  function applyMasterSlide() {
    const primaryColor =
      document.getElementById("masterPrimaryColor")?.value || "#4285f4";
    const secondaryColor =
      document.getElementById("masterSecondaryColor")?.value || "#34a853";
    const accentColor =
      document.getElementById("masterAccentColor")?.value || "#fbbc04";
    const bgColor =
      document.getElementById("masterBgColor")?.value || "#ffffff";
    const textColor =
      document.getElementById("masterTextColor")?.value || "#212121";
    const textLightColor =
      document.getElementById("masterTextLightColor")?.value || "#666666";
    const headingFont =
      document.getElementById("masterHeadingFont")?.value || "Arial";
    const bodyFont =
      document.getElementById("masterBodyFont")?.value || "Arial";

    saveToHistory();

    state.theme = {
      name: "Custom",
      colors: {
        primary: primaryColor,
        secondary: secondaryColor,
        accent: accentColor,
        background: bgColor,
        text: textColor,
        text_light: textLightColor,
      },
      fonts: {
        heading: headingFont,
        body: bodyFont,
      },
    };

    state.slides.forEach((slide) => {
      slide.background = slide.background || {};
      slide.background.color = bgColor;

      if (slide.elements) {
        slide.elements.forEach((el) => {
          if (el.element_type === "text") {
            el.style = el.style || {};
            const isHeading =
              el.style.fontSize >= 24 || el.style.fontWeight === "bold";
            el.style.fontFamily = isHeading ? headingFont : bodyFont;
            el.style.color = isHeading ? textColor : textLightColor;
          }
        });
      }
    });

    hideModal("masterSlideModal");
    renderThumbnails();
    renderCurrentSlide();

    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Master slide theme applied to all slides!");
  }

  function resetMasterSlide() {
    setColorInput("masterPrimaryColor", "#4285f4");
    setColorInput("masterSecondaryColor", "#34a853");
    setColorInput("masterAccentColor", "#fbbc04");
    setColorInput("masterBgColor", "#ffffff");
    setColorInput("masterTextColor", "#212121");
    setColorInput("masterTextLightColor", "#666666");
    setSelectValue("masterHeadingFont", "Arial");
    setSelectValue("masterBodyFont", "Arial");

    updateMasterPreview();
  }

  window.slidesApp = {
    init,
    addSlide,
    addTextBox,
    addShape,
    addImage,
    duplicateSlide,
    deleteSlide,
    goToSlide,
    startPresentation,
    exitPresentation,
    showModal,
    hideModal,
    toggleChatPanel,
    savePresentation,
    showTransitionsModal,
    showAnimationsModal,
    showSlideSorter,
    exportToPdf,
    showMasterSlideModal,
    showSlideContextMenu,
  };
