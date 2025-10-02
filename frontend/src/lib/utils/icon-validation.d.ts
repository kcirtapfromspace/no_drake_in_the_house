/**
 * Type definitions for icon validation utilities
 */

export interface IconValidationIssue {
  line: number;
  pattern: string;
  suggestion: string;
}

export interface IconUsageReport {
  totalIcons: number;
  properlyClassified: number;
  oversized: number;
  missingAria: number;
  suggestions: string[];
}

export interface IconValidationOptions {
  maxSize?: number;
  enableAutoFix?: boolean;
  skipAvatars?: boolean;
}

/**
 * Validates icon sizes at runtime and applies constraints if needed
 */
export declare function validateIconSizes(options?: IconValidationOptions): void;

/**
 * Scans component code for oversized icon patterns
 */
export declare function scanForOversizedPatterns(componentCode: string): IconValidationIssue[];

/**
 * Automatically replaces common oversized patterns with design system classes
 */
export declare function migrateOversizedPatterns(componentCode: string): string;

/**
 * Creates a development mode observer to watch for dynamically added oversized icons
 */
export declare function createIconSizeObserver(): MutationObserver | null;

/**
 * Generates a report of icon usage across the application
 */
export declare function generateIconUsageReport(): IconUsageReport;