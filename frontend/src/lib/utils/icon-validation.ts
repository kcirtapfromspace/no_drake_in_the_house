/**
 * Icon Validation and Migration Utilities
 * 
 * This module provides utilities for:
 * 1. Scanning and identifying oversized SVG patterns in components
 * 2. Runtime validation to detect and fix oversized icons in development mode
 * 3. Automated replacement of Tailwind sizing classes with design system classes
 */

// Common oversized Tailwind patterns that should be replaced
const OVERSIZED_PATTERNS = [
  { pattern: /class="([^"]*?)h-10 w-10([^"]*?)"/, replacement: 'class="$1icon icon--xl$2"' },
  { pattern: /class="([^"]*?)h-12 w-12([^"]*?)"/, replacement: 'class="$1icon icon--xl$2"' },
  { pattern: /class="([^"]*?)h-16 w-16([^"]*?)"/, replacement: 'class="$1icon icon--xl$2"' },
  { pattern: /class="([^"]*?)h-8 w-8([^"]*?)"/, replacement: 'class="$1icon icon--lg$2"' },
  { pattern: /class="([^"]*?)h-6 w-6([^"]*?)"/, replacement: 'class="$1icon icon--lg$2"' },
  { pattern: /class="([^"]*?)h-5 w-5([^"]*?)"/, replacement: 'class="$1icon icon--md$2"' },
  { pattern: /class="([^"]*?)h-4 w-4([^"]*?)"/, replacement: 'class="$1icon icon--sm$2"' }
];

// Maximum allowed dimensions for icons (in pixels)
const MAX_ICON_SIZE = 32; // 2rem

/**
 * Validates icon sizes at runtime and applies constraints if needed
 * Should only be used in development mode
 */
export function validateIconSizes(): void {
  if (typeof window === 'undefined' || process.env.NODE_ENV === 'production') {
    return;
  }

  const svgElements = document.querySelectorAll('svg');
  let violationCount = 0;

  svgElements.forEach((svg) => {
    const rect = svg.getBoundingClientRect();

    // Skip if element is not visible
    if (rect.width === 0 && rect.height === 0) {
      return;
    }

    // Check if icon exceeds maximum size
    if (rect.width > MAX_ICON_SIZE || rect.height > MAX_ICON_SIZE) {
      violationCount++;
      
      console.warn('🚨 Oversized icon detected:', {
        element: svg,
        currentSize: { width: rect.width, height: rect.height },
        maxAllowed: MAX_ICON_SIZE,
        classes: svg.className.baseVal || svg.getAttribute('class'),
        suggestion: 'Add proper icon classes (icon icon--sm, icon--md, etc.)'
      });

      // Auto-fix in development mode
      if (!svg.classList.contains('icon')) {
        svg.classList.add('icon', 'icon--lg');
        console.log('✅ Auto-applied icon constraints to oversized SVG');
      }
    }

    // Check for missing design system classes
    if (svg.tagName.toLowerCase() === 'svg' && !svg.classList.contains('icon')) {
      // Skip if it's clearly an avatar or image container
      const isAvatar = svg.closest('.avatar') || 
                      svg.closest('[class*="rounded-full"]') ||
                      svg.closest('[class*="avatar"]');
      
      if (!isAvatar) {
        console.info('💡 SVG without design system classes:', {
          element: svg,
          suggestion: 'Add icon class and size variant (icon icon--sm, icon--md, etc.)'
        });
      }
    }
  });

  if (violationCount > 0) {
    console.warn(`🚨 Found ${violationCount} oversized icons. Consider migrating to design system classes.`);
  }
}

/**
 * Scans component code for oversized icon patterns
 * Useful for build-time analysis
 */
export function scanForOversizedPatterns(componentCode: string): Array<{
  line: number;
  pattern: string;
  suggestion: string;
}> {
  const issues: Array<{ line: number; pattern: string; suggestion: string }> = [];
  const lines = componentCode.split('\n');

  lines.forEach((line, index) => {
    // Check for oversized Tailwind classes
    const oversizedMatch = line.match(/class="[^"]*h-(10|12|16|20|24)[^"]*w-(10|12|16|20|24)[^"]*"/);
    if (oversizedMatch) {
      issues.push({
        line: index + 1,
        pattern: oversizedMatch[0],
        suggestion: 'Replace with design system icon classes (icon icon--lg, icon--xl)'
      });
    }

    // Check for SVGs without proper classes
    const svgMatch = line.match(/<svg[^>]*class="[^"]*"[^>]*>/);
    if (svgMatch && !line.includes('icon')) {
      issues.push({
        line: index + 1,
        pattern: svgMatch[0],
        suggestion: 'Add icon class and size variant'
      });
    }

    // Check for missing aria-hidden on decorative icons
    const decorativeSvgMatch = line.match(/<svg(?![^>]*aria-label)(?![^>]*aria-labelledby)[^>]*>/);
    if (decorativeSvgMatch && !line.includes('aria-hidden')) {
      issues.push({
        line: index + 1,
        pattern: decorativeSvgMatch[0],
        suggestion: 'Add aria-hidden="true" for decorative icons'
      });
    }
  });

  return issues;
}

