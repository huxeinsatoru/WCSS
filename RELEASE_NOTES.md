# WCSS v0.5.0 Release Notes

## 🎉 Comprehensive CSS Parser Improvements

Version 0.5.0 brings major enhancements to the CSS parser, fixing 10 critical bugs and adding support for modern CSS patterns.

## ✨ New Features

### Enhanced CSS Selector Support
- **Standalone pseudo-class selectors**: `:root`, `:is()`, `:where()` now work as top-level selectors
- **Descendant combinators**: `.parent .child` syntax fully supported
- **CSS nesting improvements**: `& .child` pattern now works correctly alongside `&:pseudo`

### New At-Rules
- **@page support**: Full support for print styles with `@page` at-rule
- **Nested @media**: `@media` inside `@media` at top level now works
- **Comma-separated @layer**: `@layer base, components, utilities;` syntax supported

### Robust Value Parser
- **Multi-token values**: Font-family, background shorthand, grid-template-areas
- **Balanced parentheses**: `url()`, `rgba()`, `calc()` functions handled correctly
- **Quoted strings**: Multiple quoted values in properties work properly
- **Complex shorthands**: All CSS shorthand properties now parse correctly

## 🐛 Bug Fixes

1. ✅ Font-family with comma-separated quoted values
2. ✅ Background shorthand with `url()` function
3. ✅ Grid-template-areas with multiple quoted strings
4. ✅ Aspect-ratio with slash notation
5. ✅ Shorthand properties (margin, padding, border, flex)
6. ✅ Multi-value properties (box-shadow, text-shadow)
7. ✅ Descendant selectors (`.a .b`)
8. ✅ Standalone pseudo-classes (`:root`, `:is()`, `:where()`)
9. ✅ CSS nesting with `& .child`
10. ✅ Nested `@media` at top level

## 🧹 Code Cleanup

- Removed 11 obsolete documentation files
- Deleted temporary test files
- Cleaned up development artifacts
- Improved code organization

## 📊 Test Coverage

- **244 unit tests** passing
- All property-based tests passing
- Integration tests verified
- Zero regressions

## 🚀 Performance

No performance regressions. All benchmarks maintain previous performance:
- 1.73ms compilation time for 5000 rules with tree-shaking
- 98% size reduction with tree-shaking enabled

## 📦 What's Included

### Rust Crates
- `wcss-compiler` v0.5.0 - Core compiler
- `wcss-cli` v0.5.0 - Command-line interface
- `wcss-wasm` v0.5.0 - WebAssembly build

### npm Packages
- `vite-plugin-wcss` v0.6.0 - Vite plugin
- `@wcss/wasm` v0.4.0 - WASM package
- Other framework integrations

## 📝 Documentation

- Updated README.md with latest features
- Added CHANGELOG.md for version history
- Created PUBLISHING.md for maintainers

## 🔗 Links

- **GitHub**: https://github.com/huxeinsatoru/WCSS
- **Documentation**: See README.md
- **Issues**: https://github.com/huxeinsatoru/WCSS/issues

## 🙏 Acknowledgments

Thanks to all contributors and users who reported issues and provided feedback!

## 📅 Release Date

April 8, 2026

---

For detailed changes, see [CHANGELOG.md](./CHANGELOG.md)
For publishing instructions, see [PUBLISHING.md](./PUBLISHING.md)
