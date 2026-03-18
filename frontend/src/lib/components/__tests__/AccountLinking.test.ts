import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import AccountLinking from '../AccountLinking.svelte';

const mocks = vi.hoisted(() => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
  },
  currentUser: {
    subscribe: vi.fn(),
  },
}));

vi.mock('$lib/utils/api', () => ({
  api: mocks.api,
}));

vi.mock('$lib/stores/auth', () => ({
  currentUser: mocks.currentUser,
}));

const mockPopup = {
  closed: false,
  close: vi.fn(),
};

const mockConfirm = vi.fn();
const mockSetInterval = vi.fn(() => 1);
const mockClearInterval = vi.fn();

vi.stubGlobal('confirm', mockConfirm);
vi.stubGlobal('setInterval', mockSetInterval);
vi.stubGlobal('clearInterval', mockClearInterval);

describe('AccountLinking', () => {
  const linkedAccounts = [
    {
      provider: 'google',
      provider_user_id: 'google123',
      email: 'test@gmail.com',
      display_name: 'Test User',
      avatar_url: 'https://example.com/avatar.jpg',
      linked_at: '2023-01-01T00:00:00Z',
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    mockPopup.closed = false;
    window.sessionStorage.clear();
    vi.spyOn(window, 'open').mockImplementation(() => mockPopup as unknown as Window);

    mocks.currentUser.subscribe.mockImplementation((callback: (value: unknown) => void) => {
      callback({ id: '1', email: 'test@example.com' });
      return () => {};
    });

    mocks.api.get.mockResolvedValue({ success: true, data: [] });
    mocks.api.post.mockResolvedValue({ success: true, data: { authorization_url: 'https://accounts.google.com/oauth/authorize', state: 'test-state-token' } });
    mocks.api.delete.mockResolvedValue({ success: true });
    mockConfirm.mockReturnValue(true);
  });

  it('does not render when hidden', () => {
    render(AccountLinking, { isVisible: false });

    expect(screen.queryByText(/linked accounts/i)).not.toBeInTheDocument();
  });

  it('renders the modal and loads linked accounts when visible', async () => {
    mocks.api.get.mockResolvedValue({ success: true, data: linkedAccounts });
    const linkedDate = new Date(linkedAccounts[0].linked_at).toLocaleDateString();

    render(AccountLinking, { isVisible: true });

    expect(screen.getByText(/linked accounts/i)).toBeInTheDocument();
    expect(screen.getByText(/link your social accounts/i)).toBeInTheDocument();

    await waitFor(() => {
      expect(mocks.api.get).toHaveBeenCalledWith('/auth/oauth/accounts');
      expect(screen.getByText(/test user/i)).toBeInTheDocument();
      expect(screen.getByText(`Linked on ${linkedDate}`)).toBeInTheDocument();
    });
  });

  it('shows all providers as linkable when no accounts are connected', async () => {
    render(AccountLinking, { isVisible: true });

    await waitFor(() => {
      expect(screen.getByText(/^google$/i)).toBeInTheDocument();
      expect(screen.getByText(/^apple$/i)).toBeInTheDocument();
      expect(screen.getByText(/^github$/i)).toBeInTheDocument();
    });

    expect(screen.getAllByRole('button', { name: /^link$/i })).toHaveLength(3);
  });

  it('initiates an account link flow and shows the linking state', async () => {
    render(AccountLinking, { isVisible: true });

    await waitFor(() => {
      expect(screen.getByText(/^google$/i)).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole('button', { name: /^link$/i })[0]);

    await waitFor(() => {
      expect(mocks.api.post).toHaveBeenCalledWith('/auth/oauth/google/link');
      expect(screen.getByRole('button', { name: /linking\.\.\./i })).toBeDisabled();
    });
  });

  it('renders linking errors from the API response', async () => {
    mocks.api.post.mockResolvedValue({
      success: false,
      message: 'OAuth provider not configured',
    });

    render(AccountLinking, { isVisible: true });

    await waitFor(() => {
      expect(screen.getByText(/^google$/i)).toBeInTheDocument();
    });

    await fireEvent.click(screen.getAllByRole('button', { name: /^link$/i })[0]);

    await waitFor(() => {
      expect(screen.getByText(/oauth provider not configured/i)).toBeInTheDocument();
    });
  });

  it('unlinks an account after confirmation and reloads accounts', async () => {
    mocks.api.get.mockResolvedValue({ success: true, data: linkedAccounts });

    render(AccountLinking, { isVisible: true });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /unlink/i })).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByRole('button', { name: /unlink/i }));

    await waitFor(() => {
      expect(mockConfirm).toHaveBeenCalledWith('Are you sure you want to unlink your Google account?');
      expect(mocks.api.delete).toHaveBeenCalledWith('/auth/oauth/google/unlink');
    });
  });

  it('does not unlink when confirmation is declined', async () => {
    mocks.api.get.mockResolvedValue({ success: true, data: linkedAccounts });
    mockConfirm.mockReturnValue(false);

    render(AccountLinking, { isVisible: true });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /unlink/i })).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByRole('button', { name: /unlink/i }));

    expect(mocks.api.delete).not.toHaveBeenCalled();
  });

  it('renders load errors when fetching linked accounts fails', async () => {
    mocks.api.get.mockRejectedValue(new Error('Network error'));

    render(AccountLinking, { isVisible: true });

    await waitFor(() => {
      expect(screen.getByText(/network error/i)).toBeInTheDocument();
    });
  });

  it('emits a close event from the visible close button', async () => {
    const { component } = render(AccountLinking, { isVisible: true });
    let closed = false;

    component.$on('close', () => {
      closed = true;
    });

    await fireEvent.click(screen.getByRole('button', { name: /^close$/i }));

    expect(closed).toBe(true);
  });
});
