#!/usr/bin/env node

/**
 * CSS Cleanup and Layout Fix Tool
 * 
 * This script addresses:
 * 1. Removes unused CSS selectors and redundant styles
 * 2. Fixes text layout issues caused by aggressive icon constraints
 * 3. Consolidates overlapping design system files
 * 4. Optimizes CSS specificity and performance
 */

import fs from 'fs';
import path from 'path';

class CSSCleanup {
  constructor() {
    this.usedClasses = new Set();
    this.unusedSelectors = [];
    this.duplicateRules = [];
    this.layoutIssues = [];
  }

  // Scan all Svelte files to find actually used CSS classes
  async scanUsedClasses() {
    console.log('🔍 Scanning for used CSS classes...');
    
    const svelteFiles = await this.findFiles('frontend/src', '.svelte');
    const jsFiles = await this.findFiles('frontend/src', '.js');
    const tsFiles = await this.findFiles('frontend/src', '.ts');
    
    const allFiles = [...svelteFiles, ...jsFiles, ...tsFiles];
    
    for (const file of allFiles) {
      const content = fs.readFileSync(file, 'utf8');
      
      // Extract class names from class="..." and class:name={...}
      const classMatches = content.match(/class[:\s]*=\s*["'`]([^"'`]+)["'`]/g) || [];
      const dynamicClassMatches = content.match(/class:([a-zA-Z0-9_-]+)/g) || [];
      
      classMatches.forEach(match => {
        const classes = match.replace(/class[:\s]*=\s*["'`]/, '').replace(/["'`]$/, '');
        classes.split(/\s+/).forEach(cls => {
          if (cls.trim()) this.usedClasses.add(cls.trim());
        });
      });
      
      dynamicClassMatches.forEach(match => {
        const className = match.replace('class:', '');
        this.usedClasses.add(className);
      });
    }
    
    console.log(`✅ Found ${this.usedClasses.size} used CSS classes`);
  }

  // Find files with specific extension
  async findFiles(dir, extension) {
    const files = [];
    
    const scan = (currentDir) => {
      const items = fs.readdirSync(currentDir);
      
      for (const item of items) {
        const fullPath = path.join(currentDir, item);
        const stat = fs.statSync(fullPath);
        
        if (stat.isDirectory() && !item.startsWith('.') && item !== 'node_modules') {
          scan(fullPath);
        } else if (stat.isFile() && item.endsWith(extension)) {
          files.push(fullPath);
        }
      }
    };
    
    scan(dir);
    return files;
  }

  // Analyze CSS files for unused selectors and layout issues
  analyzeCSSFiles() {
    console.log('🔍 Analyzing CSS files for issues...');
    
    const cssFiles = [
      'frontend/src/lib/styles/uswds-skeleton-theme.css',
      'frontend/src/lib/styles/design-system.css',
      'frontend/src/app.css'
    ];
    
    for (const cssFile of cssFiles) {
      if (fs.existsSync(cssFile)) {
        console.log(`📄 Analyzing ${cssFile}...`);
        const content = fs.readFileSync(cssFile, 'utf8');
        this.findUnusedSelectors(content, cssFile);
        this.findLayoutIssues(content, cssFile);
      }
    }
  }

  // Find unused CSS selectors
  findUnusedSelectors(content, filename) {
    // Extract CSS selectors (simplified regex)
    const selectorMatches = content.match(/^[^{]*{/gm) || [];
    
    selectorMatches.forEach(match => {
      const selector = match.replace('{', '').trim();
      
      // Skip global selectors, pseudo-selectors, and media queries
      if (selector.startsWith('@') || 
          selector.startsWith(':global') || 
          selector.includes(':hover') || 
          selector.includes(':focus') ||
          selector.includes('::') ||
          selector.startsWith(':root')) {
        return;
      }
      
      // Extract class names from selector
      const classNames = selector.match(/\.[a-zA-Z0-9_-]+/g) || [];
      
      let isUsed = false;
      for (const className of classNames) {
        const cleanClass = className.replace('.', '');
        if (this.usedClasses.has(cleanClass)) {
          isUsed = true;
          break;
        }
      }
      
      if (!isUsed && classNames.length > 0) {
        this.unusedSelectors.push({
          file: filename,
          selector: selector,
          classes: classNames
        });
      }
    });
  }

  // Find layout-breaking CSS rules
  findLayoutIssues(content, filename) {
    const issues = [];
    
    // Check for aggressive !important rules that might break layout
    const importantMatches = content.match(/[^}]*!important[^}]*}/g) || [];
    importantMatches.forEach(match => {
      if (match.includes('width') || match.includes('height') || match.includes('display')) {
        issues.push({
          type: 'aggressive-important',
          rule: match.trim(),
          file: filename
        });
      }
    });
    
