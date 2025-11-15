document.addEventListener('alpine:init', () => {
  Alpine.data('auth', () => ({
    email: '',
    password: '',
    rememberMe: false,
    isLoading: false,
    error: '',

    async socialLogin(provider) {
      this.isLoading = true;
      this.error = '';
      
      try {
        // In a real implementation, this would redirect to the auth endpoint
        const authUrl = `${this.getAuthEndpoint()}/oauth/v2/authorize?` +
          `client_id=${this.getClientId()}&` +
          `redirect_uri=${encodeURIComponent(window.location.origin)}&` +
          `response_type=code&` +
          `scope=openid profile email&` +
          `provider=${provider}`;
        
        window.location.href = authUrl;
      } catch (err) {
        this.error = 'Failed to initiate login';
        console.error('Login error:', err);
      } finally {
        this.isLoading = false;
      }
    },

    async emailLogin() {
      this.isLoading = true;
      this.error = '';
      
      try {
        const response = await fetch('/api/auth/login', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: this.email,
            password: this.password,
            rememberMe: this.rememberMe
          })
        });

        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(errorData.message || 'Login failed');
        }

        const data = await response.json();
        localStorage.setItem('authToken', data.token);
        window.location.href = '/tables.html';
      } catch (err) {
        this.error = err.message || 'Login failed. Please check your credentials.';
        console.error('Login error:', err);
      } finally {
        this.isLoading = false;
      }
    },

    getAuthEndpoint() {
      // In a real app, this would come from config
      return 'https://auth.example.com';
    },

    getClientId() {
      // In a real app, this would come from config
      return 'general-bots-client';
    }
  }));
});
