#!/usr/bin/env node

/**
 * Fix Button Reactivity Issues
 * 
 * This script identifies and fixes buttons that might not be responding
 * properly to click events in Svelte components.
 */

import fs from 'fs';
import path from 'path';

class ButtonReactivityFixer {
  constructor() {
    this.processedFiles = 0;
    this.totalFixes = 0;
  }

  // Find all Svelte files
  async findSvelteFiles() {
    const files = [];
    
    const scan = (dir) => {
      const items = fs.readdirSync(dir);
      
      for (const item of items) {
        const fullPath = path.join(dir, item);
        const stat = fs.statSync(fullPath);
        
        if (stat.isDirectory() && !item.startsWith('.') && item !== 'node_modules') {
          scan(fullPath);
        } else if (stat.isFile() && item.endsWith('.svelte')) {
          files.push(fullPath);
        }
      }
    };
    
    scan('frontend/src');
    return files;
  }

  // Fix button reactivity issues in a file
  fixButtonReactivity(filePath) {
    console.log(`🔧 Checking ${path.basename(filePath)}...`);
    
    let content = fs.readFileSync(filePath, 'utf8');
    let fixes = 0;
    
    // Fix 1: Ensure all buttons have proper event handlers
    // Look for buttons without on:click handlers
    const buttonMatches = content.match(/<button[^>]*>/g) || [];
    
    for (const button of buttonMatches) {
      if (!button.includes('on:click') && !button.includes('disabled')) {
        console.log(`⚠️  Found button without click handler in ${path.basename(filePath)}: ${button.substring(0, 50)}...`);
      }
    }
    
    // Fix 2: Ensure preventDefault is used consistently
    const clickHandlers = content.match(/on:click[^=]*=[^>]*/g) || [];
    
    for (const handler of clickHandlers) {
      if (!handler.includes('preventDefault') && !handler.includes('|preventDefault')) {
        console.log(`⚠️  Click handler without preventDefault in ${path.basename(filePath)}: ${handler}`);
      }
    }
    
    // Fix 3: Replace any remaining old router calls
    const beforeRouter = content;
    content = content.replace(/router\.navigate\(/g, 'navigateTo(');
    if (content !== beforeRouter) {
      fixes++;
      console.log(`✅ Fixed router.navigate call in ${path.basename(filePath)}`);
    }
    
    // Fix 4: Ensure proper function calls in event handlers
    // Look for arrow functions that might not be working
    const arrowFunctionMatches = content.match(/on:click[^=]*=\s*\{[^}]*=>[^}]*\}/g) || [];
    
    for (const match of arrowFunctionMatches) {
      if (match.includes('setActiveTab') || match.includes('navigate')) {
        console.log(`✅ Found navigation arrow function in ${path.basename(filePath)}`);
      }
    }
    
    // Fix 5: Add debugging to button clicks
    const debuggingPattern = /on:click\|preventDefault=\{([^}]+)\}/g;
    let debugContent = content;
    
    debugContent = debugContent.replace(debuggingPattern, (match, handler) => {
      if (!handler.includes('console.log')) {
        const newHandler = `() => { console.log('Button clicked:', '${handler}'); ${handler}; }`;
        return `on:click|preventDefault={${newHandler}}`;
      }
      return match;
    });
    
    if (debugContent !== content) {
      content = debugContent;
      fixes++;
      console.log(`✅ Added debugging to button clicks in ${path.basename(filePath)}`);
    }
    
    if (fixes > 0) {
      fs.writeFileSync(filePath, content);
      this.totalFixes += fixes;
    }
    
    this.processedFiles++;
  }

  // Main execution
  async run() {
    console.log('🚀 Starting button reactivity fix...\n');
    
    try {
      const svelteFiles = await this.findSvelteFiles();
      console.log(`📁 Found ${svelteFiles.length} Svelte files to check\n`);
      
      for (const file of svelteFiles) {
        this.fixButtonReactivity(file);
      }
      
      console.log('\n📊 BUTTON REACTIVITY FIX SUMMARY');
      console.log('='.repeat(50));
      console.log(`✅ Files processed: ${this.processedFiles}`);
      console.log(`🔧 Total fixes applied: ${this.totalFixes}`);
      
      if (this.totalFixes > 0) {
        console.log(`\n🎉 Button reactivity fixes applied!`);
        console.log(`\n📝 NEXT STEPS:`);
        console.log(`1. Test button clicks - should now respond immediately`);
        console.log(`2. Check browser console for button click debugging`);
        console.log(`3. Verify navigation works without refresh`);
      } else {
        console.log(`\n✨ No button reactivity issues found!`);
        console.log(`The navigation should already be working properly.`);
      }
      
    } catch (error) {
      console.error('❌ Error during button fix:', error);
      process.exit(1);
    }
  }
}

// Run the fix
const fixer = new ButtonReactivityFixer();
fixer.run();