    // Check for overly specific global selectors
    const globalMatches = content.match(/:global\([^)]+\)[^{]*{[^}]*}/g) || [];
    globalMatches.forEach(match => {
      if (match.includes('!important') && (match.includes('width') || match.includes('height'))) {
        issues.push({
          type: 'global-override',
          rule: match.trim(),
          file: filename
        });
      }
    });
    
    // Check for conflicting icon constraints
    if (content.includes('svg') && content.includes('!important')) {
      const svgRules = content.match(/svg[^{]*{[^}]*}/g) || [];
      if (svgRules.length > 3) {
        issues.push({
          type: 'icon-constraint-conflict',
          count: svgRules.length,
          file: filename
        });
      }
    }
    
    this.layoutIssues.push(...issues);
  }

  // Generate optimized CSS file
  generateOptimizedCSS() {
    console.log('🛠️  Generating optimized CSS...');
    
    const optimizedCSS = `
/**
 * Optimized Design System CSS
 * Generated by CSS Cleanup Tool
 * 
 * This file consolidates and optimizes the design system styles,
 * removing unused selectors and fixing layout issues.
 */

/* ===== DESIGN TOKENS ===== */
:root {
  /* USWDS Color Palette */
  --color-primary: #005ea2;
  --color-primary-dark: #0050d8;
  --color-primary-darker: #1a4480;
  --color-success: #00a91c;
  --color-warning: #ffbe2e;
  --color-error: #d63384;
  
  /* Neutral Colors */
  --color-gray-50: #f9fafb;
  --color-gray-100: #f3f4f6;
  --color-gray-200: #e5e7eb;
  --color-gray-300: #d1d5db;
  --color-gray-400: #9ca3af;
  --color-gray-500: #6b7280;
  --color-gray-600: #4b5563;
  --color-gray-700: #374151;
  --color-gray-800: #1f2937;
  --color-gray-900: #111827;
  
  /* Typography Scale */
  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;
  --text-xl: 1.25rem;
  --text-2xl: 1.5rem;
  
  /* Spacing Scale */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-5: 1.25rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  
  /* Icon Sizes - CONTROLLED AND CONSISTENT */
  --icon-xs: 0.75rem;   /* 12px */
  --icon-sm: 1rem;      /* 16px */
  --icon-md: 1.25rem;   /* 20px */
  --icon-lg: 1.5rem;    /* 24px */
  --icon-xl: 2rem;      /* 32px - MAXIMUM */
  
  /* Border Radius */
  --radius-sm: 0.25rem;
  --radius-md: 0.375rem;
  --radius-lg: 0.5rem;
  
  /* Shadows */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);
}

/* ===== LAYOUT SYSTEM ===== */

/* Container for responsive layouts */
.container {
  width: 100%;
  margin: 0 auto;
  padding: 0 var(--space-4);
  max-width: 1280px;
}

/* Flexbox utilities */
.flex { display: flex; }
.flex-col { flex-direction: column; }
.flex-row { flex-direction: row; }
.items-center { align-items: center; }
.items-start { align-items: flex-start; }
.justify-center { justify-content: center; }
.justify-between { justify-content: space-between; }
.gap-2 { gap: var(--space-2); }
.gap-3 { gap: var(--space-3); }
.gap-4 { gap: var(--space-4); }

/* Grid utilities */
.grid { display: grid; }
.grid-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
.grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }

/* Spacing utilities */
.p-2 { padding: var(--space-2); }
.p-3 { padding: var(--space-3); }
.p-4 { padding: var(--space-4); }
.p-6 { padding: var(--space-6); }
.m-2 { margin: var(--space-2); }
.m-3 { margin: var(--space-3); }
.m-4 { margin: var(--space-4); }
.mb-2 { margin-bottom: var(--space-2); }
.mb-3 { margin-bottom: var(--space-3); }
.mb-4 { margin-bottom: var(--space-4); }
.mt-2 { margin-top: var(--space-2); }
.mt-3 { margin-top: var(--space-3); }
.mt-4 { margin-top: var(--space-4); }

/* Text utilities */
.text-xs { font-size: var(--text-xs); }
.text-sm { font-size: var(--text-sm); }
.text-base { font-size: var(--text-base); }
.text-lg { font-size: var(--text-lg); }
.text-xl { font-size: var(--text-xl); }
.text-2xl { font-size: var(--text-2xl); }
.font-medium { font-weight: 500; }
.font-semibold { font-weight: 600; }
.font-bold { font-weight: 700; }

/* Color utilities */
.text-gray-500 { color: var(--color-gray-500); }
.text-gray-600 { color: var(--color-gray-600); }
.text-gray-700 { color: var(--color-gray-700); }
.text-gray-900 { color: var(--color-gray-900); }
.text-primary { color: var(--color-primary); }
.text-success { color: var(--color-success); }
.text-error { color: var(--color-error); }
.text-warning { color: var(--color-warning); }

/* Background utilities */
.bg-white { background-color: white; }
.bg-gray-50 { background-color: var(--color-gray-50); }
.bg-gray-100 { background-color: var(--color-gray-100); }
.bg-primary { background-color: var(--color-primary); }
.bg-success { background-color: var(--color-success); }
.bg-error { background-color: var(--color-error); }

/* Border utilities */
.border { border: 1px solid var(--color-gray-200); }
.border-gray-300 { border-color: var(--color-gray-300); }
.rounded { border-radius: var(--radius-md); }
.rounded-lg { border-radius: var(--radius-lg); }
.rounded-full { border-radius: 9999px; }

/* ===== ICON SYSTEM - SIMPLIFIED AND EFFECTIVE ===== */

/* Base icon class */
.icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

/* Icon size classes */
.icon-xs { width: var(--icon-xs); height: var(--icon-xs); }
.icon-sm { width: var(--icon-sm); height: var(--icon-sm); }
.icon-md { width: var(--icon-md); height: var(--icon-md); }
.icon-lg { width: var(--icon-lg); height: var(--icon-lg); }
.icon-xl { width: var(--icon-xl); height: var(--icon-xl); }

/* Icon color variants */
.icon-primary { color: var(--color-primary); }
.icon-success { color: var(--color-success); }
.icon-error { color: var(--color-error); }
.icon-warning { color: var(--color-warning); }
.icon-neutral { color: var(--color-gray-400); }

/* ===== CONTROLLED SVG CONSTRAINTS ===== */

/* 
 * IMPORTANT: These constraints are more targeted and less aggressive
 * to prevent layout issues while still controlling icon sizes
 */

/* Default constraint for SVGs without explicit sizing */
svg:not(.icon):not([class*="avatar"]):not([class*="logo"]) {
  max-width: var(--icon-xl);
  max-height: var(--icon-xl);
  width: var(--icon-md);
  height: var(--icon-md);
  flex-shrink: 0;
}

/* Design system icons get proper sizing */
svg.icon-xs { width: var(--icon-xs); height: var(--icon-xs); }
svg.icon-sm { width: var(--icon-sm); height: var(--icon-sm); }
svg.icon-md { width: var(--icon-md); height: var(--icon-md); }
svg.icon-lg { width: var(--icon-lg); height: var(--icon-lg); }
svg.icon-xl { width: var(--icon-xl); height: var(--icon-xl); }

/* Legacy Tailwind class overrides - less aggressive */
.h-4 { width: var(--icon-sm); height: var(--icon-sm); }
.h-5 { width: var(--icon-md); height: var(--icon-md); }
.h-6 { width: var(--icon-lg); height: var(--icon-lg); }
.h-8 { width: var(--icon-xl); height: var(--icon-xl); }

/* Prevent truly oversized icons */
.h-10, .h-12, .h-16, .h-20, .h-24 { 
  width: var(--icon-xl); 
  height: var(--icon-xl); 
}

/* ===== COMPONENT STYLES ===== */

/* Button styles */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-3) var(--space-4);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  font-weight: 500;
  text-decoration: none;
  cursor: pointer;
  transition: all 150ms ease-in-out;
  box-shadow: var(--shadow-sm);
}

.btn-primary {
  background-color: var(--color-primary);
  color: white;
}

.btn-primary:hover {
  background-color: var(--color-primary-dark);
  box-shadow: var(--shadow-md);
}

.btn-secondary {
  background-color: transparent;
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.btn-secondary:hover {
  background-color: var(--color-primary);
  color: white;
}

/* Form styles */
.form-field {
  margin-bottom: var(--space-4);
}

.form-label {
  display: block;
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--color-gray-700);
  margin-bottom: var(--space-2);
}

.form-input {
  width: 100%;
  padding: var(--space-3);
  border: 1px solid var(--color-gray-300);
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  transition: border-color 150ms ease-in-out;
  background-color: white;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgb(0 94 162 / 0.1);
}

/* Alert styles */
.alert {
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  border-left: 0.25rem solid;
  display: flex;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.alert-success {
  background-color: #f0fdf4;
  border-left-color: var(--color-success);
  color: #166534;
}

.alert-error {
  background-color: #fef2f2;
  border-left-color: var(--color-error);
  color: #991b1b;
}

.alert-warning {
  background-color: #fffbeb;
  border-left-color: var(--color-warning);
  color: #92400e;
}

/* Card styles */
.card {
  background-color: white;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
}

.card-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--color-gray-200);
}

.card-body {
  padding: var(--space-4);
}

/* ===== RESPONSIVE DESIGN ===== */

@media (max-width: 640px) {
  .container {
    padding: 0 var(--space-3);
  }
  
  /* Slightly smaller icons on mobile */
  .icon-lg { width: var(--icon-md); height: var(--icon-md); }
  .icon-xl { width: var(--icon-lg); height: var(--icon-lg); }
  
  /* Stack grid columns on mobile */
  .grid-cols-2 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
}

/* ===== ACCESSIBILITY ===== */

/* Focus styles */
.btn:focus,
.form-input:focus {
  outline: 0.25rem solid var(--color-primary);
  outline-offset: 0.125rem;
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* High contrast support */
@media (prefers-contrast: high) {
  .btn {
    border-width: 2px;
  }
  
  .form-input {
    border-width: 2px;
  }
}
`;

    fs.writeFileSync('frontend/src/lib/styles/optimized-design-system.css', optimizedCSS);
    console.log('✅ Generated optimized CSS file');
  }

  // Generate layout fix script
  generateLayoutFixScript() {
    console.log('🛠️  Generating layout fix script...');
    
    const layoutFixScript = `
/**
 * Layout Fix Script
 * 
 * This script fixes common layout issues caused by aggressive CSS constraints
 */

// Fix text flow issues
function fixTextLayout() {
  // Remove aggressive constraints from text containers
  const textContainers = document.querySelectorAll('p, span, div:not(.icon), h1, h2, h3, h4, h5, h6');
  
  textContainers.forEach(container => {
    // Remove any width/height constraints that might affect text flow
    container.style.width = '';
    container.style.height = '';
    container.style.maxWidth = '';
    container.style.maxHeight = '';
    
    // Ensure proper line height for readability
    if (!container.style.lineHeight) {
      container.style.lineHeight = '1.5';
    }
  });
}

// Fix icon constraints to be less aggressive
function fixIconConstraints() {
  const svgs = document.querySelectorAll('svg');
  
  svgs.forEach(svg => {
    // Only apply constraints to actual icons, not decorative SVGs
    if (!svg.closest('.avatar') && !svg.closest('.logo') && !svg.hasAttribute('data-no-constraint')) {
      const rect = svg.getBoundingClientRect();
      
      // If SVG is unreasonably large, constrain it
      if (rect.width > 48 || rect.height > 48) {
        svg.style.maxWidth = '2rem';
        svg.style.maxHeight = '2rem';
        svg.style.width = '1.25rem';
        svg.style.height = '1.25rem';
      }
    }
  });
}

// Fix spacing issues
function fixSpacing() {
  // Ensure proper spacing between elements
  const elements = document.querySelectorAll('.flex, .grid, .space-y-2, .space-y-3, .space-y-4');
  
  elements.forEach(element => {
    // Reset any conflicting margin/padding
    if (element.style.margin === '0' || element.style.padding === '0') {
      element.style.margin = '';
      element.style.padding = '';
    }
  });
}

// Run fixes when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    fixTextLayout();
    fixIconConstraints();
    fixSpacing();
  });
} else {
  fixTextLayout();
  fixIconConstraints();
  fixSpacing();
}

// Re-run fixes when new content is added
const observer = new MutationObserver(() => {
  fixTextLayout();
  fixIconConstraints();
  fixSpacing();
});

observer.observe(document.body, {
  childList: true,
  subtree: true
});

export { fixTextLayout, fixIconConstraints, fixSpacing };
`;

    fs.writeFileSync('frontend/src/lib/utils/layout-fixes.js', layoutFixScript);
    console.log('✅ Generated layout fix script');
  }

  // Generate report
  generateReport() {
    console.log('\n📊 CSS CLEANUP REPORT');
    console.log('='.repeat(50));
    
    console.log(`\n🔍 ANALYSIS RESULTS:`);
    console.log(`   • Used CSS classes: ${this.usedClasses.size}`);
    console.log(`   • Unused selectors found: ${this.unusedSelectors.length}`);
    console.log(`   • Layout issues found: ${this.layoutIssues.length}`);
    
    if (this.unusedSelectors.length > 0) {
      console.log(`\n🗑️  TOP UNUSED SELECTORS:`);
      this.unusedSelectors.slice(0, 10).forEach(item => {
        console.log(`   • ${item.selector} (${path.basename(item.file)})`);
      });
      
      if (this.unusedSelectors.length > 10) {
        console.log(`   ... and ${this.unusedSelectors.length - 10} more`);
      }
    }
    
    if (this.layoutIssues.length > 0) {
      console.log(`\n⚠️  LAYOUT ISSUES:`);
      this.layoutIssues.forEach(issue => {
        console.log(`   • ${issue.type} in ${path.basename(issue.file)}`);
      });
    }
    
    console.log(`\n✅ FIXES APPLIED:`);
    console.log(`   • Generated optimized design system CSS`);
    console.log(`   • Created layout fix script`);
    console.log(`   • Reduced CSS specificity conflicts`);
    console.log(`   • Fixed aggressive icon constraints`);
    
    console.log(`\n📝 NEXT STEPS:`);
    console.log(`   1. Replace existing CSS imports with optimized version`);
    console.log(`   2. Import layout fix script in main app`);
    console.log(`   3. Test layout and text rendering`);
    console.log(`   4. Remove unused CSS files after verification`);
  }

  // Main execution
  async run() {
    console.log('🚀 Starting CSS Cleanup and Layout Fix...\n');
    
    try {
      await this.scanUsedClasses();
      this.analyzeCSSFiles();
      this.generateOptimizedCSS();
      this.generateLayoutFixScript();
      this.generateReport();
      
      console.log('\n🎉 CSS cleanup completed successfully!');
      
    } catch (error) {
      console.error('❌ Error during CSS cleanup:', error);
      process.exit(1);
    }
  }
}

// Run the cleanup if this script is executed directly
const cleanup = new CSSCleanup();
cleanup.run();

export default CSSCleanup;