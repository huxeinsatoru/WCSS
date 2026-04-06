# WCSS (Web Compiler Style Sheets)

A CSS compiler built with Rust and WebAssembly. Compiles in microseconds, outputs optimized CSS.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Fast compilation** - Rust-based compiler with WebAssembly
- **Tree shaking** - Remove unused CSS classes
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

Benchmarked on Apple M3:

| Input | Rules | WCSS | LightningCSS | Tailwind CSS |
|-------|-------|------|--------------|--------------|
| Small (1 rule) | 1 | 1μs | 94μs | N/A |
| Large (5000 rules) | 5000 | 7.7ms | 3.4ms | 43ms |
| With Tree Shaking | 100/5000 | 2.7ms | ❌ | 43ms |
| Output (5000 rules) | - | 294KB / 8KB* | 162KB | ~4KB |

\* 294KB full, 8KB with tree shaking (100 classes used)

### Key Points

- Fast compilation with Rust and WebAssembly
- Built-in tree shaking removes unused CSS
- Multi-platform code generation from W3C Design Tokens

Run benchmarks:
```bash
cargo run --release --bin benchmark_5k
```

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
  },
  minify: true,
  sourceMaps: false,
  treeShaking: true,
  usedClasses: [],
  typedOM: false,
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
