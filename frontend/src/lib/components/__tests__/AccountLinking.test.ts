import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import AccountLinking from '../AccountLinking.svelte';

// Mock the API module
const mockApi = {
  get: vi.fn(),
  post: vi.fn(),
  delete: vi.fn(),
};

vi.mock('$lib/utils/api', () => ({
  api: mockApi,
}));

// Mock the auth store
const mockCurrentUser = {
  subscribe: vi.fn(),
};

vi.mock('$lib/stores/auth', () => ({
  currentUser: mockCurrentUser,
}));

// Mock window.open and related functionality
const mockPopup = {
  closed: false,
  close: vi.fn(),
};

const mockWindow = {
  open: vi.fn(() => mockPopup),
};

Object.defineProperty(window, 'open', {
  value: mockWindow.open,
});

// Mock sessionStorage
const mockSessionStorage = {
  setItem: vi.fn(),
  getItem: vi.fn(),
  removeItem: vi.fn(),
};

Object.defineProperty(window, 'sessionStorage', {
  value: mockSessionStorage,
});

// Mock setInterval and clearInterval
vi.stubGlobal('setInterval', (fn: Function, delay: number) => {
  // Simulate popup closing after a short delay
  setTimeout(() => {
    mockPopup.closed = true;
    fn();
  }, 100);
  return 1;
});

vi.stubGlobal('clearInterval', vi.fn());

