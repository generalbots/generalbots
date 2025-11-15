// Settings module JavaScript
document.addEventListener('alpine:init', () => {
  Alpine.data('settings', () => ({
    username: '',
    email: '',
    bio: '',
    errors: {},

    submitForm() {
      this.errors = {};
      let isValid = true;

      // Validate username
      if (this.username.length < 2) {
        this.errors.username = "Username must be at least 2 characters.";
        isValid = false;
      } else if (this.username.length > 30) {
        this.errors.username = "Username must not be longer than 30 characters.";
        isValid = false;
      }

      // Validate email
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(this.email)) {
        this.errors.email = "Please enter a valid email address.";
        isValid = false;
      }

      // Validate bio
      if (this.bio.length < 4) {
        this.errors.bio = "Bio must be at least 4 characters.";
        isValid = false;
      } else if (this.bio.length > 160) {
        this.errors.bio = "Bio must not be longer than 160 characters.";
        isValid = false;
      }

      if (isValid) {
        this.saveSettings();
      }
    },

    saveSettings() {
      const settings = {
        username: this.username,
        email: this.email,
        bio: this.bio
      };
      console.log('Saving settings:', settings);
      // Here you would typically send the data to a server
    }
  }));
});
