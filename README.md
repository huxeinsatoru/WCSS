# Euis

CSS compiler built with Rust + WebAssembly. **1.73ms** for 5000 rules. **9.68x faster** than Tailwind CSS.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Install

```bash
npm install -g @euis/cli
```

## Quick Start

```css
/* styles.euis */
.button {
  padding: 1rem 2rem;
  background: $colors.primary;
  color: white;
  border-radius: 0.5rem;
  &:hover { background: $colors.secondary; }
}
```

```bash
euis build styles.euis -o output.css
```

## Features

| Feature | Description |
|---------|-------------|
| **Tailwind CSS first** | Full @tailwind, @apply, @theme, @utility, @variant, @source, @plugin, @config (v3 + v4) |
| **Tree-shaking** | Auto-scan HTML/JSX/TSX/Vue/Svelte. 98% size reduction. Supports clsx, cva, twMerge |
| **Preflight/Reset** | Built-in CSS reset (modern-normalize based) with configurable options |
| **Theming** | CSS variables theming with multi-theme support (light/dark/custom) |
| **Migration tool** | Tailwind config + utility class converter to Euis format |
| **CSS Modules** | Local scoping, class hashing, composes, :local/:global |
| **Design Tokens** | W3C tokens → CSS, iOS, Android, Flutter, TypeScript, Docs |
| **Modern CSS** | oklch, color-mix(), light-dark(), container queries, @layer, nesting |
| **Vendor prefixing** | Auto -webkit-, -moz-, -ms- for 30+ properties |
| **Parallel processing** | Rayon-based multi-file compilation |
| **VS Code** | LSP with autocomplete, diagnostics, hover, formatting |

## Framework Integration

**Vite**
```js
import euis from 'vite-plugin-euis';
export default { plugins: [euis()] };
```

**Next.js**
```js
const withEuis = require('next-euis');
module.exports = withEuis({});
```

**Webpack**
```js
{ test: /\.euis$/, use: ['style-loader', 'css-loader', 'euis-loader'] }
```

## Configuration

```js
// euis.config.js
export default {
  tokens: {
    colors: { primary: '#3b82f6', secondary: '#8b5cf6' },
    spacing: { sm: '0.5rem', md: '1rem', lg: '1.5rem' },
  },
  treeShaking: true,
  contentPaths: ['src/**/*.{html,jsx,tsx,vue,svelte}'],
  safelist: [/^btn-/, 'active'],
  preflight: { enabled: true },
  theming: {
    defaultTheme: 'light',
    strategy: 'CSSVariables',
    themes: {
      light: { colors: { primary: '#3b82f6' } },
      dark: { colors: { primary: '#60a5fa' } },
    },
  },
  minify: true,
  autoprefixer: true,
  darkMode: { strategy: 'media' },
};
```

## Tailwind Migration

```bash
euis migrate --config tailwind.config.js --input src/styles.css --output src/styles.euis
```

Converts Tailwind config (colors, spacing, fonts, breakpoints) to Euis tokens, and transforms @tailwind/@apply directives.

## CLI

```bash
euis build input.euis -o output.css          # Compile
euis build input.euis -o output.css --minify  # Minified
euis build input.euis -o output.css --analyze # Bundle analysis
euis watch input.euis -o output.css           # Watch mode
euis format input.euis --write                # Format
euis tokens tokens.json --platform css        # W3C Design Tokens
```

## Performance

| Framework | Time | Output (100 used) |
|-----------|------|--------------------|
| **Euis** | **1.73ms** | **6.6KB** |
| UnoCSS | 2.33ms | 3.42KB |
| Tailwind CSS | 16.75ms | 11.52KB |

## Packages

| Package | Description |
|---------|-------------|
| `@euis/cli` | CLI |
| `@euis/wasm` | WASM compiler |
| `vite-plugin-euis` | Vite plugin |
| `next-euis` | Next.js plugin |
| `euis-loader` | Webpack loader |
| `vscode-euis` | VS Code extension |

## License

MIT
