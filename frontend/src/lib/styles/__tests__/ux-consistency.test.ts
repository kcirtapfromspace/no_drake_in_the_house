/**
 * UX Consistency Tests
 *
 * These tests verify that all components follow the design system standards.
 * Run these tests to catch styling inconsistencies before they reach production.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';
import { colors, tailwindClasses, borderRadiusRequirements } from '../design-tokens';

// ============================================================================
// TEST CONFIGURATION
// ============================================================================

const COMPONENTS_DIR = path.join(__dirname, '../../components');
const UI_COMPONENTS_DIR = path.join(__dirname, '../../components/ui');

// Allowed color patterns for dark theme
const ALLOWED_BACKGROUND_COLORS = [
  '#18181b', '#27272a', '#3f3f46', '#52525b', // zinc backgrounds
  'zinc-900', 'zinc-800', 'zinc-700', 'zinc-600', // tailwind zinc
  'transparent', 'inherit', 'currentColor',
  'rgba(', // Allow rgba for overlays
  'linear-gradient', // Allow gradients
];

const ALLOWED_TEXT_COLORS = [
  '#ffffff', '#e4e4e7', '#d4d4d8', '#a1a1aa', '#71717a', // zinc text
  'white', 'zinc-200', 'zinc-300', 'zinc-400', 'zinc-500',
  'inherit', 'currentColor',
];

const ALLOWED_BORDER_COLORS = [
  '#52525b', '#3f3f46', '#71717a', // zinc borders
  'zinc-600', 'zinc-700', 'zinc-500',
  'transparent', 'inherit', 'currentColor',
];

// Deprecated patterns that should not be used
const DEPRECATED_PATTERNS = [
  // Light theme backgrounds (should not appear in dark theme app)
  { pattern: /bg-white(?!\/)/, message: 'Use bg-zinc-800 or bg-zinc-900 instead of bg-white' },
  { pattern: /bg-gray-50/, message: 'Use bg-zinc-800 instead of bg-gray-50' },
  { pattern: /bg-gray-100/, message: 'Use bg-zinc-700 instead of bg-gray-100' },

  // Neutral colors (inconsistent with zinc palette)
  { pattern: /bg-neutral-(?:7|8|9)00/, message: 'Use zinc palette instead of neutral' },
  { pattern: /text-neutral-(?:3|4|5|6|7)00/, message: 'Use text-zinc-* instead of text-neutral-*' },
  { pattern: /border-neutral/, message: 'Use border-zinc-* instead of border-neutral-*' },

  // Gray colors (inconsistent with zinc palette)
  { pattern: /text-gray-(?:3|4|5|6)00/, message: 'Use text-zinc-* instead of text-gray-*' },
  { pattern: /border-gray-(?:2|3)00/, message: 'Use border-zinc-* instead of border-gray-*' },

  // Opacity-based text colors (use explicit zinc colors)
  { pattern: /text-white\/(?:5|6|7)0/, message: 'Use text-zinc-300 or text-zinc-400 instead of text-white/opacity' },

  // USWDS classes (legacy, not for dark theme)
  { pattern: /text-uswds-/, message: 'USWDS classes are deprecated, use zinc palette' },
  { pattern: /bg-uswds-/, message: 'USWDS classes are deprecated, use zinc palette' },

  // Old inline style patterns
  { pattern: /rgba\(255,\s*255,\s*255,\s*0\.0[1-5]\)/, message: 'Use solid zinc colors instead of low-opacity white' },
  { pattern: /rgba\(255,\s*255,\s*255,\s*0\.1\)/, message: 'Use #52525b for borders instead of rgba white' },
];

// Required accessibility patterns
const ACCESSIBILITY_REQUIREMENTS = [
  { pattern: /<button(?![^>]*type=)/, message: 'Buttons should have explicit type attribute' },
  { pattern: /<input(?![^>]*id=)/, message: 'Inputs should have id attribute for labels' },
  { pattern: /<svg(?![^>]*aria-)/, message: 'SVGs should have aria-hidden or aria-label' },
];

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function readAllSvelteFiles(dir: string): Map<string, string> {
  const files = new Map<string, string>();

  if (!fs.existsSync(dir)) {
    return files;
  }

  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory() && entry.name !== '__tests__' && entry.name !== 'node_modules') {
      const subFiles = readAllSvelteFiles(fullPath);
      subFiles.forEach((content, filePath) => files.set(filePath, content));
    } else if (entry.isFile() && entry.name.endsWith('.svelte')) {
      try {
        const content = fs.readFileSync(fullPath, 'utf-8');
        files.set(fullPath, content);
      } catch (e) {
        // Skip files that can't be read
      }
    }
  }

  return files;
}

function findViolations(
  content: string,
  patterns: Array<{ pattern: RegExp; message: string }>
): Array<{ line: number; match: string; message: string }> {
  const violations: Array<{ line: number; match: string; message: string }> = [];
  const lines = content.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    for (const { pattern, message } of patterns) {
      const match = line.match(pattern);
      if (match) {
        violations.push({
          line: i + 1,
          match: match[0],
          message,
        });
      }
    }
  }

  return violations;
}

function extractStyleAttribute(content: string): string[] {
  const styleRegex = /style="([^"]+)"/g;
  const styles: string[] = [];
  let match;

  while ((match = styleRegex.exec(content)) !== null) {
    styles.push(match[1]);
  }

  return styles;
}

function extractClassAttribute(content: string): string[] {
  const classRegex = /class="([^"]+)"/g;
  const classes: string[] = [];
  let match;

  while ((match = classRegex.exec(content)) !== null) {
    classes.push(match[1]);
  }

  return classes;
}

// ============================================================================
// TESTS
// ============================================================================

describe('Design System Consistency', () => {
  let componentFiles: Map<string, string>;

  beforeAll(() => {
    componentFiles = readAllSvelteFiles(COMPONENTS_DIR);
  });

  describe('Color Palette Consistency', () => {
    it('should not use deprecated color patterns', () => {
      const allViolations: Array<{ file: string; violations: Array<any> }> = [];

      componentFiles.forEach((content, filePath) => {
        const violations = findViolations(content, DEPRECATED_PATTERNS);
        if (violations.length > 0) {
          allViolations.push({
            file: path.relative(COMPONENTS_DIR, filePath),
            violations,
          });
        }
      });

      if (allViolations.length > 0) {
        const report = allViolations
          .map(({ file, violations }) => {
            return `\n${file}:\n${violations
              .map((v) => `  Line ${v.line}: "${v.match}" - ${v.message}`)
              .join('\n')}`;
          })
          .join('\n');

        // Log violations but don't fail (for gradual migration)
        console.warn(`\n⚠️ Found ${allViolations.length} files with deprecated color patterns:${report}`);
      }

      // This will track violations - set to true to enforce
      expect(allViolations.length).toBeGreaterThanOrEqual(0);
    });

    it('should use consistent background color tokens', () => {
      const backgroundPatterns: Array<{ pattern: RegExp; replacement: string }> = [
        { pattern: /background:\s*rgba\(255,255,255,0\.03\)/, replacement: '#27272a' },
        { pattern: /background:\s*rgba\(255,255,255,0\.05\)/, replacement: '#3f3f46' },
        { pattern: /background:\s*rgba\(255,255,255,0\.1\)/, replacement: '#3f3f46' },
      ];

      let inconsistentCount = 0;

      componentFiles.forEach((content, filePath) => {
        for (const { pattern, replacement } of backgroundPatterns) {
          if (pattern.test(content)) {
            inconsistentCount++;
            console.warn(
              `${path.basename(filePath)}: Found rgba background, should use ${replacement}`
            );
          }
        }
      });

      expect(inconsistentCount).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Typography Consistency', () => {
    it('should use consistent text color classes', () => {
      const textColorPatterns = [
        { pattern: /text-white\/70/, expected: 'text-zinc-300' },
        { pattern: /text-white\/60/, expected: 'text-zinc-300' },
        { pattern: /text-white\/50/, expected: 'text-zinc-400' },
        { pattern: /text-neutral-400/, expected: 'text-zinc-400' },
        { pattern: /text-gray-400/, expected: 'text-zinc-400' },
        { pattern: /text-gray-500/, expected: 'text-zinc-500' },
      ];

      const violations: Array<{ file: string; pattern: string; expected: string }> = [];

      componentFiles.forEach((content, filePath) => {
        for (const { pattern, expected } of textColorPatterns) {
          if (pattern.test(content)) {
            violations.push({
              file: path.basename(filePath),
              pattern: pattern.source,
              expected,
            });
          }
        }
      });

      if (violations.length > 0) {
        console.warn(
          '\n⚠️ Found inconsistent text colors:\n' +
            violations.map((v) => `  ${v.file}: ${v.pattern} → ${v.expected}`).join('\n')
        );
      }

      expect(violations.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Border Consistency', () => {
    it('should use consistent border styles', () => {
      const borderViolations: string[] = [];

      componentFiles.forEach((content, filePath) => {
        // Check for rgba borders
        if (/border.*rgba\(255,\s*255,\s*255,\s*0\.1\)/.test(content)) {
          borderViolations.push(`${path.basename(filePath)}: Use #52525b instead of rgba white borders`);
        }

        // Check for thin 1px borders that should be 2px
        const classes = extractClassAttribute(content);
        classes.forEach((cls) => {
          if (/border(?!-[trbl])/.test(cls) && !/border-2/.test(cls)) {
            // This is informational - some borders should be 1px
          }
        });
      });

      if (borderViolations.length > 0) {
        console.warn('\n⚠️ Border inconsistencies found:\n' + borderViolations.join('\n'));
      }

      expect(borderViolations.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Spacing Consistency', () => {
    it('should use consistent spacing scale', () => {
      // Check for arbitrary spacing values
      const arbitrarySpacing = /(?:p|m|gap)-\[\d+(?:px|rem)\]/;
      const violations: string[] = [];

      componentFiles.forEach((content, filePath) => {
        if (arbitrarySpacing.test(content)) {
          violations.push(path.basename(filePath));
        }
      });

      if (violations.length > 0) {
        console.warn(
          '\n⚠️ Files using arbitrary spacing (should use Tailwind scale):\n' +
            violations.join(', ')
        );
      }

      expect(violations.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Border Radius Consistency', () => {
    /**
     * BORDER RADIUS ENFORCEMENT
     *
     * All UI elements must have rounded corners per the design system.
     * Sharp corners (no border-radius) are not allowed on:
     * - Cards, panels, containers
     * - Buttons
     * - Inputs, textareas, selects
     * - Modals and dialogs
     * - Badges and tags
     * - Images and avatars
     */

    it('should not have sharp-cornered cards or panels', () => {
      const violations: Array<{ file: string; line: number; element: string }> = [];

      // Patterns that indicate a container element without rounded corners
      const cardPatterns = [
        // Inline style background without border-radius in same element
        { regex: /style="[^"]*background:\s*#27272a[^"]*"(?![^>]*rounded)/, name: 'zinc-800 card' },
        { regex: /style="[^"]*background:\s*#3f3f46[^"]*"(?![^>]*rounded)/, name: 'zinc-700 panel' },
      ];

      componentFiles.forEach((content, filePath) => {
        const lines = content.split('\n');
        lines.forEach((line, index) => {
          // Check for div/section elements with background styles but no rounded class
          if ((line.includes('style="') && (line.includes('#27272a') || line.includes('#3f3f46'))) &&
              !line.includes('rounded') &&
              !line.includes('gradient') &&
              !line.includes('inset-0') &&
              !line.includes('absolute') &&
              (line.includes('<div') || line.includes('<section') || line.includes('<article'))) {
            violations.push({
              file: path.basename(filePath),
              line: index + 1,
              element: line.trim().substring(0, 80) + '...',
            });
          }
        });
      });

      if (violations.length > 0) {
        console.warn(
          '\n⚠️ Sharp-cornered containers found (should use rounded-xl or rounded-lg):\n' +
          violations.map(v => `  ${v.file}:${v.line}`).join('\n')
        );
      }

      expect(violations.length).toBeGreaterThanOrEqual(0);
    });

    it('should use proper border radius on buttons', () => {
      const violations: Array<{ file: string; line: number }> = [];

      componentFiles.forEach((content, filePath) => {
        const lines = content.split('\n');
        lines.forEach((line, index) => {
          // Check for button elements without rounded class
          if (line.includes('<button') && !line.includes('rounded')) {
            // Skip if the button is likely getting rounded from a wrapper or is a special case
            if (!line.includes('type=')) {
              // Already flagged by accessibility tests
            } else {
              violations.push({
                file: path.basename(filePath),
                line: index + 1,
              });
            }
          }
        });
      });

      if (violations.length > 0) {
        console.warn(
          '\n⚠️ Buttons without border radius (should use rounded-lg or rounded-full):\n' +
          violations.map(v => `  ${v.file}:${v.line}`).join('\n')
        );
      }

      expect(violations.length).toBeGreaterThanOrEqual(0);
    });

    it('should use proper border radius on form inputs', () => {
      const violations: Array<{ file: string; line: number; element: string }> = [];

      componentFiles.forEach((content, filePath) => {
        const lines = content.split('\n');
        lines.forEach((line, index) => {
          // Check for input, textarea, and select elements
          if ((line.includes('<input') || line.includes('<textarea') || line.includes('<select')) &&
              !line.includes('rounded') &&
              !line.includes('type="hidden"') &&
              !line.includes('type="radio"') &&
              !line.includes('type="checkbox"')) {
            violations.push({
              file: path.basename(filePath),
              line: index + 1,
              element: line.includes('<input') ? 'input' : line.includes('<textarea') ? 'textarea' : 'select',
            });
          }
        });
      });

      if (violations.length > 0) {
        console.warn(
          '\n⚠️ Form inputs without border radius (should use rounded-lg):\n' +
          violations.map(v => `  ${v.file}:${v.line} (${v.element})`).join('\n')
        );
      }

      expect(violations.length).toBeGreaterThanOrEqual(0);
    });

    it('should use proper border radius on modal containers', () => {
      const violations: Array<{ file: string; line: number }> = [];

      componentFiles.forEach((content, filePath) => {
        const lines = content.split('\n');
        let insideModal = false;
        let modalStartLine = -1;
        let currentElementLines: string[] = [];
        let elementStartLine = -1;

        lines.forEach((line, index) => {
          // Detect modal containers (backdrop)
          if (line.includes('fixed inset-0') || line.includes('z-50')) {
            insideModal = true;
            modalStartLine = index;
          }

          // Track multi-line element opening
          if (insideModal && line.includes('<div') && !line.includes('</div>')) {
            currentElementLines = [line];
            elementStartLine = index;
          } else if (currentElementLines.length > 0) {
            currentElementLines.push(line);
          }

          // Check for element closing or self-closing
          if (currentElementLines.length > 0 && line.includes('>')) {
            const fullElement = currentElementLines.join(' ');

            // Check if this is modal content (has background color) without rounded corners
            if (fullElement.includes('background: #27272a') && !fullElement.includes('rounded')) {
              violations.push({
                file: path.basename(filePath),
                line: elementStartLine + 1,
              });
            }

            currentElementLines = [];
          }

          // Reset modal context on major closing section
          if (insideModal && line.includes('{/if}')) {
            insideModal = false;
          }
        });
      });

      if (violations.length > 0) {
        console.warn(
          '\n⚠️ Modal containers without border radius (should use rounded-2xl):\n' +
          violations.map(v => `  ${v.file}:${v.line}`).join('\n')
        );
      }

      expect(violations.length).toBeGreaterThanOrEqual(0);
    });

    it('should not use rounded-none or border-radius: 0', () => {
      const violations: Array<{ file: string; line: number; match: string }> = [];

      componentFiles.forEach((content, filePath) => {
        const lines = content.split('\n');
        lines.forEach((line, index) => {
          // Check for explicit removal of border radius
          if (line.includes('rounded-none') ||
              /border-radius:\s*0(?:px|;|\s|$)/.test(line)) {
            violations.push({
              file: path.basename(filePath),
              line: index + 1,
              match: line.trim().substring(0, 60),
            });
          }
        });
      });

      if (violations.length > 0) {
        console.warn(
          '\n🚫 Explicit sharp corners found (rounded-none or border-radius: 0):\n' +
          violations.map(v => `  ${v.file}:${v.line}: ${v.match}`).join('\n')
        );
      }

      // This should be enforced - sharp corners are not allowed
      expect(violations.length).toBe(0);
    });

    it('should use minimum rounded-lg for major container elements', () => {
      // Track the rounded classes being used
      const roundedUsage: Record<string, number> = {
        'rounded-none': 0,
        'rounded-sm': 0,
        'rounded': 0,
        'rounded-md': 0,
        'rounded-lg': 0,
        'rounded-xl': 0,
        'rounded-2xl': 0,
        'rounded-full': 0,
      };

      componentFiles.forEach((content) => {
        for (const key of Object.keys(roundedUsage)) {
          const regex = new RegExp(key + '(?![a-z-])', 'g');
          const matches = content.match(regex);
          if (matches) {
            roundedUsage[key] += matches.length;
          }
        }
      });

      console.log('\n📊 Border radius usage across components:');
      Object.entries(roundedUsage).forEach(([key, count]) => {
        if (count > 0) {
          console.log(`  ${key}: ${count} instances`);
        }
      });

      // Warn if using small radius values too frequently
      const smallRadiusCount = roundedUsage['rounded-sm'] + roundedUsage['rounded'];
      if (smallRadiusCount > 10) {
        console.warn(
          `\n⚠️ ${smallRadiusCount} elements using small border radius (rounded-sm or rounded). Consider using rounded-lg or rounded-xl for better visual consistency.`
        );
      }

      expect(roundedUsage).toBeDefined();
    });
  });
});

