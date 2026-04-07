# CodeSandbox Testing Guide for vite-plugin-wcss

This guide provides step-by-step instructions for testing vite-plugin-wcss in CodeSandbox, a browser-based development environment.

## Prerequisites

- A CodeSandbox account (free tier is sufficient)
- Modern browser (Chrome, Firefox, Safari, or Edge)
- Basic knowledge of Vite and WCSS

## Test Objectives

This test verifies that vite-plugin-wcss works correctly in CodeSandbox by validating:

1. **Installation**: Plugin and dependencies install correctly from npm
2. **Compilation**: .wcss files compile to CSS successfully
3. **Production Build**: Production builds generate optimized CSS
4. **Error Handling**: Compilation errors display correctly
5. **Configuration**: Plugin configuration options work as expected

## Test Setup

### Step 1: Create a New Vite Project in CodeSandbox

1. Go to [CodeSandbox](https://codesandbox.io/)
2. Click "Create Sandbox"
3. Select "Vite" → "Vanilla" (or "React" if you prefer)
4. Wait for the project to initialize

### Step 2: Install vite-plugin-wcss

Open the terminal in CodeSandbox and run:

```bash
npm install vite-plugin-wcss
```

**Expected Result:**
- Installation completes without errors
- Both `vite-plugin-wcss` and `@wcss/wasm` appear in `package.json` dependencies
- Dependencies are automatically installed by CodeSandbox

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
    <h1>WCSS CodeSandbox Test</h1>
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

### Test Case 2: File Changes and Recompilation

**Objective:** Verify that editing .wcss files triggers recompilation

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
- ✅ Change appears within a few seconds
- ✅ No errors in console or terminal

**Verification:**
- Verify the button color changed to red
- Check terminal for compilation success message

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
- ✅ Error message appears in browser or terminal
- ✅ Error message indicates undefined token
- ✅ Error includes line and column number
- ✅ Terminal shows compilation error

**Verification:**
- Read the error message - should mention "undefined-token"
- Check that line/column numbers are present

4. Fix the error:

```wcss
.test-button {
  background: $colors.primary; /* Fixed */
  padding: $spacing.md;
}
```

5. Save the file

**Expected Result:**
- ✅ Error clears
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
      <h2 class="card-title">WCSS CodeSandbox Test</h2>
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

### Test Case 5: Production Build

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
- ✅ CSS is properly formatted

**Verification:**
- Check `dist/assets` for `.css` files
- Verify no `.wasm` files in `dist`
- Open generated CSS and verify it contains the compiled styles
- Check that the preview looks identical to development

### Test Case 6: Configuration Options

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
      tokens: {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6',
        },
        spacing: {
          sm: '0.5rem',
          md: '1rem',
        },
      },
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

**Verification:**
- Open the generated CSS file
- Verify it's minified (single line, no extra whitespace)
- Check that no `.map` files exist

### Test Case 7: TypeScript Support (Optional)

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
4. Refresh the browser preview

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
4. Restart the dev server

### Issue: Styles Not Updating

**Symptom:**
- Changes to .wcss files don't appear in browser

**Solution:**
1. Check terminal for compilation errors
2. Hard refresh browser (Ctrl+Shift+R or Cmd+Shift+R)
3. Restart dev server
4. Check that the file is saved properly

### Issue: Build Fails in CodeSandbox

**Symptom:**
- Production build command fails

**Solution:**
1. Check terminal for specific error messages
2. Ensure all dependencies are installed
3. Try clearing node_modules and reinstalling:
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   ```
4. Check CodeSandbox status for any platform issues

### Issue: Preview Not Loading

**Symptom:**
- Browser preview shows blank page or loading spinner

**Solution:**
1. Check browser console for JavaScript errors
2. Verify dev server is running (check terminal)
3. Try refreshing the preview
4. Check that port 5173 (or configured port) is accessible
5. Restart the sandbox if issues persist

## Test Results Template

Use this template to document your test results:

```markdown
## CodeSandbox Test Results

**Date:** [Date]
**Tester:** [Your Name]
**Browser:** [Browser Name and Version]
**CodeSandbox Project:** [Project URL]

### Test Case Results

| Test Case | Status | Notes |
|-----------|--------|-------|
| 1. Basic Compilation | ✅ Pass / ❌ Fail | |
| 2. File Changes | ✅ Pass / ❌ Fail | |
| 3. Error Handling | ✅ Pass / ❌ Fail | |
| 4. Multiple WCSS Files | ✅ Pass / ❌ Fail | |
| 5. Production Build | ✅ Pass / ❌ Fail | |
| 6. Configuration Options | ✅ Pass / ❌ Fail | |
| 7. TypeScript Support | ✅ Pass / ❌ Fail / ⏭️ Skip | |

### Issues Encountered

[List any issues or unexpected behavior]

### Overall Assessment

- [ ] All critical tests passed
- [ ] Plugin works as expected in CodeSandbox
- [ ] Ready for production use

### Additional Notes

[Any other observations or comments]
```

## Success Criteria

The test is considered successful if:

1. ✅ Plugin installs without errors
2. ✅ .wcss files compile to valid CSS
3. ✅ File changes trigger recompilation
4. ✅ Compilation errors display correctly
5. ✅ Multiple .wcss files can be imported
6. ✅ Production builds generate optimized CSS
7. ✅ No WASM files in production bundle
8. ✅ Configuration options work as expected

## CodeSandbox-Specific Notes

### Differences from StackBlitz

- **Container Technology**: CodeSandbox uses Docker containers instead of WebContainers
- **File System**: Full Node.js file system access (more similar to local development)
- **Build Performance**: May be slower than StackBlitz for large projects
- **Hot Reload**: May require manual refresh in some cases

### Best Practices for CodeSandbox

1. **Save Frequently**: CodeSandbox auto-saves, but manual saves ensure changes are processed
2. **Check Terminal**: Always monitor the terminal for build output and errors
3. **Use Preview**: Use the built-in preview pane for testing
4. **Share Sandboxes**: Use the share feature to collaborate or report issues
5. **Fork for Testing**: Fork the sandbox before making major changes

## Next Steps

After completing this test:

1. Document any issues in the test results template
2. Report bugs or unexpected behavior to the development team
3. Test in other cloud environments (StackBlitz, Lovable)
4. Share the CodeSandbox project URL for reference

## Reference Links

- [CodeSandbox](https://codesandbox.io/)
- [vite-plugin-wcss Documentation](./README.md)
- [WCSS Lovable Guide](./LOVABLE.md)
- [StackBlitz Testing Guide](./STACKBLITZ_TESTING.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

---

**Happy testing! 🧪**
