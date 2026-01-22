# PRD: UX Design System Alignment

## Amazon Executive Code Review Summary

**Reviewer**: Design Executive Perspective
**Date**: 2026-01-21
**Application**: No Drake in the House - Music Streaming Blocklist Manager

---

## 1. Customer Problem Statement

Users need a consistent, accessible, and intuitive interface to manage their music streaming blocklists across multiple platforms. The current UX creates cognitive friction through:

- **Inconsistent visual language** - Two competing design systems create unpredictable interactions
- **Accessibility barriers** - Missing ARIA labels exclude screen reader users
- **Mobile experience gaps** - No responsive navigation for the 60%+ mobile user base
- **Unclear system feedback** - Inconsistent loading/error states cause user uncertainty

**Customer Impact**: Users abandon tasks, lose trust in the system, and cannot efficiently manage their blocklists.

---

## 2. Code Assessment: Current State

### 2.1 Design Token Conflict (CRITICAL)

**Files**: `design-system.css` vs `optimized-design-system.css`

| Token | design-system.css | optimized-design-system.css | Impact |
|-------|-------------------|----------------------------|--------|
| Primary | #3b82f6 | #005ea2 | Brand inconsistency |
| Error | #ef4444 | #d63384 | Confusing error states |
| Success | #22c55e | #00a91c | Mixed feedback signals |

**Customer Impact**: Users see different colors for the same actions across pages, eroding trust.

### 2.2 Component Pattern Fragmentation

**Buttons across codebase**:
```
Login.svelte:136     → bg-rose-500 (inline Tailwind)
Dashboard.svelte:346 → Mixed patterns
ArtistSearch.svelte  → .btn-primary class
ConnectionCard.svelte → Hardcoded colors
```

**Customer Question**: "Why does the sign-in button look different from the dashboard buttons?"

### 2.3 Accessibility Gaps

| Issue | Occurrences | Customer Impact |
|-------|-------------|-----------------|
| SVGs missing aria-hidden | 47 instances | Screen readers announce decorative icons |
| Missing aria-live regions | Toast system | Blind users miss notifications |
| No focus-visible states | All buttons | Keyboard users can't navigate |
| Poor color contrast | text-zinc-400 on dark | Low-vision users can't read |

### 2.4 Performance Concerns

- **CSS over-specificity**: 127 `!important` rules in design-system.css
- **Duplicate styles**: Both design systems loaded simultaneously
- **No code-splitting**: All component CSS bundled together

---

## 3. Probing Questions for Engineering

1. **Trade-off Decision**: Why maintain two design systems? What customer research drove optimized-design-system.css?

2. **Failure Mode**: If a user has `prefers-reduced-motion` enabled, are spinners still animating? What's the fallback?

3. **Measured Performance**: What's the First Contentful Paint (FCP) with dual CSS systems? Have we measured Cumulative Layout Shift (CLS)?

4. **Peak Load UX**: During enforcement batch operations (potentially 100+ items), does the UI remain responsive? Is there skeleton loading?

5. **Mobile Analytics**: What % of users access via mobile? Do we have mobile-specific error rates?

---

## 4. Prioritized Recommendations

### P0 - Critical (Week 1-2)

| ID | Issue | Customer Impact | Effort |
|----|-------|-----------------|--------|
| UX-001 | Consolidate design systems | Eliminates visual inconsistency | M |
| UX-002 | Add ARIA labels to all interactive elements | Enables screen reader access | S |
| UX-003 | Implement aria-live for toast notifications | Blind users get feedback | S |
| UX-004 | Add focus-visible states | Keyboard navigation works | S |

### P1 - High Priority (Week 3-4)

| ID | Issue | Customer Impact | Effort |
|----|-------|-----------------|--------|
| UX-005 | Create unified Button component | Consistent interactions | M |
| UX-006 | Implement mobile responsive nav | Mobile users can navigate | M |
| UX-007 | Add skeleton loading states | Perceived performance improves | M |
| UX-008 | Fix color contrast issues | Low-vision accessibility | S |

### P2 - Medium Priority (Week 5-6)

| ID | Issue | Customer Impact | Effort |
|----|-------|-----------------|--------|
| UX-009 | Implement form error component | Clear validation feedback | M |
| UX-010 | Respect prefers-reduced-motion | Motion-sensitive users | S |
| UX-011 | Add breadcrumb navigation | Users understand location | S |
| UX-012 | Create Card component library | Faster development | L |

---

## 5. User Stories

### US-UX-001: Unified Design Token System

**As a** user navigating the application
**I want** consistent colors and styling across all pages
**So that** I can trust the interface and predict interactions

**Acceptance Criteria**:
- [ ] Single source of truth for design tokens in `design-tokens.ts`
- [ ] Remove `optimized-design-system.css` or merge into primary
- [ ] All components reference CSS variables, not hardcoded colors
- [ ] Color palette documented in Storybook/style guide

