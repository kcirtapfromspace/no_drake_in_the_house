/**
 * Design System Tokens
 *
 * Centralized design tokens for consistent styling across the application.
 * All components should import and use these tokens instead of hardcoding values.
 *
 * Color Hierarchy:
 * - zinc-900 (#18181b): Page backgrounds
 * - zinc-800 (#27272a): Card backgrounds, elevated surfaces
 * - zinc-700 (#3f3f46): Interactive element backgrounds, inputs
 * - zinc-600 (#52525b): Borders, dividers
 * - zinc-500 (#71717a): Disabled states
 * - zinc-400 (#a1a1aa): Secondary text, icons
 * - zinc-300 (#d4d4d8): Primary text on dark
 * - zinc-200 (#e4e4e7): Emphasized text
 * - white (#ffffff): Headings, important text
 */

// ============================================================================
// COLOR TOKENS
// ============================================================================

export const colors = {
  // Background colors (dark to light)
  background: {
    page: '#18181b',        // zinc-900 - Main page background
    elevated: '#27272a',    // zinc-800 - Cards, modals, elevated surfaces
    interactive: '#3f3f46', // zinc-700 - Inputs, buttons, hover states
    hover: '#52525b',       // zinc-600 - Hover on interactive elements
  },

  // Border colors
  border: {
    default: '#52525b',     // zinc-600 - Standard borders
    subtle: '#3f3f46',      // zinc-700 - Subtle dividers
    focus: '#f43f5e',       // rose-500 - Focus rings
    accent: '#a1a1aa',      // zinc-400 - Accent borders
  },

  // Text colors (dark to light on dark bg)
  text: {
    primary: '#ffffff',     // white - Headings, important text
    secondary: '#d4d4d8',   // zinc-300 - Body text
    tertiary: '#a1a1aa',    // zinc-400 - Captions, labels
    muted: '#71717a',       // zinc-500 - Disabled, placeholders
    inverse: '#18181b',     // zinc-900 - Text on light backgrounds
  },

  // Brand colors
  brand: {
    primary: '#f43f5e',     // rose-500 - Primary actions
    primaryHover: '#e11d48', // rose-600 - Primary hover
    primaryMuted: 'rgba(244, 63, 94, 0.15)', // rose-500/15 - Backgrounds
  },

  // Semantic colors
  semantic: {
    success: '#10B981',     // emerald-500
    successMuted: 'rgba(16, 185, 129, 0.15)',
    warning: '#F59E0B',     // amber-500
    warningMuted: 'rgba(245, 158, 11, 0.15)',
    error: '#EF4444',       // red-500
    errorMuted: 'rgba(239, 68, 68, 0.15)',
    info: '#3B82F6',        // blue-500
    infoMuted: 'rgba(59, 130, 246, 0.15)',
  },

  // Category colors for offense types
  category: {
    domestic_violence: { icon: '#F43F5E', bg: 'rgba(244, 63, 94, 0.15)' },
    sexual_misconduct: { icon: '#EC4899', bg: 'rgba(236, 72, 153, 0.15)' },
    sexual_assault: { icon: '#DB2777', bg: 'rgba(219, 39, 119, 0.15)' },
    child_abuse: { icon: '#BE185D', bg: 'rgba(190, 24, 93, 0.15)' },
    certified_creeper: { icon: '#8B5CF6', bg: 'rgba(139, 92, 246, 0.15)' },
    hate_speech: { icon: '#DC2626', bg: 'rgba(220, 38, 38, 0.15)' },
    racism: { icon: '#B91C1C', bg: 'rgba(185, 28, 28, 0.15)' },
    homophobia: { icon: '#7C3AED', bg: 'rgba(124, 58, 237, 0.15)' },
    antisemitism: { icon: '#6D28D9', bg: 'rgba(109, 40, 217, 0.15)' },
    violent_crime: { icon: '#EF4444', bg: 'rgba(239, 68, 68, 0.15)' },
    drug_trafficking: { icon: '#059669', bg: 'rgba(5, 150, 105, 0.15)' },
    fraud: { icon: '#0891B2', bg: 'rgba(8, 145, 178, 0.15)' },
    animal_abuse: { icon: '#F97316', bg: 'rgba(249, 115, 22, 0.15)' },
    other: { icon: '#6B7280', bg: 'rgba(107, 114, 128, 0.15)' },
  },

  // Platform colors
  platform: {
    spotify: '#1DB954',
    apple: '#FC3C44',
    youtube: '#FF0000',
    tidal: '#000000',
  },
} as const;

