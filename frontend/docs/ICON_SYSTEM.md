# Icon System Documentation

## Overview

The design system includes a comprehensive icon constraint system that prevents icons from scaling beyond their intended sizes. This system addresses the critical UX issue where SVG icons were expanding to fill all available browser space.

## Quick Start

### Basic Usage

```html
<!-- Standard UI icon -->
<svg class="icon icon--sm" aria-hidden="true">
  <!-- SVG content -->
</svg>

<!-- Emphasized icon -->
<svg class="icon icon--lg icon--primary" aria-hidden="true">
  <!-- SVG content -->
</svg>

<!-- Status icon -->
<svg class="icon icon--md icon--success" aria-label="Success">
  <!-- SVG content -->
</svg>
```

### Import Design System

Make sure to import the design system CSS in your component or globally:

```css
@import '../styles/design-system.css';
```

## Icon Sizes

| Class | Size | Usage |
|-------|------|-------|
| `icon--xs` | 12px | Inline text icons |
| `icon--sm` | 16px | Standard UI icons |
| `icon--md` | 20px | Emphasized icons |
| `icon--lg` | 24px | Section headers |
| `icon--xl` | 32px | Hero/feature icons (maximum) |

## Semantic Colors

| Class | Color | Usage |
|-------|-------|-------|
| `icon--success` | Green | Success states, checkmarks |
| `icon--error` | Red | Error states, warnings |
| `icon--warning` | Yellow | Caution, alerts |
| `icon--neutral` | Gray | Neutral information |
| `icon--primary` | Blue | Primary actions, highlights |

## Context-Specific Classes

### Button Icons
```html
<button class="btn btn--primary">
  <svg class="btn-icon" aria-hidden="true">
    <!-- Icon content -->
  </svg>
  Button Text
</button>
```

### Form Field Icons
```html
<div class="relative">
  <input class="form-input pl-10" />
  <div class="absolute inset-y-0 left-0 pl-3 flex items-center">
    <svg class="field-icon" aria-hidden="true">
      <!-- Icon content -->
    </svg>
  </div>
</div>
```

### Loading Spinners
```html
<svg class="loading-icon text-primary-600" aria-hidden="true">
  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
  <path class="opacity-75" fill="currentColor" d="..."></path>
</svg>
```

## Accessibility

### Decorative Icons
Icons that are purely decorative should have `aria-hidden="true"`:

```html
<svg class="icon icon--sm" aria-hidden="true">
  <!-- Decorative icon -->
</svg>
```

### Meaningful Icons
Icons that convey meaning should have appropriate labels:

```html
<!-- With aria-label -->
<svg class="icon icon--sm icon--success" aria-label="Success">
  <!-- Meaningful icon -->
</svg>

<!-- With aria-labelledby -->
<svg class="icon icon--sm" aria-labelledby="icon-label">
  <!-- Meaningful icon -->
</svg>
<span id="icon-label" class="sr-only">Delete item</span>
```

## Avatar System

For user profile images and artist photos, use the avatar system instead of icon classes:

```html
<!-- Avatar with image -->
<div class="avatar avatar--md">
  <img src="user-photo.jpg" alt="User Name" />
</div>

<!-- Avatar placeholder -->
<div class="avatar avatar--lg">
  <div class="avatar__placeholder">
    <svg class="icon icon--lg text-gray-600" aria-hidden="true">
      <!-- User icon -->
    </svg>
  </div>
</div>
```

Avatar sizes: `avatar--xs` (24px), `avatar--sm` (32px), `avatar--md` (40px), `avatar--lg` (48px), `avatar--xl` (64px)

## Global Constraints

The system includes ultra-aggressive global constraints that prevent ANY SVG from becoming oversized:

- All SVGs are constrained to a maximum of 32px (2rem)
- Design system icons override these constraints with proper sizing
- Tailwind sizing classes are automatically overridden
- Images and avatar containers are excluded from constraints

## Migration Tools

### Automated Migration

```bash
# Dry run to see what would change
npm run icons:migrate:dry-run

# Apply migrations
npm run icons:migrate

# Validate icons
npm run icons:validate

# Strict validation (for CI/CD)
npm run icons:validate:strict
```

### Manual Migration

Replace oversized Tailwind classes:

```html
<!-- Before -->
<svg class="h-12 w-12 text-gray-400">

<!-- After -->
<svg class="icon icon--xl text-gray-400" aria-hidden="true">
```

## Development Tools

### Runtime Validation

In development mode, the system automatically:
- Validates icon sizes and logs warnings for oversized icons
- Suggests proper design system classes
- Auto-fixes oversized icons by applying constraints
- Monitors for dynamically added oversized icons

### Build-Time Validation

The build process includes icon validation:
- Scans all Svelte components for oversized patterns
- Reports violations with suggestions
- Fails the build in strict mode if errors are found
- Generates detailed reports

### Debug Mode

Add the `debug-icons` class to visualize icon boundaries:

```html
<div class="debug-icons">
  <!-- Icons will show visual boundaries -->
</div>
```

## Best Practices

### Do's
- ✅ Always use `icon` base class with a size variant
- ✅ Add appropriate `aria-hidden` or `aria-label` attributes
- ✅ Use semantic color classes for status indicators
- ✅ Use avatar classes for profile images, not icon classes
- ✅ Test icons across different viewport sizes

### Don'ts
- ❌ Don't use Tailwind sizing classes (`h-8 w-8`) for icons
- ❌ Don't use inline `width` or `height` attributes
- ❌ Don't create icons larger than `icon--xl` (32px)
- ❌ Don't forget accessibility attributes
- ❌ Don't use icon classes for non-icon elements

## Troubleshooting

### Icons Still Oversized?

1. Check that design system CSS is imported
2. Verify the icon has the `icon` base class
3. Check browser dev tools for CSS conflicts
4. Run `npm run icons:validate` to identify issues

### Icons Too Small on Mobile?

The system automatically adjusts icon sizes on mobile. If you need custom mobile sizing:

```css
@media (max-width: 640px) {
  .my-custom-icon {
    width: var(--icon-md);
    height: var(--icon-md);
  }
}
```

### Build Failing Due to Icon Validation?

1. Run `npm run icons:migrate:dry-run` to see suggested changes
2. Apply migrations with `npm run icons:migrate`
3. Manually fix any remaining issues
4. Re-run validation with `npm run icons:validate`

## Performance Considerations

- Global constraints use `!important` for reliability but are overridden by design system classes
- Icon validation only runs in development mode
- CSS is optimized for minimal bundle size impact
- Responsive adjustments use CSS custom properties for efficiency

## Browser Support

- Modern browsers: Full support
- IE11: Partial support (no CSS custom properties)
- Fallbacks provided for older browsers
- Print styles included for proper icon sizing

## Contributing

When adding new icons:

1. Use the established size scale
2. Follow accessibility guidelines
3. Test across viewport sizes
4. Run validation tools
5. Update documentation if adding new patterns