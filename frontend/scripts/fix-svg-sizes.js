#!/usr/bin/env node

/**
 * Script to automatically fix oversized SVGs in Svelte components
 * Replaces common large SVG classes with design system classes
 */

const fs = require('fs');
const path = require('path');
const glob = require('glob');

// SVG size mappings
const sizeReplacements = {
  'h-4 w-4': 'icon icon--sm',
  'h-5 w-5': 'icon icon--md', 
  'h-6 w-6': 'icon icon--lg',
  'h-8 w-8': 'icon icon--xl',
  'h-10 w-10': 'icon icon--xl',
  'h-12 w-12': 'icon icon--xl'
};

// Color mappings to semantic classes
const colorMappings = {
  'text-green-400': 'icon--success',
  'text-green-500': 'icon--success',
  'text-green-600': 'icon--success',
  'text-red-400': 'icon--error',
  'text-red-500': 'icon--error',
  'text-red-600': 'icon--error',
  'text-yellow-400': 'icon--warning',
  'text-yellow-500': 'icon--warning',
  'text-yellow-600': 'icon--warning',
  'text-blue-400': 'icon--primary',
  'text-blue-500': 'icon--primary',
  'text-blue-600': 'icon--primary',
  'text-indigo-400': 'icon--primary',
  'text-indigo-500': 'icon--primary',
  'text-indigo-600': 'icon--primary',
  'text-gray-400': 'icon--neutral',
  'text-gray-500': 'icon--neutral',
  'text-gray-600': 'icon--neutral'
};

function fixSvgSizes(content) {
  let fixed = content;
  
  // Replace size classes
  Object.entries(sizeReplacements).forEach(([oldClass, newClass]) => {
    const regex = new RegExp(`class="([^"]*?)${oldClass}([^"]*?)"`, 'g');
    fixed = fixed.replace(regex, (match, before, after) => {
      // Remove old size classes and add new icon class
      let newClasses = before.replace(/h-\d+\s*/g, '').replace(/w-\d+\s*/g, '').trim();
      if (newClasses) newClasses += ' ';
      newClasses += newClass;
      if (after.trim()) newClasses += ' ' + after.trim();
      return `class="${newClasses}"`;
    });
  });
  
  // Add aria-hidden to SVGs that don't have it
  fixed = fixed.replace(/<svg([^>]*?)>/g, (match, attributes) => {
    if (!attributes.includes('aria-hidden')) {
      return `<svg${attributes} aria-hidden="true">`;
    }
    return match;
  });
  
  return fixed;
}

function processFile(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    const fixed = fixSvgSizes(content);
    
    if (content !== fixed) {
      fs.writeFileSync(filePath, fixed);
      console.log(`Fixed SVGs in: ${filePath}`);
      return true;
    }
    return false;
  } catch (error) {
    console.error(`Error processing ${filePath}:`, error.message);
    return false;
  }
}

function addDesignSystemImport(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    
    // Check if design system is already imported
    if (content.includes("@import '../styles/design-system.css'")) {
      return false;
    }
    
    // Find the script tag end
    const scriptEndMatch = content.match(/^<\/script>/m);
    if (scriptEndMatch) {
      const insertIndex = scriptEndMatch.index + scriptEndMatch[0].length;
      const beforeScript = content.substring(0, insertIndex);
      const afterScript = content.substring(insertIndex);
      
      const styleSection = `

<style>
  @import '../styles/design-system.css';
</style>`;
      
      const newContent = beforeScript + styleSection + afterScript;
      fs.writeFileSync(filePath, newContent);
      console.log(`Added design system import to: ${filePath}`);
      return true;
    }
    return false;
  } catch (error) {
    console.error(`Error adding import to ${filePath}:`, error.message);
    return false;
  }
}

// Main execution
const componentFiles = glob.sync('src/lib/components/*.svelte');
let fixedFiles = 0;
let importedFiles = 0;

console.log('🔧 Fixing SVG sizes in Svelte components...\n');

componentFiles.forEach(filePath => {
  console.log(`Processing: ${filePath}`);
  
  if (processFile(filePath)) {
    fixedFiles++;
  }
  
  if (addDesignSystemImport(filePath)) {
    importedFiles++;
  }
});

console.log(`\n✅ Processing complete!`);
console.log(`📝 Fixed SVGs in ${fixedFiles} files`);
console.log(`📦 Added design system imports to ${importedFiles} files`);
console.log(`\n🎨 All SVGs should now be properly sized using the design system!`);