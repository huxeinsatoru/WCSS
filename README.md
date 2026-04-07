# WCSS (Web Compiler Style Sheets)

A CSS compiler built with Rust and WebAssembly. Compiles in microseconds, outputs optimized CSS.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Fast compilation** - Rust-based compiler with WebAssembly (1.73ms for 5000 rules with tree-shaking)
- **Full CSS spec support** - All selectors (#id, .class, [attr], :pseudo), at-rules (@keyframes, @layer, @media, @container), 55+ units, 40+ pseudo-classes
- **Tree shaking** - Remove unused CSS classes (98% size reduction)
- **Plugin system** - 6 hooks for custom transformations (before/after parse, optimize, codegen)
- **Vendor prefixing** - Automatic -webkit-, -moz-, -ms- prefixes for 30+ properties
- **Dark mode** - Built-in support (media query, class, attribute strategies)
- **Incremental cache** - Content-hash based AST/CSS caching
- **Modern colors** - oklch, oklab, hwb, lab, lch, color-mix(), light-dark()
- **W3C Design Tokens** - Industry standard token format with multi-platform code generation
- **Zero runtime** - Pure CSS output, no JavaScript in browser
- **Framework plugins** - Vite, Webpack, Next.js, Nuxt, Astro

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

Run benchmarks:
```bash
cargo run --release --bin benchmark_5k
```

See [BENCHMARK_RESULTS.md](./BENCHMARK_RESULTS.md) for detailed results.

## CLI Commands

```bash
# Build
wcss build input.wcss -o output.css

# Watch mode
wcss watch input.wcss -o output.css

# Format
wcss format input.wcss --write

# W3C Tokens
wcss tokens tokens.json --platform css -o tokens.css
wcss tokens tokens.json --platform ios -o DesignTokens.swift
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
| `wcss-wasm` | WebAssembly build |
| `@wcss/cli` | Command-line interface |
| `vite-plugin-wcss` | Vite plugin |
| `next-wcss` | Next.js plugin |
| `wcss-loader` | Webpack loader |
| `astro-wcss` | Astro integration |
| `vscode-wcss` | VS Code extension |

## Testing

```bash
# Rust tests
cargo test

# JavaScript tests
npm test
```

## License

MIT

## Links

- [Examples](./examples)
- [W3C Design Tokens Spec](https://design-tokens.github.io/community-group/format/)
