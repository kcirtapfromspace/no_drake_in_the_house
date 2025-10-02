#!/usr/bin/env node

/**
 * Remove CSS imports from Svelte components
 * 
 * Since we now have a centralized optimized design system in app.css,
 * we need to remove the individual imports from components.
 */

import fs from 'fs';
import path from 'path';

class CSSImportRemover {
  constructor() {
    this.processedFiles = 0;
    this.totalRemovals = 0;
  }

  // Remove CSS imports from a file
  cleanFile(filePath) {
    console.log(`🧹 Cleaning ${path.basename(filePath)}...`);
    
    let content = fs.readFileSync(filePath, 'utf8');
    let removals = 0;
    
    // Remove @import statements for design-system.css
    const beforeLength = content.length;
    content = content.replace(/@import\s+['"]\.\.\/styles\/design-system\.css['"];?\s*/g, '');
    
    // Remove empty style blocks
    content = content.replace(/<style>\s*<\/style>/g, '');
    content = content.replace(/<style>\s*\/\*\s*Component-specific overrides\s*\*\/\s*<\/style>/g, '');
    
    // Clean up extra whitespace
    content = content.replace(/\n\n\n+/g, '\n\n');
    
    if (content.length !== beforeLength) {
      fs.writeFileSync(filePath, content);
      removals++;
      console.log(`✅ Removed CSS import from ${path.basename(filePath)}`);
    } else {
      console.log(`⏭️  No changes needed for ${path.basename(filePath)}`);
    }
    
    this.totalRemovals += removals;
    this.processedFiles++;
  }

  // Main execution
  async run() {
    console.log('🚀 Removing CSS imports from Svelte components...\n');
    
    const filesToClean = [
      'frontend/src/lib/components/ActionHistory.svelte',
      'frontend/src/lib/components/RegisterForm.svelte',
      'frontend/src/lib/components/LoginForm.svelte',
      'frontend/src/lib/components/ServiceConnections.svelte',
      'frontend/src/lib/components/Navigation.svelte',
      'frontend/src/lib/components/Login.svelte'
    ];
    
    try {
      for (const file of filesToClean) {
        if (fs.existsSync(file)) {
          this.cleanFile(file);
        }
      }
      
      console.log('\n📊 CLEANUP SUMMARY');
      console.log('='.repeat(50));
      console.log(`✅ Files processed: ${this.processedFiles}`);
      console.log(`🗑️  CSS imports removed: ${this.totalRemovals}`);
      
      if (this.totalRemovals > 0) {
        console.log(`\n🎉 Successfully cleaned up CSS imports!`);
        console.log(`\n📝 RESULT:`);
        console.log(`• All components now use the centralized design system`);
        console.log(`• No more duplicate CSS imports`);
        console.log(`• Cleaner component files`);
        console.log(`• Better performance with single CSS file`);
      }
      
    } catch (error) {
      console.error('❌ Error during cleanup:', error);
      process.exit(1);
    }
  }
}

// Run the cleanup
const remover = new CSSImportRemover();
remover.run();