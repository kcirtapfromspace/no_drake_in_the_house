/**
 * Centralized API client for consistent request/response handling
 * Handles authentication, error handling, and retry logic
 */

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  error_code?: string;
  timestamp?: string;
}

export interface ApiError {
  success: false;
  message: string;
  error_code?: string;
  field_errors?: Record<string, string>;
  retry_after?: number;
}

export class ApiClient {
  private baseUrl: string;
  private maxRetries: number = 3;
  private retryDelay: number = 1000; // Base delay in ms

  constructor(baseUrl: string = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  /**
   * Get authentication token from localStorage
   */
  public getAuthToken(): string | null {
    return localStorage.getItem('auth_token');
  }

  /**
   * Set authentication token in localStorage
   */
  public setAuthToken(token: string): void {
    localStorage.setItem('auth_token', token);
  }

  /**
   * Clear authentication token
   */
  public clearAuthToken(): void {
    localStorage.removeItem('auth_token');
  }

  /**
   * Create request headers with authentication
   */
  private createHeaders(includeAuth: boolean = true): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (includeAuth) {
      const token = this.getAuthToken();
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
    }

    return headers;
  }

  /**
   * Handle API response and extract data
   */
  private async handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
    const contentType = response.headers.get('content-type');
    
    let data: any;
    if (contentType && contentType.includes('application/json')) {
      data = await response.json();
    } else {
      data = { message: await response.text() };
    }

    // If response has the expected format, return it
    if (typeof data === 'object' && data !== null && 'success' in data) {
      return data as ApiResponse<T>;
    }

    // Handle non-standard responses
    if (response.ok) {
      return {
        success: true,
        data: data as T,
        timestamp: new Date().toISOString(),
      };
    } else {
      return {
        success: false,
        message: data.message || `HTTP ${response.status}: ${response.statusText}`,
        error_code: `HTTP_${response.status}`,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * Retry logic with exponential backoff
   */
  private async withRetry<T>(
    operation: () => Promise<Response>,
    retries: number = this.maxRetries
  ): Promise<ApiResponse<T>> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const response = await operation();
        
        // Don't retry on authentication errors or client errors (4xx)
        if (response.status === 401 || response.status === 403 || 
            (response.status >= 400 && response.status < 500)) {
          return this.handleResponse<T>(response);
        }

        // Don't retry on success
        if (response.ok) {
          return this.handleResponse<T>(response);
        }

        // Retry on server errors (5xx) or network issues
        if (attempt < retries) {
          const delay = this.retryDelay * Math.pow(2, attempt);
          await new Promise(resolve => setTimeout(resolve, delay));
          continue;
        }

        return this.handleResponse<T>(response);
      } catch (error) {
        lastError = error as Error;
        
        if (attempt < retries) {
          const delay = this.retryDelay * Math.pow(2, attempt);
          await new Promise(resolve => setTimeout(resolve, delay));
          continue;
        }
      }
    }

    // If all retries failed, return error response
    return {
      success: false,
      message: lastError?.message || 'Network request failed after retries',
      error_code: 'NETWORK_ERROR',
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * GET request
   */
  async get<T>(endpoint: string, includeAuth: boolean = true): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    return this.withRetry<T>(() =>
      fetch(url, {
        method: 'GET',
        headers: this.createHeaders(includeAuth),
      })
    );
  }

  /**
   * POST request
   */
  async post<T>(
    endpoint: string, 
    data?: any, 
    includeAuth: boolean = true
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    return this.withRetry<T>(() =>
      fetch(url, {
        method: 'POST',
        headers: this.createHeaders(includeAuth),
        body: data ? JSON.stringify(data) : undefined,
      })
    );
  }

  /**
   * PUT request
   */
  async put<T>(
    endpoint: string, 
    data?: any, 
    includeAuth: boolean = true
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    return this.withRetry<T>(() =>
      fetch(url, {
        method: 'PUT',
        headers: this.createHeaders(includeAuth),
        body: data ? JSON.stringify(data) : undefined,
      })
    );
  }

  /**
   * DELETE request
   */
  async delete<T>(endpoint: string, includeAuth: boolean = true): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    return this.withRetry<T>(() =>
      fetch(url, {
        method: 'DELETE',
        headers: this.createHeaders(includeAuth),
      })
    );
  }

  /**
   * Handle authentication errors by attempting token refresh
   */
  async handleAuthError(): Promise<boolean> {
    try {
      const refreshToken = localStorage.getItem('refresh_token');
      if (!refreshToken) {
        return false;
      }

      const response = await this.post<{access_token: string, refresh_token: string}>(
        '/api/v1/auth/refresh',
        { refresh_token: refreshToken },
        false // Don't include auth for refresh
      );

      if (response.success && response.data) {
        this.setAuthToken(response.data.access_token);
        localStorage.setItem('refresh_token', response.data.refresh_token);
        return true;
      }

      return false;
    } catch (error) {
      console.error('Token refresh failed:', error);
      return false;
    }
  }

  /**
   * Make authenticated request with automatic token refresh
   */
  async authenticatedRequest<T>(
    method: 'GET' | 'POST' | 'PUT' | 'DELETE',
    endpoint: string,
    data?: any
  ): Promise<ApiResponse<T>> {
    let response: ApiResponse<T>;

    // Make initial request
    switch (method) {
      case 'GET':
        response = await this.get<T>(endpoint);
        break;
      case 'POST':
        response = await this.post<T>(endpoint, data);
        break;
      case 'PUT':
        response = await this.put<T>(endpoint, data);
        break;
      case 'DELETE':
        response = await this.delete<T>(endpoint);
        break;
    }

    // If unauthorized, try to refresh token and retry once
    if (!response.success && response.error_code === 'HTTP_401') {
      const refreshed = await this.handleAuthError();
      
      if (refreshed) {
        // Retry the original request
        switch (method) {
          case 'GET':
            response = await this.get<T>(endpoint);
            break;
          case 'POST':
            response = await this.post<T>(endpoint, data);
            break;
          case 'PUT':
            response = await this.put<T>(endpoint, data);
            break;
          case 'DELETE':
            response = await this.delete<T>(endpoint);
            break;
        }
      } else {
        // Clear tokens and redirect to login
        this.clearAuthToken();
        localStorage.removeItem('refresh_token');
        
        // Dispatch custom event for auth failure
        window.dispatchEvent(new CustomEvent('auth:logout', {
          detail: { reason: 'token_refresh_failed' }
        }));
      }
    }

    return response;
  }
}

// Create singleton instance
export const apiClient = new ApiClient();