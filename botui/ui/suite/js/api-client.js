/**
 * API Client - Centralized API request handler with authentication
 *
 * This module provides a consistent way to make API requests with:
 * - Automatic auth token injection
 * - Request/response logging in development
 * - Error handling
 * - Retry logic for failed requests
 */

(function () {
  "use strict";

  const AUTH_TOKEN_KEY = "gb-access-token";
  const REFRESH_TOKEN_KEY = "gb-refresh-token";
  const SESSION_ID_KEY = "gb-session-id";

  /**
   * Get the current auth token from storage
   */
  function getAuthToken() {
    // Try localStorage first (persistent login)
    let token = localStorage.getItem(AUTH_TOKEN_KEY);
    if (token) return token;

    // Fall back to sessionStorage
    token = sessionStorage.getItem(AUTH_TOKEN_KEY);
    if (token) return token;

    // Try to get from cookie
    const cookies = document.cookie.split(";");
    for (const cookie of cookies) {
      const [name, value] = cookie.trim().split("=");
      if (name === "gb_session" || name === "session_id") {
        return value;
      }
    }

    return null;
  }

  /**
   * Set the auth token in storage
   */
  function setAuthToken(token, persistent = true) {
    if (persistent) {
      localStorage.setItem(AUTH_TOKEN_KEY, token);
    } else {
      sessionStorage.setItem(AUTH_TOKEN_KEY, token);
    }
  }

  /**
   * Clear the auth token from storage
   */
  function clearAuthToken() {
    localStorage.removeItem(AUTH_TOKEN_KEY);
    sessionStorage.removeItem(AUTH_TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    sessionStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(SESSION_ID_KEY);
    sessionStorage.removeItem(SESSION_ID_KEY);
    localStorage.removeItem("gb-token-expires");
    sessionStorage.removeItem("gb-token-expires");
  }

  /**
   * Get session ID
   */
  function getSessionId() {
    return (
      localStorage.getItem(SESSION_ID_KEY) ||
      sessionStorage.getItem(SESSION_ID_KEY)
    );
  }

  /**
   * Set session ID
   */
  function setSessionId(sessionId, persistent = true) {
    if (persistent) {
      localStorage.setItem(SESSION_ID_KEY, sessionId);
    } else {
      sessionStorage.setItem(SESSION_ID_KEY, sessionId);
    }
  }

  /**
   * Build headers with auth token
   */
  function buildHeaders(customHeaders = {}) {
    const headers = {
      "Content-Type": "application/json",
      ...customHeaders,
    };

    const token = getAuthToken();
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const sessionId = getSessionId();
    if (sessionId) {
      headers["X-Session-ID"] = sessionId;
    }

    return headers;
  }

  /**
   * Make an API request with auth
   */
  async function request(url, options = {}) {
    const {
      method = "GET",
      body = null,
      headers = {},
      retries = 0,
      retryDelay = 1000,
      timeout = 30000,
      credentials = "same-origin",
      skipAuth = false,
    } = options;

    const requestHeaders = skipAuth
      ? { "Content-Type": "application/json", ...headers }
      : buildHeaders(headers);

    const fetchOptions = {
      method,
      headers: requestHeaders,
      credentials,
    };

    if (body && method !== "GET" && method !== "HEAD") {
      fetchOptions.body =
        typeof body === "string" ? body : JSON.stringify(body);
    }

    // Add timeout support
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);
    fetchOptions.signal = controller.signal;

    let lastError;
    let attempt = 0;

    while (attempt <= retries) {
      try {
        const response = await fetch(url, fetchOptions);
        clearTimeout(timeoutId);

        // Handle auth errors
        if (response.status === 401) {
          const error = new Error("Unauthorized");
          error.status = 401;
          error.response = response;

          // Dispatch event for global handling
          window.dispatchEvent(
            new CustomEvent("api:unauthorized", { detail: { url, response } }),
          );

          throw error;
        }

        if (response.status === 403) {
          const error = new Error("Forbidden");
          error.status = 403;
          error.response = response;
          throw error;
        }

        if (!response.ok) {
          let errorBody;
          try {
            errorBody = await response.json();
          } catch {
            errorBody = { error: response.statusText };
          }
          const error = new Error(
            errorBody.message || errorBody.error || "Request failed",
          );
          error.status = response.status;
          error.body = errorBody;
          error.response = response;
          throw error;
        }

        // Handle empty responses
        const contentType = response.headers.get("content-type");
        if (contentType && contentType.includes("application/json")) {
          return await response.json();
        } else if (response.status === 204) {
          return null;
        } else {
          return await response.text();
        }
      } catch (err) {
        clearTimeout(timeoutId);
        lastError = err;

        // Don't retry on auth errors or client errors
        if (err.status && err.status >= 400 && err.status < 500) {
          throw err;
        }

        // Don't retry on abort
        if (err.name === "AbortError") {
          const error = new Error("Request timeout");
          error.status = 408;
          throw error;
        }

        attempt++;
        if (attempt <= retries) {
          await new Promise((resolve) =>
            setTimeout(resolve, retryDelay * attempt),
          );
        }
      }
    }

    throw lastError;
  }

  /**
   * GET request helper
   */
  async function get(url, options = {}) {
    return request(url, { ...options, method: "GET" });
  }

  /**
   * POST request helper
   */
  async function post(url, body, options = {}) {
    return request(url, { ...options, method: "POST", body });
  }

  /**
   * PUT request helper
   */
  async function put(url, body, options = {}) {
    return request(url, { ...options, method: "PUT", body });
  }

  /**
   * PATCH request helper
   */
  async function patch(url, body, options = {}) {
    return request(url, { ...options, method: "PATCH", body });
  }

  /**
   * DELETE request helper
   */
  async function del(url, options = {}) {
    return request(url, { ...options, method: "DELETE" });
  }

  /**
   * Upload file with progress tracking
   */
  async function upload(url, file, options = {}) {
    const { onProgress, ...restOptions } = options;

    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.open("POST", url);

      // Set auth header
      const token = getAuthToken();
      if (token) {
        xhr.setRequestHeader("Authorization", `Bearer ${token}`);
      }

      const sessionId = getSessionId();
      if (sessionId) {
        xhr.setRequestHeader("X-Session-ID", sessionId);
      }

      // Set custom headers
      if (restOptions.headers) {
        Object.entries(restOptions.headers).forEach(([key, value]) => {
          xhr.setRequestHeader(key, value);
        });
      }

      // Track progress
      if (onProgress && xhr.upload) {
        xhr.upload.onprogress = (event) => {
          if (event.lengthComputable) {
            const progress = Math.round((event.loaded / event.total) * 100);
            onProgress(progress, event.loaded, event.total);
          }
        };
      }

      xhr.onload = () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          try {
            resolve(JSON.parse(xhr.responseText));
          } catch {
            resolve(xhr.responseText);
          }
        } else if (xhr.status === 401) {
          window.dispatchEvent(
            new CustomEvent("api:unauthorized", { detail: { url } }),
          );
          reject(new Error("Unauthorized"));
        } else {
          reject(new Error(xhr.statusText || "Upload failed"));
        }
      };

      xhr.onerror = () => reject(new Error("Network error"));
      xhr.ontimeout = () => reject(new Error("Upload timeout"));

      // Create FormData
      const formData = new FormData();
      if (file instanceof File) {
        formData.append("file", file);
      } else if (file instanceof FormData) {
        // If already FormData, use it directly
        xhr.send(file);
        return;
      } else {
        formData.append("file", file);
      }

      xhr.send(formData);
    });
  }

  /**
   * Initialize auth from login response
   */
  function initFromLoginResponse(response) {
    if (response.access_token) {
      setAuthToken(response.access_token, response.remember !== false);
    }
    if (response.session_id) {
      setSessionId(response.session_id, response.remember !== false);
    }
  }

  /**
   * Check if user is authenticated (has token)
   */
  function isAuthenticated() {
    return !!getAuthToken();
  }

  // Global handler for unauthorized responses
  window.addEventListener("api:unauthorized", () => {
    // Clear invalid tokens
    clearAuthToken();

    // Optionally redirect to login
    // Only redirect if not already on login page
    if (
      !window.location.hash.includes("login") &&
      !window.location.pathname.includes("/auth/")
    ) {
      console.warn("Session expired. Please log in again.");
      // Could dispatch event for UI to handle
      window.dispatchEvent(new CustomEvent("auth:expired"));
    }
  });

  // Export API client
  window.ApiClient = {
    request,
    get,
    post,
    put,
    patch,
    delete: del,
    upload,
    getAuthToken,
    setAuthToken,
    clearAuthToken,
    getSessionId,
    setSessionId,
    buildHeaders,
    initFromLoginResponse,
    isAuthenticated,
  };

  // Also export as gbApi for backwards compatibility
  window.gbApi = window.ApiClient;
})();
