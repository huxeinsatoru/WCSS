# Publishing Guide

Panduan untuk mempublikasikan WCSS ke GitHub dan npm.

## Persiapan

### 1. Verifikasi Semua Test Passing

```bash
# Test Rust
cargo test --workspace

# Test JavaScript
npm test

# Build release
cargo build --release
```

### 2. Update Versi

Versi saat ini: **0.5.0**

Jika perlu update versi, edit file berikut:
- `Cargo.toml` (workspace.package.version)
- `crates/wcss-compiler/Cargo.toml`
- `crates/wcss-cli/Cargo.toml`
- `crates/wcss-wasm/Cargo.toml`
- `packages/vite-plugin-wcss/package.json`
- `packages/*/package.json` (untuk package lainnya)

### 3. Update CHANGELOG.md

Pastikan semua perubahan terdokumentasi di `CHANGELOG.md`.

## Publish ke GitHub

### 1. Push ke Repository

```bash
# Pastikan semua perubahan sudah di-commit
git status

# Push ke GitHub
git push origin main

# Buat tag untuk release
git tag -a v0.5.0 -m "Release v0.5.0: Comprehensive CSS parser improvements"
git push origin v0.5.0
```

### 2. Buat GitHub Release

1. Buka https://github.com/huxeinsatoru/WCSS/releases
2. Klik "Draft a new release"
3. Pilih tag `v0.5.0`
4. Judul: "v0.5.0 - Comprehensive CSS Parser Improvements"
5. Deskripsi: Copy dari CHANGELOG.md
6. Klik "Publish release"

## Publish ke npm

### 1. Login ke npm

```bash
npm login
```

### 2. Build WASM Package

```bash
# Build untuk bundler (browser)
npm run build:wasm

# Build untuk Node.js
npm run build:wasm:nodejs
```

### 3. Publish @wcss/wasm

```bash
cd pkg/bundler
npm publish --access public
cd ../..
```

### 4. Publish vite-plugin-wcss

```bash
cd packages/vite-plugin-wcss
npm run build
npm publish --access public
cd ../..
```

### 5. Publish Package Lainnya (Opsional)

```bash
# Next.js plugin
cd packages/next-wcss
npm run build
npm publish --access public
cd ../..

# Webpack loader
cd packages/wcss-loader
npm run build
npm publish --access public
cd ../..

# Astro integration
cd packages/astro-wcss
npm run build
npm publish --access public
cd ../..
```

## Publish ke crates.io (Rust)

### 1. Login ke crates.io

```bash
cargo login
```

### 2. Publish Crates

```bash
# Publish compiler (dependency untuk yang lain)
cd crates/wcss-compiler
cargo publish
cd ../..

# Tunggu beberapa menit untuk propagasi

# Publish CLI
cd crates/wcss-cli
cargo publish
cd ../..

# Publish WASM
cd crates/wcss-wasm
cargo publish
cd ../..
```

## Verifikasi Publikasi

### npm

```bash
# Cek versi terbaru
npm view vite-plugin-wcss version
npm view @wcss/wasm version

# Test install
npm install -g vite-plugin-wcss
```

### crates.io

```bash
# Cek di https://crates.io/crates/wcss-compiler
# Cek di https://crates.io/crates/wcss-cli
```

### GitHub

```bash
# Cek release di https://github.com/huxeinsatoru/WCSS/releases
# Cek tag di https://github.com/huxeinsatoru/WCSS/tags
```

## Checklist Publikasi

- [ ] Semua test passing (`cargo test --workspace && npm test`)
- [ ] Build release berhasil (`cargo build --release`)
- [ ] CHANGELOG.md sudah diupdate
- [ ] Versi sudah diupdate di semua file
- [ ] Commit dan push ke GitHub
- [ ] Tag release dibuat (`git tag v0.5.0`)
- [ ] GitHub Release dibuat
- [ ] WASM package di-build
- [ ] @wcss/wasm dipublish ke npm
- [ ] vite-plugin-wcss dipublish ke npm
- [ ] wcss-compiler dipublish ke crates.io
- [ ] wcss-cli dipublish ke crates.io
- [ ] Verifikasi instalasi dari npm dan crates.io

## Troubleshooting

### Error: "You cannot publish over the previously published versions"

Solusi: Increment versi di package.json atau Cargo.toml

### Error: "failed to verify package tarball"

Solusi: Pastikan semua file yang diperlukan ada di `files` field di package.json

### Error: "crate not found in registry"

Solusi: Tunggu beberapa menit setelah publish untuk propagasi

## Catatan

- Selalu test di environment lokal sebelum publish
- Gunakan `npm publish --dry-run` untuk preview sebelum publish
- Gunakan `cargo publish --dry-run` untuk preview sebelum publish
- Backup kode sebelum publish
- Dokumentasikan breaking changes di CHANGELOG.md
