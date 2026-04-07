# Lovable Testing Guide for vite-plugin-wcss

This guide provides step-by-step instructions for testing vite-plugin-wcss in Lovable (formerly GPT Engineer), a cloud-based AI-powered development environment.

## Prerequisites

- A Lovable account (free tier is sufficient)
- Modern browser (Chrome, Firefox, Safari, or Edge)
- Basic knowledge of Vite and WCSS

## Test Objectives

This test verifies that vite-plugin-wcss works correctly in Lovable by validating:

1. **Installation**: Plugin and dependencies install correctly from npm
2. **Compilation**: .wcss files compile to CSS successfully
3. **HMR**: Hot Module Replacement updates styles without page reload
4. **Error Handling**: Compilation errors display correctly in the error overlay
5. **Production Build**: Production builds generate optimized CSS
6. **Full Development Workflow**: Complete development cycle works seamlessly

## Test Setup

### Step 1: Create a New Project in Lovable

1. Go to [Lovable](https://lovable.dev/)
2. Click "Create New Project" or start a conversation with the AI
3. Ask Lovable to create a new Vite + React project (or vanilla Vite)
4. Wait for the project to initialize

**Example prompt:**
```
Create a new Vite React project with TypeScript
```

### Step 2: Install vite-plugin-wcss

Ask Lovable to install the plugin, or use the terminal:

**Via AI prompt:**
```
Install vite-plugin-wcss package
```

**Via terminal:**
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

Ask Lovable to configure the plugin, or manually create/update `vite.config.js`:

**Via AI prompt:**
```
Configure vite-plugin-wcss in vite.config.js with basic options including color and spacing tokens
```

**Manual configuration:**

```javascript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wcss from 'vite-plugin-wcss';

export default defineConfig({
  plugins: [
    react(),
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
- No configuration errors in terminal

## Test Cases

### Test Case 1: Basic Compilation

**Objective:** Verify that .wcss files compile to CSS

**Steps:**

1. Create a new file `src/styles.wcss`:

**Via AI prompt:**
```
Create a src/styles.wcss file with a test button style using color and spacing tokens
```

**Manual creation:**

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
  font-weight: 500;
  transition: opacity 0.2s;
}

.test-button:hover {
  opacity: 0.9;
}

.test-button:active {
  opacity: 0.8;
}
```

2. Import the WCSS file in your component (e.g., `src/App.jsx`):

**Via AI prompt:**
```
Import styles.wcss in App.jsx and create a test button using the test-button class
```

**Manual implementation:**

```jsx
import React from 'react';
import styles from './styles.wcss';

function App() {
  return (
    <>
      <style>{styles}</style>
      <div style={{ padding: '2rem' }}>
        <h1>WCSS Lovable Test</h1>
        <button className="test-button">Test Button</button>
      </div>
    </>
  );
}

export default App;
```

3. Check the browser preview

**Expected Result:**
- ✅ No compilation errors in terminal
- ✅ Button appears with blue background
- ✅ Console shows compiled CSS (if you add console.log)
- ✅ Compiled CSS contains expanded token values (not `$colors.primary`)
- ✅ Button hover effect works smoothly
- ✅ Button active state works

**Verification:**
- Inspect the button element in DevTools
- Check that styles are applied correctly
- Verify CSS contains actual color values, not token references
- Test hover and active states

### Test Case 2: Hot Module Replacement (HMR)

**Objective:** Verify that editing .wcss files triggers HMR

**Steps:**

1. With the dev server running, edit `src/styles.wcss`
2. Change the primary color:

**Via AI prompt:**
```
Change the button background color from blue to red in styles.wcss
```

**Manual edit:**

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
- ✅ Component state is preserved (if any)

**Verification:**
- Check that the page doesn't reload (any console logs remain)
- Verify the button color changed to red
- Check terminal for HMR update message
- Verify React component state wasn't reset

### Test Case 3: Compilation Error Handling

**Objective:** Verify that syntax errors are displayed correctly

**Steps:**

1. Edit `src/styles.wcss` to introduce a syntax error:

**Via AI prompt:**
```
Add a reference to an undefined token in styles.wcss to test error handling
```

**Manual edit:**

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
- ✅ Error message is clear and actionable

**Verification:**
- Read the error message - should mention "undefined-token"
- Check that line/column numbers are present
- Verify the page is still functional (not completely broken)
- Check that error overlay is dismissible

4. Fix the error:

**Via AI prompt:**
```
Fix the undefined token error in styles.wcss
```

**Manual fix:**

```wcss
.test-button {
  background: $colors.primary; /* Fixed */
  padding: $spacing.md;
}
```

5. Save the file

**Expected Result:**
- ✅ Error overlay disappears automatically
- ✅ Styles update correctly
- ✅ No errors in console or terminal
- ✅ Button returns to correct appearance

### Test Case 4: Multiple WCSS Files

**Objective:** Verify that multiple .wcss files can be imported

**Steps:**

1. Create `src/components.wcss`:

**Via AI prompt:**
```
Create a src/components.wcss file with card component styles
```

**Manual creation:**

```wcss
.card {
  background: white;
  border-radius: 8px;
  padding: $spacing.lg;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  margin-bottom: $spacing.md;
}

.card-title {
  font-size: 1.5rem;
  font-weight: bold;
  margin-bottom: $spacing.sm;
  color: $colors.primary;
}

.card-content {
  color: #666;
  line-height: 1.6;
}
```

2. Import both files in your component:

**Via AI prompt:**
```
Update App.jsx to import both styles.wcss and components.wcss, and create a card with a button inside
```

**Manual implementation:**

```jsx
import React from 'react';
import buttonStyles from './styles.wcss';
import componentStyles from './components.wcss';

function App() {
  return (
    <>
      <style>{buttonStyles}</style>
      <style>{componentStyles}</style>
      <div style={{ padding: '2rem' }}>
        <div className="card">
          <h2 className="card-title">WCSS Lovable Test</h2>
          <p className="card-content">
            This card demonstrates multiple WCSS files working together.
          </p>
          <button className="test-button">Test Button</button>
        </div>
      </div>
    </>
  );
}

export default App;
```

3. Check the browser preview

**Expected Result:**
- ✅ Both stylesheets compile successfully
- ✅ Card and button styles both apply
- ✅ No conflicts between stylesheets
- ✅ Styles cascade correctly
- ✅ Both files support HMR independently

**Verification:**
- Edit one file and verify only those styles update
- Check that both stylesheets use the same tokens
- Verify no duplicate CSS rules in output

### Test Case 5: AI-Assisted Development Workflow

**Objective:** Verify WCSS works seamlessly with Lovable's AI assistance

**Steps:**

1. Ask Lovable to create a new component with WCSS styles:

**AI prompt:**
```
Create a new Alert component in src/components/Alert.jsx with its own alert.wcss file. 
The alert should have success, warning, and danger variants using the color tokens.
```

2. Ask Lovable to modify the styles:

**AI prompt:**
```
Update the alert styles to add an icon area on the left and make the alert dismissible
```

3. Ask Lovable to fix any issues:

**AI prompt:**
```
The alert padding looks too small on mobile. Can you adjust it?
```

**Expected Result:**
- ✅ Lovable creates .wcss files correctly
- ✅ Lovable understands WCSS syntax
- ✅ Lovable can modify existing .wcss files
- ✅ Lovable handles token references correctly
- ✅ All generated code compiles without errors
- ✅ AI suggestions are contextually appropriate

**Verification:**
- Check that generated WCSS is valid
- Verify token usage is correct
- Test that modifications work as expected
- Confirm AI understands WCSS-specific concepts

### Test Case 6: TypeScript Support

**Objective:** Verify TypeScript definitions work correctly

**Steps:**

1. Ensure `src/vite-env.d.ts` includes WCSS types:

**Via AI prompt:**
```
Add TypeScript support for .wcss file imports
```

**Manual addition:**

```typescript
/// <reference types="vite/client" />
/// <reference types="vite-plugin-wcss/wcss" />
```

2. Create a TypeScript component that imports WCSS:

**Via AI prompt:**
```
Create a TypeScript component Button.tsx that imports button.wcss
```

**Manual creation:**

```typescript
import React from 'react';
import styles from './button.wcss';

interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary';
}

const Button: React.FC<ButtonProps> = ({ children, onClick, variant = 'primary' }) => {
  return (
    <>
      <style>{styles}</style>
      <button className={`btn btn-${variant}`} onClick={onClick}>
        {children}
      </button>
    </>
  );
};

export default Button;
```

**Expected Result:**
- ✅ No TypeScript errors
- ✅ Autocomplete works for .wcss imports
- ✅ Type checking passes
- ✅ `styles` is correctly typed as `string`
- ✅ No import errors in IDE

**Verification:**
- Hover over the import to see type information
- Check that TypeScript compiler doesn't report errors
- Verify autocomplete suggests the import

### Test Case 7: Production Build

**Objective:** Verify production builds work correctly

**Steps:**

1. Run the production build command:

**Via AI prompt:**
```
Build the project for production
```

**Via terminal:**

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
- ✅ CSS is properly compiled
- ✅ All token references are resolved
- ✅ Build time is reasonable (< 30 seconds for small projects)

**Verification:**
- Check `dist/assets` for `.css` files
- Verify no `.wasm` files in `dist`
- Open generated CSS and verify it contains the compiled styles
- Check that the preview looks identical to development
- Verify file sizes are reasonable

### Test Case 8: Configuration Options

**Objective:** Verify configuration options work correctly

**Steps:**

1. Update `vite.config.js` to enable minification:

**Via AI prompt:**
```
Enable minification and disable source maps in the WCSS plugin configuration
```

**Manual update:**

```javascript
export default defineConfig({
  plugins: [
    react(),
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
- ✅ File size is smaller than unminified version

**Verification:**
- Open the CSS file and verify it's minified
- Check that no `.map` files exist
- Compare file size with previous build

### Test Case 9: Complex Styling Scenarios

**Objective:** Verify WCSS handles complex real-world scenarios

**Steps:**

1. Create a complex component with nested styles:

**Via AI prompt:**
```
Create a responsive navigation bar component with dropdown menus using WCSS. 
Include hover states, mobile menu, and use breakpoint tokens.
```

2. Test responsive behavior:
   - Resize browser window
   - Test mobile menu toggle
   - Verify breakpoints work correctly

3. Test state variations:
   - Hover states
   - Active states
   - Disabled states
   - Focus states

**Expected Result:**
- ✅ Complex selectors compile correctly
- ✅ Nested rules work as expected
- ✅ Media queries function properly
- ✅ Pseudo-classes and pseudo-elements work
- ✅ All states render correctly
- ✅ Responsive design works across breakpoints

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
5. Ask Lovable: "Reinstall vite-plugin-wcss and restart the dev server"

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
4. Ask Lovable: "Fix the vite-plugin-wcss import error"

### Issue: Styles Not Updating

**Symptom:**
- Changes to .wcss files don't appear in browser

**Solution:**
1. Check terminal for compilation errors
2. Hard refresh browser (Ctrl+Shift+R or Cmd+Shift+R)
3. Restart dev server
4. Check that HMR is enabled in Vite config
5. Ask Lovable: "The WCSS styles aren't updating, can you help debug?"

### Issue: Lovable AI Not Understanding WCSS

**Symptom:**
- AI generates invalid WCSS syntax
- AI doesn't recognize .wcss files

**Solution:**
1. Be explicit in prompts: "Create a .wcss file (not .css)"
2. Provide examples of correct WCSS syntax
3. Reference the WCSS documentation in your prompt
4. Ask Lovable to review the LOVABLE.md guide

### Issue: TypeScript Import Errors

**Symptom:**
```
Cannot find module './styles.wcss' or its corresponding type declarations
```

**Solution:**
1. Ensure `vite-env.d.ts` includes:
   ```typescript
   /// <reference types="vite-plugin-wcss/wcss" />
   ```
2. Restart TypeScript server in IDE
3. Ask Lovable: "Add TypeScript support for .wcss imports"

### Issue: Production Build Includes WASM

**Symptom:**
- `.wasm` files appear in `dist` folder

**Solution:**
1. This shouldn't happen - WASM is build-time only
2. Check Vite configuration for incorrect settings
3. Verify you're using the latest version of vite-plugin-wcss
4. Report this as a bug if it persists

## Test Results Template

Use this template to document your test results:

```markdown
## Lovable Test Results

**Date:** [Date]
**Tester:** [Your Name]
**Browser:** [Browser Name and Version]
**Lovable Project:** [Project URL]

### Test Case Results

| Test Case | Status | Notes |
|-----------|--------|-------|
| 1. Basic Compilation | ✅ Pass / ❌ Fail | |
| 2. Hot Module Replacement | ✅ Pass / ❌ Fail | |
| 3. Error Handling | ✅ Pass / ❌ Fail | |
| 4. Multiple WCSS Files | ✅ Pass / ❌ Fail | |
| 5. AI-Assisted Workflow | ✅ Pass / ❌ Fail | |
| 6. TypeScript Support | ✅ Pass / ❌ Fail / ⏭️ Skip | |
| 7. Production Build | ✅ Pass / ❌ Fail | |
| 8. Configuration Options | ✅ Pass / ❌ Fail | |
| 9. Complex Scenarios | ✅ Pass / ❌ Fail / ⏭️ Skip | |

### AI Interaction Quality

- [ ] AI understood WCSS syntax
- [ ] AI created valid .wcss files
- [ ] AI handled token references correctly
- [ ] AI provided helpful debugging assistance

### Issues Encountered

[List any issues or unexpected behavior]

### Overall Assessment

- [ ] All critical tests passed
- [ ] Plugin works as expected in Lovable
- [ ] AI integration works smoothly
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
6. ✅ TypeScript support works correctly
7. ✅ Production builds generate optimized CSS
8. ✅ No WASM files in production bundle
9. ✅ Configuration options work as expected
10. ✅ Lovable AI can work with WCSS files effectively

## Lovable-Specific Considerations

### AI Prompting Best Practices

When working with Lovable's AI:

1. **Be explicit about file types:**
   - ❌ "Create a stylesheet"
   - ✅ "Create a .wcss file"

2. **Reference WCSS concepts:**
   - ✅ "Use WCSS tokens for colors"
   - ✅ "Create a .wcss file with token definitions"

3. **Provide context:**
   - ✅ "Update the button.wcss file to use the $colors.primary token"

4. **Ask for verification:**
   - ✅ "Verify that the WCSS compiles correctly"

### Lovable Environment Characteristics

- **Auto-save**: Files save automatically, triggering HMR
- **AI Integration**: AI can create and modify .wcss files
- **Terminal Access**: Full npm and build command access
- **Preview**: Live preview updates automatically
- **File System**: Virtual file system, all resources from npm

## Next Steps

After completing this test:

1. Document any issues in the test results template
2. Report bugs or unexpected behavior to the development team
3. Share feedback about AI integration quality
4. Test in other cloud environments (StackBlitz, CodeSandbox)
5. Share the Lovable project URL for reference

## Reference Links

- [Lovable](https://lovable.dev/)
- [vite-plugin-wcss Documentation](./README.md)
- [WCSS Lovable Setup Guide](./LOVABLE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [Configuration Guide](./CONFIGURATION.md)

---

**Happy testing with Lovable! 🚀**
