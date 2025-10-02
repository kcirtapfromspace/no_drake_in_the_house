#!/usr/bin/env node

/**
 * Remove Unused CSS from Svelte Components
 * 
 * This script removes the duplicate design system CSS that was copied
 * into individual Svelte components, since we now have a centralized
 * optimized design system.
 */

import fs from 'fs';
import path from 'path';

class UnusedCSSRemover {
  constructor() {
    this.processedFiles = 0;
    this.totalRemovals = 0;
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

  // Remove duplicate CSS from a Svelte file
  cleanSvelteFile(filePath) {
    console.log(`🧹 Cleaning ${path.basename(filePath)}...`);
    
    let content = fs.readFileSync(filePath, 'utf8');
    let removals = 0;
    
    // Check if file has a <style> section with design system CSS
    const styleMatch = content.match(/<style[^>]*>([\s\S]*?)<\/style>/);
    
    if (!styleMatch) {
      console.log(`⏭️  No styles found in ${path.basename(filePath)}`);
      return;
    }
    
    const styleContent = styleMatch[1];
    
    // Check if this contains design system CSS (look for design tokens)
    if (styleContent.includes('--icon-xs:') || 
        styleContent.includes('--space-1:') || 
        styleContent.includes('.icon {') ||
        styleContent.includes('.avatar {') ||
        styleContent.includes('.form-field {')) {
      
      console.log(`🗑️  Removing duplicate design system CSS from ${path.basename(filePath)}`);
      
      // Remove the entire style block since we now use the centralized CSS
      content = content.replace(/<style[^>]*>[\s\S]*?<\/style>/, '');
      removals++;
      
      // Clean up any extra whitespace
      content = content.replace(/\n\n\n+/g, '\n\n');
      
      fs.writeFileSync(filePath, content);
      this.totalRemovals += removals;
    } else {
      console.log(`✅ ${path.basename(filePath)} has component-specific styles, keeping them`);
    }
    
    this.processedFiles++;
  }

  // Main execution
  async run() {
    console.log('🚀 Starting unused CSS removal from Svelte components...\n');
    
    try {
      const svelteFiles = await this.findSvelteFiles();
      console.log(`📁 Found ${svelteFiles.length} Svelte files to process\n`);
      
      for (const file of svelteFiles) {
        this.cleanSvelteFile(file);
      }
      
      console.log('\n📊 CLEANUP SUMMARY');
      console.log('='.repeat(50));
      console.log(`✅ Files processed: ${this.processedFiles}`);
      console.log(`🗑️  CSS blocks removed: ${this.totalRemovals}`);
      
      if (this.totalRemovals > 0) {
        console.log(`\n🎉 Successfully removed duplicate CSS from ${this.totalRemovals} components!`);
        console.log(`\n📝 NEXT STEPS:`);
        console.log(`1. Test the application to ensure styles still work`);
        console.log(`2. All styling now comes from the optimized design system`);
        console.log(`3. Components are now much cleaner and easier to maintain`);
      } else {
        console.log(`\n✨ No duplicate CSS found - components are already clean!`);
      }
      
    } catch (error) {
      console.error('❌ Error during CSS cleanup:', error);
      process.exit(1);
    }
  }
}

// Run the cleanup
const remover = new UnusedCSSRemover();
remover.run();