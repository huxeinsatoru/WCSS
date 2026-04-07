# WCSS Setup Guide for Lovable

This guide shows you how to use WCSS (Web Compiler Style Sheets) in Lovable (formerly GPT Engineer), a cloud-based development environment.

## Why WCSS Works in Lovable

WCSS is fully compatible with cloud-based development environments like Lovable because:

- All dependencies are delivered via npm packages
- No local file system access required
- WASM compiler is bundled in the npm package
- Works entirely within the browser environment

## Cloud Environment Compatibility

### No Local Setup Required

WCSS is designed to work seamlessly in cloud-based development environments without any local installation or configuration. Unlike traditional build tools that require local file system access, WCSS uses an npm-based architecture that works entirely within the browser environment.

**What this means for you:**
- No need to install Rust or any native tooling
- No local WASM compilation required
- No file system dependencies outside node_modules
- Works immediately after `npm install`

### Supported Platforms

WCSS has been tested and verified to work in the following cloud development environments:

- **Lovable** (formerly GPT Engineer) - Full support with HMR
- **StackBlitz** - Full support for Vite projects
- **CodeSandbox** - Full support for Vite projects
- **GitHub Codespaces** - Full support
- **Gitpod** - Full support
- **Replit** - Full support for Node.js environments

All these platforms support the npm-based WASM loading mechanism that WCSS uses.

### npm-Based Architecture

WCSS uses a modern architecture that eliminates traditional build tool limitations:

**How it works:**
1. **WASM Package**: The WCSS compiler is built as a WebAssembly module and published to npm as `@wcss/wasm`
2. **Dynamic Loading**: The Vite plugin (`vite-plugin-wcss`) dynamically imports the WASM module from node_modules
3. **Browser Execution**: The WASM compiler runs directly in the browser's JavaScript environment
4. **Zero File System Access**: All resources are loaded from npm packages, not local file paths

**Benefits:**
- **Universal Compatibility**: Works in any JavaScript environment that supports WASM
- **Fast Installation**: No compilation step during installation
- **Consistent Behavior**: Same compilation results across all environments
- **Automatic Updates**: Standard npm versioning and dependency management

**Technical Details:**
- WASM binary size: ~150KB (compressed)
- Initialization time: ~10-50ms
- Compilation speed: ~1-5ms per file
- Memory usage: Minimal (WASM runs in isolated sandbox)

## Installation

### Step 1: Install the Plugin

In your Lovable project, open the terminal and run:

```bash
npm install vite-plugin-wcss
```

This will automatically install both `vite-plugin-wcss` and its dependency `@wcss/wasm`.

### Step 2: Configure Vite

Create or update your `vite.config.js` file in the project root:

```javascript
import { defineConfig } from 'vite';
import wcss from 'vite-plugin-wcss';

export default defineConfig({
  plugins: [
    wcss({
      // Optional: Enable minification for production
      minify: false,
      
      // Optional: Enable source maps for debugging
      sourceMaps: true,
      
      // Optional: Enable tree-shaking (removes unused styles)
      treeShaking: false,
      
      // Optional: Define design tokens
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6',
          success: '#10b981',
          danger: '#ef4444',
        },
        spacing: {
          sm: '0.5rem',
          md: '1rem',
          lg: '1.5rem',
          xl: '2rem',
        },
      },
    }),
  ],
});
```

### Step 3: Create a WCSS File

Create a new file called `styles.wcss` in your `src` directory:

```wcss
/* src/styles.wcss */

/* Define reusable tokens */
$colors.primary: #3b82f6;
$colors.primary-dark: #2563eb;
$spacing.md: 1rem;
$spacing.sm: 0.5rem;

/* Button component */
.button {
  background: $colors.primary;
  color: white;
  padding: $spacing.sm $spacing.md;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  transition: background 0.2s;
}

.button:hover {
  background: $colors.primary-dark;
}

.button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Card component */
.card {
  background: white;
  border-radius: 8px;
  padding: $spacing.md;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.card-title {
  font-size: 1.5rem;
  font-weight: bold;
  margin-bottom: $spacing.sm;
}

.card-content {
  color: #666;
  line-height: 1.6;
}
```

### Step 4: Import and Use WCSS

Import your WCSS file in your JavaScript or TypeScript code:

**In a React component:**

```javascript
// src/App.jsx
import React from 'react';
import styles from './styles.wcss';

function App() {
  return (
    <div>
      <style>{styles}</style>
      
      <div className="card">
        <h2 className="card-title">Welcome to WCSS</h2>
        <p className="card-content">
          This is a card component styled with WCSS.
        </p>
        <button className="button">Click Me</button>
      </div>
    </div>
  );
}

export default App;
```

**In a vanilla JavaScript project:**

```javascript
// src/main.js
import styles from './styles.wcss';

// Inject styles into the page
const styleElement = document.createElement('style');
styleElement.textContent = styles;
document.head.appendChild(styleElement);

// Use the classes
document.body.innerHTML = `
  <div class="card">
    <h2 class="card-title">Welcome to WCSS</h2>
    <p class="card-content">
      This is a card component styled with WCSS.
    </p>
    <button class="button">Click Me</button>
  </div>
`;
```

