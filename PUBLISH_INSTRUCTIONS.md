# Panduan Publish Euis ke npm

## Status Rebranding
✅ Semua file sudah diganti dari WCSS ke Euis
✅ Repository GitHub sudah di-rename ke `Euis`
✅ 344 tests passing
✅ Build release berhasil

## Package yang Perlu Dipublish

### 1. euis-wasm (PRIORITAS PERTAMA)
```bash
cd pkg/bundler
npm publish --access public --otp=<kode-dari-authenticator>
```

### 2. vite-plugin-euis
```bash
cd packages/vite-plugin-euis
npm run build
npm publish --access public --otp=<kode-dari-authenticator>
```

### 3. euis-cli
```bash
cd packages/euis-cli
npm run build
npm publish --access public --otp=<kode-dari-authenticator>
```

### 4. next-euis
```bash
cd packages/next-euis
npm install
npm run build
npm publish --access public --otp=<kode-dari-authenticator>
```

## Verifikasi Setelah Publish

```bash
npm view euis-wasm version
npm view vite-plugin-euis version
npm view euis-cli version
npm view next-euis version
```

## Test Install

```bash
mkdir /tmp/test-euis
cd /tmp/test-euis
npm init -y
npm install euis-wasm@latest
npm install vite-plugin-euis@latest
npm install -g euis-cli@latest
```

## Catatan Penting

- Semua package memerlukan OTP karena akun npm Anda menggunakan 2FA
- Publish euis-wasm TERLEBIH DAHULU karena package lain depend padanya
- Tunggu 2-3 menit setelah publish euis-wasm sebelum publish yang lain
- Versi saat ini: 0.5.0

## Breaking Changes dari WCSS

Beritahu user tentang breaking changes:
- Package name: `@wcss/wasm` → `euis-wasm`
- Package name: `vite-plugin-wcss` → `vite-plugin-euis`
- Package name: `wcss-cli` → `euis-cli`
- Package name: `next-wcss` → `next-euis`
- File extension: `.wcss` → `.euis`
- CLI command: `wcss` → `euis`
- Config file: `wcss.config.js` → `euis.config.js`

## Migration Guide untuk User

```bash
# Uninstall old packages
npm uninstall @wcss/wasm vite-plugin-wcss wcss-cli next-wcss

# Install new packages
npm install euis-wasm vite-plugin-euis
npm install -g euis-cli
npm install next-euis

# Rename files
find . -name "*.wcss" -exec rename 's/\.wcss$/.euis/' {} \;

# Update imports
sed -i 's/@wcss\/wasm/euis-wasm/g' **/*.{js,ts}
sed -i 's/vite-plugin-wcss/vite-plugin-euis/g' **/*.{js,ts}
sed -i 's/next-wcss/next-euis/g' **/*.{js,ts}

# Rename config
mv wcss.config.js euis.config.js

# Update CLI commands
# wcss build → euis build
# wcss watch → euis watch
```
