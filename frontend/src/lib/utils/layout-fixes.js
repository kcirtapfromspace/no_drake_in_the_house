
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
