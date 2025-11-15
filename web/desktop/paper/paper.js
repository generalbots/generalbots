// Paper module JavaScript
document.addEventListener('alpine:init', () => {
  Alpine.data('paper', () => ({
    showToolbar: false,
    selection: null,

    initEditor() {
      const editor = this.$refs.editor;
      
      // Track selection for floating toolbar
      editor.addEventListener('mouseup', this.updateSelection.bind(this));
      editor.addEventListener('keyup', this.updateSelection.bind(this));
      
      // Show/hide toolbar based on selection
      document.addEventListener('selectionchange', () => {
        const selection = window.getSelection();
        this.showToolbar = !selection.isCollapsed && 
                          editor.contains(selection.anchorNode);
      });
    },

    updateSelection() {
      this.selection = window.getSelection();
    },

    formatText(format) {
      document.execCommand(format, false);
      this.updateSelection();
    },

    alignText(align) {
      document.execCommand('justify' + align, false);
      this.updateSelection();
    },

    isActive(format) {
      return document.queryCommandState(format);
    },

    isAligned(align) {
      return document.queryCommandValue('justify' + align) === align;
    },

    addLink() {
      const url = prompt('Enter URL:');
      if (url) {
        document.execCommand('createLink', false, url);
      }
      this.updateSelection();
    }
  }));
});
