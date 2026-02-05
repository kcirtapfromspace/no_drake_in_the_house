# PRD: Frontend Visual Redesign

## Introduction

The "No Drake in the House" frontend currently suffers from a dated, cluttered design that undermines usability despite having a solid feature set. The UI mixes inconsistent spacing, poor information hierarchy, and a dark-only theme that feels oppressive rather than premium. This redesign will modernize the visual layer of the key pages — Home, Blocklist, Search, Layout/Navigation, and Settings — delivering a clean, Spotify-inspired aesthetic with proper light/dark theming, while preserving all existing functionality.

## Goals

- Deliver a modern, polished UI that looks like a shipping product, not a prototype
- Establish clear visual hierarchy so users instantly know where to look and what to do
- Support light and dark themes with system-preference auto-detection and manual toggle
- Improve mobile usability with touch-friendly targets and responsive layouts
- Keep all existing features intact — this is a visual overhaul, not a feature cut

## User Stories

### US-001: Redesign design tokens and CSS foundation
**Description:** As a developer, I need updated design tokens (colors, spacing, typography, shadows, radii) so all components inherit a modern, cohesive look.

**Acceptance Criteria:**
- [ ] Update `unified-design-system.css` with new color palette supporting both light and dark themes via CSS custom properties
- [ ] Light theme: clean whites/grays with rose accent. Dark theme: refined zinc/slate palette (not pure black)
- [ ] Add `[data-theme="light"]` and `[data-theme="dark"]` selectors on `:root`/`html`
- [ ] Typography uses Inter or system font stack (not Source Sans Pro)
- [ ] Spacing scale remains 4px-based but add comfortable padding defaults for cards/sections
- [ ] Shadows are subtle in light mode, near-invisible in dark mode
- [ ] Border radii standardized: 8px default, 12px for cards, 16px for modals
- [ ] Typecheck/lint passes

### US-002: Implement theme toggle with system preference detection
**Description:** As a user, I want the app to match my system theme automatically and let me override it, so the interface is comfortable in any lighting.

**Acceptance Criteria:**
- [ ] On first visit, detect `prefers-color-scheme` and apply matching theme
- [ ] Theme toggle button in Layout header (sun/moon icon)
- [ ] Toggle cycles: System → Light → Dark → System
- [ ] Preference persisted in localStorage under key `theme-preference`
- [ ] Theme applies instantly without flash of wrong theme on page load
- [ ] `data-theme` attribute set on `<html>` element
- [ ] Typecheck/lint passes
- [ ] Verify in browser

### US-003: Redesign Layout and Navigation
**Description:** As a user, I want a clear, uncluttered navigation that helps me find features without hunting through menus.

**Acceptance Criteria:**
- [ ] Sticky top bar: logo (left), primary nav links (center), theme toggle + user avatar (right)
- [ ] Desktop nav items: **Home**, **Blocklist**, **Community**, **Settings** — no more than 5 top-level items
- [ ] Active nav item has a clear indicator (underline or filled pill, not just color change)
- [ ] Mobile: hamburger menu opens a slide-over panel with all nav items
- [ ] User avatar/menu shows username, connected services status, sign-out
- [ ] Logo is text-based ("NDITH" or app name), not emoji
- [ ] Service connection status (Spotify/Apple) shown as small colored dots near user avatar, not full buttons in nav
- [ ] Remove legacy `SimpleNavigation.svelte` usage
- [ ] Typecheck/lint passes
- [ ] Verify in browser

### US-004: Redesign Home page
**Description:** As a user, I want the home page to give me an at-a-glance summary of my blocking activity and quick access to key actions.

**Acceptance Criteria:**
- [ ] Hero section: welcome message with user's name, quick stats (artists blocked, categories subscribed, enforcement status)
- [ ] Quick actions row: "Search Artists", "View Blocklist", "Browse Community Lists" as prominent cards/buttons
- [ ] Category subscriptions shown as compact cards in a grid (2-3 columns desktop, 1 on mobile)
- [ ] Each category card shows: name, artist count, subscribe/unsubscribe toggle, brief description
- [ ] Recently blocked artists shown as a horizontal scrollable list or compact table (last 5-10)
- [ ] Search bar prominently placed at top of page (not sticky — that's the nav bar's job)
- [ ] Search results appear in a dropdown overlay below the search input, not inline
- [ ] Empty state for new users: friendly illustration/icon + clear CTA to start blocking
- [ ] All interactive elements have hover/focus states
- [ ] Typecheck/lint passes
- [ ] Verify in browser

### US-005: Redesign Blocklist page
**Description:** As a user, I want to manage my blocked artists in a clean, scannable list with sorting and filtering.

**Acceptance Criteria:**
- [ ] Page header shows total blocked count and a search/filter input
- [ ] Blocked artists displayed in a clean list/table: artist name, genre badges, date blocked, source (manual/category), unblock button
- [ ] Artist rows have hover highlight and click-to-expand or click-to-profile
- [ ] Empty state with illustration and CTA to search artists
- [ ] Loading state uses skeleton placeholders (already exists, polish the styling)
- [ ] Error state is a dismissible banner, not a full-page block
- [ ] Unblock action has a confirmation step (small inline confirm, not a modal)
- [ ] Excepted artists section clearly separated with explanation text
- [ ] Responsive: list becomes card layout on mobile
- [ ] Typecheck/lint passes
- [ ] Verify in browser

### US-006: Redesign Artist Search experience
**Description:** As a user, I want to search for artists and block them in as few clicks as possible.

