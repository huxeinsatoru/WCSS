# wcss-cli

Command-line interface for WCSS (Web Compiler Style Sheets) - a fast CSS compiler built with Rust and WebAssembly.

## Installation

```bash
npm install -g wcss-cli
```

## Quick Start

Create a WCSS file:

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

Compile to CSS:

```bash
wcss build styles.wcss -o output.css
```

## Commands

### Build

Compile WCSS to CSS:

```bash
wcss build <input> -o <output>
wcss build styles.wcss -o dist/styles.css
```

Options:
- `-o, --output <file>` - Output file path
- `--minify` - Minify output CSS
- `--sourcemap` - Generate source maps

### Watch

Watch for changes and recompile:

```bash
wcss watch <input> -o <output>
wcss watch styles.wcss -o dist/styles.css
```

### Format

Format WCSS files:

```bash
wcss format <input> --write
wcss format styles.wcss --write
```

Options:
- `--write` - Write formatted output to file
- `--check` - Check if files are formatted

### Tokens

Generate code from W3C Design Tokens:

```bash
wcss tokens <input> --platform <platform> -o <output>
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
wcss tokens tokens.json --platform css -o tokens.css

# Generate iOS Swift
wcss tokens tokens.json --platform ios -o DesignTokens.swift

# Generate Android XML
wcss tokens tokens.json --platform android -o res/values/

# Generate Flutter Dart
wcss tokens tokens.json --platform flutter -o design_tokens.dart

# Generate TypeScript
wcss tokens tokens.json --platform typescript -o tokens.ts

# Generate documentation
wcss tokens tokens.json --platform docs -o docs/
```

## W3C Design Tokens

WCSS supports the [W3C Design Tokens Community Group](https://design-tokens.github.io/community-group/format/) format.

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
  minify: true,
  sourceMaps: false,
  treeShaking: true,
};
```

Use tokens in WCSS:

```wcss
.card {
  padding: $spacing.md;
  background: $colors.primary;
}
```

## Features

- **Fast compilation** - Rust-based compiler with WebAssembly
- **Tree shaking** - Remove unused CSS classes
- **W3C Design Tokens** - Multi-platform code generation
- **Zero runtime** - Pure CSS output
- **Watch mode** - Auto-recompile on changes
- **Source maps** - Debug support

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

### Token References

```wcss
.button {
  padding: $spacing.md;
  background: $colors.primary;
  color: $colors.white;
}
```

## Framework Integration

WCSS integrates with popular frameworks:

- **Vite**: `vite-plugin-wcss`
- **Next.js**: `next-wcss`
- **Webpack**: `wcss-loader`
- **Astro**: `astro-wcss`
- **Nuxt**: `nuxt-wcss`

## Performance

Benchmarked on Apple M3:

| Input | WCSS | LightningCSS | Tailwind CSS |
|-------|------|--------------|--------------|
| Small (1 rule) | 1μs | 94μs | N/A |
| Large (5000 rules) | 7.7ms | 3.4ms | 43ms |
| With Tree Shaking | 2.7ms | ❌ | 43ms |

## Requirements

- Node.js >= 16.0.0

## License

MIT

## Links

- [GitHub Repository](https://github.com/huxeinsatoru/WCSS)
- [W3C Design Tokens Spec](https://design-tokens.github.io/community-group/format/)
- [NPM Package](https://www.npmjs.com/package/wcss-cli)