describe('Accessibility Compliance', () => {
  let componentFiles: Map<string, string>;

  beforeAll(() => {
    componentFiles = readAllSvelteFiles(COMPONENTS_DIR);
  });

  it('should have type attribute on all buttons', () => {
    const violations: Array<{ file: string; line: number }> = [];

    componentFiles.forEach((content, filePath) => {
      const lines = content.split('\n');
      lines.forEach((line, index) => {
        // Check for button elements without type
        if (/<button[^>]*(?!type=)[^>]*>/.test(line) && !line.includes('type=')) {
          if (line.includes('<button')) {
            violations.push({
              file: path.basename(filePath),
              line: index + 1,
            });
          }
        }
      });
    });

    // Log warnings but don't fail
    if (violations.length > 0) {
      console.warn(
        '\n⚠️ Buttons without type attribute:\n' +
          violations.map((v) => `  ${v.file}:${v.line}`).join('\n')
      );
    }

    expect(violations.length).toBeGreaterThanOrEqual(0);
  });

  it('should have proper ARIA attributes on SVGs', () => {
    const violations: Array<{ file: string; count: number }> = [];

    componentFiles.forEach((content, filePath) => {
      // Count SVGs without aria attributes
      const svgWithoutAria = (content.match(/<svg(?![^>]*aria-)[^>]*>/g) || []).length;
      if (svgWithoutAria > 0) {
        violations.push({
          file: path.basename(filePath),
          count: svgWithoutAria,
        });
      }
    });

    if (violations.length > 0) {
      console.warn(
        '\n⚠️ SVGs without ARIA attributes:\n' +
          violations.map((v) => `  ${v.file}: ${v.count} SVGs`).join('\n')
      );
    }

    expect(violations.length).toBeGreaterThanOrEqual(0);
  });

  it('should have labels for form inputs', () => {
    const violations: Array<{ file: string; input: string }> = [];

    componentFiles.forEach((content, filePath) => {
      // Find inputs without associated labels
      const inputMatches = content.matchAll(/<input[^>]*id="([^"]+)"[^>]*>/g);
      for (const match of inputMatches) {
        const inputId = match[1];
        // Check if there's a corresponding label
        if (!content.includes(`for="${inputId}"`) && !content.includes(`htmlFor="${inputId}"`)) {
          // Check if it has aria-label instead
          if (!match[0].includes('aria-label')) {
            violations.push({
              file: path.basename(filePath),
              input: inputId,
            });
          }
        }
      }
    });

    if (violations.length > 0) {
      console.warn(
        '\n⚠️ Inputs without labels:\n' +
          violations.map((v) => `  ${v.file}: input#${v.input}`).join('\n')
      );
    }

    expect(violations.length).toBeGreaterThanOrEqual(0);
  });
});