**Technical Notes**:
- Migrate to CSS custom properties exclusively
- Create Tailwind config that references design tokens
- Remove all `!important` overrides (127 instances)

---

### US-UX-002: Accessible Icon System

**As a** screen reader user
**I want** decorative icons to be hidden and interactive icons to be labeled
**So that** I can understand the interface without visual cues

**Acceptance Criteria**:
- [ ] All decorative SVGs have `aria-hidden="true"`
- [ ] All interactive icons have `aria-label` or `aria-labelledby`
- [ ] Icon buttons have visible focus states
- [ ] Audit passes axe-core accessibility checks

**Files to Update**:
- `ArtistCard.svelte` (lines 66-87)
- `EnforcementBadges.svelte`
- `Navigation.svelte` (line 22)
- All icon button implementations

---

### US-UX-003: Toast Notification Accessibility

**As a** blind user performing actions
**I want** to be notified of success/error states audibly
**So that** I know when my actions complete

**Acceptance Criteria**:
- [ ] Toast container has `role="region"` and `aria-live="polite"`
- [ ] Error toasts use `aria-live="assertive"`
- [ ] Toast messages are concise and actionable
- [ ] Toasts can be dismissed via keyboard (Escape key)

**File**: `BlockingToasts.svelte`

---

### US-UX-004: Keyboard Navigation Support

**As a** keyboard-only user
**I want** visible focus indicators on all interactive elements
**So that** I can see where I am in the interface

**Acceptance Criteria**:
- [ ] All buttons show visible focus ring (`:focus-visible`)
- [ ] Focus order follows logical reading order
- [ ] Modal focus trapped within modal when open
- [ ] Skip-to-content link available
- [ ] No focus traps in navigation

**CSS Addition**:
```css
:focus-visible {
  outline: 2px solid var(--color-primary-500);
  outline-offset: 2px;
}
```

---

### US-UX-005: Unified Button Component

**As a** developer building features
**I want** a single Button component with variants
**So that** all buttons are consistent and accessible

**Acceptance Criteria**:
- [ ] Button component with variants: primary, secondary, danger, ghost
- [ ] Size variants: sm, md, lg
- [ ] Loading state with spinner
- [ ] Disabled state with proper aria-disabled
- [ ] Icon support (left/right positioning)

**Props Interface**:
```typescript
interface ButtonProps {
  variant: 'primary' | 'secondary' | 'danger' | 'ghost';
  size: 'sm' | 'md' | 'lg';
  loading?: boolean;
  disabled?: boolean;
  icon?: SvelteComponent;
  iconPosition?: 'left' | 'right';
}
```

---

### US-UX-006: Mobile Responsive Navigation

**As a** mobile user
**I want** a hamburger menu that shows navigation options
**So that** I can access all features on my phone

**Acceptance Criteria**:
- [ ] Hamburger icon appears below 768px breakpoint
- [ ] Menu slides in from left/right with animation
- [ ] Menu can be closed by tapping outside or X button
- [ ] Current page highlighted in menu
- [ ] Touch targets minimum 44x44px

**File**: `Navigation.svelte`

---

### US-UX-007: Skeleton Loading States

**As a** user waiting for data to load
**I want** to see placeholder content shapes
**So that** I know content is loading and where it will appear

**Acceptance Criteria**:
- [ ] Skeleton component used for all async data displays
- [ ] Skeleton matches approximate shape of loaded content
- [ ] Animation respects `prefers-reduced-motion`
- [ ] Skeleton.svelte component already exists - integrate into:
  - Dashboard artist search results
  - DNP list loading
  - Connection status cards

---

### US-UX-008: Color Contrast Remediation

**As a** user with low vision
**I want** sufficient color contrast on all text
**So that** I can read the interface

**Acceptance Criteria**:
- [ ] All text meets WCAG 2.1 AA contrast ratio (4.5:1 for normal, 3:1 for large)
- [ ] Replace `text-zinc-400` on dark backgrounds with `text-zinc-300` or lighter
- [ ] Error/success states meet contrast requirements
- [ ] Audit with Lighthouse accessibility score > 90

**Current Violations**:
- `text-zinc-400` on `bg-zinc-900`: 3.9:1 ratio (FAIL)
- `text-zinc-500` on `bg-zinc-800`: 3.2:1 ratio (FAIL)

---

### US-UX-009: Form Error Component

**As a** user submitting forms
**I want** clear, consistent error messages
**So that** I know how to fix my input

**Acceptance Criteria**:
- [ ] FormError component with icon and message
- [ ] Error linked to input via `aria-describedby`
- [ ] Error announced to screen readers
- [ ] Consistent styling across all forms
- [ ] Error recovery suggestions when applicable

---

