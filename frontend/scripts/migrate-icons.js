#!/usr/bin/env node

/**
 * Icon Migration Script
 * 
 * This script scans Svelte components for oversized icon patterns and
 * automatically migrates them to use the design system classes.
 * 
 * Usage:
 *   node scripts/migrate-icons.js [--dry-run] [--path=src/lib/components]
 */

import fs from 'fs';
import path from 'path';

// Simple glob implementation for .svelte files
function findSvelteFiles(dir) {
  const files = [];
  
  function walk(currentPath) {
    const items = fs.readdirSync(currentPath);
    
    for (const item of items) {
      const fullPath = path.join(currentPath, item);
      const stat = fs.statSync(fullPath);
      
      if (stat.isDirectory() && !item.startsWith('.') && item !== 'node_modules') {
        walk(fullPath);
      } else if (stat.isFile() && item.endsWith('.svelte')) {
        files.push(fullPath);
      }
    }
  }
  
  walk(dir);
  return files;
}

// Migration patterns - from Tailwind classes to design system classes
const MIGRATION_PATTERNS = [
  // Oversized icons that should be capped at xl
  { 
    from: /class="([^"]*?)h-(?:10|12|16|20|24) w-(?:10|12|16|20|24)([^"]*?)"/g, 
    to: 'class="$1icon icon--xl$2"',
    description: 'Replace oversized Tailwind classes with icon--xl'
  },
  
  // Standard size mappings
  { 
    from: /class="([^"]*?)h-8 w-8([^"]*?)"/g, 
    to: 'class="$1icon icon--lg$2"',
    description: 'Replace h-8 w-8 with icon--lg'
  },
  { 
    from: /class="([^"]*?)h-6 w-6([^"]*?)"/g, 
    to: 'class="$1icon icon--lg$2"',
    description: 'Replace h-6 w-6 with icon--lg'
  },
  { 
    from: /class="([^"]*?)h-5 w-5([^"]*?)"/g, 
    to: 'class="$1icon icon--md$2"',
    description: 'Replace h-5 w-5 with icon--md'
  },
  { 
    from: /class="([^"]*?)h-4 w-4([^"]*?)"/g, 
    to: 'class="$1icon icon--sm$2"',
    description: 'Replace h-4 w-4 with icon--sm'
  },
  
  // Add aria-hidden to decorative SVGs
  {
    from: /<svg(?![^>]*aria-label)(?![^>]*aria-labelledby)(?![^>]*aria-hidden)([^>]*)>/g,
    to: '<svg aria-hidden="true"$1>',
    description: 'Add aria-hidden to decorative SVGs'
  },
  
  // Fix loading spinners
  {
    from: /class="([^"]*?)animate-spin([^"]*?)h-\d+ w-\d+([^"]*?)"/g,
    to: 'class="$1animate-spin$2icon icon--loading$3"',
    description: 'Fix loading spinner sizing'
  }
];

// Avatar patterns - these should NOT be converted to icon classes
const AVATAR_PATTERNS = [
  /class="[^"]*h-(?:8|10|12|16|20|24) w-(?:8|10|12|16|20|24)[^"]*rounded-full[^"]*"/g,
  /class="[^"]*rounded-full[^"]*h-(?:8|10|12|16|20|24) w-(?:8|10|12|16|20|24)[^"]*"/g
];

function isAvatarPattern(line) {
  return AVATAR_PATTERNS.some(pattern => pattern.test(line));
}

function migrateFile(filePath, dryRun = false) {
  const content = fs.readFileSync(filePath, 'utf8');
  let migratedContent = content;
  let changesMade = 0;
  const changes = [];

  // Apply migration patterns
  MIGRATION_PATTERNS.forEach(({ from, to, description }) => {
    const lines = migratedContent.split('\n');
    
    lines.forEach((line, index) => {
      // Skip avatar patterns
      if (isAvatarPattern(line)) {
        return;
      }

      const matches = line.match(from);
      if (matches) {
        const newLine = line.replace(from, to);
        if (newLine !== line) {
          changes.push({
            line: index + 1,
            description,
            before: line.trim(),
            after: newLine.trim()
          });
          changesMade++;
        }
      }
    });

    migratedContent = migratedContent.replace(from, (match, ...args) => {
      // Skip if this looks like an avatar pattern
      if (isAvatarPattern(match)) {
        return match;
      }
      return to.replace(/\$(\d+)/g, (_, num) => args[parseInt(num) - 1] || '');
    });
  });

  if (changesMade > 0) {
    console.log(`\n📁 ${filePath}`);
    console.log(`   ${changesMade} changes found`);
    
    if (process.argv.includes('--verbose')) {
      changes.forEach(change => {
        console.log(`   Line ${change.line}: ${change.description}`);
        console.log(`     - ${change.before}`);
        console.log(`     + ${change.after}`);
      });
    }

    if (!dryRun) {
      fs.writeFileSync(filePath, migratedContent);
      console.log(`   ✅ Changes applied`);
    } else {
      console.log(`   🔍 Dry run - no changes applied`);
    }
  }

  return { changesMade, changes };
}

function scanDirectory(directory) {
  const files = findSvelteFiles(directory);
  
  console.log(`🔍 Scanning ${files.length} Svelte files in ${directory}...`);
  
  let totalChanges = 0;
  let filesChanged = 0;
  const dryRun = process.argv.includes('--dry-run');

  files.forEach(file => {
    const result = migrateFile(file, dryRun);
    if (result.changesMade > 0) {
      totalChanges += result.changesMade;
      filesChanged++;
    }
  });

  console.log(`\n📊 Migration Summary:`);
  console.log(`   Files scanned: ${files.length}`);
  console.log(`   Files with changes: ${filesChanged}`);
  console.log(`   Total changes: ${totalChanges}`);
  
  if (dryRun) {
    console.log(`\n💡 This was a dry run. To apply changes, run without --dry-run flag.`);
  } else if (totalChanges > 0) {
    console.log(`\n✅ Migration complete! Remember to:`);
    console.log(`   1. Test your components to ensure they look correct`);
    console.log(`   2. Import the design system CSS in your components`);
    console.log(`   3. Run the icon validation utility in development mode`);
  } else {
    console.log(`\n🎉 No migration needed - all icons are already properly sized!`);
  }
}

function main() {
  const args = process.argv.slice(2);
  const pathArg = args.find(arg => arg.startsWith('--path='));
  const targetPath = pathArg ? pathArg.split('=')[1] : 'src/lib/components';

  console.log('🚀 Icon Migration Tool');
  console.log('====================');
  
  if (!fs.existsSync(targetPath)) {
    console.error(`❌ Path does not exist: ${targetPath}`);
    process.exit(1);
  }

  scanDirectory(targetPath);
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { migrateFile, scanDirectory };