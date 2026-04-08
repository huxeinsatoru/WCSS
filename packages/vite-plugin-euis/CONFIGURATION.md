# Euis Plugin Configuration Guide

Complete reference for all configuration options available in `vite-plugin-euis`.

## Table of Contents

- [Overview](#overview)
- [Configuration Options](#configuration-options)
  - [minify](#minify)
  - [sourceMaps](#sourcemaps)
  - [treeShaking](#treeshaking)
  - [typedOM](#typedom)
  - [deduplicate](#deduplicate)
  - [usedClasses](#usedclasses)
  - [contentPaths](#contentpaths)
  - [safelist](#safelist)
  - [tokens](#tokens)
- [Configuration Examples](#configuration-examples)
- [Optimization Features](#optimization-features)

## Overview

The Euis plugin accepts a configuration object with various options to control compilation behavior, optimization, and output format.

```typescript
interface EuisPluginOptions {
  treeShaking?: boolean;
  minify?: boolean;
  sourceMaps?: boolean;
  typedOM?: boolean;
  deduplicate?: boolean;
  usedClasses?: string[];
  contentPaths?: string[];
  safelist?: string[];
  tokens?: {
    colors?: Record<string, string>;
    spacing?: Record<string, string>;
    typography?: Record<string, string>;
    breakpoints?: Record<string, string>;
  };
}
```

## Configuration Options

### minify

**Type:** `boolean`  
**Default:** `false`

Enables CSS minification to reduce output file size. When enabled, the compiler removes whitespace, comments, and optimizes CSS rules.

**Example:**

```javascript
// vite.config.js
export default defineConfig({
  plugins: [
    euis({
      minify: true
    })
  ]
});
```

**Input:**

```euis
.button {
  background: #3b82f6;
  padding: 1rem;
  border-radius: 4px;
}
```

**Output (minified):**

```css
.button{background:#3b82f6;padding:1rem;border-radius:4px}
```

**Use Cases:**
- Production builds to reduce bundle size
- Optimizing CSS delivery for performance
- Reducing bandwidth usage

**Recommendation:** Enable in production, disable in development for better debugging.

```javascript
euis({
  minify: process.env.NODE_ENV === 'production'
})
```

---

### sourceMaps

**Type:** `boolean`  
**Default:** `true`

Generates source maps that map compiled CSS back to original Euis source files. Essential for debugging in development.

**Example:**

```javascript
euis({
  sourceMaps: true
})
```

**Benefits:**
- Browser DevTools show original Euis file locations
- Accurate line numbers for debugging
- Better development experience

**Source Map Output:**

When enabled, the plugin generates inline source maps that allow you to see the original `.euis` file and line numbers in browser DevTools when inspecting elements.

**Recommendation:** Enable in development, optionally disable in production to reduce file size.

```javascript
euis({
  sourceMaps: process.env.NODE_ENV !== 'production'
})
```

---

### treeShaking

**Type:** `boolean`  
**Default:** `false`

Removes unused CSS classes from the output. Requires either `usedClasses` or `contentPaths` to determine which classes are actually used in your application.

**Example:**

```javascript
euis({
  treeShaking: true,
  usedClasses: ['button', 'card', 'header']
})
```

**How It Works:**

1. The compiler analyzes your Euis files and identifies all defined classes
2. It compares them against the `usedClasses` list or scans `contentPaths` for class usage
3. Only classes that are actually used are included in the output
4. Unused classes are eliminated, reducing bundle size

**Input Euis:**

```euis
.button { background: blue; }
.card { padding: 1rem; }
.unused { color: red; }
```

**With `usedClasses: ['button', 'card']`:**

```css
.button { background: blue; }
.card { padding: 1rem; }
/* .unused is removed */
```

**Benefits:**
- Significantly reduces CSS bundle size
- Removes dead code automatically
- Improves page load performance

**Requirements:**
- Must specify either `usedClasses` or `contentPaths`
- Works best with static class names (not dynamic class generation)

**Recommendation:** Enable in production with content scanning for maximum optimization.

---

### typedOM

**Type:** `boolean`  
**Default:** `false`

Enables CSS Typed OM runtime output. When enabled, the compiler generates JavaScript code that uses the CSS Typed Object Model API for programmatic style manipulation.

**Example:**

```javascript
euis({
  typedOM: true
})
```

**Output:**

When enabled, the plugin exports an additional `runtime` export containing Typed OM helper code:

```javascript
import styles, { runtime } from './styles.euis';

// styles: compiled CSS string
// runtime: Typed OM helper code (if typedOM is enabled)
```

**Use Cases:**
- Dynamic style manipulation with better performance
- Type-safe CSS value manipulation
- Advanced animation and transition control

**Note:** This is an advanced feature. Most applications don't need Typed OM and should keep this disabled.

**Recommendation:** Keep disabled unless you specifically need Typed OM functionality.

---

### deduplicate

**Type:** `boolean`  
**Default:** `true`

Removes duplicate CSS rule blocks to reduce output size. When multiple selectors have identical declarations, they are merged.

**Example:**

```javascript
euis({
  deduplicate: true
})
```

**Input:**

```euis
.button-primary {
  background: blue;
  padding: 1rem;
}

.button-secondary {
  background: blue;
  padding: 1rem;
}
```

**Output (deduplicated):**

```css
.button-primary,
.button-secondary {
  background: blue;
  padding: 1rem;
}
```

**Benefits:**
- Reduces CSS file size
- Improves compression ratio
- No runtime impact

**Recommendation:** Keep enabled (default) for optimal output size.

---

### usedClasses

**Type:** `string[]`  
**Default:** `[]`

Explicit list of CSS class names that are used in your application. Used by tree-shaking to determine which classes to keep.

**Example:**

```javascript
euis({
  treeShaking: true,
  usedClasses: [
    'button',
    'button-primary',
    'button-secondary',
    'card',
    'card-header',
    'card-body',
    'header',
    'footer'
  ]
})
```

**Use Cases:**
- When you know exactly which classes are used
- For libraries with a fixed set of components
- When content scanning is not feasible

**Advantages:**
- Faster than content scanning
- Deterministic output
- No file system access needed

**Disadvantages:**
- Must be manually maintained
- Can become outdated as code changes

**Recommendation:** Use `contentPaths` for automatic scanning instead, unless you have specific requirements for manual control.

---

### contentPaths

**Type:** `string[]`  
**Default:** `[]`

Glob patterns for files to scan for CSS class usage. The compiler scans these files to automatically detect which classes are used, enabling automatic tree-shaking.

**Example:**

```javascript
euis({
  treeShaking: true,
  contentPaths: [
    './src/**/*.{js,jsx,ts,tsx}',
    './src/**/*.html',
    './src/**/*.vue'
  ]
})
```

**How It Works:**

1. The compiler scans all files matching the glob patterns
2. It extracts class names from `className`, `class`, and other attributes
3. Only classes found in these files are included in the output
4. Unused classes are automatically removed

**Supported Patterns:**

```javascript
contentPaths: [
  './src/**/*.jsx',           // All JSX files in src
  './src/components/**/*.tsx', // TypeScript React in components
  './public/**/*.html',        // HTML files
  './src/**/*.vue',            // Vue components
  './src/**/*.svelte'          // Svelte components
]
```

**Benefits:**
- Automatic tree-shaking without manual maintenance
- Catches all class usage across your codebase
- Updates automatically as code changes

**Performance:**
- Scanning happens during build time
- Minimal impact on development server
- Results are cached for subsequent builds

**Recommendation:** Use this for automatic tree-shaking in production builds.

```javascript
euis({
  treeShaking: process.env.NODE_ENV === 'production',
  contentPaths: process.env.NODE_ENV === 'production' 
    ? ['./src/**/*.{js,jsx,ts,tsx}']
    : []
})
```

---

### safelist

**Type:** `string[]`  
**Default:** `[]`

List of class names that should never be removed by tree-shaking, even if they're not detected in content scanning.

**Example:**

```javascript
euis({
  treeShaking: true,
  contentPaths: ['./src/**/*.jsx'],
  safelist: [
    'error',
    'success',
    'warning',
    'dynamic-class-*'  // Supports wildcards
  ]
})
```

**Use Cases:**

1. **Dynamically Generated Classes:**
   ```javascript
   // These won't be detected by content scanning
   const className = `status-${status}`;
   ```

2. **Classes Added by Third-Party Libraries:**
   ```javascript
   // Library might add classes at runtime
   <div className="user-content">
     {/* Third-party content with unknown classes */}
   </div>
   ```

3. **Classes Used in HTML Strings:**
   ```javascript
   element.innerHTML = '<div class="dynamic">...</div>';
   ```

**Wildcard Support:**

```javascript
safelist: [
  'status-*',      // Matches status-error, status-success, etc.
  'btn-*',         // Matches btn-primary, btn-secondary, etc.
  '*-active'       // Matches any class ending with -active
]
```

**Recommendation:** Use sparingly and only for classes that are truly dynamic or added at runtime.

---

### tokens

**Type:** `object`  
**Default:** `{}`

Design tokens configuration that defines reusable values for colors, spacing, typography, and breakpoints. These tokens can be referenced in Euis files using the `$` syntax.

**Structure:**

```typescript
tokens?: {
  colors?: Record<string, string>;
  spacing?: Record<string, string>;
  typography?: Record<string, string>;
  breakpoints?: Record<string, string>;
}
```

**Example:**

```javascript
euis({
  tokens: {
    colors: {
      primary: '#3b82f6',
      secondary: '#8b5cf6',
      success: '#10b981',
      danger: '#ef4444',
      warning: '#f59e0b',
      info: '#06b6d4',
      'gray-50': '#f9fafb',
      'gray-900': '#111827'
    },
    spacing: {
      xs: '0.25rem',
      sm: '0.5rem',
      md: '1rem',
      lg: '1.5rem',
      xl: '2rem',
      '2xl': '3rem',
      '3xl': '4rem'
    },
    typography: {
      'font-sans': 'system-ui, -apple-system, sans-serif',
      'font-serif': 'Georgia, serif',
      'font-mono': 'Menlo, Monaco, monospace',
      'text-xs': '0.75rem',
      'text-sm': '0.875rem',
      'text-base': '1rem',
      'text-lg': '1.125rem',
      'text-xl': '1.25rem'
    },
    breakpoints: {
      sm: '640px',
      md: '768px',
      lg: '1024px',
      xl: '1280px',
      '2xl': '1536px'
    }
  }
})
```

**Usage in Euis:**

```euis
.button {
  background: $colors.primary;
  color: white;
  padding: $spacing.sm $spacing.md;
  font-family: $typography.font-sans;
  font-size: $typography.text-base;
}

.button:hover {
  background: $colors.secondary;
}

@media (min-width: $breakpoints.md) {
  .button {
    padding: $spacing.md $spacing.lg;
    font-size: $typography.text-lg;
  }
}

.card {
  background: $colors.gray-50;
  border: 1px solid $colors.gray-900;
  padding: $spacing.xl;
}
```

**Benefits:**

1. **Centralized Design System:**
   - Define tokens once, use everywhere
   - Easy to maintain and update
   - Consistent styling across the application

2. **Type Safety:**
   - Tokens are validated at compile time
   - Undefined token references produce errors
   - Prevents typos and mistakes

3. **Flexibility:**
   - Override tokens per environment
   - Support theming and customization
   - Easy to integrate with design tools

**Token Organization:**

```javascript
// tokens.config.js
export const tokens = {
  colors: {
    // Brand colors
    primary: '#3b82f6',
    secondary: '#8b5cf6',
    
    // Semantic colors
    success: '#10b981',
    danger: '#ef4444',
    warning: '#f59e0b',
    info: '#06b6d4',
    
    // Neutral colors
    'gray-50': '#f9fafb',
    'gray-100': '#f3f4f6',
    // ... more grays
    'gray-900': '#111827'
  },
  spacing: {
    // Base spacing scale
    xs: '0.25rem',   // 4px
    sm: '0.5rem',    // 8px
    md: '1rem',      // 16px
    lg: '1.5rem',    // 24px
    xl: '2rem',      // 32px
    '2xl': '3rem',   // 48px
    '3xl': '4rem'    // 64px
  }
};

// vite.config.js
import { tokens } from './tokens.config.js';

export default defineConfig({
  plugins: [
    euis({ tokens })
  ]
});
```

**Recommendation:** Define tokens in the config for centralized management and easy updates across your entire application.

---

## Configuration Examples

### Development Configuration

Optimized for fast iteration and debugging:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';

export default defineConfig({
  plugins: [
    euis({
      minify: false,
      sourceMaps: true,
      treeShaking: false,
      deduplicate: true,
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6'
        },
        spacing: {
          sm: '0.5rem',
          md: '1rem',
          lg: '1.5rem'
        }
      }
    })
  ]
});
```

### Production Configuration

Optimized for performance and bundle size:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';

export default defineConfig({
  plugins: [
    euis({
      minify: true,
      sourceMaps: false,
      treeShaking: true,
      deduplicate: true,
      contentPaths: [
        './src/**/*.{js,jsx,ts,tsx}',
        './src/**/*.html'
      ],
      safelist: [
        'error',
        'success',
        'warning'
      ],
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6'
        },
        spacing: {
          sm: '0.5rem',
          md: '1rem',
          lg: '1.5rem'
        }
      }
    })
  ]
});
```

### Environment-Based Configuration

Automatically adjust settings based on environment:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';

const isProduction = process.env.NODE_ENV === 'production';

export default defineConfig({
  plugins: [
    euis({
      minify: isProduction,
      sourceMaps: !isProduction,
      treeShaking: isProduction,
      deduplicate: true,
      contentPaths: isProduction ? [
        './src/**/*.{js,jsx,ts,tsx}'
      ] : [],
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6'
        },
        spacing: {
          sm: '0.5rem',
          md: '1rem',
          lg: '1.5rem'
        }
      }
    })
  ]
});
```

### Component Library Configuration

For building a component library with Euis:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';

export default defineConfig({
  plugins: [
    euis({
      minify: true,
      sourceMaps: true,
      treeShaking: false, // Include all styles for library consumers
      deduplicate: true,
      tokens: {
        colors: {
          primary: 'var(--lib-primary, #3b82f6)',
          secondary: 'var(--lib-secondary, #8b5cf6)'
        },
        spacing: {
          sm: 'var(--lib-spacing-sm, 0.5rem)',
          md: 'var(--lib-spacing-md, 1rem)',
          lg: 'var(--lib-spacing-lg, 1.5rem)'
        }
      }
    })
  ]
});
```

### Full-Featured Configuration

All options enabled with comprehensive settings:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';
import { tokens } from './design-tokens.js';

export default defineConfig({
  plugins: [
    euis({
      // Optimization
      minify: process.env.NODE_ENV === 'production',
      deduplicate: true,
      
      // Tree-shaking
      treeShaking: process.env.NODE_ENV === 'production',
      contentPaths: [
        './src/**/*.{js,jsx,ts,tsx}',
        './src/**/*.vue',
        './src/**/*.html'
      ],
      safelist: [
        'error',
        'success',
        'warning',
        'info',
        'status-*',
        'dynamic-*'
      ],
      
      // Development
      sourceMaps: process.env.NODE_ENV !== 'production',
      
      // Advanced features
      typedOM: false,
      
      // Design tokens
      tokens: tokens
    })
  ]
});
```

---

## Optimization Features

### Tree-Shaking

Tree-shaking removes unused CSS classes from your output, significantly reducing bundle size.

**How to Enable:**

```javascript
euis({
  treeShaking: true,
  contentPaths: ['./src/**/*.{js,jsx,ts,tsx}']
})
```

**How It Works:**

1. **Content Scanning:** The compiler scans all files matching `contentPaths` patterns
2. **Class Detection:** Extracts class names from `className`, `class` attributes, and string literals
3. **Usage Analysis:** Compares detected classes against defined classes in Euis
4. **Elimination:** Removes classes that are defined but never used
5. **Safelist:** Preserves classes in the `safelist` regardless of usage

**Example:**

**Euis Input:**

```euis
.button { background: blue; }
.card { padding: 1rem; }
.unused-class { color: red; }
.another-unused { margin: 2rem; }
```

**JavaScript Usage:**

```jsx
function App() {
  return (
    <div>
      <button className="button">Click</button>
      <div className="card">Content</div>
    </div>
  );
}
```

**Output (with tree-shaking):**

```css
.button { background: blue; }
.card { padding: 1rem; }
/* .unused-class and .another-unused are removed */
```

**Benefits:**
- Reduces CSS bundle size by 30-70% on average
- Improves page load performance
- Automatic dead code elimination

**Best Practices:**

1. **Use in Production Only:**
   ```javascript
   treeShaking: process.env.NODE_ENV === 'production'
   ```

2. **Comprehensive Content Paths:**
   ```javascript
   contentPaths: [
     './src/**/*.{js,jsx,ts,tsx}',
     './public/**/*.html'
   ]
   ```

3. **Safelist Dynamic Classes:**
   ```javascript
   safelist: ['status-*', 'theme-*']
   ```

### Minification

Minification reduces CSS file size by removing whitespace, comments, and optimizing syntax.

**How to Enable:**

```javascript
euis({
  minify: true
})
```

**Optimizations Applied:**

1. **Whitespace Removal:** Removes all unnecessary whitespace
2. **Comment Removal:** Strips all comments
3. **Color Optimization:** Converts colors to shortest form (`#ffffff` → `#fff`)
4. **Zero Removal:** Removes unnecessary zeros (`0.5rem` → `.5rem`)
5. **Semicolon Removal:** Removes trailing semicolons
6. **Quote Optimization:** Removes unnecessary quotes

**Example:**

**Input:**

```euis
/* Button styles */
.button {
  background: #ffffff;
  padding: 0.5rem 1.0rem;
  margin: 0px;
  border-radius: 4px;
}
```

**Output (minified):**

```css
.button{background:#fff;padding:.5rem 1rem;margin:0;border-radius:4px}
```

**Size Reduction:** Typically 20-40% smaller than unminified CSS.

**Recommendation:** Enable in production, disable in development.

### Deduplication

Deduplication merges identical CSS rule blocks to reduce redundancy.

**How to Enable:**

```javascript
euis({
  deduplicate: true  // Enabled by default
})
```

**Example:**

**Input:**

```euis
.button-primary {
  background: blue;
  padding: 1rem;
  border-radius: 4px;
}

.button-secondary {
  background: blue;
  padding: 1rem;
  border-radius: 4px;
}

.button-tertiary {
  background: green;
  padding: 1rem;
}
```

**Output (deduplicated):**

```css
.button-primary,
.button-secondary {
  background: blue;
  padding: 1rem;
  border-radius: 4px;
}

.button-tertiary {
  background: green;
  padding: 1rem;
}
```

**Benefits:**
- Reduces CSS file size
- Improves gzip compression ratio
- No runtime performance impact

**Recommendation:** Keep enabled (default) for optimal output.

### Combined Optimizations

For maximum optimization, combine all features:

```javascript
euis({
  minify: true,
  treeShaking: true,
  deduplicate: true,
  contentPaths: ['./src/**/*.{js,jsx,ts,tsx}'],
  safelist: ['dynamic-*']
})
```

**Expected Results:**
- 50-80% reduction in CSS bundle size
- Faster page loads
- Better compression ratios
- Cleaner output

---

## Performance Considerations

### Build Time Impact

| Feature | Build Time Impact | Bundle Size Reduction |
|---------|------------------|----------------------|
| Minification | Low (~5-10ms) | 20-40% |
| Deduplication | Low (~5-10ms) | 5-15% |
| Tree-shaking | Medium (~50-200ms) | 30-70% |
| Source Maps | Low (~10-20ms) | N/A (dev only) |

### Recommendations by Project Size

**Small Projects (<10 Euis files):**
```javascript
euis({
  minify: true,
  deduplicate: true,
  sourceMaps: false
})
```

**Medium Projects (10-50 Euis files):**
```javascript
euis({
  minify: true,
  deduplicate: true,
  treeShaking: true,
  contentPaths: ['./src/**/*.{js,jsx,ts,tsx}']
})
```

**Large Projects (50+ Euis files):**
```javascript
euis({
  minify: true,
  deduplicate: true,
  treeShaking: true,
  contentPaths: ['./src/**/*.{js,jsx,ts,tsx}'],
  safelist: ['dynamic-*', 'status-*']
})
```

---

## Next Steps

- [Quick Start Guide](./README.md)
- [Lovable Setup Guide](./LOVABLE.md)
- [TypeScript Support](./TYPESCRIPT.md)
