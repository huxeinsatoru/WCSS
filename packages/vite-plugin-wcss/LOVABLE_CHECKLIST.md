# Lovable Testing Checklist

Quick reference checklist for testing vite-plugin-wcss in Lovable (formerly GPT Engineer).

## Pre-Test Setup

- [ ] Lovable account created
- [ ] Modern browser (Chrome/Firefox/Edge/Safari)
- [ ] New Vite project created in Lovable

## Installation (Requirement 3.1)

- [ ] Run `npm install vite-plugin-wcss` or ask AI to install
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
- [ ] Import in component: `import styles from './styles.wcss'`
- [ ] Inject styles into page
- [ ] Verify button appears with correct styles
- [ ] Check console for compiled CSS (optional)
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
- [ ] Confirm component state preserved (if applicable)

**Expected:** ✅ HMR updates styles instantly

## Test 3: Error Handling

- [ ] Introduce syntax error (undefined token)
- [ ] Save file
- [ ] Verify error overlay appears in browser
- [ ] Check error message includes line/column
- [ ] Verify terminal shows error
- [ ] Confirm page remains functional
- [ ] Fix the error
- [ ] Verify error overlay disappears
- [ ] Verify styles update correctly

**Expected:** ✅ Errors display correctly, recovery works

## Test 4: Multiple Files

- [ ] Create second file `src/components.wcss`
- [ ] Define different styles (e.g., `.card`)
- [ ] Import both files in component
- [ ] Inject both stylesheets
- [ ] Verify both sets of styles apply
- [ ] Check no conflicts between files
- [ ] Test HMR on each file independently

**Expected:** ✅ Multiple .wcss files work together

## Test 5: AI-Assisted Workflow (Lovable-Specific)

- [ ] Ask AI to create new component with .wcss file
- [ ] Verify AI generates valid WCSS syntax
- [ ] Ask AI to modify existing .wcss file
- [ ] Verify AI handles token references correctly
- [ ] Ask AI to fix styling issues
- [ ] Verify AI suggestions compile without errors
- [ ] Test AI understanding of WCSS concepts

**Expected:** ✅ Lovable AI works seamlessly with WCSS

## Test 6: TypeScript Support

- [ ] Ensure `src/vite-env.d.ts` includes WCSS types
- [ ] Create TypeScript component importing .wcss
- [ ] Verify no TypeScript errors
- [ ] Check autocomplete works for .wcss imports
- [ ] Verify `styles` typed as `string`
- [ ] Confirm type checking passes

**Expected:** ✅ TypeScript integration works

## Test 7: Production Build (Requirement 3.6)

- [ ] Run `npm run build`
- [ ] Check build completes without errors
- [ ] Verify CSS files in `dist/assets`
- [ ] Verify NO `.wasm` files in `dist`
- [ ] Run `npm run preview`
- [ ] Verify preview looks identical to dev
- [ ] Check CSS is properly compiled
- [ ] Verify reasonable build time

**Expected:** ✅ Production build generates CSS only

## Test 8: Configuration Options

- [ ] Enable `minify: true` in config
- [ ] Enable `deduplicate: true`
- [ ] Disable `sourceMaps: false`
- [ ] Rebuild project
- [ ] Check generated CSS is minified
- [ ] Verify no source map files
- [ ] Confirm smaller file size

**Expected:** ✅ Configuration options work

## Test 9: Complex Scenarios (Optional)

- [ ] Create component with nested styles
- [ ] Test responsive design with breakpoints
- [ ] Verify hover/active/focus states
- [ ] Test pseudo-classes and pseudo-elements
- [ ] Check media queries work correctly
- [ ] Verify complex selectors compile

**Expected:** ✅ Complex styling scenarios work

## Verification Checklist

### Requirements Coverage

- [ ] **Requirement 3.1**: Plugin installs from npm ✅
- [ ] **Requirement 3.2**: .wcss files compile successfully ✅
- [ ] **Requirement 3.5**: HMR functionality works ✅
- [ ] **Requirement 3.6**: Full development workflow functional ✅

### Additional Checks

- [ ] No file system errors
- [ ] No WASM loading errors
- [ ] All resources loaded from node_modules
- [ ] Works in browser environment
- [ ] No local file dependencies
- [ ] AI integration quality is good

## AI Prompting Tips

### Good Prompts ✅

- "Create a .wcss file (not .css) with button styles"
- "Use WCSS tokens for the color values"
- "Update button.wcss to use $colors.primary token"
- "Add TypeScript support for .wcss imports"

### Avoid ❌

- "Create a stylesheet" (too vague)
- "Make it look better" (not specific)
- Assuming AI knows WCSS without context

## Common Issues

### WASM Init Error
```bash
npm install @wcss/wasm
# Restart dev server
```

Or ask AI:
```
Reinstall vite-plugin-wcss and restart the dev server
```

### Module Not Found
```bash
npm list vite-plugin-wcss
npm install vite-plugin-wcss
```

Or ask AI:
```
Fix the vite-plugin-wcss import error
```

### Styles Not Updating
- Hard refresh (Ctrl+Shift+R)
- Restart dev server
- Check terminal for errors

Or ask AI:
```
The WCSS styles aren't updating, can you help debug?
```

### AI Not Understanding WCSS
- Be explicit: "Create a .wcss file"
- Provide examples of correct syntax
- Reference WCSS documentation
- Ask AI to review LOVABLE.md guide

### TypeScript Errors
Add to `vite-env.d.ts`:
```typescript
/// <reference types="vite-plugin-wcss/wcss" />
```

Or ask AI:
```
Add TypeScript support for .wcss imports
```

## Test Result

**Overall Status:** ⬜ Not Started / 🟡 In Progress / ✅ Passed / ❌ Failed

**Critical Issues:** [None / List issues]

**AI Integration Quality:** ⬜ Not Tested / ✅ Excellent / 🟡 Good / ❌ Poor

**Notes:**

---

**Test Date:** ___________
**Tester:** ___________
**Browser:** ___________
**Lovable Project URL:** ___________

## Quick Reference

### Installation Command
```bash
npm install vite-plugin-wcss
```

### Basic Config
```javascript
import wcss from 'vite-plugin-wcss';

export default defineConfig({
  plugins: [wcss({ minify: false, sourceMaps: true })],
});
```

### Basic WCSS File
```wcss
$colors.primary: #3b82f6;

.button {
  background: $colors.primary;
  padding: 1rem;
}
```

### Import in Component
```javascript
import styles from './styles.wcss';
// Use: <style>{styles}</style>
```

### Verify Installation
```bash
npm list vite-plugin-wcss @wcss/wasm
```

### Build Commands
```bash
npm run dev      # Development
npm run build    # Production
npm run preview  # Preview build
```

## Success Criteria Summary

✅ **Must Pass:**
1. Installation works
2. Compilation works
3. HMR works
4. Error handling works
5. Production build works

✅ **Should Pass:**
6. Multiple files work
7. TypeScript works
8. Configuration works
9. AI integration works

✅ **Nice to Have:**
10. Complex scenarios work

---

**For detailed instructions, see [LOVABLE_TESTING.md](./LOVABLE_TESTING.md)**
