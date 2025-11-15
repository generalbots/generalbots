// Editor module JavaScript using Alpine.js
document.addEventListener('alpine:init', () => {
  Alpine.data('editor', () => ({
    fileName: 'Document 1',
    fontSize: '12',
    fontFamily: 'Calibri',
    textColor: '#000000',
    highlightColor: '#ffff00',
    activeTab: 'home',
    zoom: 100,
    pages: [1],
    content: '',

    init() {
      // Initialize with default content
      this.content = `
        <h1 style="text-align: center; font-size: 24px; margin-bottom: 20px;">${this.fileName}</h1>
        <p><br></p>
        <p>Start typing your document here...</p>
        <p><br></p>
      `;
    },

    // Ribbon tab switching
    setActiveTab(tab) {
      this.activeTab = tab;
    },

    // Formatting methods
    formatBold() {
      document.execCommand('bold', false);
    },
    formatItalic() {
      document.execCommand('italic', false);
    },
    formatUnderline() {
      document.execCommand('underline', false);
    },
    alignLeft() {
      document.execCommand('justifyLeft', false);
    },
    alignCenter() {
      document.execCommand('justifyCenter', false);
    },
    alignRight() {
      document.execCommand('justifyRight', false);
    },
    alignJustify() {
      document.execCommand('justifyFull', false);
    },

    // Zoom controls
    zoomOut() {
      this.zoom = Math.max(50, this.zoom - 10);
      this.updateZoom();
    },
    zoomIn() {
      this.zoom = Math.min(200, this.zoom + 10);
      this.updateZoom();
    },
    updateZoom() {
      document.querySelector('.pages-container').style.transform = `scale(${this.zoom / 100})`;
    },

    // Save document
    saveDocument() {
      const content = document.getElementById('editor-content').innerHTML;
      const blob = new Blob([content], { type: 'text/html' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${this.fileName}.html`;
      a.click();
      URL.revokeObjectURL(url);
    }
  }));
});
