import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import RegisterForm from '../RegisterForm.svelte';
import { mockFetch, createMockAuthResponse, createMockAuthStore } from '../../../test/helpers';

// Mock the auth store
const mockAuthStore = createMockAuthStore();
vi.mock('$lib/stores/auth', () => ({
  auth: mockAuthStore,
}));

describe('RegisterForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders registration form with all required fields', () => {
    render(RegisterForm);
    
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/^password$/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /create account/i })).toBeInTheDocument();
  });

  it('disables submit button when form is invalid', async () => {
    render(RegisterForm);
    
    const submitButton = screen.getByRole('button', { name: /create account/i });
    expect(submitButton).toBeDisabled();
  });

  it('shows validation error for invalid email format', async () => {
    render(RegisterForm);
    
    const emailInput = screen.getByLabelText(/email/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(emailInput, { target: { value: 'invalid-email' } });
    await fireEvent.click(submitButton);
    
    await waitFor(() => {
      expect(screen.getByText(/please enter a valid email/i)).toBeInTheDocument();
    });
  });

  it('shows password strength requirements', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    
    await fireEvent.input(passwordInput, { target: { value: '123' } });
    
    // Should show password requirements
    expect(screen.getByText(/password must contain:/i)).toBeInTheDocument();
    expect(screen.getByText(/at least 8 characters/i)).toBeInTheDocument();
  });

  it('shows validation error when passwords do not match', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(passwordInput, { target: { value: 'password123' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'different123' } });
    await fireEvent.click(submitButton);
    
    await waitFor(() => {
      expect(screen.getByText(/passwords do not match/i)).toBeInTheDocument();
    });
  });

  it('shows password strength indicator', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    
    // Weak password
    await fireEvent.input(passwordInput, { target: { value: '123' } });
    expect(screen.getByText(/weak/i)).toBeInTheDocument();
    
    // Fair password
    await fireEvent.input(passwordInput, { target: { value: 'password123' } });
    expect(screen.getByText(/fair/i)).toBeInTheDocument();
    
    // Strong password
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    expect(screen.getByText(/strong/i)).toBeInTheDocument();
  });

  it('dispatches register event with valid data', async () => {
    const component = render(RegisterForm);
    let dispatchedEvent: any = null;
    
    component.component.$on('register', (event) => {
      dispatchedEvent = event.detail;
    });
    
    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/^password$/i);
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.click(submitButton);
    
    expect(dispatchedEvent).toEqual({
      email: 'test@example.com',
      password: 'SecurePassword123!',
    });
  });

  it('displays error message when error prop is set', () => {
    render(RegisterForm, { error: 'Email already exists' });
    
    expect(screen.getByText(/email already exists/i)).toBeInTheDocument();
  });

  it('shows loading state when isLoading prop is true', () => {
    render(RegisterForm, { isLoading: true });
    
    const submitButton = screen.getByRole('button', { name: /creating account/i });
    expect(submitButton).toBeInTheDocument();
    expect(submitButton).toBeDisabled();
  });

  it('enables submit button when form is valid', async () => {
    render(RegisterForm);
    
    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/^password$/i);
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'SecurePassword123!' } });
    
    expect(submitButton).not.toBeDisabled();
  });

  it('shows switch to login button', () => {
    render(RegisterForm);
    
    const switchButton = screen.getByRole('button', { name: /already have an account/i });
    expect(switchButton).toBeInTheDocument();
  });

  it('has accessible form labels and ARIA attributes', () => {
    render(RegisterForm);
    
    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/^password$/i);
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    expect(emailInput).toHaveAttribute('type', 'email');
    expect(emailInput).toHaveAttribute('required');
    expect(passwordInput).toHaveAttribute('type', 'password');
    expect(passwordInput).toHaveAttribute('required');
    expect(confirmPasswordInput).toHaveAttribute('type', 'password');
    expect(confirmPasswordInput).toHaveAttribute('required');
    expect(submitButton).toHaveAttribute('type', 'submit');
  });
});