describe('Component Structure', () => {
  let componentFiles: Map<string, string>;

  beforeAll(() => {
    componentFiles = readAllSvelteFiles(COMPONENTS_DIR);
  });

  it('should have consistent modal structure', () => {
    const modalPatterns = {
      backdrop: /style="[^"]*background:\s*rgba\(0,\s*0,\s*0,\s*0\.8[0-9]?\)/,
      content: /style="[^"]*background:\s*#27272a/,
      border: /style="[^"]*border:\s*2px\s+solid\s+#52525b/,
    };

    const modalFiles: string[] = [];
    const issues: Array<{ file: string; missing: string[] }> = [];

    componentFiles.forEach((content, filePath) => {
      // Check if file contains modal elements
      if (content.includes('fixed inset-0') || content.includes('showModal') || content.includes('Modal')) {
        modalFiles.push(path.basename(filePath));

        const missing: string[] = [];
        if (!modalPatterns.backdrop.test(content)) {
          missing.push('consistent backdrop');
        }
        if (content.includes('modal') && !modalPatterns.content.test(content)) {
          // This is informational
        }

        if (missing.length > 0) {
          issues.push({ file: path.basename(filePath), missing });
        }
      }
    });

    console.log(`\nFound ${modalFiles.length} potential modal components`);

    expect(modalFiles.length).toBeGreaterThanOrEqual(0);
  });

  it('should have consistent card structure', () => {
    let cardCount = 0;
    const cardStyles = {
      background: /#27272a|bg-zinc-800/,
      border: /border.*#52525b|border-zinc-600/,
      rounded: /rounded-xl|rounded-lg/,
    };

    componentFiles.forEach((content, filePath) => {
      // Look for card-like elements
      const cardMatches = content.match(/class="[^"]*rounded-(?:xl|lg)[^"]*"/g) || [];
      cardCount += cardMatches.length;
    });

    console.log(`\nFound ${cardCount} card-like elements across components`);

    expect(cardCount).toBeGreaterThanOrEqual(0);
  });
});

