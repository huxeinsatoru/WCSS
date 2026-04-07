# StackBlitz Test Project Configuration

This document provides the complete configuration for a minimal StackBlitz test project for vite-plugin-wcss.

## Project Structure

```
stackblitz-wcss-test/
├── index.html
├── package.json
├── vite.config.js
└── src/
    ├── main.js
    ├── styles.wcss
    └── vite-env.d.ts (optional, for TypeScript)
```

## File Contents

### package.json

```json
{
  "name": "wcss-stackblitz-test",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "vite-plugin-wcss": "latest"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}
```

### vite.config.js

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

### index.html

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>WCSS StackBlitz Test</title>
    <style>
      body {
        font-family: system-ui, -apple-system, sans-serif;
        margin: 0;
        padding: 2rem;
        background: #f5f5f5;
      }
    </style>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>
```

### src/styles.wcss

```wcss
/* WCSS Test Styles */

/* Define tokens */
$colors.primary: #3b82f6;
$colors.primary-dark: #2563eb;
$colors.success: #10b981;
$spacing.sm: 0.5rem;
$spacing.md: 1rem;
$spacing.lg: 1.5rem;

/* Button component */
.test-button {
  background: $colors.primary;
  color: white;
  padding: $spacing.sm $spacing.md;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  font-weight: 500;
  transition: all 0.2s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.test-button:hover {
  background: $colors.primary-dark;
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.test-button:active {
  transform: translateY(0);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.test-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

/* Card component */
.card {
  background: white;
  border-radius: 8px;
  padding: $spacing.lg;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  max-width: 600px;
  margin: 0 auto;
}

.card-title {
  font-size: 1.5rem;
  font-weight: bold;
  margin: 0 0 $spacing.md 0;
  color: $colors.primary;
}

.card-content {
  color: #666;
  line-height: 1.6;
  margin-bottom: $spacing.lg;
}

.success-message {
  background: $colors.success;
  color: white;
  padding: $spacing.sm $spacing.md;
  border-radius: 4px;
  margin-top: $spacing.md;
  font-size: 0.875rem;
}

/* Test status indicator */
.status-indicator {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: $colors.success;
  margin-right: $spacing.sm;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
```

### src/main.js

```javascript
import styles from './styles.wcss';

// Inject compiled styles
const styleElement = document.createElement('style');
styleElement.textContent = styles;
document.head.appendChild(styleElement);

// Create test UI
const app = document.querySelector('#app');
app.innerHTML = `
  <div class="card">
    <h1 class="card-title">
      <span class="status-indicator"></span>
      WCSS StackBlitz Test
    </h1>
    
    <div class="card-content">
      <p>
        This is a test project for vite-plugin-wcss running in StackBlitz.
        If you can see this styled card and the button below, the plugin is working correctly!
      </p>
      
      <p>
        <strong>Test Instructions:</strong>
      </p>
      <ol>
        <li>Verify this card has proper styling</li>
        <li>Click the button below to test interactivity</li>
        <li>Edit <code>src/styles.wcss</code> to test HMR</li>
        <li>Change the primary color and save to see instant updates</li>
      </ol>
    </div>
    
    <button class="test-button" id="testButton">
      Click to Test
    </button>
    
    <div id="message"></div>
  </div>
`;

// Add button click handler
const button = document.getElementById('testButton');
const messageDiv = document.getElementById('message');
let clickCount = 0;

button.addEventListener('click', () => {
  clickCount++;
  messageDiv.innerHTML = `
    <div class="success-message">
      ✓ Button clicked ${clickCount} time${clickCount !== 1 ? 's' : ''}! 
      WCSS styles are working correctly.
    </div>
  `;
});

// Log compiled CSS for verification
console.log('=== WCSS Compilation Test ===');
console.log('Compiled CSS length:', styles.length, 'characters');
console.log('Contains token expansions:', !styles.includes('$'));
console.log('Sample output:', styles.substring(0, 200) + '...');
console.log('=== Test Complete ===');
```

### src/vite-env.d.ts (Optional - for TypeScript)

```typescript
/// <reference types="vite/client" />
/// <reference types="vite-plugin-wcss/wcss" />
```

## Quick Start in StackBlitz

### Option 1: Manual Setup

1. Go to [StackBlitz](https://stackblitz.com/)
2. Create new Vite project
3. Copy the files above into your project
4. Run `npm install`
5. The dev server should start automatically

### Option 2: Import from GitHub

If this project is available on GitHub:

1. Go to `stackblitz.com/github/[your-repo]/[path-to-example]`
2. StackBlitz will automatically import and run the project

## Expected Behavior

When the project runs successfully:

1. ✅ A styled card appears with a blue title
2. ✅ A blue button is visible and interactive
3. ✅ Clicking the button shows a green success message
4. ✅ Console shows compilation information
5. ✅ No errors in console or terminal

## Testing HMR

To test Hot Module Replacement:

1. Open `src/styles.wcss`
2. Change `$colors.primary: #3b82f6;` to `$colors.primary: #ef4444;`
3. Save the file
4. The card title and button should turn red instantly
5. No page reload should occur

## Troubleshooting

### Styles Not Appearing

Check:
- Terminal for compilation errors
- Console for JavaScript errors
- Network tab for failed requests
- Verify `vite-plugin-wcss` is installed

### WASM Error

Run:
```bash
npm install @wcss/wasm
```

Then restart the dev server.

### HMR Not Working

Try:
- Hard refresh (Ctrl+Shift+R)
- Restart dev server
- Check StackBlitz WebContainers are enabled

## Verification Checklist

- [ ] Project starts without errors
- [ ] Styles are applied correctly
- [ ] Button is interactive
- [ ] Console shows compilation info
- [ ] HMR updates work
- [ ] No WASM loading errors
- [ ] Production build works (`npm run build`)

## Next Steps

After verifying this basic test:

1. Try adding more complex WCSS features
2. Test with multiple .wcss files
3. Test error handling with invalid syntax
4. Test production build output
5. Test with different Vite configurations

## Reference

- [StackBlitz Testing Guide](../STACKBLITZ_TESTING.md)
- [StackBlitz Testing Checklist](../STACKBLITZ_CHECKLIST.md)
- [Main README](../README.md)

---

**This configuration is ready to use in StackBlitz!** 🚀
