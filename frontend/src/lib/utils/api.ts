import config from './config';
import { apiClient, type ApiResponse } from './api-client';

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiCall<T = unknown>(
  endpoint: string,
  options: RequestInit = {},
): Promise<ApiResponse<T>> {
  const normalizedEndpoint =
    endpoint.startsWith('/api/') || /^https?:\/\//i.test(endpoint)
      ? endpoint
      : config.getApiEndpoint(endpoint);
  const method = (options.method || 'GET').toUpperCase() as
    | 'GET'
    | 'POST'
    | 'PUT'
    | 'DELETE';
  const includeAuth = Boolean(localStorage.getItem('auth_token'));
  const parsedBody =
    typeof options.body === 'string' && options.body.length > 0
      ? JSON.parse(options.body)
      : undefined;

  switch (method) {
    case 'POST':
      return await apiClient.post<T>(normalizedEndpoint, parsedBody, includeAuth);
    case 'PUT':
      return await apiClient.put<T>(normalizedEndpoint, parsedBody, includeAuth);
    case 'DELETE':
      return await apiClient.delete<T>(normalizedEndpoint, includeAuth);
    case 'GET':
    default:
      return await apiClient.get<T>(normalizedEndpoint, includeAuth);
  }
}

export const api = {
  get: <T = unknown>(endpoint: string) => apiCall<T>(endpoint),
  post: <T = unknown>(endpoint: string, data?: unknown) =>
    apiCall<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    }),
  put: <T = unknown>(endpoint: string, data?: unknown) =>
    apiCall<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    }),
  delete: <T = unknown>(endpoint: string, data?: unknown) =>
    apiCall<T>(endpoint, {
      method: 'DELETE',
      body: data ? JSON.stringify(data) : undefined,
    }),
};