// ============================================================================
// SPACING TOKENS
// ============================================================================

export const spacing = {
  0: '0',
  0.5: '0.125rem',  // 2px
  1: '0.25rem',     // 4px
  1.5: '0.375rem',  // 6px
  2: '0.5rem',      // 8px
  2.5: '0.625rem',  // 10px
  3: '0.75rem',     // 12px
  3.5: '0.875rem',  // 14px
  4: '1rem',        // 16px
  5: '1.25rem',     // 20px
  6: '1.5rem',      // 24px
  7: '1.75rem',     // 28px
  8: '2rem',        // 32px
  9: '2.25rem',     // 36px
  10: '2.5rem',     // 40px
  12: '3rem',       // 48px
  14: '3.5rem',     // 56px
  16: '4rem',       // 64px
} as const;

// ============================================================================
// TYPOGRAPHY TOKENS
// ============================================================================

export const typography = {
  fontFamily: {
    sans: "'Source Sans Pro', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen-Sans, Ubuntu, Cantarell, 'Helvetica Neue', sans-serif",
    mono: "ui-monospace, SFMono-Regular, 'SF Mono', Consolas, 'Liberation Mono', Menlo, monospace",
  },

  fontSize: {
    xs: '0.75rem',    // 12px
    sm: '0.875rem',   // 14px
    base: '1rem',     // 16px
    lg: '1.125rem',   // 18px
    xl: '1.25rem',    // 20px
    '2xl': '1.5rem',  // 24px
    '3xl': '1.875rem', // 30px
    '4xl': '2.25rem', // 36px
    '5xl': '3rem',    // 48px
  },

  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },

  lineHeight: {
    tight: '1.25',
    snug: '1.375',
    normal: '1.5',
    relaxed: '1.625',
    loose: '2',
  },
} as const;

// ============================================================================
// BORDER TOKENS
// ============================================================================

export const borders = {
  width: {
    none: '0',
    thin: '1px',
    default: '2px',
    thick: '3px',
  },

  /**
   * Border Radius Standards - ENFORCED ACROSS ALL COMPONENTS
   *
   * IMPORTANT: Sharp corners (no border-radius) are NOT allowed on:
   * - Cards, panels, containers
   * - Buttons
   * - Inputs, textareas, selects
   * - Modals and dialogs
   * - Badges and tags
   * - Images and avatars
   *
   * Use these values consistently:
   * - Cards/Panels: rounded-xl (1rem/16px) or rounded-lg (0.75rem/12px)
   * - Buttons: rounded-lg (12px) or rounded-full for pill buttons
   * - Inputs: rounded-lg (12px)
   * - Small elements (badges): rounded-md (8px) or rounded-full
   * - Avatars: rounded-full or rounded-lg
   */
  radius: {
    none: '0',
    sm: '0.25rem',    // 4px - minimal, rarely used
    default: '0.375rem', // 6px - legacy, avoid
    md: '0.5rem',     // 8px - badges, small elements
    lg: '0.75rem',    // 12px - buttons, inputs, small cards
    xl: '1rem',       // 16px - cards, panels, containers (PREFERRED)
    '2xl': '1.5rem',  // 24px - large modals, hero sections
    full: '9999px',   // pill buttons, avatars
  },
} as const;

/**
 * COMPONENT BORDER RADIUS REQUIREMENTS
 *
 * These are the REQUIRED border radius values for different component types.
 * UX tests will verify compliance.
 */