## TypeScript Support

If you're using TypeScript, add type definitions for `.wcss` imports.

Create or update `src/vite-env.d.ts`:

```typescript
/// <reference types="vite/client" />
/// <reference types="vite-plugin-wcss/wcss" />
```

Now TypeScript will recognize `.wcss` imports:

```typescript
import styles from './styles.wcss';
// styles: string (compiled CSS)
```

## Advanced Configuration

For detailed documentation of all configuration options including tree-shaking, optimization features, and design tokens, see the [Configuration Guide](./CONFIGURATION.md).

### Tree-Shaking (Remove Unused Styles)

Enable tree-shaking to remove unused CSS classes in production:

```javascript
// vite.config.js
export default defineConfig({
  plugins: [
    wcss({
      treeShaking: true,
      contentPaths: ['./src/**/*.{js,jsx,ts,tsx,html}'],
      safelist: ['button', 'card'], // Always include these classes
    }),
  ],
});
```

### Design Tokens from Config

Define design tokens in your Vite config instead of in WCSS files:

```javascript
// vite.config.js
export default defineConfig({
  plugins: [
    wcss({
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6',
          success: '#10b981',
          danger: '#ef4444',
          warning: '#f59e0b',
          info: '#06b6d4',
        },
        spacing: {
          xs: '0.25rem',
          sm: '0.5rem',
          md: '1rem',
          lg: '1.5rem',
          xl: '2rem',
          '2xl': '3rem',
        },
        typography: {
          'font-sans': 'system-ui, sans-serif',
          'font-mono': 'monospace',
        },
        breakpoints: {
          sm: '640px',
          md: '768px',
          lg: '1024px',
          xl: '1280px',
        },
      },
    }),
  ],
});
```

Then use them in your WCSS:

```wcss
.button {
  background: $colors.primary;
  padding: $spacing.sm $spacing.md;
  font-family: $typography.font-sans;
}

@media (min-width: $breakpoints.md) {
  .button {
    padding: $spacing.md $spacing.lg;
  }
}
```

### Production Optimization

For production builds, enable minification:

```javascript
// vite.config.js
export default defineConfig({
  plugins: [
    wcss({
      minify: process.env.NODE_ENV === 'production',
      sourceMaps: process.env.NODE_ENV !== 'production',
      deduplicate: true,
    }),
  ],
});
```

## Hot Module Replacement

WCSS supports Hot Module Replacement (HMR) in development. When you edit a `.wcss` file, your styles will update instantly without a full page reload.

Try it:
1. Start the dev server: `npm run dev`
2. Edit your `.wcss` file
3. Save the file
4. See the changes instantly in the browser

## Troubleshooting

### Quick Fixes

**Plugin Not Working:**
1. Check installation: `npm list vite-plugin-wcss @wcss/wasm`
2. Restart dev server: Stop (Ctrl+C) and run `npm run dev`
3. Verify plugin is in `vite.config.js`

**WASM Initialization Error:**
```bash
npm install @wcss/wasm
```

**TypeScript Import Errors:**
Add to `src/vite-env.d.ts`:
```typescript
/// <reference types="vite-plugin-wcss/wcss" />
```

**Compilation Errors:**
- Check token definitions: `$colors.primary: #3b82f6;`
- Verify syntax (semicolons, braces)
- Ensure file ends with `.wcss`

### Comprehensive Troubleshooting

For detailed solutions to all issues including cloud environment problems, WASM loading issues, production build problems, and more, see the [Troubleshooting Guide](./TROUBLESHOOTING.md).

## Example Project Structure

Here's a typical project structure using WCSS in Lovable:

```
my-lovable-project/
├── src/
│   ├── components/
│   │   ├── Button.jsx
│   │   └── button.wcss
│   ├── styles/
│   │   ├── global.wcss
│   │   └── tokens.wcss
│   ├── App.jsx
│   ├── main.jsx
│   └── vite-env.d.ts
├── vite.config.js
├── package.json
└── index.html
```

## Testing in Lovable

Want to verify that WCSS works correctly in your Lovable project? Check out our comprehensive testing resources:

- [Lovable Testing Guide](./LOVABLE_TESTING.md) - Step-by-step test cases covering installation, compilation, HMR, error handling, and AI-assisted workflows
- [Lovable Testing Checklist](./LOVABLE_CHECKLIST.md) - Quick reference checklist for rapid verification

These guides help you:
- Verify all features work correctly in Lovable
- Test AI integration with WCSS files
- Troubleshoot common issues
- Document test results

## Next Steps

- Explore [WCSS syntax and features](../README.md)
- Learn about [configuration options](./CONFIGURATION.md)
- Check out [TypeScript support](./TYPESCRIPT.md)
- Test your setup with the [Lovable Testing Guide](./LOVABLE_TESTING.md)

## Need Help?

If you encounter issues:

1. Check the [Troubleshooting Guide](./TROUBLESHOOTING.md) for comprehensive solutions
2. Review the [troubleshooting section](#troubleshooting) above for quick fixes
3. Check the [main README](./README.md) for general documentation
4. Open an issue on the [GitHub repository](https://github.com/your-repo/wcss)

---

**Happy styling with WCSS in Lovable! 🎨**
