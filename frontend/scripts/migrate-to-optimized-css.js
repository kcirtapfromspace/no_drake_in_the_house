#!/usr/bin/env node

/**
 * Migration Script for Optimized CSS Classes
 * 
 * This script migrates existing components to use the new optimized design system classes
 * and removes problematic CSS patterns that cause layout issues.
 */

import fs from 'fs';
import path from 'path';

class CSSMigration {
  constructor() {
    this.migrations = [
      // Icon size migrations
      { from: /class="([^"]*)\bh-4\b([^"]*)"/, to: 'class="$1icon icon-sm$2"' },
      { from: /class="([^"]*)\bh-5\b([^"]*)"/, to: 'class="$1icon icon-md$2"' },
      { from: /class="([^"]*)\bh-6\b([^"]*)"/, to: 'class="$1icon icon-lg$2"' },
      { from: /class="([^"]*)\bh-8\b([^"]*)"/, to: 'class="$1icon icon-xl$2"' },
      { from: /class="([^"]*)\bw-4\b([^"]*)"/, to: 'class="$1$2"' }, // Remove w-4 as icon class handles both
      { from: /class="([^"]*)\bw-5\b([^"]*)"/, to: 'class="$1$2"' }, // Remove w-5 as icon class handles both
      { from: /class="([^"]*)\bw-6\b([^"]*)"/, to: 'class="$1$2"' }, // Remove w-6 as icon class handles both
      { from: /class="([^"]*)\bw-8\b([^"]*)"/, to: 'class="$1$2"' }, // Remove w-8 as icon class handles both
      
      // Oversized icon prevention
      { from: /class="([^"]*)\bh-10\b([^"]*)"/, to: 'class="$1icon icon-xl$2"' },
      { from: /class="([^"]*)\bh-12\b([^"]*)"/, to: 'class="$1icon icon-xl$2"' },
      { from: /class="([^"]*)\bh-16\b([^"]*)"/, to: 'class="$1icon icon-xl$2"' },
      { from: /class="([^"]*)\bh-20\b([^"]*)"/, to: 'class="$1icon icon-xl$2"' },
      { from: /class="([^"]*)\bh-24\b([^"]*)"/, to: 'class="$1icon icon-xl$2"' },
      
      // Color migrations to design system
      { from: /class="([^"]*)\btext-indigo-600\b([^"]*)"/, to: 'class="$1text-primary$2"' },
      { from: /class="([^"]*)\btext-red-600\b([^"]*)"/, to: 'class="$1text-error$2"' },
      { from: /class="([^"]*)\btext-green-600\b([^"]*)"/, to: 'class="$1text-success$2"' },
      { from: /class="([^"]*)\btext-yellow-600\b([^"]*)"/, to: 'class="$1text-warning$2"' },
      
      // Background color migrations
      { from: /class="([^"]*)\bbg-indigo-600\b([^"]*)"/, to: 'class="$1bg-primary$2"' },
      { from: /class="([^"]*)\bbg-red-600\b([^"]*)"/, to: 'class="$1bg-error$2"' },
      { from: /class="([^"]*)\bbg-green-600\b([^"]*)"/, to: 'class="$1bg-success$2"' },
      
      // Button class migrations
      { from: /class="([^"]*)\bbg-indigo-600 hover:bg-indigo-700\b([^"]*)"/, to: 'class="$1btn btn-primary$2"' },
      { from: /class="([^"]*)\bborder border-indigo-600 text-indigo-600 hover:bg-indigo-600 hover:text-white\b([^"]*)"/, to: 'class="$1btn btn-secondary$2"' },
      
      // Spacing migrations to design system
      { from: /class="([^"]*)\bp-2\b([^"]*)"/, to: 'class="$1p-2$2"' }, // Keep as is, matches design system
      { from: /class="([^"]*)\bp-3\b([^"]*)"/, to: 'class="$1p-3$2"' }, // Keep as is, matches design system
      { from: /class="([^"]*)\bp-4\b([^"]*)"/, to: 'class="$1p-4$2"' }, // Keep as is, matches design system
      
      // Clean up redundant classes
      { from: /class="([^"]*)\bflex-shrink-0\b([^"]*)"/, to: 'class="$1$2"' }, // Icon class handles this
      { from: /class="([^"]*)\binline-flex\b([^"]*)"/, to: 'class="$1flex$2"' }, // Simplify to flex
    ];
    
    this.processedFiles = 0;
    this.totalChanges = 0;
  }

  // Find all Svelte files to migrate
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

  // Migrate a single file
  migrateFile(filePath) {
    console.log(`🔄 Migrating ${filePath}...`);
    
    let content = fs.readFileSync(filePath, 'utf8');
    let changes = 0;
    
    // Apply all migrations
    for (const migration of this.migrations) {
      const before = content;
      content = content.replace(migration.from, migration.to);
      if (content !== before) {
        changes++;
      }
    }
    
    // Special handling for SVG elements - add icon class if missing
    const svgMatches = content.match(/<svg[^>]*class="([^"]*)"[^>]*>/g) || [];
    for (const match of svgMatches) {
      if (!match.includes('icon') && !match.includes('avatar') && !match.includes('logo')) {
        const newMatch = match.replace(/class="([^"]*)"/, 'class="icon icon-md $1"');
        content = content.replace(match, newMatch);
        changes++;
      }
    }
    
    // Clean up multiple spaces in class attributes
    content = content.replace(/class="([^"]*)"/, (match, classes) => {
      const cleanClasses = classes.split(/\s+/).filter(c => c.trim()).join(' ');
      return `class="${cleanClasses}"`;
    });
    
    if (changes > 0) {
      fs.writeFileSync(filePath, content);
      console.log(`✅ Applied ${changes} changes to ${path.basename(filePath)}`);
      this.totalChanges += changes;
    } else {
      console.log(`⏭️  No changes needed for ${path.basename(filePath)}`);
    }
    
    this.processedFiles++;
  }

  // Main migration process
  async run() {
    console.log('🚀 Starting CSS class migration to optimized design system...\n');
    
    try {
      const svelteFiles = await this.findSvelteFiles();
      console.log(`📁 Found ${svelteFiles.length} Svelte files to process\n`);
      
      for (const file of svelteFiles) {
        this.migrateFile(file);
      }
      
      console.log('\n📊 MIGRATION SUMMARY');
      console.log('='.repeat(50));
      console.log(`✅ Files processed: ${this.processedFiles}`);
      console.log(`🔄 Total changes applied: ${this.totalChanges}`);
      console.log(`\n🎉 Migration completed successfully!`);
      
      if (this.totalChanges > 0) {
        console.log(`\n📝 NEXT STEPS:`);
        console.log(`1. Test the application to ensure layout is fixed`);
        console.log(`2. Check for any remaining layout issues`);
        console.log(`3. Remove old CSS files if everything works correctly`);
        console.log(`4. Run the development server to verify changes`);
      }
      
    } catch (error) {
      console.error('❌ Error during migration:', error);
      process.exit(1);
    }
  }
}

// Run the migration
const migration = new CSSMigration();
migration.run();