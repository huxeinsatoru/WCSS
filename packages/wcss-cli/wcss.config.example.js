/**
 * Example WCSS configuration file
 * Copy this to wcss.config.js and customize for your project
 */

export default {
  // Minify CSS output
  minify: false,

  // Generate source maps: false, 'inline', or 'external'
  sourceMaps: false,

  // Enable Typed OM runtime for dynamic styles
  typedOM: false,

  // Enable tree shaking to remove unused styles
  treeShaking: false,

  // Design tokens
  tokens: {
    // Color tokens
    colors: {
      primary: '#3b82f6',
      secondary: '#8b5cf6',
      accent: '#f59e0b',
      danger: '#ef4444',
      success: '#10b981',
      warning: '#f59e0b',
      info: '#3b82f6',
      
      // Grayscale
      black: '#000000',
      white: '#ffffff',
      gray: {
        50: '#f9fafb',
        100: '#f3f4f6',
        200: '#e5e7eb',
        300: '#d1d5db',
        400: '#9ca3af',
        500: '#6b7280',
        600: '#4b5563',
        700: '#374151',
        800: '#1f2937',
        900: '#111827',
      },
    },

    // Spacing tokens
    spacing: {
      xs: '0.25rem',    // 4px
      sm: '0.5rem',     // 8px
      md: '1rem',       // 16px
      lg: '1.5rem',     // 24px
      xl: '2rem',       // 32px
      '2xl': '3rem',    // 48px
      '3xl': '4rem',    // 64px
      '4xl': '6rem',    // 96px
    },

    // Typography tokens
    typography: {
      // Font families
      'font-sans': 'system-ui, -apple-system, sans-serif',
      'font-serif': 'Georgia, serif',
      'font-mono': 'Menlo, Monaco, monospace',

      // Font sizes
      'text-xs': '0.75rem',
      'text-sm': '0.875rem',
      'text-base': '1rem',
      'text-lg': '1.125rem',
      'text-xl': '1.25rem',
      'text-2xl': '1.5rem',
      'text-3xl': '1.875rem',
      'text-4xl': '2.25rem',

      // Line heights
      'leading-tight': '1.25',
      'leading-normal': '1.5',
      'leading-loose': '2',
    },

    // Breakpoint tokens
    breakpoints: {
      sm: '640px',
      md: '768px',
      lg: '1024px',
      xl: '1280px',
      '2xl': '1536px',
    },
  },
};
