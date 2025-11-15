function driveApp() {
  return {
    current: 'All Files',
    search: '',
    selectedFile: null,
    navItems: [
      { name: 'All Files', icon: '📁' },
      { name: 'Recent', icon: '🕐' },
      { name: 'Starred', icon: '⭐' },
      { name: 'Shared', icon: '👥' },
      { name: 'Trash', icon: '🗑' }
    ],
    files: [
      { id: 1, name: 'Project Proposal.pdf', type: 'PDF', icon: '📄', size: '2.4 MB', date: 'Nov 10, 2025' },
      { id: 2, name: 'Design Assets', type: 'Folder', icon: '📁', size: '—', date: 'Nov 12, 2025' },
      { id: 3, name: 'Meeting Notes.docx', type: 'Document', icon: '📝', size: '156 KB', date: 'Nov 14, 2025' },
      { id: 4, name: 'Budget 2025.xlsx', type: 'Spreadsheet', icon: '📊', size: '892 KB', date: 'Nov 13, 2025' },
      { id: 5, name: 'Presentation.pptx', type: 'Presentation', icon: '📽', size: '5.2 MB', date: 'Nov 11, 2025' },
      { id: 6, name: 'team-photo.jpg', type: 'Image', icon: '🖼', size: '3.1 MB', date: 'Nov 9, 2025' },
      { id: 7, name: 'source-code.zip', type: 'Archive', icon: '📦', size: '12.8 MB', date: 'Nov 8, 2025' },
      { id: 8, name: 'video-demo.mp4', type: 'Video', icon: '🎬', size: '45.2 MB', date: 'Nov 7, 2025' }
    ],
    get filteredFiles() {
      return this.files.filter(file => 
        file.name.toLowerCase().includes(this.search.toLowerCase())
      );
    }
  };
}