**Acceptance Criteria:**
- [ ] Search input with clear placeholder text, search icon, and clear (X) button
- [ ] Results appear as cards in a grid: artist image/placeholder, name, genres, provider badges (Spotify/Apple/MusicBrainz)
- [ ] Each result card has a prominent "Block" button (or "Blocked" state if already on DNP list)
- [ ] Clicking artist name/image navigates to artist profile
- [ ] Debounced search (300ms) with loading spinner in input
- [ ] "No results" state with helpful suggestions
- [ ] Search works from both Home page and Blocklist page (via ArtistSearchBar component)
- [ ] Typecheck/lint passes
- [ ] Verify in browser

### US-007: Redesign Settings page
**Description:** As a user, I want a clean settings page where I can manage my account, connected services, and preferences in organized sections.

**Acceptance Criteria:**
- [ ] Settings organized into sections with clear headings: Account, Connected Services, Blocking Preferences, Theme
- [ ] Account section: username display, email, logout button
- [ ] Connected Services section: Spotify and Apple Music cards with connect/disconnect buttons and status
- [ ] Blocking Preferences: toggles for "Block featured artists", "Block producers", notification preferences
- [ ] Theme section: theme toggle with preview (same as nav toggle but with labels)
- [ ] Each section is a bordered card with consistent padding
- [ ] Destructive actions (disconnect, logout) use danger-styled buttons with confirmation
- [ ] Typecheck/lint passes
- [ ] Verify in browser

### US-008: Polish shared UI components
**Description:** As a developer, I need the shared components (buttons, badges, cards, inputs, toasts) to look consistent with the new design.

**Acceptance Criteria:**
- [ ] Buttons: consistent padding (px-4 py-2), rounded-lg, clear primary/secondary/ghost/danger variants
- [ ] Badges: pill-shaped with subtle background, not too bright
- [ ] Cards: consistent border-radius (12px), subtle border in light mode, subtle bg elevation in dark mode
- [ ] Form inputs: consistent height (40px), rounded-lg, clear focus ring (rose-500), placeholder text styling
- [ ] Toast notifications: clean slide-in from bottom-right, auto-dismiss, close button
- [ ] Skeleton loaders: match new card dimensions and spacing
- [ ] All interactive elements have `:hover`, `:focus-visible`, `:active`, and `:disabled` states
- [ ] Typecheck/lint passes
- [ ] Verify in browser

## Functional Requirements

- FR-1: The design system must support light and dark themes via CSS custom properties on a `data-theme` attribute
- FR-2: Theme preference must be auto-detected from `prefers-color-scheme` on first visit and stored in localStorage
- FR-3: Theme toggle must be accessible from the navigation bar on every page
- FR-4: The navigation must show no more than 5 primary items on desktop and collapse into a hamburger menu on mobile (<768px)
- FR-5: The Home page must display user stats, category subscriptions, and recent blocks without scrolling on a 1080p viewport
- FR-6: Artist search must return results within 300ms of the user stopping typing (debounced)
- FR-7: The Blocklist page must handle 300+ blocked artists without performance degradation (virtual scrolling if needed)
- FR-8: All pages must be fully usable on mobile viewports (375px minimum width)
- FR-9: All interactive elements must have visible focus indicators for keyboard navigation
- FR-10: Unblock/disconnect destructive actions must require explicit user confirmation before executing

## Non-Goals (Out of Scope)

- Redesigning analytics, enforcement, graph explorer, or community detail pages (future iteration)
- Changing the routing system or store architecture
- Adding new features or API endpoints
- Migrating to SvelteKit or another framework
- Adding animations beyond basic hover/focus transitions
- Redesigning the Login/Register pages (functional as-is)
- Adding a component library (Shadcn, Melt UI, etc.) — use custom CSS

## Design Considerations

- **Aesthetic**: Clean, minimal, Spotify-inspired. Generous whitespace. Muted colors with rose as the accent.
- **Typography**: Inter (or system font stack: `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`) at 14px base, 1.5 line-height
- **Color palette (dark)**: Zinc-950 background, zinc-900 cards, zinc-800 elevated, rose-500 accent, zinc-400 secondary text
- **Color palette (light)**: White background, gray-50 cards, gray-100 elevated, rose-500 accent, gray-600 secondary text
- **Spacing**: 16px base padding for cards, 24px section gaps, 32px page margins
- **Icons**: Continue using inline SVG Heroicons pattern (no new dependencies)
- **Existing components to preserve**: `BlockedArtistCard`, `ArtistSearchBar`, `ConnectionCard`, `EnforcementBadges` (restyle, don't rewrite logic)

## Technical Considerations

- All styling changes are CSS-only or Svelte template changes — no backend modifications needed
- Theme detection must run before first paint (inline `<script>` in `index.html` or `app.html`)
- The `unified-design-system.css` file is the single source of truth for design tokens
- `app.css` compatibility layer (Tailwind-like utilities) should be updated to reference new tokens
- Component-scoped `<style>` blocks should use CSS variables, not hard-coded colors
- Test the theme toggle doesn't cause layout shifts or FOUC (flash of unstyled content)

## Success Metrics

- Every redesigned page passes a visual "would I use this app?" gut check
- Theme toggle works correctly across page navigations and browser refreshes
- No accessibility regressions: all existing ARIA labels, focus management, and skip links preserved
- Mobile layout is usable on a 375px viewport without horizontal scrolling
- Page load doesn't flash wrong theme

## Open Questions

- Should the app name in the nav be "No Drake in the House", "NDITH", or something shorter?
- Should we add subtle micro-animations (card hover lift, button press) or keep it strictly static?
- Is the current 4-tab desktop nav sufficient, or do users need quick access to Analytics/Enforcement from the top bar?
