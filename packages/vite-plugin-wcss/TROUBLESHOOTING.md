# WCSS Troubleshooting Guide

This guide helps you diagnose and fix common issues when using `vite-plugin-wcss` in development and production environments.

## Table of Contents

- [Installation Issues](#installation-issues)
- [WASM Loading Problems](#wasm-loading-problems)
- [Compilation Errors](#compilation-errors)
- [Cloud Environment Issues](#cloud-environment-issues)
- [TypeScript Issues](#typescript-issues)
- [Hot Module Replacement Issues](#hot-module-replacement-issues)
- [Production Build Issues](#production-build-issues)
- [Performance Issues](#performance-issues)

---

## Installation Issues

### Package Not Found

**Symptom:**

```
npm ERR! 404 Not Found - GET https://registry.npmjs.org/vite-plugin-wcss
```

**Cause:** Package name is incorrect or package is not published.

**Solution:**

1. Verify the package name:
   ```bash
   npm install vite-plugin-wcss
   ```

2. Check npm registry status:
   ```bash
   npm view vite-plugin-wcss
   ```

3. If the package is not available, check the repository for installation instructions.

---

### Dependency Installation Fails

**Symptom:**

```
npm ERR! Could not resolve dependency:
npm ERR! peer vite@"^4.0.0 || ^5.0.0" from vite-plugin-wcss
```

**Cause:** Incompatible Vite version.

**Solution:**

1. Check your Vite version:
   ```bash
   npm list vite
   ```

2. Upgrade Vite to a supported version (4.x or 5.x):
   ```bash
   npm install vite@latest
   ```

3. If you must use an older Vite version, check the plugin documentation for compatible versions.

---

### @wcss/wasm Not Installed

**Symptom:**

```
Error: Cannot find module '@wcss/wasm'
```

**Cause:** The WASM package dependency is missing.

**Solution:**

1. Install the WASM package explicitly:
   ```bash
   npm install @wcss/wasm
   ```

2. Verify installation:
   ```bash
   npm list @wcss/wasm
   ```

3. If the issue persists, delete `node_modules` and reinstall:
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   ```

---

## WASM Loading Problems

### WASM Initialization Failed

**Symptom:**

```
Failed to initialize WCSS WASM compiler: WebAssembly module is not a valid module
```

**Cause:** WASM binary is corrupted or incompatible with the environment.

**Solution:**

1. **Reinstall the WASM package:**
   ```bash
   npm uninstall @wcss/wasm
   npm install @wcss/wasm
   ```

2. **Clear npm cache:**
   ```bash
   npm cache clean --force
   npm install
   ```

3. **Check Node.js version:**
   ```bash
   node --version
   ```
   Ensure you're using Node.js 16.x or higher.

4. **Verify WASM support:**
   ```bash
   node -e "console.log(typeof WebAssembly)"
   ```
   Should output `object`. If not, upgrade Node.js.

---

### WASM Module Not Found

**Symptom:**

```
Error: Cannot find module '@wcss/wasm/wcss_wasm_bg.wasm'
```

**Cause:** Bundler is not configured to handle WASM files.

**Solution:**

1. **Vite (should work by default):**
   Vite supports WASM out of the box. If this error occurs, check your Vite version:
   ```bash
   npm install vite@latest
   ```

2. **Webpack:**
   Add WASM support to `webpack.config.js`:
   ```javascript
   module.exports = {
     experiments: {
       asyncWebAssembly: true
     }
   };
   ```

3. **Rollup:**
   Install the WASM plugin:
   ```bash
   npm install @rollup/plugin-wasm
   ```
   
   Add to `rollup.config.js`:
   ```javascript
   import wasm from '@rollup/plugin-wasm';
   
   export default {
     plugins: [wasm()]
   };
   ```

---

### WASM Initialization Timeout

**Symptom:**

```
Error: WASM initialization timed out
```

**Cause:** Slow network or large WASM file taking too long to load.

**Solution:**

1. **Check network connection:**
   Ensure you have a stable internet connection, especially in cloud environments.

2. **Increase timeout (if configurable):**
   Some environments allow configuring WASM load timeout.

3. **Use local development:**
   If in a cloud environment, try running locally to isolate the issue.

4. **Check CDN/proxy:**
   If using a proxy or CDN, ensure WASM files are not blocked.

---

### WASM Memory Issues

**Symptom:**

```
RuntimeError: memory access out of bounds
```

**Cause:** WASM module ran out of memory or accessed invalid memory.

**Solution:**

1. **Reduce input size:**
   If compiling very large WCSS files, try splitting them into smaller files.

2. **Check for infinite loops:**
   Ensure your WCSS doesn't have circular token references:
   ```wcss
   /* BAD: Circular reference */
   $colors.primary: $colors.secondary;
   $colors.secondary: $colors.primary;
   ```

3. **Update WASM package:**
   ```bash
   npm update @wcss/wasm
   ```

4. **Report the issue:**
   If the problem persists, file a bug report with the WCSS file that causes the issue.

---

## Compilation Errors

### Undefined Token Error

**Symptom:**

```
WCSS compilation errors:
  error: Undefined token '$colors.primary' (line 5:12)
```

**Cause:** Token is referenced but not defined.

**Solution:**

1. **Define the token in your WCSS file:**
   ```wcss
   $colors.primary: #3b82f6;
   
   .button {
     background: $colors.primary;
   }
   ```

2. **Or define in Vite config:**
   ```javascript
   // vite.config.js
   export default defineConfig({
     plugins: [
       wcss({
         tokens: {
           colors: {
             primary: '#3b82f6'
           }
         }
       })
     ]
   });
   ```

3. **Check token name spelling:**
   Token names are case-sensitive. Ensure exact match:
   ```wcss
   $colors.Primary  // Wrong
   $colors.primary  // Correct
   ```

---

### Syntax Error

**Symptom:**

```
WCSS compilation errors:
  error: Unexpected token '}' (line 8:1)
```

**Cause:** Invalid WCSS syntax.

**Solution:**

1. **Check for missing semicolons:**
   ```wcss
   /* BAD */
   .button {
     background: blue
     padding: 1rem;
   }
   
   /* GOOD */
   .button {
     background: blue;
     padding: 1rem;
   }
   ```

2. **Check for unmatched braces:**
   ```wcss
   /* BAD */
   .button {
     background: blue;
   /* Missing closing brace */
   
   /* GOOD */
   .button {
     background: blue;
   }
   ```

3. **Check for invalid property values:**
   ```wcss
   /* BAD */
   .button {
     display: invalid-value;
   }
   
   /* GOOD */
   .button {
     display: block;
   }
   ```

---

### Invalid Property Value

**Symptom:**

```
WCSS compilation errors:
  error: Invalid property value 'invalid' for 'display' (line 8:15)
```

**Cause:** CSS property has an invalid value.

**Solution:**

1. **Check CSS property documentation:**
   Verify the value is valid for the property:
   ```wcss
   /* BAD */
   .button {
     display: invalid;
   }
   
   /* GOOD */
   .button {
     display: block; /* or flex, inline, etc. */
   }
   ```

2. **Check for typos:**
   ```wcss
   /* BAD */
   .button {
     background: blu;
   }
   
   /* GOOD */
   .button {
     background: blue;
   }
   ```

---

### Circular Token Reference

**Symptom:**

```
WCSS compilation errors:
  error: Circular token reference detected: $colors.primary -> $colors.secondary -> $colors.primary
```

**Cause:** Tokens reference each other in a loop.

**Solution:**

1. **Break the circular reference:**
   ```wcss
   /* BAD */
   $colors.primary: $colors.secondary;
   $colors.secondary: $colors.primary;
   
   /* GOOD */
   $colors.primary: #3b82f6;
   $colors.secondary: $colors.primary;
   ```

2. **Use literal values:**
   ```wcss
   $colors.primary: #3b82f6;
   $colors.secondary: #8b5cf6;
   ```

---

## Cloud Environment Issues

### Lovable: Plugin Not Working

**Symptom:**

- `.wcss` files are not compiled
- No errors in console
- Styles not applied

**Solution:**

1. **Verify plugin installation:**
   ```bash
   npm list vite-plugin-wcss @wcss/wasm
   ```

2. **Check Vite config:**
   Ensure the plugin is properly imported and added:
   ```javascript
   // vite.config.js
   import { defineConfig } from 'vite';
   import wcss from 'vite-plugin-wcss';
   
   export default defineConfig({
     plugins: [
       wcss()
     ]
   });
   ```

3. **Restart the dev server:**
   Stop the server (Ctrl+C) and restart:
   ```bash
   npm run dev
   ```

4. **Check file extension:**
   Ensure your file ends with `.wcss` (not `.css` or `.wcss.css`).

5. **Check import statement:**
   ```javascript
   // Correct
   import styles from './styles.wcss';
   
   // Wrong
   import './styles.wcss'; // No default import
   ```

---

### StackBlitz: WASM Loading Fails

**Symptom:**

```
Failed to fetch WASM module
```

**Cause:** StackBlitz may have issues with WASM loading in some configurations.

**Solution:**

1. **Use WebContainers mode:**
   Ensure your StackBlitz project is using WebContainers (not the legacy mode).

2. **Check browser compatibility:**
   Use a modern browser (Chrome, Firefox, Edge, Safari 11+).

3. **Clear browser cache:**
   Hard refresh the page (Ctrl+Shift+R or Cmd+Shift+R).

4. **Try a different browser:**
   If the issue persists, test in a different browser.

5. **Check StackBlitz status:**
   Visit [StackBlitz status page](https://status.stackblitz.com/) to check for outages.

---

### CodeSandbox: Module Not Found

**Symptom:**

```
Module not found: Can't resolve 'vite-plugin-wcss'
```

**Cause:** Dependencies not installed or CodeSandbox cache issue.

**Solution:**

1. **Refresh dependencies:**
   Click "Refresh" in the Dependencies panel in CodeSandbox.

2. **Manually add dependency:**
   Add to `package.json`:
   ```json
   {
     "dependencies": {
       "vite-plugin-wcss": "latest",
       "@wcss/wasm": "latest"
     }
   }
   ```

3. **Restart sandbox:**
   Click "Restart Sandbox" in the menu.

4. **Fork the sandbox:**
   Create a fresh fork of the sandbox.

---

### Cloud Environment: File System Access Error

**Symptom:**

```
Error: ENOENT: no such file or directory
```

**Cause:** Plugin is trying to access local file system (should not happen with npm-based loading).

**Solution:**

1. **Update plugin version:**
   Ensure you're using the latest version that supports cloud environments:
   ```bash
   npm install vite-plugin-wcss@latest
   ```

2. **Check configuration:**
   Remove any file system paths from configuration:
   ```javascript
   // BAD
   wcss({
     wasmPath: '/path/to/wasm' // Don't do this
   })
   
   // GOOD
   wcss({
     // No file system paths needed
   })
   ```

3. **Report the issue:**
   If the problem persists, file a bug report with your configuration.

---

## TypeScript Issues

### Cannot Find Module '.wcss'

**Symptom:**

```typescript
// TypeScript error
Cannot find module './styles.wcss' or its corresponding type declarations.
```

**Cause:** TypeScript doesn't recognize `.wcss` file imports.

**Solution:**

1. **Add type reference:**
   Create or update `src/vite-env.d.ts`:
   ```typescript
   /// <reference types="vite/client" />
   /// <reference types="vite-plugin-wcss/wcss" />
   ```

2. **Restart TypeScript server:**
   In VS Code: `Cmd+Shift+P` → "TypeScript: Restart TS Server"

3. **Check tsconfig.json:**
   Ensure `vite-env.d.ts` is included:
   ```json
   {
     "include": ["src/**/*", "src/vite-env.d.ts"]
   }
   ```

---

### Type Definitions Not Found

**Symptom:**

```
Could not find a declaration file for module 'vite-plugin-wcss'
```

**Cause:** Plugin package doesn't include TypeScript definitions.

**Solution:**

1. **Update plugin:**
   ```bash
   npm install vite-plugin-wcss@latest
   ```

2. **Check package contents:**
   ```bash
   npm list vite-plugin-wcss
   ls node_modules/vite-plugin-wcss
   ```
   Should include `.d.ts` files.

3. **Add type declaration manually:**
   Create `types/vite-plugin-wcss.d.ts`:
   ```typescript
   declare module 'vite-plugin-wcss' {
     import { Plugin } from 'vite';
     
     export interface WCSSPluginOptions {
       minify?: boolean;
       sourceMaps?: boolean;
       treeShaking?: boolean;
       // ... other options
     }
     
     export default function wcss(options?: WCSSPluginOptions): Plugin;
   }
   ```

---

## Hot Module Replacement Issues

### HMR Not Working

**Symptom:**

- Changes to `.wcss` files don't update in the browser
- Full page reload required to see changes

**Solution:**

1. **Check dev server is running:**
   Ensure `npm run dev` is active.

2. **Check file is being watched:**
   Vite should log file changes:
   ```
   [vite] hmr update /src/styles.wcss
   ```

3. **Check browser console:**
   Look for HMR errors in the browser console.

4. **Restart dev server:**
   Stop and restart the dev server.

5. **Check Vite config:**
   Ensure HMR is not disabled:
   ```javascript
   // vite.config.js
   export default defineConfig({
     server: {
       hmr: true // Should be enabled (default)
     }
   });
   ```

---

### HMR Causes Full Page Reload

**Symptom:**

- `.wcss` changes trigger full page reload instead of CSS-only update

**Cause:** This is expected behavior in the current implementation.

**Solution:**

This is not a bug. The plugin currently triggers full page reload for `.wcss` changes to ensure consistency. CSS-only HMR may be added in a future version.

**Workaround:**

If full page reloads are disruptive, consider:
1. Using smaller, more focused `.wcss` files
2. Organizing styles by component for faster reloads

---

## Production Build Issues

### Build Fails with WASM Error

**Symptom:**

```
Build failed: WASM initialization failed
```

**Cause:** WASM not available during build.

**Solution:**

1. **Ensure @wcss/wasm is installed:**
   ```bash
   npm install @wcss/wasm
   ```

2. **Check build command:**
   ```bash
   npm run build
   ```

3. **Clear build cache:**
   ```bash
   rm -rf dist node_modules/.vite
   npm run build
   ```

4. **Check Node.js version:**
   ```bash
   node --version
   ```
   Ensure Node.js 16+ is installed.

---

### Production CSS Missing Styles

**Symptom:**

- Styles work in development but not in production
- Some CSS classes are missing

**Cause:** Tree-shaking removed classes that are actually used.

**Solution:**

1. **Check tree-shaking configuration:**
   ```javascript
   // vite.config.js
   wcss({
     treeShaking: true,
     contentPaths: ['./src/**/*.{js,jsx,ts,tsx}']
   })
   ```

2. **Add missing classes to safelist:**
   ```javascript
   wcss({
     treeShaking: true,
     safelist: [
       'dynamic-class',
       'status-*',
       'theme-*'
     ]
   })
   ```

3. **Disable tree-shaking temporarily:**
   ```javascript
   wcss({
     treeShaking: false
   })
   ```
   If styles appear, the issue is with tree-shaking configuration.

4. **Check for dynamic class names:**
   ```javascript
   // These won't be detected by tree-shaking
   const className = `status-${status}`;
   const theme = `theme-${themeName}`;
   ```
   Add these patterns to `safelist`.

---

### Production Build Size Too Large

**Symptom:**

- CSS bundle is larger than expected
- Unused styles are included

**Solution:**

1. **Enable minification:**
   ```javascript
   wcss({
     minify: true
   })
   ```

2. **Enable tree-shaking:**
   ```javascript
   wcss({
     treeShaking: true,
     contentPaths: ['./src/**/*.{js,jsx,ts,tsx}']
   })
   ```

3. **Enable deduplication:**
   ```javascript
   wcss({
     deduplicate: true
   })
   ```

4. **Analyze bundle:**
   Use a bundle analyzer to identify large CSS files:
   ```bash
   npm install -D rollup-plugin-visualizer
   ```

---

## Performance Issues

### Slow Compilation

**Symptom:**

- `.wcss` files take a long time to compile
- Dev server is slow to start

**Solution:**

1. **Reduce file size:**
   Split large `.wcss` files into smaller, focused files.

2. **Disable source maps in development:**
   ```javascript
   wcss({
     sourceMaps: false
   })
   ```

3. **Disable tree-shaking in development:**
   ```javascript
   wcss({
     treeShaking: process.env.NODE_ENV === 'production'
   })
   ```

4. **Check system resources:**
   Ensure your system has sufficient RAM and CPU available.

5. **Update dependencies:**
   ```bash
   npm update
   ```

---

### High Memory Usage

**Symptom:**

- Build process uses excessive memory
- Out of memory errors

**Solution:**

1. **Increase Node.js memory limit:**
   ```bash
   NODE_OPTIONS="--max-old-space-size=4096" npm run build
   ```

2. **Reduce concurrent compilations:**
   If compiling many `.wcss` files, consider reducing parallelism.

3. **Split large files:**
   Break large `.wcss` files into smaller modules.

4. **Disable source maps:**
   ```javascript
   wcss({
     sourceMaps: false
   })
   ```

---

## Getting Help

If your issue is not covered in this guide:

1. **Check existing issues:**
   Search the [GitHub issues](https://github.com/your-repo/wcss/issues) for similar problems.

2. **Create a minimal reproduction:**
   Create a minimal example that demonstrates the issue.

3. **File a bug report:**
   Open a new issue with:
   - Description of the problem
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Node.js version, browser)
   - Relevant configuration files

4. **Community support:**
   Ask questions in the project's discussion forum or community chat.

---

## Diagnostic Checklist

Use this checklist to diagnose issues systematically:

- [ ] Plugin is installed: `npm list vite-plugin-wcss`
- [ ] WASM package is installed: `npm list @wcss/wasm`
- [ ] Vite version is 4.x or 5.x: `npm list vite`
- [ ] Node.js version is 16+: `node --version`
- [ ] Plugin is added to `vite.config.js`
- [ ] File extension is `.wcss`
- [ ] Import statement is correct: `import styles from './file.wcss'`
- [ ] TypeScript types are configured (if using TypeScript)
- [ ] Dev server is running: `npm run dev`
- [ ] Browser console shows no errors
- [ ] Network tab shows WASM file loaded (if applicable)

---

## Common Error Messages Reference

| Error Message | Likely Cause | Solution |
|--------------|--------------|----------|
| `Cannot find module '@wcss/wasm'` | WASM package not installed | `npm install @wcss/wasm` |
| `Failed to initialize WCSS WASM compiler` | WASM loading failed | Reinstall @wcss/wasm, check Node.js version |
| `Undefined token '$colors.primary'` | Token not defined | Define token in WCSS or config |
| `Unexpected token '}'` | Syntax error | Check for missing semicolons or braces |
| `Cannot find module './styles.wcss'` | TypeScript type missing | Add `/// <reference types="vite-plugin-wcss/wcss" />` |
| `ENOENT: no such file or directory` | File system access issue | Update plugin, remove file system paths |
| `memory access out of bounds` | WASM memory issue | Reduce file size, check for circular references |

---

**Last Updated:** 2024
**Plugin Version:** 1.0.0+

For the latest troubleshooting information, visit the [GitHub repository](https://github.com/your-repo/wcss).
