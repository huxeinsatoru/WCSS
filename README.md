# WCSS (Web Compiler Style Sheets)

A CSS compiler built with Rust and WebAssembly. Compiles in microseconds, outputs optimized CSS.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Fast compilation** - Rust-based compiler with WebAssembly (1.73ms for 5000 rules with tree-shaking)
- **Full CSS spec support** - All selectors (#id, .class, [attr], :pseudo), at-rules (@keyframes, @layer, @media, @container), 55+ units, 40+ pseudo-classes
- **Tailwind CSS first** - Full support for @tailwind, @apply, @theme, @utility, @variant, @source, @plugin, @config directives (v3 + v4)
- **Modern CSS syntax** - Space-separated colors (hsl/rgb/hwb/oklch/oklab with slash alpha), nested @media/@supports/@container, vendor prefixes
- **Automatic tree shaking** - Auto-scan HTML/JSX/TSX/Vue/Svelte files for used classes (98% size reduction)
- **CSS Modules** - Local scoping with `.module.wcss`, class hashing, `:local()/:global()`, `composes:` support
- **Bundle Analyzer** - Tree-shaking stats per component, size analysis, top 10 largest rules
- **VS Code LSP** - Autocomplete, diagnostics, hover info, formatting support
- **Plugin system** - 6 hooks for custom transformations (before/after parse, optimize, codegen)
- **Vendor prefixing** - Automatic -webkit-, -moz-, -ms- prefixes for 30+ properties
- **Dark mode** - Built-in support (media query, class, attribute strategies)
- **Incremental cache** - Content-hash based AST/CSS caching
- **Modern colors** - oklch, oklab, hwb, lab, lch, color-mix(), light-dark()
- **W3C Design Tokens** - Industry standard token format with multi-platform code generation
- **Parallel processing** - Rayon-based multi-file compilation (auto-parallel for 2+ files)
- **Rich diagnostics** - E001-E010 error codes, Levenshtein suggestions, colored output
- **Zero runtime** - Pure CSS output, no JavaScript in browser
- **Framework plugins** - Vite, Webpack, Next.js, Nuxt, Astro
- **Cloud environment support** - Works in Lovable, StackBlitz, CodeSandbox via WASM

## Install

```bash
npm install -g @wcss/cli
```

## Usage

Create `styles.wcss`:

```wcss
.button {
  padding: 1rem 2rem;
  background: #3b82f6;
  color: white;
  border-radius: 0.5rem;
}

.button:hover {
  background: #2563eb;
}
```

Compile:

```bash
wcss build styles.wcss -o output.css
```

## Framework Integration

### Vite

```bash
npm install vite-plugin-wcss
```

```javascript
// vite.config.js
import wcss from 'vite-plugin-wcss';

export default {
  plugins: [wcss()],
};
```

### Next.js

```bash
npm install next-wcss
```

```javascript
// next.config.js
const withWCSS = require('next-wcss');

module.exports = withWCSS({});
```

### Webpack

```bash
npm install wcss-loader
```

```javascript
// webpack.config.js
module.exports = {
  module: {
    rules: [
      {
        test: /\.wcss$/,
        use: ['style-loader', 'css-loader', 'wcss-loader'],
      },
    ],
  },
};
```

## Design Tokens

### W3C Design Tokens

WCSS supports the [W3C Design Tokens Community Group](https://design-tokens.github.io/community-group/format/) format.

```json
{
  "color": {
    "primary": {
      "$value": "#3b82f6",
      "$type": "color",
      "$description": "Primary brand color"
    }
  },
  "spacing": {
    "base": {
      "$value": "16px",
      "$type": "dimension"
    }
  }
}
```

Generate code for multiple platforms:

```bash
# CSS
wcss tokens tokens.json --platform css -o tokens.css

# iOS
wcss tokens tokens.json --platform ios -o DesignTokens.swift

# Android
wcss tokens tokens.json --platform android -o res/values/

# Flutter
wcss tokens tokens.json --platform flutter -o design_tokens.dart

# TypeScript
wcss tokens tokens.json --platform typescript -o tokens.ts

# Documentation
wcss tokens tokens.json --platform docs -o docs/
```

### Native Tokens

Create `wcss.config.js`:

```javascript
export default {
  tokens: {
    colors: {
      primary: '#3b82f6',
      secondary: '#8b5cf6',
    },
    spacing: {
      sm: '0.5rem',
      md: '1rem',
      lg: '1.5rem',
    },
  },
};
```

Use in styles:

```wcss
.card {
  padding: $spacing.md;
  background: $colors.primary;
}
```

## Language Features

### Tailwind CSS First

WCSS fully supports Tailwind CSS as a first-class workflow, with all v3 and v4 directives:

**@tailwind Directives (v3/v4):**
```wcss
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**@apply Directive:**
```wcss
.btn-primary {
  @apply px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600;
}

.hero {
  @apply container mx-auto px-4;

  @media (min-width: 768px) {
    @apply px-8;
  }
}
```

**@layer with @apply:**
```wcss
@layer components {
  .card {
    @apply p-4 bg-white shadow rounded;
  }
  
  .btn {
    @apply px-4 py-2 rounded font-semibold;
  }
}
```

**Tailwind v4 Directives:**
```wcss
@theme {
  --color-primary: #3b82f6;
  --font-display: "Inter", sans-serif;
}

@source "../src/**/*.{html,jsx,tsx}";
@plugin "@tailwindcss/typography";
@config "./tailwind.config.ts";

@utility content-auto {
  content-visibility: auto;
}

@variant hocus (&:hover, &:focus);
```

**Tailwind-First Workflow:**
- All `@tailwind`, `@theme`, `@utility`, `@variant`, `@source`, `@plugin`, `@config` directives pass through to the output
- `@apply` composes Tailwind utilities into custom components
- Nested `@media`, `@supports`, `@container` work inside rules with `@apply`
- Compatible with Tailwind CSS v3 and v4

### Modern CSS Syntax Support

WCSS parser now supports modern CSS syntax for production use:

**Modern Color Syntax:**
```wcss
.card {
  /* Modern space-separated with slash for alpha */
  background: hsl(240 2% 12% / 0.82);
  color: rgb(255 0 0 / 0.5);
  
  /* Legacy comma-separated (still supported) */
  border: 1px solid hsla(240, 2%, 12%, 0.82);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}
```

**Vendor Prefixes:**
```wcss
.glass {
  /* All vendor prefixes supported */
  -webkit-backdrop-filter: blur(48px);
  -moz-backdrop-filter: blur(48px);
  backdrop-filter: blur(48px);
  
  -webkit-transform: translateY(0);
  -moz-transform: translateY(0);
  -ms-transform: translateY(0);
  transform: translateY(0);
}
```

**Modern Properties:**
```wcss
.modern {
  /* Shorthand properties */
  inset: 0;  /* top, right, bottom, left */
  
  /* Performance hints */
  will-change: transform;
  
  /* Transform utilities */
  transform-origin: center bottom;
  
  /* Box model */
  box-sizing: content-box;
}
```

### Automatic Content Scanning

WCSS automatically scans your source files to detect which CSS classes are used, enabling tree-shaking without manual configuration (similar to Tailwind JIT):

```javascript
// wcss.config.js
export default {
  treeShaking: true,
  contentPaths: [
    'src/**/*.{html,jsx,tsx,vue,svelte}',
    'components/**/*.{js,ts}'
  ],
  // Optional: safelist patterns for dynamic classes
  safelist: [
    /^btn-/,      // Keep all btn-* classes
    /^text-/,     // Keep all text-* classes
  ]
};
```

**Supported file formats:**
- HTML: `<div class="btn primary">`
- JSX/TSX: `<button className="btn-large hover:bg-blue">`
- Vue: `<div :class="['btn', 'primary']">`
- Svelte: `<div class:active class:btn>`
- Template literals: `className={\`btn ${variant}\`}`

**Features:**
- Automatic class extraction from source files
- Glob pattern support for file matching
- Safelist patterns for dynamic classes (regex)
- Validates class names (filters invalid CSS identifiers)
- Supports pseudo-class syntax (e.g., `hover:bg-blue`)

### CSS Modules

WCSS supports CSS Modules for local scoping with `.module.wcss` files:

```wcss
/* button.module.wcss */
.button {
  padding: 1rem 2rem;
  background: #3b82f6;
}

.primary {
  composes: button;
  background: #2563eb;
}

:global(.reset) {
  margin: 0;
  padding: 0;
}
```

Features:
- **Class hashing** - Automatic content-based hash appending (e.g., `button_a1b2c3`)
- **Local/Global scoping** - Use `:local()` and `:global()` to control scope
- **Composition** - Compose classes with `composes: className` or `composes: className from './other.module.css'`
- **Export maps** - Generate JSON/JS exports for use in JavaScript

```javascript
import styles from './button.module.wcss';
// styles = { button: 'button_a1b2c3', primary: 'primary_d4e5f6 button_a1b2c3' }
```

### Full CSS Selector Support

```wcss
/* Element, class, ID */
button { }
.button { }
#submit { }

/* Universal and combinators */
* { }
.parent > .child { }
.sibling + .next { }
.prev ~ .following { }

/* Attribute selectors */
[type="text"] { }
[class^="btn-"] { }
[href$=".pdf"] { }
[data-state~="active"] { }

/* Pseudo-classes (40+) */
:hover, :focus, :active { }
:first-child, :last-child, :nth-child(2n) { }
:not(.disabled), :is(.primary, .secondary) { }
:has(> img), :where(.card) { }
```

### At-Rules

```wcss
/* Keyframes */
@keyframes slide {
  from { transform: translateX(0); }
  to { transform: translateX(100px); }
}

/* Media queries */
@media (min-width: 768px) {
  .card { padding: 2rem; }
}

/* Container queries */
@container (min-width: 400px) {
  .item { flex: 1; }
}

/* Layers */
@layer base, components, utilities;

/* Supports */
@supports (display: grid) {
  .layout { display: grid; }
}

/* Import */
@import "base.css";

/* Font face */
@font-face {
  font-family: "Custom";
  src: url("font.woff2");
}
```

### Modern Colors

```wcss
.element {
  /* OKLCH - perceptually uniform */
  color: oklch(70% 0.15 180);
  
  /* Color mixing */
  background: color-mix(in oklch, blue 50%, red);
  
  /* Light/dark mode */
  border: 1px solid light-dark(#ccc, #333);
  
  /* Other formats */
  fill: oklab(60% 0.1 -0.1);
  stroke: hwb(180 20% 30%);
}
```

### Dark Mode

```wcss
/* Media query strategy */
@media (prefers-color-scheme: dark) {
  .card { background: #1a1a1a; }
}

/* Class strategy */
.dark .card { background: #1a1a1a; }

/* Attribute strategy */
[data-theme="dark"] .card { background: #1a1a1a; }
```

### Vendor Prefixing

Automatic prefixing for 30+ properties:

```wcss
.box {
  /* Auto-prefixed */
  transform: scale(1.2);
  /* Generates: -webkit-transform, -moz-transform, -ms-transform, transform */
  
  user-select: none;
  /* Generates: -webkit-user-select, -moz-user-select, -ms-user-select, user-select */
  
  backdrop-filter: blur(10px);
  /* Generates: -webkit-backdrop-filter, backdrop-filter */
}
```

### States

```wcss
.button {
  background: #3b82f6;
  
  :hover {
    background: #2563eb;
  }
  
  :active {
    background: #1d4ed8;
  }
}
```

### Responsive Queries

```wcss
.card {
  padding: 1rem;
  
  @media (min-width: 768px) {
    padding: 1.5rem;
  }
}
```

## Performance

Benchmarked on Apple M3 with 5000 utility classes:

| Framework | Compilation Time | Output Size (100 used) | Notes |
|-----------|-----------------|----------------------|-------|
| **WCSS (tree-shaking)** | **1.73ms** | **6.6KB** | Fastest with optimizations |
| UnoCSS | 2.33ms | 3.42KB | Pure utility generator |
| WCSS (full) | 4.95ms | 340KB | No tree-shaking |
| Tailwind CSS | 16.75ms | 11.52KB | PostCSS-based |
| Panda CSS | ~17ms | ~12KB | Build-time generation |

### Speed Comparison

- **1.35x faster** than UnoCSS
- **9.68x faster** than Tailwind CSS
- **98% size reduction** with tree-shaking (340KB → 6.6KB)

### Key Points

- Fast compilation with Rust and WebAssembly
- Built-in tree shaking removes unused CSS
- Multi-platform code generation from W3C Design Tokens
- Full compiler features: parser, validator, optimizer, source maps
- Parallel processing with Rayon (auto-parallel for 2+ files)

Run benchmarks:
```bash
cargo run --release --bin benchmark_5k
```

See [BENCHMARK_RESULTS.md](./BENCHMARK_RESULTS.md) for detailed results.

## Bundle Analyzer

Analyze your CSS bundle to understand optimization impact:

```bash
wcss build styles.wcss -o output.css --analyze
```

Output includes:
- **Size comparison** - Original vs optimized size with reduction percentage
- **Rule statistics** - Total rules, removed rules, kept rules
- **Declaration stats** - Duplicate declarations removed, shorthand merges
- **Top 10 largest rules** - Identify optimization opportunities
- **Unused selectors** - List of selectors removed by tree-shaking

Example output:
```
╔════════════════════════════════════════╗
║      WCSS Bundle Analysis              ║
╠════════════════════════════════════════╣
║ Original:                    340.0 KB  ║
║ Optimized:                     6.6 KB  ║
║ Saved:              333.4 KB (98.1%)   ║
╠════════════════════════════════════════╣
║ Rules: 5000 -> 100 (-4900)             ║
║ Declarations: 10000 -> 200 (-9800)     ║
║ Duplicates removed: 50                 ║
║ Shorthand merges: 12                   ║
╠════════════════════════════════════════╣
║ Top 10 Largest Rules:                  ║
║  1. .card          (245 B, 8 decl)     ║
║  2. .button        (198 B, 6 decl)     ║
║ ...                                    ║
╚════════════════════════════════════════╝
```

## CLI Commands

The native Rust CLI provides fast compilation with rich diagnostics:

```bash
# Build with parallel processing (auto-enabled for 2+ files)
wcss build input.wcss -o output.css

# Build with minification
wcss build input.wcss -o output.css --minify

# Build with tree-shaking
wcss build input.wcss -o output.css --tree-shaking

# Build with source maps
wcss build input.wcss -o output.css --source-maps inline

# Watch mode with auto-recompilation
wcss watch input.wcss -o output.css

# Format WCSS files
wcss format input.wcss --write

# W3C Design Tokens compilation
wcss tokens tokens.json --platform css -o tokens.css
wcss tokens tokens.json --platform ios -o DesignTokens.swift
wcss tokens tokens.json --platform android -o res/values/
wcss tokens tokens.json --platform flutter -o design_tokens.dart
wcss tokens tokens.json --platform typescript -o tokens.ts
wcss tokens tokens.json --platform docs -o docs/
```

### Rich Diagnostics

The compiler provides detailed error messages with:
- **Error codes** - E001-E010 for common mistakes
- **Levenshtein suggestions** - "Did you mean `color`?" for typos
- **Colored output** - Red for errors, yellow for warnings
- **Source snippets** - Show the exact line with the error
- **Caret underlines** - Point to the exact location

Example error output:
```
error[E001]: Unknown property 'colr'
  ┌─ styles.wcss:3:3
  │
3 │   colr: red;
  │   ^^^^
  = help: Did you mean `color`?
  = note: [E001] The property name is not a recognized CSS property.
```

## Configuration

```javascript
// wcss.config.js
export default {
  tokens: {
    colors: {},
    spacing: {},
    typography: {},
    breakpoints: {},
    shadows: {},      // New
    borders: {},      // New
    radii: {},        // New
    zindex: {},       // New
    opacity: {},      // New
  },
  minify: true,
  sourceMaps: false,
  treeShaking: true,
  
  // Automatic content scanning (NEW!)
  contentPaths: [
    'src/**/*.{html,jsx,tsx,vue,svelte}',
    'pages/**/*.{js,ts}',
  ],
  
  // Safelist patterns for dynamic classes (NEW!)
  safelist: [
    /^btn-/,        // Keep all btn-* classes
    /^text-/,       // Keep all text-* classes
    'active',       // Keep specific class
  ],
  
  // Manual class list (fallback if contentPaths not provided)
  usedClasses: [],
  
  typedOM: false,
  
  // Plugin system
  plugins: [
    {
      name: 'custom-plugin',
      beforeParse: (input) => input,
      afterParse: (ast) => ast,
      beforeOptimize: (ast) => ast,
      afterOptimize: (ast) => ast,
      beforeCodegen: (ast) => ast,
      afterCodegen: (css) => css,
    }
  ],
  
  // Vendor prefixing
  prefixer: {
    enabled: true,
    browsers: ['> 1%', 'last 2 versions'],
  },
  
  // Dark mode
  darkMode: {
    strategy: 'media', // 'media' | 'class' | 'attribute'
    className: 'dark',
    attribute: 'data-theme',
  },
  
  // Incremental cache
  cache: {
    enabled: true,
    directory: '.wcss-cache',
  },
};
```

## Packages

| Package | Description |
|---------|-------------|
| `wcss-compiler` | Core compiler (Rust) |
| `wcss-cli` | Native Rust CLI with parallel processing |
| `wcss-wasm` | WebAssembly build |
| `@wcss/wasm` | WASM package for cloud environments (npm) |
| `@wcss/cli` | Command-line interface (npm) |
| `vite-plugin-wcss` | Vite plugin with HMR support |
| `next-wcss` | Next.js plugin |
| `wcss-loader` | Webpack loader |
| `astro-wcss` | Astro integration |
| `wcss-lsp` | Language Server Protocol implementation |
| `vscode-wcss` | VS Code extension |

## IDE Support

### VS Code Extension

Install the WCSS extension for VS Code to get:
- **Autocomplete** - IntelliSense for CSS properties and values
- **Diagnostics** - Real-time error checking with E001-E010 codes
- **Hover info** - Documentation for properties and values
- **Formatting** - Auto-format WCSS files on save
- **Syntax highlighting** - Full WCSS syntax support

The extension uses the `wcss-lsp` Language Server Protocol implementation for fast, accurate language features.

## CI/CD

WCSS includes GitHub Actions workflows for continuous integration:

### Workflows

- **CI** (`.github/workflows/ci.yml`) - Runs on push/PR
  - Rust format check (`cargo fmt`)
  - Rust clippy linting (`cargo clippy`)
  - Rust tests on Ubuntu, macOS, Windows
  - WASM build for bundler and Node.js targets
  - JavaScript tests (350 total tests)
  - Benchmark comparison on PRs

- **Release** (`.github/workflows/release.yml`) - Automated releases
  - Build native binaries for Linux, macOS, Windows
  - Publish to crates.io
  - Publish to npm registry
  - Create GitHub releases with artifacts

- **Benchmark** (`.github/workflows/benchmark.yml`) - Performance tracking
  - Run benchmarks on schedule
  - Compare PR performance vs base branch
  - Post results as PR comments

## Testing

WCSS has comprehensive test coverage with 381+ tests:

```bash
# Rust tests (218 tests)
cargo test

# JavaScript tests (142 tests)
npm test

# Tailwind directive tests
cargo test --test tailwind_directives

# Property-based tests
cargo test --test deduplication
cargo test --test minification
cargo test --test parse_format_roundtrip
cargo test --test token_resolution
cargo test --test tree_shaking
cargo test --test valid_css_generation
cargo test --test zero_runtime

# Parallel processing tests (8 tests)
cargo test --test parallel_processing

# Diagnostics tests (60+ tests)
cargo test --test diagnostics_tests
cargo test --test error_reporting_completeness
cargo test --test error_suggestions
cargo test --test multiple_error_detection

# Run all tests
cargo test --workspace && npm test
```

Test categories:
- **Unit tests** - Core compiler functionality
- **Property-based tests** - Fuzzing with proptest for edge cases
- **Integration tests** - End-to-end compilation workflows
- **Parallel processing tests** - Multi-file compilation
- **Diagnostics tests** - Error reporting and suggestions
- **Framework tests** - Vite, Next.js, Astro plugins
- **Tailwind compatibility tests** - @tailwind, @apply, @theme, @utility, @variant, @source, @plugin, @config directives (v3 + v4)

## License

MIT

## Links

- [Examples](./examples)
- [W3C Design Tokens Spec](https://design-tokens.github.io/community-group/format/)
