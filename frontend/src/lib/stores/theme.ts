import { writable, derived } from 'svelte/store';

export type Theme = 'light' | 'dark' | 'system';

// Get initial theme from localStorage or default to system
function getInitialTheme(): Theme {
  if (typeof window === 'undefined') return 'system';

  const stored = localStorage.getItem('theme');
  if (stored === 'light' || stored === 'dark' || stored === 'system') {
    return stored;
  }
  return 'system';
}

// Get the actual resolved theme (light or dark)
function getResolvedTheme(theme: Theme): 'light' | 'dark' {
  if (theme === 'system') {
    if (typeof window !== 'undefined') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'light';
  }
  return theme;
}

// Create the theme store
function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>(getInitialTheme());

  return {
    subscribe,

    setTheme(newTheme: Theme) {
      set(newTheme);
      if (typeof window !== 'undefined') {
        localStorage.setItem('theme', newTheme);
        applyTheme(newTheme);
      }
    },

    toggle() {
      update(current => {
        const resolved = getResolvedTheme(current);
        const newTheme: Theme = resolved === 'light' ? 'dark' : 'light';
        if (typeof window !== 'undefined') {
          localStorage.setItem('theme', newTheme);
          applyTheme(newTheme);
        }
        return newTheme;
      });
    },

    init() {
      if (typeof window !== 'undefined') {
        const theme = getInitialTheme();
        applyTheme(theme);

        // Listen for system theme changes
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
          update(current => {
            if (current === 'system') {
              applyTheme('system');
            }
            return current;
          });
        });
      }
    }
  };
}

// Apply theme to document
function applyTheme(theme: Theme) {
  if (typeof window === 'undefined') return;

  const resolved = getResolvedTheme(theme);
  const root = document.documentElement;

  if (resolved === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

export const theme = createThemeStore();

// Derived store for the resolved theme
export const resolvedTheme = derived(theme, ($theme) => getResolvedTheme($theme));

// Derived store for checking if dark mode is active
export const isDarkMode = derived(resolvedTheme, ($resolved) => $resolved === 'dark');
