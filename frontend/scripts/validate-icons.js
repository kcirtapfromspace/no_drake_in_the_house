#!/usr/bin/env node

/**
 * Icon Validation Script
 * 
 * This script validates that all icons in Svelte components follow
 * the design system guidelines and reports any violations.
 * 
 * Usage:
 *   node scripts/validate-icons.js [--strict] [--path=src/lib/components]
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

// Validation rules
const VALIDATION_RULES = [
  {
    name: 'No oversized Tailwind classes',
    pattern: /class="[^"]*h-(?:10|12|16|20|24|32) w-(?:10|12|16|20|24|32)[^"]*"/g,
    severity: 'error',
    message: 'Found oversized Tailwind classes. Use design system icon classes instead.',
    suggestion: 'Replace with: icon icon--sm, icon--md, icon--lg, or icon--xl'
  },
  
  {
    name: 'SVGs should have design system classes',
    pattern: /<svg(?![^>]*class="[^"]*icon[^"]*")[^>]*>/g,
    severity: 'warning',
    message: 'SVG without design system icon class.',
    suggestion: 'Add: class="icon icon--sm" (or appropriate size)'
  },
  
  {
    name: 'Decorative SVGs should have aria-hidden',
    pattern: /<svg(?![^>]*aria-label)(?![^>]*aria-labelledby)(?![^>]*aria-hidden)[^>]*>/g,
    severity: 'warning',
    message: 'Decorative SVG missing aria-hidden attribute.',
    suggestion: 'Add: aria-hidden="true"'
  },
  
  {
    name: 'No inline SVG sizing',
    pattern: /<svg[^>]*(?:width|height)=["']\d+["'][^>]*>/g,
    severity: 'warning',
    message: 'SVG with inline width/height attributes.',
    suggestion: 'Remove inline sizing and use CSS classes instead'
  }
];

// Patterns that should be ignored (avatars, images, etc.)
const IGNORE_PATTERNS = [
  /class="[^"]*rounded-full[^"]*"/,  // Avatar containers
  /class="[^"]*avatar[^"]*"/,        // Avatar classes
  /<img[^>]*>/,                      // Image elements
  /<!-- ignore-icon-validation -->/   // Explicit ignore comment
];

function shouldIgnoreLine(line) {
  return IGNORE_PATTERNS.some(pattern => pattern.test(line));
}

function validateFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  const lines = content.split('\n');
  const violations = [];

  lines.forEach((line, index) => {
    // Skip ignored patterns
    if (shouldIgnoreLine(line)) {
      return;
    }

    VALIDATION_RULES.forEach(rule => {
      const matches = line.match(rule.pattern);
      if (matches) {
        matches.forEach(match => {
          violations.push({
            rule: rule.name,
            severity: rule.severity,
            line: index + 1,
            content: line.trim(),
            match: match,
            message: rule.message,
            suggestion: rule.suggestion
          });
        });
      }
    });
  });

  return violations;
}

function validateDirectory(directory, strict = false) {
  const files = findSvelteFiles(directory);
  
  console.log(`🔍 Validating ${files.length} Svelte files in ${directory}...`);
  
  let totalViolations = 0;
  let errorCount = 0;
  let warningCount = 0;
  let filesWithIssues = 0;

  files.forEach(file => {
    const violations = validateFile(file);
    
    if (violations.length > 0) {
      filesWithIssues++;
      console.log(`\n📁 ${file}`);
      
      violations.forEach(violation => {
        totalViolations++;
        
        const icon = violation.severity === 'error' ? '❌' : '⚠️';
        console.log(`   ${icon} Line ${violation.line}: ${violation.message}`);
        console.log(`      ${violation.content}`);
        console.log(`      💡 ${violation.suggestion}`);
        
        if (violation.severity === 'error') {
          errorCount++;
        } else {
          warningCount++;
        }
      });
    }
  });

  console.log(`\n📊 Validation Summary:`);
  console.log(`   Files scanned: ${files.length}`);
  console.log(`   Files with issues: ${filesWithIssues}`);
  console.log(`   Total violations: ${totalViolations}`);
  console.log(`   Errors: ${errorCount}`);
  console.log(`   Warnings: ${warningCount}`);

  if (totalViolations === 0) {
    console.log(`\n🎉 All icons are properly configured!`);
    return 0;
  } else {
    console.log(`\n💡 To fix these issues:`);
    console.log(`   1. Run: node scripts/migrate-icons.js --dry-run`);
    console.log(`   2. Review the suggested changes`);
    console.log(`   3. Run: node scripts/migrate-icons.js (to apply changes)`);
    console.log(`   4. Import design system CSS in your components`);
    
    if (strict && errorCount > 0) {
      console.log(`\n❌ Validation failed in strict mode due to ${errorCount} errors.`);
      return 1;
    } else if (errorCount > 0) {
      console.log(`\n⚠️  Found ${errorCount} errors. Consider running in --strict mode for CI/CD.`);
      return 0;
    } else {
      console.log(`\n✅ No errors found, only warnings.`);
      return 0;
    }
  }
}

function generateReport(directory) {
  const files = findSvelteFiles(directory);
  
  const report = {
    timestamp: new Date().toISOString(),
    filesScanned: files.length,
    violations: [],
    summary: {
      errors: 0,
      warnings: 0,
      filesWithIssues: 0
    }
  };

  files.forEach(file => {
    const violations = validateFile(file);
    
    if (violations.length > 0) {
      report.summary.filesWithIssues++;
      
      violations.forEach(violation => {
        report.violations.push({
          file: file,
          ...violation
        });
        
        if (violation.severity === 'error') {
          report.summary.errors++;
        } else {
          report.summary.warnings++;
        }
      });
    }
  });

  return report;
}

function main() {
  const args = process.argv.slice(2);
  const strict = args.includes('--strict');
  const generateReportFlag = args.includes('--report');
  const pathArg = args.find(arg => arg.startsWith('--path='));
  const targetPath = pathArg ? pathArg.split('=')[1] : 'src/lib/components';

  console.log('🔍 Icon Validation Tool');
  console.log('======================');
  
  if (!fs.existsSync(targetPath)) {
    console.error(`❌ Path does not exist: ${targetPath}`);
    process.exit(1);
  }

  if (generateReportFlag) {
    const report = generateReport(targetPath);
    const reportPath = 'icon-validation-report.json';
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    console.log(`📄 Report generated: ${reportPath}`);
    return;
  }

  const exitCode = validateDirectory(targetPath, strict);
  process.exit(exitCode);
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { validateFile, validateDirectory, generateReport };