function mailApp() {
  return {
    currentFolder: 'Inbox',
    selectedMail: null,
    
    folders: [
      { name: 'Inbox', icon: '📥', count: 4 },
      { name: 'Sent', icon: '📤', count: 0 },
      { name: 'Drafts', icon: '📝', count: 2 },
      { name: 'Starred', icon: '⭐', count: 0 },
      { name: 'Trash', icon: '🗑', count: 0 }
    ],
    
    mails: [
      {
        id: 1,
        from: 'Sarah Johnson',
        to: 'me@example.com',
        subject: 'Q4 Project Update',
        preview: 'Hi team, I wanted to share the latest updates on our Q4 projects...',
        body: '<p>Hi team,</p><p>I wanted to share the latest updates on our Q4 projects. We\'ve made significant progress on the main deliverables and are on track to meet our goals.</p><p>Please review the attached documents and let me know if you have any questions.</p><p>Best regards,<br>Sarah</p>',
        time: '10:30 AM',
        date: 'Nov 15, 2025',
        read: false
      },
      {
        id: 2,
        from: 'Mike Chen',
        to: 'me@example.com',
        subject: 'Meeting Tomorrow',
        preview: 'Don\'t forget about our meeting tomorrow at 2 PM...',
        body: '<p>Hi,</p><p>Don\'t forget about our meeting tomorrow at 2 PM to discuss the new features.</p><p>See you then!<br>Mike</p>',
        time: '9:15 AM',
        date: 'Nov 15, 2025',
        read: false
      },
      {
        id: 3,
        from: 'Emma Wilson',
        to: 'me@example.com',
        subject: 'Design Review Complete',
        preview: 'The design review for the new dashboard is complete...',
        body: '<p>Hi,</p><p>The design review for the new dashboard is complete. Overall, the team is happy with the direction.</p><p>I\'ve made the requested changes and updated the Figma file.</p><p>Thanks,<br>Emma</p>',
        time: 'Yesterday',
        date: 'Nov 14, 2025',
        read: true
      },
      {
        id: 4,
        from: 'David Lee',
        to: 'me@example.com',
        subject: 'Budget Approval Needed',
        preview: 'Could you please review and approve the Q1 budget?',
        body: '<p>Hi,</p><p>Could you please review and approve the Q1 budget when you get a chance?</p><p>It\'s attached to this email.</p><p>Thanks,<br>David</p>',
        time: 'Yesterday',
        date: 'Nov 14, 2025',
        read: false
      }
    ],
    
    get filteredMails() {
      return this.mails;
    },
    
    selectMail(mail) {
      this.selectedMail = mail;
      mail.read = true;
      this.updateFolderCounts();
    },
    
    updateFolderCounts() {
      const inbox = this.folders.find(f => f.name === 'Inbox');
      if (inbox) {
        inbox.count = this.mails.filter(m => !m.read).length;
      }
    }
  };
}
