# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-04-08

### Added
- Support for standalone pseudo-class selectors (`:root`, `:is()`, `:where()`)
- Descendant combinator support (`.a .b`)
- CSS nesting with `& .child` pattern
- `@page` at-rule support for print styles
- Nested `@media` inside `@media` at top level
- Comma-separated `@layer` declarations (`@layer base, components, utilities;`)
- Balanced parentheses handling in value parser (`url()`, `rgba()`, etc)
- Multi-token value support (font-family, background shorthand, grid-template-areas)

### Fixed
- Value parser now correctly handles complex multi-token values
- Font-family with comma-separated quoted values
- Background shorthand with `url()` function
- Grid-template-areas with multiple quoted strings
- Aspect-ratio with slash notation
- Shorthand properties (margin, padding, border, flex)
- Multi-value properties (box-shadow, text-shadow)

### Changed
- Enhanced value parser to respect balanced parentheses and quoted strings
- Improved selector parser to handle more CSS patterns
- Updated state block parser to distinguish between `&:pseudo` and `& .child`

### Removed
- Temporary test files (stress_test.rs, debug_values.rs)
- Obsolete documentation files (LOVABLE, STACKBLITZ, CODESANDBOX testing docs)

## [0.4.0] - Previous Release

### Added
- Tailwind CSS v3 and v4 directive support
- W3C Design Tokens compilation
- Automatic content scanning for tree-shaking
- CSS Modules support
- Vendor prefixing
- Dark mode support
- Parallel processing with Rayon
- Rich diagnostics with error codes

## [0.3.0] - Earlier Release

### Added
- Initial WCSS compiler implementation
- WebAssembly build
- CLI tool
- Vite plugin
- Basic CSS parsing and compilation

[0.5.0]: https://github.com/huxeinsatoru/WCSS/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/huxeinsatoru/WCSS/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/huxeinsatoru/WCSS/releases/tag/v0.3.0
