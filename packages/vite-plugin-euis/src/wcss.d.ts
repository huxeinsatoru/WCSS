/**
 * TypeScript module declaration for .euis file imports
 * 
 * This declaration provides type safety when importing .euis files in TypeScript projects.
 * The vite-plugin-euis transforms .euis files into JavaScript modules that export:
 * - A default export containing the compiled CSS as a string
 * - An optional runtime export containing JavaScript runtime code (when typedOM is enabled)
 * 
 * @example
 * ```typescript
 * // Import compiled CSS
 * import styles from './styles.euis';
 * console.log(styles); // string containing CSS
 * 
 * // Import with runtime (when typedOM is enabled)
 * import styles, { runtime } from './styles.euis';
 * console.log(styles); // CSS string
 * console.log(runtime); // Optional runtime JavaScript
 * ```
 */
declare module '*.euis' {
  /**
   * The compiled CSS content as a string.
   * This is the result of compiling the .euis file to standard CSS.
   */
  const css: string;
  
  /**
   * Optional runtime JavaScript code.
   * This export is only present when the typedOM option is enabled in the plugin configuration.
   * It contains JavaScript helpers for CSS Typed OM support.
   */
  export const runtime: string | undefined;
  
  export default css;
}
