# Panduan Publish ke npm

## 📦 Package yang Siap Dipublish

### 1. @wcss/wasm (v0.5.0) - WASM Core
**Status**: ✅ Siap publish
**Lokasi**: `pkg/bundler/`
**Prioritas**: TINGGI (dependency untuk package lain)

### 2. vite-plugin-wcss (v0.6.0) - Vite Plugin
**Status**: ✅ Siap publish
**Lokasi**: `packages/vite-plugin-wcss/`
**Prioritas**: TINGGI (plugin utama)

### 3. wcss-cli (v0.5.0) - CLI Tool
**Status**: ✅ Siap publish
**Lokasi**: `packages/wcss-cli/`
**Prioritas**: SEDANG

### 4. astro-wcss (v0.1.0) - Astro Integration
**Status**: ✅ Siap publish
**Lokasi**: `packages/astro-wcss/`
**Prioritas**: RENDAH (opsional)

### 5. Package Lainnya
- next-wcss (v0.1.0)
- wcss-loader (v0.1.0)
- wcss-cdn (v0.1.0)

## 🚀 Langkah-langkah Publish

### Persiapan

1. **Login ke npm**
```bash
npm login
# Masukkan username, password, dan email npm Anda
```

2. **Verifikasi login**
```bash
npm whoami
```

### Publish @wcss/wasm (PRIORITAS PERTAMA)

Package ini adalah dependency untuk package lain, jadi harus dipublish terlebih dahulu.

```bash
cd pkg/bundler

# Verifikasi isi package
npm pack --dry-run

# Publish
npm publish --access public

cd ../..
```

**Catatan**: Jika belum build WASM, jalankan dulu:
```bash
# Install wasm target (jika belum)
rustup target add wasm32-unknown-unknown

# Build WASM
npm run build:wasm
```

### Publish vite-plugin-wcss

```bash
cd packages/vite-plugin-wcss

# Build TypeScript
npm run build

# Verifikasi isi package
npm pack --dry-run

# Publish
npm publish --access public

cd ../..
```

### Publish wcss-cli

```bash
cd packages/wcss-cli

# Build TypeScript
npm run build

# Verifikasi isi package
npm pack --dry-run

# Publish
npm publish --access public

cd ../..
```

### Publish astro-wcss (Opsional)

```bash
cd packages/astro-wcss

# Build TypeScript
npm run build

# Verifikasi isi package
npm pack --dry-run

# Publish
npm publish --access public

cd ../..
```

## 🔍 Verifikasi Publikasi

### Cek di npm Registry

```bash
# Cek @wcss/wasm
npm view @wcss/wasm version
npm view @wcss/wasm

# Cek vite-plugin-wcss
npm view vite-plugin-wcss version
npm view vite-plugin-wcss

# Cek wcss-cli
npm view wcss-cli version
npm view wcss-cli
```

### Test Install

```bash
# Test install di direktori temporary
mkdir /tmp/test-wcss-install
cd /tmp/test-wcss-install

npm init -y
npm install @wcss/wasm@latest
npm install vite-plugin-wcss@latest
npm install -g wcss-cli@latest

# Verifikasi
wcss --version

cd -
rm -rf /tmp/test-wcss-install
```

## ⚠️ Troubleshooting

### Error: "You cannot publish over the previously published versions"

**Solusi**: Versi sudah ada di npm. Increment versi di package.json:
```bash
# Edit package.json, ubah version
# Contoh: "0.5.0" -> "0.5.1"
```

### Error: "You must be logged in to publish packages"

**Solusi**: Login ke npm:
```bash
npm login
```

### Error: "You do not have permission to publish"

**Solusi**: 
1. Pastikan Anda adalah owner/maintainer package
2. Atau gunakan `--access public` untuk scoped packages (@wcss/wasm)

### Error: "failed to verify package tarball"

**Solusi**: Pastikan semua file yang diperlukan ada:
```bash
# Cek files field di package.json
# Pastikan dist/ folder sudah di-build
npm run build
```

### WASM Build Error: "wasm32-unknown-unknown target not found"

**Solusi**: Install wasm target:
```bash
rustup target add wasm32-unknown-unknown
```

Jika menggunakan Homebrew Rust (bukan rustup):
```bash
# Uninstall Homebrew Rust
brew uninstall rust

# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm target
rustup target add wasm32-unknown-unknown
```

## 📋 Checklist Publish

- [ ] Login ke npm (`npm login`)
- [ ] Build WASM (`npm run build:wasm`)
- [ ] Publish @wcss/wasm
- [ ] Tunggu 2-3 menit untuk propagasi
- [ ] Build vite-plugin-wcss (`cd packages/vite-plugin-wcss && npm run build`)
- [ ] Publish vite-plugin-wcss
- [ ] Build wcss-cli (`cd packages/wcss-cli && npm run build`)
- [ ] Publish wcss-cli
- [ ] Verifikasi dengan `npm view`
- [ ] Test install di environment baru
- [ ] Update dokumentasi jika perlu

## 🔗 Links

- npm Registry: https://www.npmjs.com/
- @wcss/wasm: https://www.npmjs.com/package/@wcss/wasm
- vite-plugin-wcss: https://www.npmjs.com/package/vite-plugin-wcss
- wcss-cli: https://www.npmjs.com/package/wcss-cli

## 💡 Tips

1. **Dry run terlebih dahulu**: Gunakan `npm pack --dry-run` untuk melihat file apa saja yang akan dipublish
2. **Test lokal**: Test package dengan `npm link` sebelum publish
3. **Semantic versioning**: Ikuti semver (major.minor.patch)
4. **Changelog**: Update CHANGELOG.md sebelum publish
5. **Git tag**: Buat git tag setelah publish berhasil

## 📝 Catatan Penting

- **@wcss/wasm** harus dipublish PERTAMA karena menjadi dependency package lain
- Tunggu 2-3 menit setelah publish @wcss/wasm sebelum publish package yang depend padanya
- Gunakan `--access public` untuk scoped packages (@wcss/*)
- Versi di package.json harus lebih tinggi dari versi yang sudah ada di npm
- Pastikan semua test passing sebelum publish

## 🎯 Urutan Publish yang Benar

1. @wcss/wasm (v0.5.0) ← Dependency utama
2. vite-plugin-wcss (v0.6.0) ← Depends on @wcss/wasm
3. wcss-cli (v0.5.0) ← Independent
4. astro-wcss (v0.1.0) ← Depends on vite-plugin-wcss
5. Package lainnya (opsional)
