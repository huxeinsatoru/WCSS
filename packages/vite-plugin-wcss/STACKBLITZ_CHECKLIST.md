# StackBlitz Testing Checklist

Quick reference checklist for testing vite-plugin-wcss in StackBlitz.

## Pre-Test Setup

- [ ] StackBlitz account created
- [ ] Modern browser (Chrome/Firefox/Edge/Safari)
- [ ] New Vite project created in StackBlitz

## Installation (Requirement 3.1)

- [ ] Run `npm install vite-plugin-wcss`
- [ ] Verify both `vite-plugin-wcss` and `@wcss/wasm` installed
- [ ] Check `package.json` dependencies
- [ ] Run `npm list vite-plugin-wcss @wcss/wasm` to confirm

## Configuration

- [ ] Create/update `vite.config.js`
- [ ] Import `wcss` from `vite-plugin-wcss`
- [ ] Add plugin to `plugins` array
- [ ] Configure basic options (minify, sourceMaps, tokens)
- [ ] Dev server restarts without errors

## Test 1: Basic Compilation (Requirement 3.2)

- [ ] Create `src/styles.wcss` file
- [ ] Define tokens (`$colors.primary`, `$spacing.md`)
- [ ] Create `.test-button` class using tokens
- [ ] Import in `src/main.js`: `import styles from './styles.wcss'`
- [ ] Inject styles into page
- [ ] Verify button appears with correct styles
- [ ] Check console for compiled CSS
- [ ] Verify tokens are expanded (no `$` in output)

**Expected:** ✅ .wcss compiles to CSS successfully

## Test 2: Hot Module Replacement (Requirement 3.5)

- [ ] Edit `src/styles.wcss` while dev server running
- [ ] Change color value (e.g., blue to red)
- [ ] Save file
- [ ] Observe browser preview
- [ ] Verify styles update without page reload
- [ ] Check no console errors
- [ ] Verify update happens within 1-2 seconds

**Expected:** ✅ HMR updates styles instantly

## Test 3: Error Handling

- [ ] Introduce syntax error (undefined token)
- [ ] Save file
- [ ] Verify error overlay appears in browser
- [ ] Check error message includes line/column
- [ ] Verify terminal shows error
- [ ] Fix the error
- [ ] Verify error overlay disappears
- [ ] Verify styles update correctly

**Expected:** ✅ Errors display correctly, recovery works

## Test 4: Multiple Files

- [ ] Create second file `src/components.wcss`
- [ ] Define different styles (e.g., `.card`)
- [ ] Import both files in `src/main.js`
- [ ] Inject both stylesheets
- [ ] Verify both sets of styles apply
- [ ] Check no conflicts between files

**Expected:** ✅ Multiple .wcss files work together

## Test 5: Production Build

- [ ] Run `npm run build`
- [ ] Check build completes without errors
- [ ] Verify CSS files in `dist/assets`
- [ ] Verify NO `.wasm` files in `dist`
- [ ] Run `npm run preview`
- [ ] Verify preview looks identical to dev
- [ ] Check CSS is properly compiled

**Expected:** ✅ Production build generates CSS only

## Test 6: Configuration Options

- [ ] Enable `minify: true` in config
- [ ] Enable `deduplicate: true`
- [ ] Disable `sourceMaps: false`
- [ ] Rebuild project
- [ ] Check generated CSS is minified
- [ ] Verify no source map files

**Expected:** ✅ Configuration options work

## Verification Checklist

### Requirements Coverage

- [ ] **Requirement 3.1**: Plugin installs from npm ✅
- [ ] **Requirement 3.2**: .wcss files compile successfully ✅
- [ ] **Requirement 3.5**: HMR functionality works ✅

### Additional Checks

- [ ] No file system errors
- [ ] No WASM loading errors
- [ ] All resources loaded from node_modules
- [ ] Works in browser environment
- [ ] No local file dependencies

## Common Issues

### WASM Init Error
```bash
npm install @wcss/wasm
# Restart dev server
```

### Module Not Found
```bash
npm list vite-plugin-wcss
npm install vite-plugin-wcss
```

### Styles Not Updating
- Hard refresh (Ctrl+Shift+R)
- Restart dev server
- Check terminal for errors

## Test Result

**Overall Status:** ⬜ Not Started / 🟡 In Progress / ✅ Passed / ❌ Failed

**Critical Issues:** [None / List issues]

**Notes:**

---

**Test Date:** ___________
**Tester:** ___________
**Browser:** ___________
**StackBlitz URL:** ___________
