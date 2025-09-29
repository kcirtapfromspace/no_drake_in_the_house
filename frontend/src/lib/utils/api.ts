import { config } from './config';

export interface ApiResponse<T = any> {
  success: boolean;
  data: T;
  message: string;
}

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiCall<T = any>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const token = localStorage.getItem('auth_token');
  
  const requestConfig: RequestInit = {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` }),
      ...options.headers,
    },
  };

  try {
    const url = config.getApiEndpoint(endpoint);
    const response = await fetch(url, requestConfig);
    const result = await response.json();

    if (!response.ok) {
      throw new ApiError(response.status, result.message || 'Request failed');
    }

    return result;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(0, 'Network error occurred');
  }
}

export const api = {
  get: <T = any>(endpoint: string) => apiCall<T>(endpoint),
  post: <T = any>(endpoint: string, data?: any) => 
    apiCall<T>(endpoint, { method: 'POST', body: JSON.stringify(data) }),
  put: <T = any>(endpoint: string, data?: any) => 
    apiCall<T>(endpoint, { method: 'PUT', body: JSON.stringify(data) }),
  delete: <T = any>(endpoint: string, data?: any) => 
    apiCall<T>(endpoint, { 
      method: 'DELETE', 
      body: data ? JSON.stringify(data) : undefined 
    }),
};