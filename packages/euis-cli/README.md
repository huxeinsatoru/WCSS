# euis-cli

Command-line interface for Euis (Euis) - a fast CSS compiler built with Rust and WebAssembly.

**1.35x faster than UnoCSS, 9.68x faster than Tailwind CSS** with full CSS spec support, plugin system, vendor prefixing, and dark mode.

## Installation

```bash
npm install -g euis-cli
```

## Quick Start

Create a Euis file:

```euis
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

Compile to CSS:

```bash
euis build styles.euis -o output.css
```

## Why Euis?

- **Blazing fast**: 1.73ms for 5000 rules with tree-shaking
- **Full CSS spec**: All selectors, at-rules, 40+ pseudo-classes, 55+ units
- **Plugin system**: 6 hooks for custom transformations
- **Vendor prefixing**: Automatic -webkit-, -moz-, -ms- for 30+ properties
- **Dark mode**: Built-in support (media, class, attribute strategies)
- **Modern colors**: oklch, oklab, color-mix(), light-dark()
- **Tree shaking**: 98% size reduction (340KB → 6.6KB)
- **W3C Design Tokens**: Multi-platform code generation

## Commands

### Build

Compile Euis to CSS:

```bash
euis build <input> -o <output>
euis build styles.euis -o dist/styles.css
```

Options:
- `-o, --output <file>` - Output file path
- `--minify` - Minify output CSS
- `--sourcemap` - Generate source maps

### Watch

Watch for changes and recompile:

```bash
euis watch <input> -o <output>
euis watch styles.euis -o dist/styles.css
```

### Format

Format Euis files:

```bash
euis format <input> --write
euis format styles.euis --write
```

Options:
- `--write` - Write formatted output to file
- `--check` - Check if files are formatted

### Tokens

Generate code from W3C Design Tokens:

```bash
euis tokens <input> --platform <platform> -o <output>
```

Supported platforms:
- `css` - CSS custom properties
- `ios` - Swift code for iOS
- `android` - XML resources for Android
- `flutter` - Dart code for Flutter
- `typescript` - TypeScript constants
- `docs` - Markdown documentation

Examples:

```bash
# Generate CSS
euis tokens tokens.json --platform css -o tokens.css

# Generate iOS Swift
euis tokens tokens.json --platform ios -o DesignTokens.swift

# Generate Android XML
euis tokens tokens.json --platform android -o res/values/

# Generate Flutter Dart
euis tokens tokens.json --platform flutter -o design_tokens.dart

# Generate TypeScript
euis tokens tokens.json --platform typescript -o tokens.ts

# Generate documentation
euis tokens tokens.json --platform docs -o docs/
```

## W3C Design Tokens

Euis supports the [W3C Design Tokens Community Group](https://design-tokens.github.io/community-group/format/) format.

Example `tokens.json`:

```json
{
  "color": {
    "primary": {
      "$value": "#3b82f6",
      "$type": "color",
      "$description": "Primary brand color"
    },
    "secondary": {
      "$value": "#8b5cf6",
      "$type": "color"
    }
  },
  "spacing": {
    "base": {
      "$value": "16px",
      "$type": "dimension"
    },
    "large": {
      "$value": "24px",
      "$type": "dimension"
    }
  }
}
```

## Configuration

Create `euis.config.js`:

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
  minify: true,
  sourceMaps: false,
  treeShaking: true,
};
```

Use tokens in Euis:

```euis
.card {
  padding: $spacing.md;
  background: $colors.primary;
}
```

## Features

- **Fast compilation** - 1.73ms for 5000 rules with tree-shaking
- **Full CSS spec** - All selectors, at-rules, 40+ pseudo-classes, 55+ units
- **Tree shaking** - 98% size reduction (340KB → 6.6KB)
- **Plugin system** - 6 hooks for custom transformations
- **Vendor prefixing** - Automatic -webkit-, -moz-, -ms- for 30+ properties
- **Dark mode** - Built-in support (media, class, attribute strategies)
- **Modern colors** - oklch, oklab, color-mix(), light-dark()
- **W3C Design Tokens** - Multi-platform code generation
- **Zero runtime** - Pure CSS output
- **Watch mode** - Auto-recompile on changes
- **Source maps** - Debug support

## Language Features

### Full CSS Selector Support

```euis
/* Element, class, ID */
button { }
.button { }
#submit { }

/* Attribute selectors */
[type="text"] { }
[class^="btn-"] { }
[href$=".pdf"] { }

/* Pseudo-classes (40+) */
:hover, :focus, :active { }
:first-child, :nth-child(2n) { }
:not(.disabled), :is(.primary) { }
:has(> img), :where(.card) { }
```

### At-Rules

```euis
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

/* Layers, supports, import, font-face */
@layer base, components;
@supports (display: grid) { }
@import "base.css";
@font-face { }
```

### Modern Colors

```euis
.element {
  /* OKLCH - perceptually uniform */
  color: oklch(70% 0.15 180);
  
  /* Color mixing */
  background: color-mix(in oklch, blue 50%, red);
  
  /* Light/dark mode */
  border: 1px solid light-dark(#ccc, #333);
}
```

### Dark Mode

```euis
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

```euis
.box {
  transform: scale(1.2);
  /* Generates: -webkit-transform, -moz-transform, -ms-transform, transform */
  
  user-select: none;
  backdrop-filter: blur(10px);
}
```

### States

```euis
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

```euis
.card {
  padding: 1rem;
  
  @media (min-width: 768px) {
    padding: 1.5rem;
  }
}
```

### Token References

```euis
.button {
  padding: $spacing.md;
  background: $colors.primary;
  color: $colors.white;
}
```

## Framework Integration

Euis integrates with popular frameworks:

- **Vite**: `vite-plugin-euis`
- **Next.js**: `next-euis`
- **Webpack**: `euis-loader`
- **Astro**: `astro-euis`
- **Nuxt**: `nuxt-euis`

## Performance

Benchmarked on Apple M3 with 5000 utility classes:

| Framework | Time | Output (100 used) | Speed vs Euis |
|-----------|------|------------------|---------------|
| **Euis (tree-shaking)** | **1.73ms** | **6.6KB** | **1.0x (baseline)** |
| UnoCSS | 2.33ms | 3.42KB | 1.35x slower |
| Euis (full) | 4.95ms | 340KB | 2.86x slower |
| Tailwind CSS | 16.75ms | 11.52KB | 9.68x slower |
| Panda CSS | ~17ms | ~12KB | 9.83x slower |

**Key metrics:**
- 98% size reduction with tree-shaking (340KB → 6.6KB)
- 1.35x faster than UnoCSS
- 9.68x faster than Tailwind CSS

See [BENCHMARK_RESULTS.md](https://github.com/huxeinsatoru/Euis/blob/main/BENCHMARK_RESULTS.md) for detailed results.

## Requirements

- Node.js >= 16.0.0

## License

MIT

## Links

- [GitHub Repository](https://github.com/huxeinsatoru/Euis)
- [W3C Design Tokens Spec](https://design-tokens.github.io/community-group/format/)
- [NPM Package](https://www.npmjs.com/package/euis-cli)
