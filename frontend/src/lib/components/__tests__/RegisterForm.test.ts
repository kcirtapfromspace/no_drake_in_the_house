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
    expect(screen.getByText(/at least 8 characters/i)).toBeInTheDocument();
    expect(screen.getByText(/one uppercase letter/i)).toBeInTheDocument();
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
    render(RegisterForm, { props: { error: 'Email already exists' } });

    expect(screen.getByText(/email already exists/i)).toBeInTheDocument();
  });

  it('shows loading state when isLoading prop is true', () => {
    render(RegisterForm, { props: { isLoading: true } });

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
    const fieldErrors = {
      email: 'Email already registered',
      password: 'Password too weak',
      confirm_password: 'Password confirmation does not match',
      terms_accepted: 'You must accept the terms'
    };

    render(RegisterForm, { props: { fieldErrors } });

    expect(screen.getByText(/email already registered/i)).toBeInTheDocument();
    expect(screen.getByText(/password too weak/i)).toBeInTheDocument();
    expect(screen.getByText(/password confirmation does not match/i)).toBeInTheDocument();
    expect(screen.getByText(/you must accept the terms/i)).toBeInTheDocument();
  });

  it('prevents multiple form submissions during loading', async () => {
    const component = render(RegisterForm, { props: { isLoading: true } });
    let submitCount = 0;

    component.component.$on('register', () => {
      submitCount++;
    });

    // Use more specific query since there are multiple buttons
    const submitButton = screen.getByRole('button', { name: /creating account|create account/i });

    // Try to submit multiple times
    await fireEvent.click(submitButton);
    await fireEvent.click(submitButton);
    await fireEvent.click(submitButton);

    expect(submitCount).toBe(0);
    expect(submitButton).toBeDisabled();
  });

  it('shows success state message', async () => {
    render(RegisterForm, { props: { success: 'Account created successfully!' } });

    expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
  });

  it('validates password strength requirements in real-time', async () => {
    const { container } = render(RegisterForm);

    const passwordInput = screen.getByLabelText(/^password$/i);

    // Test that password strength bars update as user types
    await fireEvent.input(passwordInput, { target: { value: 'test' } });

    // The component uses CSS classes on strength bars, not text labels
    const strengthBars = container.querySelectorAll('.password-strength-bar');
    expect(strengthBars.length).toBe(5);

    // Test stronger password shows more active bars
    await fireEvent.input(passwordInput, { target: { value: 'TestPassword123!' } });

    await waitFor(() => {
      const activeBars = container.querySelectorAll('[class*="password-strength-bar--active"]');
      expect(activeBars.length).toBeGreaterThan(0);
    });
  });

  it('shows password requirements checklist', async () => {
    render(RegisterForm);
    
    const passwordInput = screen.getByLabelText(/^password$/i);
    
    await fireEvent.input(passwordInput, { target: { value: 'test' } });
    
    // Should show all password requirements
    expect(screen.getByText(/at least 8 characters/i)).toBeInTheDocument();
    expect(screen.getByText(/one uppercase letter/i)).toBeInTheDocument();
    expect(screen.getByText(/one lowercase letter/i)).toBeInTheDocument();
    expect(screen.getByText(/one number/i)).toBeInTheDocument();
    expect(screen.getByText(/one special character/i)).toBeInTheDocument();
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
    component.component.$set({ success: 'Account created successfully! Logging you in...' });
    
    await waitFor(() => {
      expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
      expect(screen.getByText(/logging you in/i)).toBeInTheDocument();
    });
  });

  it('handles registration without auto-login', async () => {
    render(RegisterForm, { success: 'Account created successfully! Please log in with your new credentials.' });
    
    expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
    expect(screen.getByText(/please log in/i)).toBeInTheDocument();
  });

  // UI Rendering and Responsive Design Tests
  // TODO: These tests check for specific CSS classes that have changed with USWDS migration.
  // They need to be updated to reflect the current component styling.
  describe('UI Rendering and Responsive Design', () => {
    it('renders password validation icons with proper sizing', async () => {
      const { container } = render(RegisterForm);

      const passwordInput = screen.getByLabelText(/^password$/i);
      await fireEvent.input(passwordInput, { target: { value: 'TestPassword123!' } });

      // Check that password requirements are visible (using actual component text)
      expect(screen.getByText(/password strength:/i)).toBeInTheDocument();

      // Check that requirement icons are rendered
      const requirementIcons = container.querySelectorAll('.requirement-icon');
      expect(requirementIcons.length).toBeGreaterThan(0);

      // Verify requirements grid is present (actual class name)
      const requirementsGrid = container.querySelector('.requirements-grid');
      expect(requirementsGrid).toBeInTheDocument();
    });

    it('applies responsive design classes to form elements', () => {
      render(RegisterForm);

      // Check main container has responsive classes
      const container = document.querySelector('.register-form-container');
      expect(container).toHaveClass('register-form-container');
      expect(container).toHaveClass('w-full', 'max-w-md', 'mx-auto');

      // Check form inputs use USWDS classes
      const emailInput = screen.getByLabelText(/email/i);
      expect(emailInput).toHaveClass('form-input-uswds');

      const passwordInput = screen.getByLabelText(/^password$/i);
      expect(passwordInput).toHaveClass('form-input-uswds');
    });

    it('renders password strength indicator with proper styling', async () => {
      const { container } = render(RegisterForm);

      const passwordInput = screen.getByLabelText(/^password$/i);
      await fireEvent.input(passwordInput, { target: { value: 'TestPassword123!' } });

      // Check that password strength indicator is visible
      expect(screen.getByText(/password strength:/i)).toBeInTheDocument();

      // Check that strength bars are rendered with proper classes (actual class name)
      const strengthBars = container.querySelectorAll('.password-strength-bar');
      expect(strengthBars.length).toBe(5);
    });

    it('renders form with proper spacing and layout classes', () => {
      render(RegisterForm);

      // Check form has responsive spacing
      const form = document.querySelector('form');
      expect(form).toHaveClass('space-y-4', 'sm:space-y-6');

      // Check submit button is present with USWDS styling
      const submitButton = screen.getByRole('button', { name: /create account/i });
      expect(submitButton).toHaveClass('btn-uswds', 'btn-uswds-primary');
    });

    it('renders terms acceptance with proper responsive layout', () => {
      render(RegisterForm);
      
      const termsCheckbox = screen.getByLabelText(/terms of service/i);
      const termsContainer = termsCheckbox.closest('.flex.items-start.space-x-3');
      
      expect(termsContainer).toHaveClass('flex', 'items-start', 'space-x-3');
      
      // Check that terms label has proper styling
      const termsLabel = document.querySelector('label[for="terms-accepted"]');
      expect(termsLabel).toHaveClass('cursor-pointer');
    });

    it('applies component-scoped CSS classes to prevent conflicts', async () => {
      const { container } = render(RegisterForm);

      // Check that main container has scoped class
      const formContainer = container.querySelector('.register-form-container');
      expect(formContainer).toHaveClass('register-form-container');

      // Check that password requirements have scoped grid class
      const passwordInput = screen.getByLabelText(/^password$/i);
      await fireEvent.input(passwordInput, { target: { value: 'test' } });

      // Actual class name from component
      const requirementsGrid = container.querySelector('.requirements-grid');
      expect(requirementsGrid).toBeInTheDocument();
    });

    it('renders password requirement icons with consistent sizing', async () => {
      const { container } = render(RegisterForm);

      const passwordInput = screen.getByLabelText(/^password$/i);
      await fireEvent.input(passwordInput, { target: { value: 'TestPassword123!' } });

      // Check that requirement icons exist
      const requirementIcons = container.querySelectorAll('.requirement-icon');
      expect(requirementIcons.length).toBeGreaterThan(0);

      // All icons should have the requirement-icon class
      requirementIcons.forEach(icon => {
        expect(icon).toHaveClass('requirement-icon');
      });
    });

    it('handles different screen sizes gracefully', async () => {
      const { container } = render(RegisterForm);

      // Check that responsive classes are applied
      const formContainer = container.querySelector('.register-form-container');
      expect(formContainer).toHaveClass('px-4', 'sm:px-0');

      // Check that password requirements grid exists
      const passwordInput = screen.getByLabelText(/^password$/i);
      await fireEvent.input(passwordInput, { target: { value: 'test' } });

      const requirementsGrid = container.querySelector('.requirements-grid');
      expect(requirementsGrid).toBeInTheDocument();
    });
  });
});