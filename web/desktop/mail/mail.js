window.mailApp = function mailApp() {
  return {
    currentFolder: "Inbox",
    selectedMail: null,
    composing: false,
    loading: false,
    sending: false,
    currentAccountId: null,

    folders: [
      { name: "Inbox", icon: "📥", count: 0 },
      { name: "Sent", icon: "📤", count: 0 },
      { name: "Drafts", icon: "📝", count: 0 },
      { name: "Starred", icon: "⭐", count: 0 },
      { name: "Trash", icon: "🗑", count: 0 },
    ],

    mails: [],

    // Compose form
    composeForm: {
      to: "",
      cc: "",
      bcc: "",
      subject: "",
      body: "",
    },

    // User accounts
    emailAccounts: [],

    get filteredMails() {
      // Filter by folder
      let filtered = this.mails;

      // TODO: Implement folder filtering based on IMAP folders
      // For now, show all in Inbox

      return filtered;
    },

    selectMail(mail) {
      this.selectedMail = mail;
      mail.read = true;
      this.updateFolderCounts();

      // TODO: Mark as read on server
      this.markEmailAsRead(mail.id);
    },

    updateFolderCounts() {
      const inbox = this.folders.find((f) => f.name === "Inbox");
      if (inbox) {
        inbox.count = this.mails.filter((m) => !m.read).length;
      }
    },

    async init() {
      console.log("✓ Mail component initialized");

      // Load email accounts first
      await this.loadEmailAccounts();

      // If we have accounts, load emails for the first/primary account
      if (this.emailAccounts.length > 0) {
        const primaryAccount =
          this.emailAccounts.find((a) => a.is_primary) || this.emailAccounts[0];
        this.currentAccountId = primaryAccount.id;
        await this.loadEmails();
      }

      // Listen for account updates
      window.addEventListener("email-accounts-updated", () => {
        this.loadEmailAccounts();
      });

      // Listen for section visibility
      const section = document.querySelector("#section-mail");
      if (section) {
        section.addEventListener("section-shown", () => {
          console.log("Mail section shown");
          if (this.currentAccountId) {
            this.loadEmails();
          }
        });

        section.addEventListener("section-hidden", () => {
          console.log("Mail section hidden");
        });
      }
    },

    async loadEmailAccounts() {
      try {
        const response = await fetch("/api/email/accounts");
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const result = await response.json();
        if (result.success && result.data) {
          this.emailAccounts = result.data;
          console.log(`Loaded ${this.emailAccounts.length} email accounts`);

          // If no current account is selected, select the first/primary one
          if (!this.currentAccountId && this.emailAccounts.length > 0) {
            const primaryAccount =
              this.emailAccounts.find((a) => a.is_primary) ||
              this.emailAccounts[0];
            this.currentAccountId = primaryAccount.id;
            await this.loadEmails();
          }
        } else {
          this.emailAccounts = [];
          console.warn("No email accounts configured");
        }
      } catch (error) {
        console.error("Error loading email accounts:", error);
        this.emailAccounts = [];
      }
    },

    async loadEmails() {
      if (!this.currentAccountId) {
        console.warn("No email account selected");
        this.showNotification(
          "Please configure an email account first",
          "warning",
        );
        return;
      }

      this.loading = true;
      try {
        const response = await fetch("/api/email/list", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            account_id: this.currentAccountId,
            folder: this.currentFolder.toUpperCase(),
            limit: 50,
            offset: 0,
          }),
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const result = await response.json();

        if (result.success && result.data) {
          this.mails = result.data.map((email) => ({
            id: email.id,
            from: email.from_name || email.from_email,
            to: email.to,
            subject: email.subject,
            preview: email.preview,
            body: email.body,
            time: email.time,
            date: email.date,
            read: email.read,
            has_attachments: email.has_attachments,
            folder: email.folder,
          }));

          this.updateFolderCounts();
          console.log(
            `Loaded ${this.mails.length} emails from ${this.currentFolder}`,
          );
        } else {
          console.warn("Failed to load emails:", result.message);
          this.mails = [];
        }
      } catch (error) {
        console.error("Error loading emails:", error);
        this.showNotification(
          "Failed to load emails: " + error.message,
          "error",
        );
        this.mails = [];
      } finally {
        this.loading = false;
      }
    },

    async switchAccount(accountId) {
      this.currentAccountId = accountId;
      this.selectedMail = null;
      await this.loadEmails();
    },

    async switchFolder(folderName) {
      this.currentFolder = folderName;
      this.selectedMail = null;
      await this.loadEmails();
    },

    async markEmailAsRead(emailId) {
      if (!this.currentAccountId) return;

      try {
        await fetch("/api/email/mark", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            account_id: this.currentAccountId,
            email_id: emailId,
            read: true,
          }),
        });
      } catch (error) {
        console.error("Error marking email as read:", error);
      }
    },

    async deleteEmail(emailId) {
      if (!this.currentAccountId) return;

      if (!confirm("Are you sure you want to delete this email?")) {
        return;
      }

      try {
        const response = await fetch("/api/email/delete", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            account_id: this.currentAccountId,
            email_id: emailId,
          }),
        });

        const result = await response.json();

        if (result.success) {
          this.showNotification("Email deleted", "success");
          this.selectedMail = null;
          await this.loadEmails();
        } else {
          throw new Error(result.message || "Failed to delete email");
        }
      } catch (error) {
        console.error("Error deleting email:", error);
        this.showNotification(
          "Failed to delete email: " + error.message,
          "error",
        );
      }
    },

    startCompose() {
      this.composing = true;
      this.composeForm = {
        to: "",
        cc: "",
        bcc: "",
        subject: "",
        body: "",
      };
    },

    startReply() {
      if (!this.selectedMail) return;

      this.composing = true;
      this.composeForm = {
        to: this.selectedMail.from,
        cc: "",
        bcc: "",
        subject: "Re: " + this.selectedMail.subject,
        body:
          "\n\n---\nOn " +
          this.selectedMail.date +
          ", " +
          this.selectedMail.from +
          " wrote:\n" +
          this.selectedMail.body,
      };
    },

    startForward() {
      if (!this.selectedMail) return;

      this.composing = true;
      this.composeForm = {
        to: "",
        cc: "",
        bcc: "",
        subject: "Fwd: " + this.selectedMail.subject,
        body:
          "\n\n---\nForwarded message:\nFrom: " +
          this.selectedMail.from +
          "\nSubject: " +
          this.selectedMail.subject +
          "\n\n" +
          this.selectedMail.body,
      };
    },

    cancelCompose() {
      if (
        this.composeForm.to ||
        this.composeForm.subject ||
        this.composeForm.body
      ) {
        if (!confirm("Discard draft?")) {
          return;
        }
      }
      this.composing = false;
    },

    async sendEmail() {
      if (!this.currentAccountId) {
        this.showNotification("Please select an email account", "error");
        return;
      }

      if (!this.composeForm.to) {
        this.showNotification("Please enter a recipient", "error");
        return;
      }

      if (!this.composeForm.subject) {
        this.showNotification("Please enter a subject", "error");
        return;
      }

      this.sending = true;
      try {
        const response = await fetch("/api/email/send", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            account_id: this.currentAccountId,
            to: this.composeForm.to,
            cc: this.composeForm.cc || null,
            bcc: this.composeForm.bcc || null,
            subject: this.composeForm.subject,
            body: this.composeForm.body,
            is_html: false,
          }),
        });

        const result = await response.json();

        if (!response.ok || !result.success) {
          throw new Error(result.message || "Failed to send email");
        }

        this.showNotification("Email sent successfully", "success");
        this.composing = false;
        this.composeForm = {
          to: "",
          cc: "",
          bcc: "",
          subject: "",
          body: "",
        };

        // Reload emails to show sent message in Sent folder
        await this.loadEmails();
      } catch (error) {
        console.error("Error sending email:", error);
        this.showNotification(
          "Failed to send email: " + error.message,
          "error",
        );
      } finally {
        this.sending = false;
      }
    },

    async saveDraft() {
      if (!this.currentAccountId) {
        this.showNotification("Please select an email account", "error");
        return;
      }

      try {
        const response = await fetch("/api/email/draft", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            account_id: this.currentAccountId,
            to: this.composeForm.to,
            cc: this.composeForm.cc || null,
            bcc: this.composeForm.bcc || null,
            subject: this.composeForm.subject,
            body: this.composeForm.body,
          }),
        });

        const result = await response.json();

        if (result.success) {
          this.showNotification("Draft saved", "success");
        } else {
          throw new Error(result.message || "Failed to save draft");
        }
      } catch (error) {
        console.error("Error saving draft:", error);
        this.showNotification(
          "Failed to save draft: " + error.message,
          "error",
        );
      }
    },

    async refreshEmails() {
      await this.loadEmails();
    },

    openAccountSettings() {
      // Trigger navigation to account settings
      if (window.showSection) {
        window.showSection("account");
      } else {
        this.showNotification(
          "Please configure email accounts in Settings",
          "info",
        );
      }
    },

    getCurrentAccountName() {
      if (!this.currentAccountId) return "No account";
      const account = this.emailAccounts.find(
        (a) => a.id === this.currentAccountId,
      );
      return account ? account.display_name || account.email : "Unknown";
    },

    showNotification(message, type = "info") {
      // Try to use the global notification system if available
      if (window.showNotification) {
        window.showNotification(message, type);
      } else {
        console.log(`[${type.toUpperCase()}] ${message}`);
      }
    },
  };
};

console.log("✓ Mail app function registered");