/**
 * Automatically replaces common oversized patterns with design system classes
 */
export function migrateOversizedPatterns(componentCode: string): string {
  let migratedCode = componentCode;

  OVERSIZED_PATTERNS.forEach(({ pattern, replacement }) => {
    migratedCode = migratedCode.replace(pattern, replacement);
  });

  // Add aria-hidden to decorative SVGs (those without aria-label)
  migratedCode = migratedCode.replace(
    /<svg(?![^>]*aria-label)(?![^>]*aria-labelledby)(?![^>]*aria-hidden)([^>]*)>/g,
    '<svg aria-hidden="true"$1>'
  );

  return migratedCode;
}

/**
 * Creates a development mode observer to watch for dynamically added oversized icons
 */
export function createIconSizeObserver(): MutationObserver | null {
  if (typeof window === 'undefined' || process.env.NODE_ENV === 'production') {
    return null;
  }

  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      mutation.addedNodes.forEach((node) => {
        if (node.nodeType === Node.ELEMENT_NODE) {
          const element = node as Element;
          
          // Check if the added node is an SVG or contains SVGs
          const svgs = element.tagName === 'SVG' 
            ? [element] 
            : Array.from(element.querySelectorAll('svg'));

          svgs.forEach((svg) => {
            // Delay validation to allow styles to be applied
            setTimeout(() => {
              const rect = svg.getBoundingClientRect();
              if (rect.width > MAX_ICON_SIZE || rect.height > MAX_ICON_SIZE) {
                console.warn('🚨 Dynamically added oversized icon:', svg);
                
                if (!svg.classList.contains('icon')) {
                  svg.classList.add('icon', 'icon--lg');
                }
              }
            }, 100);
          });
        }
      });
    });
  });

  observer.observe(document.body, {
    childList: true,
    subtree: true
  });

  return observer;
}

/**
 * Generates a report of icon usage across the application
 */
export function generateIconUsageReport(): {
  totalIcons: number;
  properlyClassified: number;
  oversized: number;
  missingAria: number;
  suggestions: string[];
} {
  const svgs = document.querySelectorAll('svg');
  let properlyClassified = 0;
  let oversized = 0;
  let missingAria = 0;
  const suggestions: string[] = [];

  svgs.forEach((svg) => {
    const rect = svg.getBoundingClientRect();
    
    // Skip invisible elements
    if (rect.width === 0 && rect.height === 0) return;

    // Check if properly classified
    if (svg.classList.contains('icon')) {
      properlyClassified++;
    }

    // Check if oversized
    if (rect.width > MAX_ICON_SIZE || rect.height > MAX_ICON_SIZE) {
      oversized++;
    }

    // Check for missing aria attributes
    if (!svg.hasAttribute('aria-hidden') && 
        !svg.hasAttribute('aria-label') && 
        !svg.hasAttribute('aria-labelledby')) {
      missingAria++;
    }
  });

  // Generate suggestions
  if (oversized > 0) {
    suggestions.push(`${oversized} icons are oversized. Consider using design system classes.`);
  }
  
  if (missingAria > 0) {
    suggestions.push(`${missingAria} icons are missing accessibility attributes.`);
  }

  const unclassified = svgs.length - properlyClassified;
  if (unclassified > 0) {
    suggestions.push(`${unclassified} icons are not using design system classes.`);
  }

  return {
    totalIcons: svgs.length,
    properlyClassified,
    oversized,
    missingAria,
    suggestions
  };
}

// Auto-initialize in development mode
if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
  // Run initial validation after DOM is loaded
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      setTimeout(validateIconSizes, 1000);
      createIconSizeObserver();
    });
  } else {
    setTimeout(validateIconSizes, 1000);
    createIconSizeObserver();
  }
}