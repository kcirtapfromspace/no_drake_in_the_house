#!/usr/bin/env node

/**
 * USWDS + Skeleton UI Migration Script
 * 
 * This script migrates existing components from the old design system
 * to the new USWDS + Skeleton UI design system.
 * 
 * Usage:
 *   node scripts/migrate-to-uswds.js [--dry-run] [--path=src/lib/components]
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

// Migration patterns - from old design system to USWDS + Skeleton UI
const MIGRATION_PATTERNS = [
  // Icon class migrations
  { 
    from: /class="([^"]*?)icon icon--xs([^"]*?)"/g, 
    to: 'class="$1icon-uswds icon-uswds--xs$2"',
    description: 'Migrate icon--xs to icon-uswds--xs'
  },
  { 
    from: /class="([^"]*?)icon icon--sm([^"]*?)"/g, 
    to: 'class="$1icon-uswds icon-uswds--sm$2"',
    description: 'Migrate icon--sm to icon-uswds--sm'
  },
  { 
    from: /class="([^"]*?)icon icon--md([^"]*?)"/g, 
    to: 'class="$1icon-uswds icon-uswds--md$2"',
    description: 'Migrate icon--md to icon-uswds--md'
  },
  { 
    from: /class="([^"]*?)icon icon--lg([^"]*?)"/g, 
    to: 'class="$1icon-uswds icon-uswds--lg$2"',
    description: 'Migrate icon--lg to icon-uswds--lg'
  },
  { 
    from: /class="([^"]*?)icon icon--xl([^"]*?)"/g, 
    to: 'class="$1icon-uswds icon-uswds--xl$2"',
    description: 'Migrate icon--xl to icon-uswds--xl'
  },
  
  // Semantic icon color migrations
  { 
    from: /class="([^"]*?)icon--success([^"]*?)"/g, 
    to: 'class="$1icon-uswds--success$2"',
    description: 'Migrate icon--success to icon-uswds--success'
  },
  { 
    from: /class="([^"]*?)icon--error([^"]*?)"/g, 
    to: 'class="$1icon-uswds--error$2"',
    description: 'Migrate icon--error to icon-uswds--error'
  },
  { 
    from: /class="([^"]*?)icon--warning([^"]*?)"/g, 
    to: 'class="$1icon-uswds--warning$2"',
    description: 'Migrate icon--warning to icon-uswds--warning'
  },
  { 
    from: /class="([^"]*?)icon--neutral([^"]*?)"/g, 
    to: 'class="$1icon-uswds--neutral$2"',
    description: 'Migrate icon--neutral to icon-uswds--neutral'
  },
  { 
    from: /class="([^"]*?)icon--primary([^"]*?)"/g, 
    to: 'class="$1icon-uswds--primary$2"',
    description: 'Migrate icon--primary to icon-uswds--primary'
  },
  
  // Button migrations
  { 
    from: /class="([^"]*?)btn btn--primary([^"]*?)"/g, 
    to: 'class="$1btn-uswds btn-uswds-primary$2"',
    description: 'Migrate btn--primary to btn-uswds-primary'
  },
  { 
    from: /class="([^"]*?)btn btn--secondary([^"]*?)"/g, 
    to: 'class="$1btn-uswds btn-uswds-secondary$2"',
    description: 'Migrate btn--secondary to btn-uswds-secondary'
  },
  
  // Form element migrations
  { 
    from: /class="([^"]*?)form-input([^"]*?)"/g, 
    to: 'class="$1form-input-uswds$2"',
    description: 'Migrate form-input to form-input-uswds'
  },
  { 
    from: /class="([^"]*?)form-label([^"]*?)"/g, 
    to: 'class="$1form-label-uswds$2"',
    description: 'Migrate form-label to form-label-uswds'
  },
  { 
    from: /class="([^"]*?)form-field([^"]*?)"/g, 
    to: 'class="$1form-field-uswds$2"',
    description: 'Migrate form-field to form-field-uswds'
  },
  
  // Alert migrations
  { 
    from: /class="([^"]*?)alert alert--success([^"]*?)"/g, 
    to: 'class="$1alert-uswds alert-uswds-success$2"',
    description: 'Migrate alert--success to alert-uswds-success'
  },
  { 
    from: /class="([^"]*?)alert alert--error([^"]*?)"/g, 
    to: 'class="$1alert-uswds alert-uswds-error$2"',
    description: 'Migrate alert--error to alert-uswds-error'
  },
  { 
    from: /class="([^"]*?)alert alert--warning([^"]*?)"/g, 
    to: 'class="$1alert-uswds alert-uswds-warning$2"',
    description: 'Migrate alert--warning to alert-uswds-warning'
  },
  
  // Color class migrations to USWDS colors
  { 
    from: /class="([^"]*?)text-gray-([0-9]+)([^"]*?)"/g, 
    to: 'class="$1text-uswds-base-darker$3"',
    description: 'Migrate gray text colors to USWDS base colors'
  },
  { 
    from: /class="([^"]*?)text-blue-([0-9]+)([^"]*?)"/g, 
    to: 'class="$1text-uswds-blue-50$3"',
    description: 'Migrate blue text colors to USWDS blue'
  },
  { 
    from: /class="([^"]*?)text-green-([0-9]+)([^"]*?)"/g, 
    to: 'class="$1text-uswds-green-50$3"',
    description: 'Migrate green text colors to USWDS green'
  },
  { 
    from: /class="([^"]*?)text-red-([0-9]+)([^"]*?)"/g, 
    to: 'class="$1text-uswds-red-50$3"',
    description: 'Migrate red text colors to USWDS red'
  },
  
  // Background color migrations
  { 
    from: /class="([^"]*?)bg-gray-([0-9]+)([^"]*?)"/g, 
    to: 'class="$1bg-uswds-base-lightest$3"',
    description: 'Migrate gray backgrounds to USWDS base colors'
  },
  { 
    from: /class="([^"]*?)bg-blue-([0-9]+)([^"]*?)"/g, 
    to: 'class="$1bg-uswds-blue-50$3"',
    description: 'Migrate blue backgrounds to USWDS blue'
  },
  
  // Typography migrations
  { 
    from: /class="([^"]*?)text-xs([^"]*?)"/g, 
    to: 'class="$1text-uswds-xs$2"',
    description: 'Migrate text-xs to text-uswds-xs'
  },
  { 
    from: /class="([^"]*?)text-sm([^"]*?)"/g, 
    to: 'class="$1text-uswds-sm$2"',
    description: 'Migrate text-sm to text-uswds-sm'
  },
  { 
    from: /class="([^"]*?)text-base([^"]*?)"/g, 
    to: 'class="$1text-uswds-base$2"',
    description: 'Migrate text-base to text-uswds-base'
  },
  { 
    from: /class="([^"]*?)text-lg([^"]*?)"/g, 
    to: 'class="$1text-uswds-lg$2"',
    description: 'Migrate text-lg to text-uswds-lg'
  },
  { 
    from: /class="([^"]*?)text-xl([^"]*?)"/g, 
    to: 'class="$1text-uswds-xl$2"',
    description: 'Migrate text-xl to text-uswds-xl'
  },
  { 
    from: /class="([^"]*?)text-2xl([^"]*?)"/g, 
    to: 'class="$1text-uswds-2xl$2"',
    description: 'Migrate text-2xl to text-uswds-2xl'
  },
  { 
    from: /class="([^"]*?)text-3xl([^"]*?)"/g, 
    to: 'class="$1text-uswds-3xl$2"',
    description: 'Migrate text-3xl to text-uswds-3xl'
  },
  
  // Spacing migrations to USWDS spacing
  { 
    from: /class="([^"]*?)p-([1-8])([^"]*?)"/g, 
    to: 'class="$1p-uswds-$2$3"',
    description: 'Migrate padding to USWDS spacing'
  },
  { 
    from: /class="([^"]*?)m-([1-8])([^"]*?)"/g, 
    to: 'class="$1m-uswds-$2$3"',
    description: 'Migrate margin to USWDS spacing'
  },
  
  // Border radius migrations
  { 
    from: /class="([^"]*?)rounded-md([^"]*?)"/g, 
    to: 'class="$1rounded-uswds-md$2"',
    description: 'Migrate border radius to USWDS'
  },
  { 
    from: /class="([^"]*?)rounded-lg([^"]*?)"/g, 
    to: 'class="$1rounded-uswds-lg$2"',
    description: 'Migrate border radius to USWDS'
  },
];

function migrateFile(filePath, dryRun = false) {
  const content = fs.readFileSync(filePath, 'utf8');
  let migratedContent = content;
  let changesMade = 0;
  const changes = [];

  // Apply migration patterns
  MIGRATION_PATTERNS.forEach(({ from, to, description }) => {
    const matches = migratedContent.match(from);
    if (matches) {
      const newContent = migratedContent.replace(from, to);
      if (newContent !== migratedContent) {
        changes.push({
          description,
          matchCount: matches.length
        });
        changesMade += matches.length;
        migratedContent = newContent;
      }
    }
  });

  if (changesMade > 0) {
    console.log(`\n📁 ${filePath}`);
    console.log(`   ${changesMade} changes found`);
    
    if (process.argv.includes('--verbose')) {
      changes.forEach(change => {
        console.log(`   - ${change.description} (${change.matchCount} instances)`);
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
  
  console.log(`🚀 USWDS + Skeleton UI Migration Tool`);
  console.log(`=====================================`);
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
    console.log(`   1. Import the new USWDS theme CSS in your components`);
    console.log(`   2. Test your components to ensure they look correct`);
    console.log(`   3. Update any custom styles to use USWDS design tokens`);
    console.log(`   4. Run the validation utility to check for any remaining issues`);
  } else {
    console.log(`\n🎉 No migration needed - all components are already using USWDS classes!`);
  }
}

function main() {
  const args = process.argv.slice(2);
  const pathArg = args.find(arg => arg.startsWith('--path='));
  const targetPath = pathArg ? pathArg.split('=')[1] : 'src/lib/components';

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