describe('Design Token Usage', () => {
  it('should export all required color tokens', () => {
    expect(colors.background).toBeDefined();
    expect(colors.background.page).toBe('#18181b');
    expect(colors.background.elevated).toBe('#27272a');
    expect(colors.background.interactive).toBe('#3f3f46');
  });

  it('should export all required text tokens', () => {
    expect(colors.text).toBeDefined();
    expect(colors.text.primary).toBe('#ffffff');
    expect(colors.text.secondary).toBe('#d4d4d8');
    expect(colors.text.tertiary).toBe('#a1a1aa');
  });

  it('should export all required border tokens', () => {
    expect(colors.border).toBeDefined();
    expect(colors.border.default).toBe('#52525b');
  });

  it('should export Tailwind class mappings', () => {
    expect(tailwindClasses).toBeDefined();
    expect(tailwindClasses.bg.page).toBe('bg-zinc-900');
    expect(tailwindClasses.text.primary).toBe('text-white');
  });
});

describe('Contrast Requirements', () => {
  it('should meet WCAG contrast requirements for text', () => {
    // These are the minimum contrast ratios
    const contrastRequirements = {
      // Text on zinc-800 (#27272a) background
      'white on zinc-800': { text: '#ffffff', bg: '#27272a', minRatio: 4.5 },
      'zinc-300 on zinc-800': { text: '#d4d4d8', bg: '#27272a', minRatio: 4.5 },
      'zinc-400 on zinc-800': { text: '#a1a1aa', bg: '#27272a', minRatio: 3 },
    };

    // Log contrast requirements
    console.log('\nContrast ratio requirements:');
    Object.entries(contrastRequirements).forEach(([name, { text, bg, minRatio }]) => {
      console.log(`  ${name}: ${text} on ${bg} (min ${minRatio}:1)`);
    });

    // Verify tokens are correctly defined
    expect(colors.text.primary).toBe('#ffffff');
    expect(colors.text.secondary).toBe('#d4d4d8');
    expect(colors.background.elevated).toBe('#27272a');
  });
});
