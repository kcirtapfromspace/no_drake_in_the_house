import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import RegisterForm from '../RegisterForm.svelte';

describe('RegisterForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the required registration fields', () => {
    render(RegisterForm);

    expect(screen.getByLabelText(/email address/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/^password$/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/terms of service/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /create account/i })).toBeInTheDocument();
  });

  it('keeps submit disabled until the form is valid', async () => {
    render(RegisterForm);

    const submitButton = screen.getByRole('button', { name: /create account/i });
    expect(submitButton).toBeDisabled();

    await fireEvent.input(screen.getByLabelText(/email address/i), {
      target: { value: 'test@example.com' },
    });
    await fireEvent.input(screen.getByLabelText(/^password$/i), {
      target: { value: 'SecurePassword123!' },
    });
    await fireEvent.input(screen.getByLabelText(/confirm password/i), {
      target: { value: 'SecurePassword123!' },
    });
    await fireEvent.click(screen.getByLabelText(/terms of service/i));

    expect(submitButton).not.toBeDisabled();
  });

  it('shows email validation after blur', async () => {
    render(RegisterForm);

    const emailInput = screen.getByLabelText(/email address/i);
    await fireEvent.input(emailInput, { target: { value: 'invalid-email' } });
    await fireEvent.blur(emailInput);

    await waitFor(() => {
      expect(screen.getByText(/please enter a valid email address/i)).toBeInTheDocument();
    });
  });

  it('shows password requirements and current strength as the user types', async () => {
    render(RegisterForm);

    const passwordInput = screen.getByLabelText(/^password$/i);

    await fireEvent.input(passwordInput, { target: { value: '123' } });
    expect(screen.getByText(/password strength:/i)).toBeInTheDocument();
    expect(screen.getByText(/weak/i)).toBeInTheDocument();
    expect(screen.getByText(/at least 8 characters/i)).toBeInTheDocument();

    await fireEvent.input(passwordInput, { target: { value: 'Password123' } });
    expect(screen.getByText(/good/i)).toBeInTheDocument();

    await fireEvent.input(passwordInput, { target: { value: 'SecurePassword123!' } });
    expect(screen.getByText(/strong/i)).toBeInTheDocument();
    expect(document.querySelectorAll('.password-strength-bar')).toHaveLength(5);
    expect(document.querySelector('.requirements-grid')).toBeInTheDocument();
  });

  it('shows a validation error when passwords do not match', async () => {
    render(RegisterForm);

    await fireEvent.input(screen.getByLabelText(/^password$/i), {
      target: { value: 'SecurePassword123!' },
    });
    await fireEvent.input(screen.getByLabelText(/confirm password/i), {
      target: { value: 'DifferentPassword123!' },
    });
    await fireEvent.blur(screen.getByLabelText(/confirm password/i));

    await waitFor(() => {
      expect(screen.getByText(/passwords do not match/i)).toBeInTheDocument();
    });
  });

  it('dispatches the register event with the current form payload', async () => {
    const { component } = render(RegisterForm);
    let submitted: Record<string, unknown> | null = null;

    component.$on('register', (event) => {
      submitted = event.detail;
    });

    await fireEvent.input(screen.getByLabelText(/email address/i), {
      target: { value: 'test@example.com' },
    });
    await fireEvent.input(screen.getByLabelText(/^password$/i), {
      target: { value: 'SecurePassword123!' },
    });
    await fireEvent.input(screen.getByLabelText(/confirm password/i), {
      target: { value: 'SecurePassword123!' },
    });
    await fireEvent.click(screen.getByLabelText(/terms of service/i));
    await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

    expect(submitted).toEqual({
      email: 'test@example.com',
      password: 'SecurePassword123!',
      confirm_password: 'SecurePassword123!',
      terms_accepted: true,
    });
  });

  it('renders field-level server errors from the fieldErrors prop', () => {
    render(RegisterForm, {
      fieldErrors: {
        email: 'Email already registered',
        password: 'Password too weak',
        confirm_password: 'Password confirmation does not match',
        terms_accepted: 'You must accept the terms',
      },
    });

    expect(screen.getByText(/email already registered/i)).toBeInTheDocument();
    expect(screen.getByText(/password too weak/i)).toBeInTheDocument();
    expect(screen.getByText(/password confirmation does not match/i)).toBeInTheDocument();
    expect(screen.getByText(/you must accept the terms/i)).toBeInTheDocument();
  });

  it('shows the loading state and prevents submission while loading', async () => {
    const { component } = render(RegisterForm, { isLoading: true });
    let submitCount = 0;

    component.$on('register', () => {
      submitCount += 1;
    });

    const submitButton = screen.getByRole('button', { name: /creating account/i });
    expect(submitButton).toBeDisabled();

    await fireEvent.click(submitButton);
    expect(submitCount).toBe(0);
  });

  it('renders success and error alerts from props', () => {
    render(RegisterForm, {
      success: 'Account created successfully! Logging you in...',
    });

    expect(screen.getByText(/account created successfully/i)).toBeInTheDocument();
    expect(screen.getByText(/logging you in/i)).toBeInTheDocument();
  });

  it('renders error alerts from props', () => {
    render(RegisterForm, {
      error: 'Email already exists',
    });

    expect(screen.getByText(/email already exists/i)).toBeInTheDocument();
  });

  it('renders the current design-system classes', () => {
    render(RegisterForm);

    const container = document.querySelector('.register-form-container');
    expect(container).toHaveClass('w-full', 'max-w-md', 'mx-auto', 'px-4', 'sm:px-0');

    expect(screen.getByLabelText(/email address/i)).toHaveClass('form-input-uswds');
    expect(screen.getByRole('button', { name: /create account/i })).toHaveClass(
      'btn-uswds',
      'btn-uswds-primary',
      'btn--full'
    );
  });

  it('renders password requirement icons with the current markup', async () => {
    render(RegisterForm);

    await fireEvent.input(screen.getByLabelText(/^password$/i), {
      target: { value: 'TestPassword123!' },
    });

    const requirementIcons = document.querySelectorAll('.requirement-icon');
    expect(requirementIcons.length).toBeGreaterThan(0);
    requirementIcons.forEach((icon) => {
      expect(icon).toHaveClass('requirement-icon');
    });
  });

  it('renders the switch-to-login action', () => {
    render(RegisterForm);

    expect(
      screen.getByRole('button', { name: /already have an account\? sign in/i })
    ).toBeInTheDocument();
  });
});
