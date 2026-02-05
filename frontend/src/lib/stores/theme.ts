import { writable, derived } from 'svelte/store';

export type Theme = 'light' | 'dark' | 'system';

function getInitialTheme(): Theme {
  if (typeof window === 'undefined') return 'system';
  const stored = localStorage.getItem('theme-preference');
  if (stored === 'light' || stored === 'dark' || stored === 'system') return stored;
  return 'system';
}

function getResolvedTheme(theme: Theme): 'light' | 'dark' {
  if (theme === 'system') {
    if (typeof window !== 'undefined') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'dark';
  }
  return theme;
}

function applyTheme(theme: Theme) {
  if (typeof window === 'undefined') return;
  const resolved = getResolvedTheme(theme);
  document.documentElement.setAttribute('data-theme', resolved);
}

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>(getInitialTheme());

  return {
    subscribe,

    setTheme(newTheme: Theme) {
      set(newTheme);
      if (typeof window !== 'undefined') {
        localStorage.setItem('theme-preference', newTheme);
        applyTheme(newTheme);
      }
    },

    cycle() {
      update(current => {
        // Cycle: system -> light -> dark -> system
        const order: Theme[] = ['system', 'light', 'dark'];
        const idx = order.indexOf(current);
        const next = order[(idx + 1) % order.length];
        if (typeof window !== 'undefined') {
          localStorage.setItem('theme-preference', next);
          applyTheme(next);
        }
        return next;
      });
    },

    toggle() {
      update(current => {
        const resolved = getResolvedTheme(current);
        const newTheme: Theme = resolved === 'light' ? 'dark' : 'light';
        if (typeof window !== 'undefined') {
          localStorage.setItem('theme-preference', newTheme);
          applyTheme(newTheme);
        }
        return newTheme;
      });
    },

    init() {
      if (typeof window !== 'undefined') {
        applyTheme(getInitialTheme());

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

export const theme = createThemeStore();
export const resolvedTheme = derived(theme, ($theme) => getResolvedTheme($theme));
export const isDarkMode = derived(resolvedTheme, ($resolved) => $resolved === 'dark');
