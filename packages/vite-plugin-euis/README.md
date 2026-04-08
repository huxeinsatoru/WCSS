# vite-plugin-euis

Vite plugin for Euis (Euis) - compile Euis to CSS with zero runtime overhead.

## Installation

```bash
npm install vite-plugin-euis
```

## Supported Versions

- **Vite**: 4.x and 5.x
- **Node.js**: 16.x or higher

The plugin is compatible with both Vite 4 and Vite 5, allowing you to use it in projects with either version.

## Quick Start

### 1. Configure Vite

Add the plugin to your `vite.config.js` or `vite.config.ts`:

```javascript
import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';

export default defineConfig({
  plugins: [
    euis({
      // Optional configuration
      minify: false,
      sourceMaps: true,
      treeShaking: false,
    }),
  ],
});
```

### 2. Create a Euis file

Create a `.euis` file in your project:

```euis
/* styles.euis */
.button {
  background: $colors.primary;
  padding: $spacing.md;
  border-radius: 4px;
}

.button:hover {
  background: $colors.primary-dark;
}
```

### 3. Import in your code

Import the Euis file in your JavaScript/TypeScript:

```javascript
import styles from './styles.euis';

// styles contains the compiled CSS string
console.log(styles);
```

## Configuration Options

The plugin supports various configuration options for optimization and customization:

```typescript
interface EuisPluginOptions {
  minify?: boolean;           // Enable CSS minification (default: false)
  sourceMaps?: boolean;        // Enable source maps (default: true)
  treeShaking?: boolean;       // Remove unused styles (default: false)
  typedOM?: boolean;           // Enable TypedOM output (default: false)
  deduplicate?: boolean;       // Deduplicate identical rules (default: true)
  usedClasses?: string[];      // Classes used in your app (for tree-shaking)
  contentPaths?: string[];     // Paths to scan for used classes
  safelist?: string[];         // Classes to always include
  tokens?: {                   // Design tokens
    colors?: Record<string, string>;
    spacing?: Record<string, string>;
    typography?: Record<string, string>;
    breakpoints?: Record<string, string>;
  };
}
```

For detailed documentation of all configuration options, see the [Configuration Guide](./CONFIGURATION.md).

## Cloud Environment Support

This plugin works seamlessly in cloud-based development environments:

- ✅ Lovable (formerly GPT Engineer)
- ✅ StackBlitz
- ✅ CodeSandbox
- ✅ GitHub Codespaces

No local file system setup required - all dependencies are loaded from npm packages.

### Setup Guides

For platform-specific setup instructions:

- [Lovable Setup Guide](./LOVABLE.md) - Complete setup guide for Lovable (formerly GPT Engineer)

### Testing in Cloud Environments

For detailed testing instructions and verification steps:

- [Lovable Testing Guide](./LOVABLE_TESTING.md) - Comprehensive test cases for Lovable
- [Lovable Testing Checklist](./LOVABLE_CHECKLIST.md) - Quick reference checklist for Lovable
- [StackBlitz Testing Guide](./STACKBLITZ_TESTING.md) - Comprehensive test cases for StackBlitz
- [StackBlitz Testing Checklist](./STACKBLITZ_CHECKLIST.md) - Quick reference checklist for StackBlitz

## TypeScript Support

The plugin includes TypeScript definitions. For `.euis` file imports, add this to your `vite-env.d.ts`:

```typescript
/// <reference types="vite-plugin-euis/euis" />
```

This provides type definitions for `.euis` imports:

```typescript
import styles from './styles.euis';
// styles: string
```

## Features

- **Zero Runtime**: Compiles to pure CSS with no JavaScript runtime
- **Hot Module Replacement**: Instant style updates during development
- **Source Maps**: Debug your Euis with accurate source mapping
- **Tree Shaking**: Remove unused styles in production
- **Minification**: Optimize CSS output for production
- **TypeScript**: Full TypeScript support with type definitions
- **Cloud Ready**: Works in browser-based development environments

## Troubleshooting

Having issues? Check the [Troubleshooting Guide](./TROUBLESHOOTING.md) for solutions to common problems including:

- Installation and dependency issues
- WASM loading problems
- Compilation errors
- Cloud environment specific issues
- TypeScript configuration
- Hot Module Replacement issues
- Production build problems

## License

MIT
