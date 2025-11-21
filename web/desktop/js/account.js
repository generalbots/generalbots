window.accountApp = function accountApp() {
  return {
    currentTab: "profile",
    loading: false,
    saving: false,
    addingAccount: false,
    testingAccount: null,
    showAddAccount: false,

    // Profile data
    profile: {
      username: "user",
      email: "user@example.com",
      displayName: "",
      phone: "",
    },

    // Email accounts
    emailAccounts: [],

    // New account form
    newAccount: {
      email: "",
      displayName: "",
      imapServer: "imap.gmail.com",
      imapPort: 993,
      smtpServer: "smtp.gmail.com",
      smtpPort: 587,
      username: "",
      password: "",
      isPrimary: false,
    },

    // Drive settings
    driveSettings: {
      server: "drive.example.com",
      autoSync: true,
      offlineMode: false,
    },

    // Storage info
    storageUsed: "12.3 GB",
    storageTotal: "50 GB",
    storageUsagePercent: 25,

    // Security
    security: {
      currentPassword: "",
      newPassword: "",
      confirmPassword: "",
    },

    activeSessions: [
      {
        id: "1",
        device: "Chrome on Windows",
        lastActive: "2 hours ago",
        ip: "192.168.1.100",
      },
      {
        id: "2",
        device: "Firefox on Linux",
        lastActive: "1 day ago",
        ip: "192.168.1.101",
      },
    ],

    // Initialize
    async init() {
      console.log("✓ Account component initialized");
      await this.loadProfile();
      await this.loadEmailAccounts();

      // Listen for section visibility
      const section = document.querySelector("#section-account");
      if (section) {
        section.addEventListener("section-shown", () => {
          console.log("Account section shown");
          this.loadEmailAccounts();
        });
      }
    },

    // Profile methods
    async loadProfile() {
      try {
        // TODO: Implement actual profile loading from API
        // const response = await fetch('/api/user/profile');
        // const data = await response.json();
        // this.profile = data;
        console.log("Profile loaded (mock data)");
      } catch (error) {
        console.error("Error loading profile:", error);
        this.showNotification("Failed to load profile", "error");
      }
    },

    async saveProfile() {
      this.saving = true;
      try {
        // TODO: Implement actual profile saving
        // const response = await fetch('/api/user/profile', {
        //   method: 'PUT',
        //   headers: { 'Content-Type': 'application/json' },
        //   body: JSON.stringify(this.profile)
        // });
        // if (!response.ok) throw new Error('Failed to save profile');

        await new Promise((resolve) => setTimeout(resolve, 1000)); // Mock delay
        this.showNotification("Profile saved successfully", "success");
      } catch (error) {
        console.error("Error saving profile:", error);
        this.showNotification("Failed to save profile", "error");
      } finally {
        this.saving = false;
      }
    },

    // Email account methods
    async loadEmailAccounts() {
      this.loading = true;
      try {
        const response = await fetch("/api/email/accounts");
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const result = await response.json();
        if (result.success && result.data) {
          this.emailAccounts = result.data;
          console.log(`Loaded ${this.emailAccounts.length} email accounts`);
        } else {
          console.warn("No email accounts found");
          this.emailAccounts = [];
        }
      } catch (error) {
        console.error("Error loading email accounts:", error);
        this.emailAccounts = [];
        // Don't show error notification on first load if no accounts exist
      } finally {
        this.loading = false;
      }
    },

    async addEmailAccount() {
      this.addingAccount = true;
      try {
        const response = await fetch("/api/email/accounts/add", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            email: this.newAccount.email,
            display_name: this.newAccount.displayName || null,
            imap_server: this.newAccount.imapServer,
            imap_port: parseInt(this.newAccount.imapPort),
            smtp_server: this.newAccount.smtpServer,
            smtp_port: parseInt(this.newAccount.smtpPort),
            username: this.newAccount.username,
            password: this.newAccount.password,
            is_primary: this.newAccount.isPrimary,
          }),
        });

        const result = await response.json();

        if (!response.ok || !result.success) {
          throw new Error(result.message || "Failed to add email account");
        }

        this.showNotification("Email account added successfully", "success");
        this.showAddAccount = false;
        this.resetNewAccountForm();
        await this.loadEmailAccounts();

        // Notify mail app to refresh if it's open
        window.dispatchEvent(new CustomEvent("email-accounts-updated"));
      } catch (error) {
        console.error("Error adding email account:", error);
        this.showNotification(
          error.message || "Failed to add email account",
          "error"
        );
      } finally {
        this.addingAccount = false;
      }
    },

    resetNewAccountForm() {
      this.newAccount = {
        email: "",
        displayName: "",
        imapServer: "imap.gmail.com",
        imapPort: 993,
        smtpServer: "smtp.gmail.com",
        smtpPort: 587,
        username: "",
        password: "",
        isPrimary: false,
      };
    },

    async testAccount(account) {
      this.testingAccount = account.id;
      try {
        // Test connection by trying to list emails
        const response = await fetch("/api/email/list", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            account_id: account.id,
            folder: "INBOX",
            limit: 1,
          }),
        });

        const result = await response.json();

        if (!response.ok || !result.success) {
          throw new Error(result.message || "Connection test failed");
        }

        this.showNotification(
          "Account connection test successful",
          "success"
        );
      } catch (error) {
        console.error("Error testing account:", error);
        this.showNotification(
          error.message || "Account connection test failed",
          "error"
        );
      } finally {
        this.testingAccount = null;
      }
    },

    editAccount(account) {
      // TODO: Implement account editing
      this.showNotification("Edit functionality coming soon", "info");
    },

    async deleteAccount(accountId) {
      if (
        !confirm(
          "Are you sure you want to delete this email account? This cannot be undone."
        )
      ) {
        return;
      }

      try {
        const response = await fetch(`/api/email/accounts/${accountId}`, {
          method: "DELETE",
        });

        const result = await response.json();

        if (!response.ok || !result.success) {
          throw new Error(result.message || "Failed to delete account");
        }

        this.showNotification("Email account deleted", "success");
        await this.loadEmailAccounts();

        // Notify mail app to refresh
        window.dispatchEvent(new CustomEvent("email-accounts-updated"));
      } catch (error) {
        console.error("Error deleting account:", error);
        this.showNotification(
          error.message || "Failed to delete account",
          "error"
        );
      }
    },

    // Quick setup for common providers
    setupGmail() {
      this.newAccount.imapServer = "imap.gmail.com";
      this.newAccount.imapPort = 993;
      this.newAccount.smtpServer = "smtp.gmail.com";
      this.newAccount.smtpPort = 587;
    },

    setupOutlook() {
      this.newAccount.imapServer = "outlook.office365.com";
      this.newAccount.imapPort = 993;
      this.newAccount.smtpServer = "smtp.office365.com";
      this.newAccount.smtpPort = 587;
    },

    setupYahoo() {
      this.newAccount.imapServer = "imap.mail.yahoo.com";
      this.newAccount.imapPort = 993;
      this.newAccount.smtpServer = "smtp.mail.yahoo.com";
      this.newAccount.smtpPort = 587;
    },

    // Drive settings methods
    async saveDriveSettings() {
      this.saving = true;
      try {
        // TODO: Implement actual drive settings saving
        await new Promise((resolve) => setTimeout(resolve, 1000)); // Mock delay
        this.showNotification("Drive settings saved successfully", "success");
      } catch (error) {
        console.error("Error saving drive settings:", error);
        this.showNotification("Failed to save drive settings", "error");
      } finally {
        this.saving = false;
      }
    },

    // Security methods
    async changePassword() {
      if (this.security.newPassword !== this.security.confirmPassword) {
        this.showNotification("Passwords do not match", "error");
        return;
      }

      if (this.security.newPassword.length < 8) {
        this.showNotification(
          "Password must be at least 8 characters",
          "error"
        );
        return;
      }

      try {
        // TODO: Implement actual password change
        // const response = await fetch('/api/user/change-password', {
        //   method: 'POST',
        //   headers: { 'Content-Type': 'application/json' },
        //   body: JSON.stringify({
        //     current_password: this.security.currentPassword,
        //     new_password: this.security.newPassword
        //   })
        // });

        await new Promise((resolve) => setTimeout(resolve, 1000)); // Mock delay
        this.showNotification("Password changed successfully", "success");
        this.security = {
          currentPassword: "",
          newPassword: "",
          confirmPassword: "",
        };
      } catch (error) {
        console.error("Error changing password:", error);
        this.showNotification("Failed to change password", "error");
      }
    },

    async revokeSession(sessionId) {
      if (
        !confirm(
          "Are you sure you want to revoke this session? The user will be logged out."
        )
      ) {
        return;
      }

      try {
        // TODO: Implement actual session revocation
        await new Promise((resolve) => setTimeout(resolve, 500)); // Mock delay

        this.activeSessions = this.activeSessions.filter(
          (s) => s.id !== sessionId
        );
        this.showNotification("Session revoked successfully", "success");
      } catch (error) {
        console.error("Error revoking session:", error);
        this.showNotification("Failed to revoke session", "error");
      }
    },

    // Notification helper
    showNotification(message, type = "info") {
      // Try to use the global notification system if available
      if (window.showNotification) {
        window.showNotification(message, type);
      } else {
        // Fallback to alert
        alert(message);
      }
    },
  };
};

console.log("✓ Account app function registered");