describe('AccountLinking', () => {
  const mockLinkedAccounts = [
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
    
    // Mock current user
    mockCurrentUser.subscribe.mockImplementation((callback) => {
      callback({ id: '1', email: 'test@example.com' });
      return () => {};
    });
  });

  it('does not render when not visible', () => {
    render(AccountLinking, { isVisible: false });
    
    expect(screen.queryByText(/linked accounts/i)).not.toBeInTheDocument();
  });

  it('renders modal when visible', () => {
    render(AccountLinking, { isVisible: true });
    
    expect(screen.getByText(/linked accounts/i)).toBeInTheDocument();
    expect(screen.getByText(/link your social accounts/i)).toBeInTheDocument();
  });

  it('loads linked accounts on mount when visible', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: mockLinkedAccounts,
    });
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(mockApi.get).toHaveBeenCalledWith('/auth/oauth/accounts');
    });
  });

  it('displays linked accounts correctly', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: mockLinkedAccounts,
    });
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/google/i)).toBeInTheDocument();
      expect(screen.getByText(/test user/i)).toBeInTheDocument();
      expect(screen.getByText(/linked on 1\/1\/2023/i)).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /unlink/i })).toBeInTheDocument();
    });
  });

  it('displays unlinked providers with link buttons', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: [], // No linked accounts
    });
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/google/i)).toBeInTheDocument();
      expect(screen.getByText(/apple/i)).toBeInTheDocument();
      expect(screen.getByText(/github/i)).toBeInTheDocument();
      
      const linkButtons = screen.getAllByRole('button', { name: /link/i });
      expect(linkButtons).toHaveLength(3);
    });
  });

  it('initiates account linking flow', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: [],
    });
    
    mockApi.post.mockResolvedValueOnce({
      success: true,
      data: {
        authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test',
        state: 'test-state-token',
      },
    });
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/google/i)).toBeInTheDocument();
    });
    
    const linkButton = screen.getByRole('button', { name: /link/i });
    await fireEvent.click(linkButton);
    
    await waitFor(() => {
      expect(mockApi.post).toHaveBeenCalledWith('/auth/oauth/google/link');
      expect(mockSessionStorage.setItem).toHaveBeenCalledWith('oauth_link_state_google', 'test-state-token');
      expect(mockWindow.open).toHaveBeenCalledWith(
        'https://accounts.google.com/oauth/authorize?client_id=test',
        'link_google',
        'width=500,height=600,scrollbars=yes,resizable=yes'
      );
    });
  });

  it('handles linking errors', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: [],
    });
    
    mockApi.post.mockResolvedValueOnce({
      success: false,
      message: 'OAuth provider not configured',
    });
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/google/i)).toBeInTheDocument();
    });
    
    const linkButton = screen.getByRole('button', { name: /link/i });
    await fireEvent.click(linkButton);
    
    await waitFor(() => {
      expect(screen.getByText(/oauth provider not configured/i)).toBeInTheDocument();
    });
  });

  it('unlinks account with confirmation', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: mockLinkedAccounts,
    });
    
    mockApi.delete.mockResolvedValueOnce({
      success: true,
    });
    
    // Mock window.confirm
    vi.stubGlobal('confirm', vi.fn(() => true));
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /unlink/i })).toBeInTheDocument();
    });
    
    const unlinkButton = screen.getByRole('button', { name: /unlink/i });
    await fireEvent.click(unlinkButton);
    
    await waitFor(() => {
      expect(window.confirm).toHaveBeenCalledWith('Are you sure you want to unlink your Google account?');
      expect(mockApi.delete).toHaveBeenCalledWith('/auth/oauth/google/unlink');
    });
  });

  it('cancels unlinking when user declines confirmation', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: mockLinkedAccounts,
    });
    
    // Mock window.confirm to return false
    vi.stubGlobal('confirm', vi.fn(() => false));
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /unlink/i })).toBeInTheDocument();
    });
    
    const unlinkButton = screen.getByRole('button', { name: /unlink/i });
    await fireEvent.click(unlinkButton);
    
    expect(window.confirm).toHaveBeenCalled();
    expect(mockApi.delete).not.toHaveBeenCalled();
  });

  it('handles unlinking errors', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: mockLinkedAccounts,
    });
    
    mockApi.delete.mockResolvedValueOnce({
      success: false,
      message: 'Failed to unlink account',
    });
    
    vi.stubGlobal('confirm', vi.fn(() => true));
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /unlink/i })).toBeInTheDocument();
    });
    
    const unlinkButton = screen.getByRole('button', { name: /unlink/i });
    await fireEvent.click(unlinkButton);
    
    await waitFor(() => {
      expect(screen.getByText(/failed to unlink account/i)).toBeInTheDocument();
    });
  });

  it('closes modal when close button is clicked', async () => {
    const component = render(AccountLinking, { isVisible: true });
    let closeEvent = false;
    
    component.component.$on('close', () => {
      closeEvent = true;
    });
    
    const closeButton = screen.getByRole('button', { name: /close/i });
    await fireEvent.click(closeButton);
    
    expect(closeEvent).toBe(true);
  });

  it('closes modal when backdrop is clicked', async () => {
    const component = render(AccountLinking, { isVisible: true });
    let closeEvent = false;
    
    component.component.$on('close', () => {
      closeEvent = true;
    });
    
    const backdrop = screen.getByRole('dialog').parentElement;
    await fireEvent.click(backdrop);
    
    expect(closeEvent).toBe(true);
  });

  it('displays loading state', () => {
    mockApi.get.mockImplementation(() => 
      new Promise(resolve => setTimeout(() => resolve({ success: true, data: [] }), 1000))
    );
    
    render(AccountLinking, { isVisible: true });
    
    expect(screen.getByRole('img', { hidden: true })).toBeInTheDocument(); // Loading spinner
  });

  it('displays error when loading accounts fails', async () => {
    mockApi.get.mockRejectedValueOnce(new Error('Network error'));
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/network error/i)).toBeInTheDocument();
    });
  });

  it('shows linking state for specific provider', async () => {
    mockApi.get.mockResolvedValueOnce({
      success: true,
      data: [],
    });
    
    mockApi.post.mockImplementation(() => 
      new Promise(resolve => setTimeout(() => resolve({
        success: true,
        data: {
          authorization_url: 'https://accounts.google.com/oauth/authorize',
          state: 'test-state',
        },
      }), 100))
    );
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/google/i)).toBeInTheDocument();
    });
    
    const linkButton = screen.getByRole('button', { name: /link/i });
    await fireEvent.click(linkButton);
    
    expect(screen.getByText(/linking.../i)).toBeInTheDocument();
  });

  it('reloads accounts after popup closes', async () => {
    mockApi.get
      .mockResolvedValueOnce({ success: true, data: [] })
      .mockResolvedValueOnce({ success: true, data: mockLinkedAccounts });
    
    mockApi.post.mockResolvedValueOnce({
      success: true,
      data: {
        authorization_url: 'https://accounts.google.com/oauth/authorize',
        state: 'test-state',
      },
    });
    
    render(AccountLinking, { isVisible: true });
    
    await waitFor(() => {
      expect(screen.getByText(/google/i)).toBeInTheDocument();
    });
    
    const linkButton = screen.getByRole('button', { name: /link/i });
    await fireEvent.click(linkButton);
    
    // Wait for popup to "close" and accounts to reload
    await waitFor(() => {
      expect(mockApi.get).toHaveBeenCalledTimes(2);
    }, { timeout: 2000 });
  });
});