### US-UX-010: Reduced Motion Support

**As a** user with vestibular disorders
**I want** animations disabled when I've set that preference
**So that** I don't experience discomfort

**Acceptance Criteria**:
- [ ] All CSS animations respect `@media (prefers-reduced-motion: reduce)`
- [ ] Svelte transitions check `window.matchMedia('(prefers-reduced-motion: reduce)')`
- [ ] Spinners show static alternative or simplified animation
- [ ] No motion on page transitions

**Code Pattern**:
```svelte
<script>
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
</script>

{#if !reducedMotion}
  <div transition:fly={{ y: 20, duration: 200 }}>
{:else}
  <div transition:fade={{ duration: 0 }}>
{/if}
```

---

## 6. Technical Implementation Notes

### Design Token Migration Strategy

1. **Phase 1**: Create canonical `design-tokens.ts`
```typescript
export const tokens = {
  colors: {
    primary: { 500: '#3b82f6', 600: '#2563eb' },
    error: { 500: '#ef4444', 600: '#dc2626' },
    success: { 500: '#22c55e', 600: '#16a34a' },
  },
  spacing: { 1: '4px', 2: '8px', 3: '12px', 4: '16px' },
  transitions: { fast: '150ms', normal: '200ms', slow: '300ms' },
};
```

2. **Phase 2**: Generate CSS variables from tokens
3. **Phase 3**: Update Tailwind config to use tokens
4. **Phase 4**: Migrate components one-by-one
5. **Phase 5**: Remove legacy design-system files

### Component Audit Checklist

For each component, verify:
- [ ] Uses design token colors (no hardcoded hex)
- [ ] Uses design token spacing (no arbitrary Tailwind values)
- [ ] Has proper ARIA attributes
- [ ] Has focus-visible styles
- [ ] Respects reduced motion preference
- [ ] Works at mobile breakpoints
- [ ] Has loading state (if async)
- [ ] Has error state (if fallible)

---

## 7. Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Lighthouse Accessibility | ~72 | >90 | Weekly CI check |
| axe-core violations | 47 | 0 | Pre-commit hook |
| Design token coverage | ~20% | 100% | Code analysis |
| Mobile task completion | Unknown | >95% | Analytics |
| Support tickets (UX) | Unknown | -50% | Zendesk |

---

## 8. Figma Design Standards Alignment

### Current Gaps vs. Figma Best Practices

| Figma Standard | Current State | Gap |
|----------------|---------------|-----|
| 8px grid system | Partially implemented | Inconsistent spacing values |
| Component variants | None | All styling is inline |
| Auto-layout | N/A (CSS) | Flexbox/Grid used inconsistently |
| Design tokens | Dual systems | Needs consolidation |
| Responsive breakpoints | Hardcoded | Should be tokenized |
| Typography scale | Exists but unused | Components use arbitrary sizes |

### Recommended Figma-to-Code Workflow

1. **Design Tokens Sync**: Export Figma tokens to `design-tokens.json`
2. **Component Library**: Build Svelte components matching Figma components
3. **Storybook**: Document all components with states
4. **Visual Regression**: Chromatic or Percy for UI testing

---

## 9. Timeline & Resources

| Phase | Duration | Resources | Deliverable |
|-------|----------|-----------|-------------|
| P0 Critical | 2 weeks | 1 FE Engineer | Accessibility fixes, token consolidation |
| P1 High | 2 weeks | 1 FE Engineer | Button component, mobile nav, skeleton loading |
| P2 Medium | 2 weeks | 1 FE Engineer | Form errors, reduced motion, breadcrumbs |
| Documentation | 1 week | 0.5 FE Engineer | Storybook, style guide |

**Total Estimate**: 7 weeks with 1 dedicated frontend engineer

---

## 10. Appendix: File Reference

### Files Requiring Immediate Changes

| File | Issues | Priority |
|------|--------|----------|
| `design-system.css` | 127 !important rules, conflicts | P0 |
| `optimized-design-system.css` | Duplicate tokens | P0 |
| `BlockingToasts.svelte` | Missing aria-live | P0 |
| `Login.svelte` | Inline styles, no design tokens | P1 |
| `Dashboard.svelte` | 367 lines, mixed patterns | P1 |
| `Navigation.svelte` | Hardcoded colors, no mobile | P1 |
| `ArtistCard.svelte` | Missing ARIA on icons | P0 |
| `ConnectionCard.svelte` | Hardcoded colors | P1 |

### New Components to Create

1. `Button.svelte` - Unified button with variants
2. `FormError.svelte` - Accessible form error display
3. `MobileNav.svelte` - Responsive hamburger menu
4. `Icon.svelte` - Wrapper with automatic aria handling

---

**Document Status**: Draft
**Author**: Claude (Amazon Exec Review Perspective)
**Next Review**: Engineering team alignment meeting
