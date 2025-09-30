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
    expect(screen.getByLabelText(/terms of service/i)).toBeInTheDocument();
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
    const termsCheckbox = screen.getByLabelText(/terms of service/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.click(termsCheckbox);
    await fireEvent.click(submitButton);
    
    expect(dispatchedEvent).toEqual({
      email: 'test@example.com',
      password: 'SecurePassword123!',
      confirm_password: 'SecurePassword123!',
      terms_accepted: true,
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
    const termsCheckbox = screen.getByLabelText(/terms of service/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.click(termsCheckbox);
    
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
    const termsCheckbox = screen.getByLabelText(/terms of service/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    expect(emailInput).toHaveAttribute('type', 'email');
    expect(emailInput).toHaveAttribute('required');
    expect(passwordInput).toHaveAttribute('type', 'password');
    expect(passwordInput).toHaveAttribute('required');
    expect(confirmPasswordInput).toHaveAttribute('type', 'password');
    expect(confirmPasswordInput).toHaveAttribute('required');
    expect(termsCheckbox).toHaveAttribute('type', 'checkbox');
    expect(termsCheckbox).toHaveAttribute('required');
    expect(submitButton).toHaveAttribute('type', 'submit');
  });

  // New tests for enhanced registration functionality

  it('shows validation error when terms are not accepted', async () => {
    render(RegisterForm);
    
    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/^password$/i);
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
    const submitButton = screen.getByRole('button', { name: /create account/i });
    
    await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'SecurePassword123!' } });
    // Don't check terms checkbox
    await fireEvent.click(submitButton);
    
    await waitFor(() => {
      expect(screen.getByText(/you must accept the terms/i)).toBeInTheDocument();
    });
  });

  it('shows real-time validation for email format', async () => {
    render(RegisterForm);
    
    const emailInput = screen.getByLabelText(/email/i);
    
    // Invalid email format
    await fireEvent.input(emailInput, { target: { value: 'invalid-email' } });
    await fireEvent.blur(emailInput);
    
    await waitFor(() => {
      expect(screen.getByText(/please enter a valid email/i)).toBeInTheDocument();
    });
    
    // Valid email format
    await fireEvent.input(emailInput, { target: { value: 'valid@example.com' } });
    await fireEvent.blur(emailInput);
    
    await waitFor(() => {
      expect(screen.queryByText(/please enter a valid email/i)).not.toBeInTheDocument();
    });
  });

  it('shows real-time password confirmation validation', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
    
    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.input(confirmPasswordInput, { target: { value: 'DifferentPassword123!' } });
    await fireEvent.blur(confirmPasswordInput);
    
    await waitFor(() => {
      expect(screen.getByText(/passwords do not match/i)).toBeInTheDocument();
    });
    
    // Fix password confirmation
    await fireEvent.input(confirmPasswordInput, { target: { value: 'SecurePassword123!' } });
    await fireEvent.blur(confirmPasswordInput);
    
    await waitFor(() => {
      expect(screen.queryByText(/passwords do not match/i)).not.toBeInTheDocument();
    });
  });

  it('displays field-specific error messages from server', async () => {
    const fieldErrors = [
      { field: 'email', message: 'Email already registered', code: 'EMAIL_ALREADY_EXISTS' },
      { field: 'password', message: 'Password too weak', code: 'PASSWORD_WEAK' },
      { field: 'confirm_password', message: 'Password confirmation does not match', code: 'PASSWORD_MISMATCH' },
      { field: 'terms_accepted', message: 'You must accept the terms', code: 'TERMS_NOT_ACCEPTED' }
    ];
    
    render(RegisterForm, { fieldErrors });
    
    expect(screen.getByText(/email already registered/i)).toBeInTheDocument();
    expect(screen.getByText(/password too weak/i)).toBeInTheDocument();
    expect(screen.getByText(/password confirmation does not match/i)).toBeInTheDocument();
    expect(screen.getByText(/you must accept the terms/i)).toBeInTheDocument();
  });

  it('prevents multiple form submissions during loading', async () => {
    const component = render(RegisterForm, { isLoading: true });
    let submitCount = 0;
    
    component.component.$on('register', () => {
      submitCount++;
    });
    
    const submitButton = screen.getByRole('button');
    
    // Try to submit multiple times
    await fireEvent.click(submitButton);
    await fireEvent.click(submitButton);
    await fireEvent.click(submitButton);
    
    expect(submitCount).toBe(0);
    expect(submitButton).toBeDisabled();
  });

  it('shows success state and auto-redirect message', async () => {
    render(RegisterForm, { success: true });
    
    expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
    expect(screen.getByText(/redirecting/i)).toBeInTheDocument();
  });

  it('validates password strength requirements in real-time', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    
    // Test various password strengths
    const testCases = [
      { password: '123', expectedStrength: 'weak' },
      { password: 'password', expectedStrength: 'weak' },
      { password: 'Password123', expectedStrength: 'fair' },
      { password: 'SecurePassword123!', expectedStrength: 'strong' }
    ];
    
    for (const testCase of testCases) {
      await fireEvent.input(passwordInput, { target: { value: testCase.password } });
      
      await waitFor(() => {
        expect(screen.getByText(new RegExp(testCase.expectedStrength, 'i'))).toBeInTheDocument();
      });
    }
  });

  it('shows password requirements checklist', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    
    await fireEvent.input(passwordInput, { target: { value: 'test' } });
    
    // Should show all password requirements
    expect(screen.getByText(/at least 8 characters/i)).toBeInTheDocument();
    expect(screen.getByText(/at least one uppercase letter/i)).toBeInTheDocument();
    expect(screen.getByText(/at least one lowercase letter/i)).toBeInTheDocument();
    expect(screen.getByText(/at least one number/i)).toBeInTheDocument();
    expect(screen.getByText(/at least one special character/i)).toBeInTheDocument();
  });

  it('updates password requirements checklist as user types', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    
    // Start with weak password
    await fireEvent.input(passwordInput, { target: { value: 'test' } });
    
    // Add length
    await fireEvent.input(passwordInput, { target: { value: 'testtest' } });
    // Length requirement should be satisfied
    
    // Add uppercase
    await fireEvent.input(passwordInput, { target: { value: 'Testtest' } });
    // Uppercase requirement should be satisfied
    
    // Add number
    await fireEvent.input(passwordInput, { target: { value: 'Testtest1' } });
    // Number requirement should be satisfied
    
    // Add special character
    await fireEvent.input(passwordInput, { target: { value: 'Testtest1!' } });
    // All requirements should be satisfied
  });

  it('handles auto-login success and redirect', async () => {
    const mockNavigate = vi.fn();
    vi.mock('$app/navigation', () => ({
      goto: mockNavigate
    }));
    
    const component = render(RegisterForm);
    
    // Simulate successful registration with auto-login
    component.component.$set({ success: true, autoLogin: true });
    
    await waitFor(() => {
      expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
      expect(screen.getByText(/logging you in/i)).toBeInTheDocument();
    });
    
    // Should redirect after a delay
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard');
    }, { timeout: 3000 });
  });

  it('handles registration without auto-login', async () => {
    render(RegisterForm, { success: true, autoLogin: false });
    
    expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
    expect(screen.getByText(/please log in/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /go to login/i })).toBeInTheDocument();
  });
});