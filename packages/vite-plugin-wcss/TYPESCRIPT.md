# TypeScript Support

The `vite-plugin-wcss` package includes full TypeScript support with type declarations for `.wcss` file imports.

## Module Declarations

When you import a `.wcss` file in TypeScript, the plugin provides type information for the compiled CSS output:

```typescript
// Import compiled CSS (default export)
import styles from './button.wcss';

// Type: string
console.log(styles); // ".button { padding: 1rem; ... }"
```

## Runtime Export

When the `typedOM` option is enabled, `.wcss` files also export optional runtime JavaScript code:

```typescript
// Import both CSS and runtime
import styles, { runtime } from './button.wcss';

// Type: string
console.log(styles); // CSS content

// Type: string | undefined
console.log(runtime); // Runtime JavaScript (when typedOM is enabled)
```

## Configuration

The TypeScript declarations are automatically available when you install `vite-plugin-wcss`. No additional configuration is needed.

### tsconfig.json

Your `tsconfig.json` should include standard Vite settings:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "types": ["vite/client"]
  }
}
```

## Type Safety

The module declarations ensure type safety when working with `.wcss` files:

```typescript
// ✅ Valid: default export is a string
import styles from './button.wcss';
const css: string = styles;

// ✅ Valid: runtime is optional
import styles, { runtime } from './button.wcss';
const runtimeCode: string | undefined = runtime;

// ❌ Invalid: default export is not an object
import styles from './button.wcss';
const obj: object = styles; // Type error!
```

## IDE Support

Modern IDEs like VS Code, WebStorm, and others will provide:

- **Autocomplete**: Import suggestions for `.wcss` files
- **Type checking**: Compile-time errors for incorrect usage
- **IntelliSense**: Hover information and documentation
- **Go to definition**: Navigate to `.wcss` source files

## Example Project Structure

```
src/
├── components/
│   ├── Button.tsx
│   └── Button.wcss
├── styles/
│   ├── global.wcss
│   └── theme.wcss
└── vite-env.d.ts
```

### Button.tsx

```typescript
import React from 'react';
import styles from './Button.wcss';

export const Button: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  // Inject CSS into the document
  React.useEffect(() => {
    const styleElement = document.createElement('style');
    styleElement.textContent = styles;
    document.head.appendChild(styleElement);
    return () => styleElement.remove();
  }, []);

  return <button className="button">{children}</button>;
};
```

### Button.wcss

```css
.button {
  padding: 1rem 2rem;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 0.5rem;
  cursor: pointer;
}

.button:hover {
  background: #2563eb;
}
```

## Troubleshooting

### "Cannot find module '*.wcss'"

If you see this error, ensure:

1. `vite-plugin-wcss` is installed: `npm install vite-plugin-wcss`
2. The plugin is configured in `vite.config.ts`
3. Your IDE has reloaded the TypeScript server (restart VS Code or run "TypeScript: Restart TS Server")

### Type declarations not working

If type declarations aren't working:

1. Check that `dist/wcss.d.ts` exists in `node_modules/vite-plugin-wcss/`
2. Verify your `tsconfig.json` includes the project files
3. Try deleting `node_modules` and reinstalling: `rm -rf node_modules && npm install`
4. Restart your IDE or TypeScript server

## Advanced Usage

### Custom Type Augmentation

You can extend the `.wcss` module declaration if needed:

```typescript
// custom-types.d.ts
declare module '*.wcss' {
  const css: string;
  export const runtime: string | undefined;
  
  // Add custom exports
  export const metadata: {
    classes: string[];
    tokens: Record<string, string>;
  };
  
  export default css;
}
```

### Type-Safe Class Names

For type-safe class name usage, consider using CSS Modules or a similar approach:

```typescript
// This is a future enhancement - not currently supported
import styles from './button.wcss';

// Hypothetical type-safe class names
const className = styles.button; // Type: string
```

## Related Documentation

- [Vite Plugin API](https://vitejs.dev/guide/api-plugin.html)
- [TypeScript Module Resolution](https://www.typescriptlang.org/docs/handbook/module-resolution.html)
- [CSS Typed OM](https://developer.mozilla.org/en-US/docs/Web/API/CSS_Typed_OM_API)
