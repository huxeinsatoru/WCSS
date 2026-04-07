# StackBlitz Testing Guide for vite-plugin-wcss

This guide provides step-by-step instructions for testing vite-plugin-wcss in StackBlitz, a browser-based development environment.

## Prerequisites

- A StackBlitz account (free tier is sufficient)
- Modern browser (Chrome, Firefox, Safari, or Edge)
- Basic knowledge of Vite and WCSS

## Test Objectives

This test verifies that vite-plugin-wcss works correctly in StackBlitz by validating:

1. **Installation**: Plugin and dependencies install correctly from npm
2. **Compilation**: .wcss files compile to CSS successfully
3. **HMR**: Hot Module Replacement updates styles without page reload
4. **Error Handling**: Compilation errors display correctly in the error overlay
5. **Production Build**: Production builds generate optimized CSS

## Test Setup

### Step 1: Create a New Vite Project in StackBlitz

1. Go to [StackBlitz](https://stackblitz.com/)
2. Click "New Project"
3. Select "Vite" → "Vanilla" (or "React" if you prefer)
4. Wait for the project to initialize

### Step 2: Install vite-plugin-wcss

Open the terminal in StackBlitz and run:

```bash
npm install vite-plugin-wcss
```

**Expected Result:**
- Installation completes without errors
- Both `vite-plugin-wcss` and `@wcss/wasm` appear in `package.json` dependencies
- `node_modules` folder contains both packages

**Verification:**
```bash
npm list vite-plugin-wcss @wcss/wasm
```

Should show both packages installed.

### Step 3: Configure Vite

Create or update `vite.config.js` in the project root:

```javascript
import { defineConfig } from 'vite';
import wcss from 'vite-plugin-wcss';

export default defineConfig({
  plugins: [
    wcss({
      minify: false,
      sourceMaps: true,
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6',
          success: '#10b981',
          danger: '#ef4444',
        },
        spacing: {
          sm: '0.5rem',
          md: '1rem',
          lg: '1.5rem',
          xl: '2rem',
        },
      },
    }),
  ],
});
```

**Expected Result:**
- File saves without errors
- Dev server restarts automatically

## Test Cases

### Test Case 1: Basic Compilation

**Objective:** Verify that .wcss files compile to CSS

**Steps:**

1. Create a new file `src/styles.wcss`:

```wcss
/* Test basic compilation */
$colors.primary: #3b82f6;
$spacing.md: 1rem;

.test-button {
  background: $colors.primary;
  padding: $spacing.md;
  border-radius: 4px;
  color: white;
  border: none;
  cursor: pointer;
}

.test-button:hover {
  opacity: 0.9;
}
```

2. Import the WCSS file in `src/main.js`:

```javascript
import styles from './styles.wcss';

// Inject styles
const styleElement = document.createElement('style');
styleElement.textContent = styles;
document.head.appendChild(styleElement);

// Create test button
document.querySelector('#app').innerHTML = `
  <div>
    <h1>WCSS StackBlitz Test</h1>
    <button class="test-button">Test Button</button>
  </div>
`;

console.log('Compiled CSS:', styles);
```

3. Check the browser preview

**Expected Result:**
- ✅ No compilation errors in terminal
- ✅ Button appears with blue background
- ✅ Console shows compiled CSS string
- ✅ Compiled CSS contains expanded token values (not `$colors.primary`)
- ✅ Button hover effect works

**Verification:**
- Inspect the button element in DevTools
- Check that styles are applied correctly
- Verify CSS contains actual color values, not token references

### Test Case 2: Hot Module Replacement (HMR)

**Objective:** Verify that editing .wcss files triggers HMR

**Steps:**

1. With the dev server running, edit `src/styles.wcss`
2. Change the primary color:

```wcss
$colors.primary: #ef4444; /* Changed from blue to red */
```

3. Save the file
4. Observe the browser preview

**Expected Result:**
- ✅ Button background changes from blue to red
- ✅ No full page reload occurs
- ✅ Change happens within 1-2 seconds
- ✅ No errors in console or terminal

**Verification:**
- Check that the page doesn't reload (any console logs remain)
- Verify the button color changed to red
- Check terminal for HMR update message

### Test Case 3: Compilation Error Handling

**Objective:** Verify that syntax errors are displayed correctly

**Steps:**

1. Edit `src/styles.wcss` to introduce a syntax error:

```wcss
.test-button {
  background: $colors.undefined-token; /* Undefined token */
  padding: $spacing.md;
}
```

2. Save the file
3. Check the browser preview and terminal

**Expected Result:**
- ✅ Error overlay appears in browser
- ✅ Error message indicates undefined token
- ✅ Error includes line and column number
- ✅ Terminal shows compilation error
- ✅ Previous styles remain applied (page doesn't break)

**Verification:**
- Read the error message - should mention "undefined-token"
- Check that line/column numbers are present
- Verify the page is still functional (not completely broken)

4. Fix the error:

```wcss
.test-button {
  background: $colors.primary; /* Fixed */
  padding: $spacing.md;
}
```

5. Save the file

**Expected Result:**
- ✅ Error overlay disappears
- ✅ Styles update correctly
- ✅ No errors in console or terminal

### Test Case 4: Multiple WCSS Files

**Objective:** Verify that multiple .wcss files can be imported

**Steps:**

1. Create `src/components.wcss`:

```wcss
.card {
  background: white;
  border-radius: 8px;
  padding: $spacing.lg;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.card-title {
  font-size: 1.5rem;
  font-weight: bold;
  margin-bottom: $spacing.sm;
  color: $colors.primary;
}
```

2. Import both files in `src/main.js`:

```javascript
import buttonStyles from './styles.wcss';
import componentStyles from './components.wcss';

// Inject both stylesheets
const styleElement = document.createElement('style');
styleElement.textContent = buttonStyles + '\n' + componentStyles;
document.head.appendChild(styleElement);

// Update HTML
document.querySelector('#app').innerHTML = `
  <div>
    <div class="card">
      <h2 class="card-title">WCSS StackBlitz Test</h2>
      <button class="test-button">Test Button</button>
    </div>
  </div>
`;
```

3. Check the browser preview

**Expected Result:**
- ✅ Both stylesheets compile successfully
- ✅ Card and button styles both apply
- ✅ No conflicts between stylesheets
- ✅ Console shows both compiled CSS strings

### Test Case 5: TypeScript Support (Optional)

**Objective:** Verify TypeScript definitions work correctly

**Steps:**

1. If using TypeScript, create `src/vite-env.d.ts`:

```typescript
/// <reference types="vite/client" />
/// <reference types="vite-plugin-wcss/wcss" />
```

2. Rename `src/main.js` to `src/main.ts`

3. Update imports with type annotations:

```typescript
import styles from './styles.wcss';
// TypeScript should recognize this as string type

const css: string = styles;
console.log('Type check passed:', typeof css === 'string');
```

**Expected Result:**
- ✅ No TypeScript errors
- ✅ Autocomplete works for .wcss imports
- ✅ Type checking passes

### Test Case 6: Production Build

**Objective:** Verify production builds work correctly

**Steps:**

1. Run the production build command:

```bash
npm run build
```

2. Check the `dist` folder
3. Preview the production build:

```bash
npm run preview
```

**Expected Result:**
- ✅ Build completes without errors
- ✅ CSS files are generated in `dist/assets`
- ✅ WASM binary is NOT included in `dist` folder
- ✅ Preview shows the same styles as development
- ✅ CSS is minified (if minify option is enabled)

**Verification:**
- Check `dist/assets` for `.css` files
- Verify no `.wasm` files in `dist`
- Open generated CSS and verify it contains the compiled styles
- Check that the preview looks identical to development

### Test Case 7: Configuration Options

**Objective:** Verify configuration options work correctly

**Steps:**

1. Update `vite.config.js` to enable minification:

```javascript
export default defineConfig({
  plugins: [
    wcss({
      minify: true,
      deduplicate: true,
      sourceMaps: false,
    }),
  ],
});
```

2. Rebuild the project:

```bash
npm run build
```

3. Check the generated CSS in `dist/assets`

**Expected Result:**
- ✅ CSS is minified (no whitespace, single line)
- ✅ No source map files generated
- ✅ Duplicate rules are removed
- ✅ Build completes successfully

## Troubleshooting

### Issue: WASM Initialization Error

**Symptom:**
```
Failed to initialize WCSS WASM compiler
```

**Solution:**
1. Check that `@wcss/wasm` is installed:
   ```bash
   npm list @wcss/wasm
   ```
2. If missing, install it:
   ```bash
   npm install @wcss/wasm
   ```
3. Restart the dev server
4. Clear browser cache and reload

### Issue: Module Not Found Error

**Symptom:**
```
Cannot find module 'vite-plugin-wcss'
```

**Solution:**
1. Verify installation:
   ```bash
   npm list vite-plugin-wcss
   ```
2. Reinstall if needed:
   ```bash
   npm install vite-plugin-wcss
   ```
3. Check that `vite.config.js` has correct import path
4. Restart StackBlitz if the issue persists

### Issue: Styles Not Updating

**Symptom:**
- Changes to .wcss files don't appear in browser

**Solution:**
1. Check terminal for compilation errors
2. Hard refresh browser (Ctrl+Shift+R or Cmd+Shift+R)
3. Restart dev server
4. Check that HMR is enabled in Vite config

### Issue: StackBlitz WebContainers Not Loading

**Symptom:**
- Project fails to start or dependencies don't install

**Solution:**
1. Ensure you're using a modern browser (Chrome, Firefox, Edge)
2. Check StackBlitz status: https://status.stackblitz.com/
3. Try creating a new project
4. Clear browser cache and cookies
5. Try a different browser

## Test Results Template

Use this template to document your test results:

```markdown
## StackBlitz Test Results

**Date:** [Date]
**Tester:** [Your Name]
**Browser:** [Browser Name and Version]
**StackBlitz Project:** [Project URL]

### Test Case Results

| Test Case | Status | Notes |
|-----------|--------|-------|
| 1. Basic Compilation | ✅ Pass / ❌ Fail | |
| 2. Hot Module Replacement | ✅ Pass / ❌ Fail | |
| 3. Error Handling | ✅ Pass / ❌ Fail | |
| 4. Multiple WCSS Files | ✅ Pass / ❌ Fail | |
| 5. TypeScript Support | ✅ Pass / ❌ Fail / ⏭️ Skip | |
| 6. Production Build | ✅ Pass / ❌ Fail | |
| 7. Configuration Options | ✅ Pass / ❌ Fail | |

### Issues Encountered

[List any issues or unexpected behavior]

### Overall Assessment

- [ ] All critical tests passed
- [ ] Plugin works as expected in StackBlitz
- [ ] Ready for production use

### Additional Notes

[Any other observations or comments]
```

## Success Criteria

The test is considered successful if:

1. ✅ Plugin installs without errors
2. ✅ .wcss files compile to valid CSS
3. ✅ HMR updates styles without page reload
4. ✅ Compilation errors display correctly
5. ✅ Multiple .wcss files can be imported
6. ✅ Production builds generate optimized CSS
7. ✅ No WASM files in production bundle
8. ✅ Configuration options work as expected

## Next Steps

After completing this test:

1. Document any issues in the test results template
2. Report bugs or unexpected behavior to the development team
3. Test in other cloud environments (CodeSandbox, Lovable)
4. Share the StackBlitz project URL for reference

## Reference Links

- [StackBlitz](https://stackblitz.com/)
- [vite-plugin-wcss Documentation](./README.md)
- [WCSS Lovable Guide](./LOVABLE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

---

**Happy testing! 🧪**