export const borderRadiusRequirements = {
  // Primary containers - must be visibly rounded
  card: 'rounded-xl',           // 16px - main content cards
  panel: 'rounded-xl',          // 16px - side panels, info boxes
  modal: 'rounded-2xl',         // 24px - modal dialogs

  // Interactive elements - softer corners
  button: 'rounded-lg',         // 12px - standard buttons
  buttonPill: 'rounded-full',   // pill-shaped buttons
  input: 'rounded-lg',          // 12px - all form inputs
  select: 'rounded-lg',         // 12px - dropdowns

  // Small elements
  badge: 'rounded-md',          // 8px - status badges
  tag: 'rounded-full',          // pill tags
  chip: 'rounded-full',         // filter chips

  // Media
  avatar: 'rounded-full',       // circular avatars (or rounded-lg for square)
  image: 'rounded-lg',          // image containers
  thumbnail: 'rounded-lg',      // small images
} as const;

// ============================================================================
// SHADOW TOKENS
// ============================================================================

export const shadows = {
  none: 'none',
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  default: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
} as const;

// ============================================================================
// TRANSITION TOKENS
// ============================================================================

export const transitions = {
  duration: {
    fast: '150ms',
    default: '200ms',
    slow: '300ms',
  },

  timing: {
    default: 'cubic-bezier(0.4, 0, 0.2, 1)',
    in: 'cubic-bezier(0.4, 0, 1, 1)',
    out: 'cubic-bezier(0, 0, 0.2, 1)',
  },
} as const;

// ============================================================================
// BREAKPOINTS
// ============================================================================

export const breakpoints = {
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px',
} as const;

// ============================================================================
// Z-INDEX SCALE
// ============================================================================

export const zIndex = {
  base: 0,
  dropdown: 10,
  sticky: 20,
  fixed: 30,
  modalBackdrop: 40,
  modal: 50,
  popover: 60,
  tooltip: 70,
} as const;

// ============================================================================
// COMPONENT STYLES (Inline style helpers)
// ============================================================================

/**
 * Pre-composed style strings for common component patterns.
 * Use these for inline styles to ensure consistency.
 */
export const componentStyles = {
  // Card styles
  card: {
    default: `background: ${colors.background.elevated}; border: 2px solid ${colors.border.default};`,
    hover: `background: ${colors.background.interactive}; border: 2px solid ${colors.border.default};`,
    active: `background: ${colors.background.interactive}; border: 2px solid ${colors.brand.primary};`,
  },

  // Input styles
  input: {
    default: `background: ${colors.background.interactive}; border: 1px solid ${colors.border.default}; color: ${colors.text.primary};`,
    focus: `background: ${colors.background.interactive}; border: 1px solid ${colors.brand.primary}; color: ${colors.text.primary};`,
  },

  // Button styles
  button: {
    primary: `background: ${colors.brand.primary}; color: white;`,
    secondary: `background: ${colors.background.interactive}; color: white; border: 1px solid ${colors.border.default};`,
    ghost: `background: transparent; color: ${colors.text.secondary}; border: 1px solid transparent;`,
    danger: `background: ${colors.semantic.error}; color: white;`,
  },

  // Modal styles
  modal: {
    backdrop: `background: rgba(0, 0, 0, 0.85);`,
    content: `background: ${colors.background.elevated}; border: 2px solid ${colors.border.default};`,
  },
} as const;

// ============================================================================
// TAILWIND CLASS MAPPINGS
// ============================================================================

/**
 * Mapping of design token concepts to Tailwind classes.
 * Use these when working with Tailwind utilities.
 */
export const tailwindClasses = {
  // Background classes
  bg: {
    page: 'bg-zinc-900',
    elevated: 'bg-zinc-800',
    interactive: 'bg-zinc-700',
    hover: 'bg-zinc-600',
  },

  // Text classes
  text: {
    primary: 'text-white',
    secondary: 'text-zinc-300',
    tertiary: 'text-zinc-400',
    muted: 'text-zinc-500',
  },

  // Border classes
  border: {
    default: 'border-zinc-600',
    subtle: 'border-zinc-700',
    focus: 'focus:border-rose-500',
  },
} as const;

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type ColorToken = typeof colors;
export type SpacingToken = typeof spacing;
export type TypographyToken = typeof typography;
export type BorderToken = typeof borders;
export type ShadowToken = typeof shadows;
export type TransitionToken = typeof transitions;
export type BreakpointToken = typeof breakpoints;
export type ZIndexToken = typeof zIndex;
