import type { Plugin, HmrContext } from 'vite';

export interface EuisPluginOptions {
  /**
   * Enable tree shaking to remove unused styles
   * @default false
   */
  treeShaking?: boolean;

  /**
   * Minify CSS output
   * @default false
   */
  minify?: boolean;

  /**
   * Generate source maps
   * @default true
   */
  sourceMaps?: boolean;

  /**
   * Enable Typed OM runtime
   * @default false
   */
  typedOM?: boolean;

  /**
   * Deduplicate identical rule blocks
   * @default true
   */
  deduplicate?: boolean;

  /**
   * List of class names known to be used (for tree shaking)
   */
  usedClasses?: string[];

  /**
   * Glob paths to scan for used classes (for tree shaking)
   */
  contentPaths?: string[];

  /**
   * Class names that should never be removed by tree shaking
   */
  safelist?: string[];

  /**
   * Design tokens configuration
   */
  tokens?: {
    colors?: Record<string, string>;
    spacing?: Record<string, string>;
    typography?: Record<string, string>;
    breakpoints?: Record<string, string>;
  };
}

/**
 * Result from WASM compiler (matches Rust CompileResult struct)
 */
interface CompileResult {
  css: string;
  js?: string;
  source_map?: string;
  errors: Array<{ message?: string; line?: number; column?: number; severity?: string }>;
  warnings: Array<{ message?: string; line?: number; column?: number; severity?: string }>;
  stats?: {
    input_size: number;
    output_size: number;
    compile_time_us: number;
    rules_processed: number;
    rules_eliminated: number;
  };
}

/**
 * Singleton WASM compiler instance
 */
let wasmCompiler: any = null;

/**
 * Promise to track WASM initialization (prevents concurrent initialization)
 */
let wasmInitPromise: Promise<any> | null = null;

/**
 * Cache of last successful compilation results per file
 * Used to preserve styles when HMR compilation fails
 */
const compilationCache = new Map<string, { code: string; map: any }>();

/**
 * Initialize WASM module dynamically from @euis/wasm npm package.
 * Uses singleton pattern with promise caching to ensure initialization happens only once.
 * 
 * @returns Promise that resolves to the EuisCompiler instance
 * @throws Error with descriptive message if initialization fails
 */
async function initWASM(): Promise<any> {
  // Return cached instance if already initialized
  if (wasmCompiler) {
    return wasmCompiler;
  }

  // If initialization is in progress, wait for it
  if (!wasmInitPromise) {
    wasmInitPromise = (async () => {
      try {
        // Dynamic import of @euis/wasm package from node_modules
        const { EuisCompiler } = await import('@euis/wasm');
        wasmCompiler = new EuisCompiler();
        return wasmCompiler;
      } catch (error: any) {
        // Clear the promise so retry is possible
        wasmInitPromise = null;
        
        // Determine the likely cause of the error
        const errorMessage = error?.message || String(error);
        const isModuleNotFound = errorMessage.includes('Cannot find module') || 
                                  errorMessage.includes('MODULE_NOT_FOUND');
        const isWasmError = errorMessage.includes('WebAssembly') || 
                           errorMessage.includes('wasm');
        
        // Build comprehensive error message with troubleshooting guidance
        let troubleshootingSteps = 'Troubleshooting:\n';
        
        if (isModuleNotFound) {
          troubleshootingSteps += 
            `1. Install the WASM package: npm install @euis/wasm\n` +
            `2. If already installed, try reinstalling: npm install --force\n` +
            `3. Clear node_modules and reinstall: rm -rf node_modules && npm install\n`;
        } else if (isWasmError) {
          troubleshootingSteps += 
            `1. Verify your environment supports WebAssembly\n` +
            `2. Check Node.js version (requires 16+): node --version\n` +
            `3. Ensure your bundler supports WASM imports (Vite does by default)\n`;
        } else {
          troubleshootingSteps += 
            `1. Ensure @euis/wasm is installed: npm install @euis/wasm\n` +
            `2. Check that your bundler supports WASM imports (Vite does by default)\n` +
            `3. Verify you're using a compatible Node.js version (16+)\n`;
        }
        
        // Add cloud environment specific guidance
        troubleshootingSteps += 
          `\nCloud Environment Steps:\n` +
          `- Lovable: Ensure dependencies are installed and the workspace has synced\n` +
          `- StackBlitz: Wait for node_modules to fully install (check terminal output)\n` +
          `- CodeSandbox: Refresh the browser if dependencies don't load initially\n` +
          `- All platforms: Try restarting the dev server after installation\n` +
          `\nIf the issue persists, check that @euis/wasm is listed in package.json dependencies.`;
        
        throw new Error(
          `Failed to initialize Euis WASM compiler: ${errorMessage}\n\n${troubleshootingSteps}`
        );
      }
    })();
  }

  return wasmInitPromise;
}

/**
 * Convert a flat token map (key -> value) into the Rust Literal format
 * expected by the compiler: { key: { Literal: "value" } }
 */
function toLiteralTokens(
  map: Record<string, string> | undefined
): Record<string, { Literal: string }> | undefined {
  if (!map) return undefined;
  const result: Record<string, { Literal: string }> = {};
  for (const [key, value] of Object.entries(map)) {
    result[key] = { Literal: value };
  }
  return result;
}

/**
 * Build the Rust CompilerConfig JSON from plugin options.
 */
function buildConfigJson(options: EuisPluginOptions): string {
  const {
    minify = false,
    treeShaking = false,
    typedOM = false,
    sourceMaps = true,
    deduplicate = true,
    usedClasses = [],
    contentPaths = [],
    safelist = [],
    tokens = {},
  } = options;

  let sourceMapMode: string;
  if (!sourceMaps) {
    sourceMapMode = 'Disabled';
  } else {
    sourceMapMode = 'Inline';
  }

  const config: Record<string, unknown> = {
    minify,
    tree_shaking: treeShaking,
    typed_om: typedOM,
    source_maps: sourceMapMode,
    deduplicate,
    used_classes: usedClasses,
    content_paths: contentPaths,
    safelist,
    tokens: {
      colors: toLiteralTokens(tokens.colors) ?? {},
      spacing: toLiteralTokens(tokens.spacing) ?? {},
      typography: toLiteralTokens(tokens.typography) ?? {},
      breakpoints: toLiteralTokens(tokens.breakpoints) ?? {},
    },
  };

  return JSON.stringify(config);
}

/**
 * Format compiler errors/warnings for Vite's error overlay.
 * 
 * @param items - Array of diagnostic items (errors or warnings)
 * @param label - Label to prefix each diagnostic (e.g., 'error' or 'warning')
 * @returns Formatted multi-line string with line/column information
 * 
 * @example
 * ```typescript
 * const errors = [
 *   { message: 'Undefined token', line: 5, column: 12, severity: 'error' },
 *   { message: 'Invalid value', line: 8, severity: 'error' }
 * ];
 * const formatted = formatDiagnostics(errors, 'error');
 * // Returns:
 * //   error: Undefined token (5:12)
 * //   error: Invalid value (8)
 * ```
 */
export function formatDiagnostics(
  items: Array<{ message?: string; line?: number; column?: number; severity?: string }>,
  label: string
): string {
  return items
    .map((item) => {
      const loc =
        item.line != null
          ? ` (${item.line}${item.column != null ? ':' + item.column : ''})`
          : '';
      return `  ${label}: ${item.message ?? item}${loc}`;
    })
    .join('\n');
}

/**
 * Euis plugin factory function type.
 * Creates a Vite plugin that transforms .euis files to CSS.
 * 
 * @param options - Configuration options for the Euis compiler
 * @returns Vite plugin instance
 * 
 * @example
 * ```typescript
 * import { defineConfig } from 'vite';
 * import euis from 'vite-plugin-euis';
 * 
 * export default defineConfig({
 *   plugins: [
 *     euis({
 *       minify: true,
 *       sourceMaps: true,
 *       treeShaking: true,
 *       usedClasses: ['btn', 'card']
 *     })
 *   ]
 * });
 * ```
 */
export type EuisPlugin = (options?: EuisPluginOptions) => Plugin;

/**
 * Vite plugin for Euis.
 * Transforms .euis files to CSS with support for design tokens, tree-shaking, and minification.
 * 
 * @param options - Configuration options for the Euis compiler
 * @returns Vite plugin instance
 * 
 * @example
 * ```typescript
 * import { defineConfig } from 'vite';
 * import euis from 'vite-plugin-euis';
 * 
 * export default defineConfig({
 *   plugins: [
 *     euis({
 *       minify: true,
 *       sourceMaps: true,
 *       tokens: {
 *         colors: { primary: '#007bff' }
 *       }
 *     })
 *   ]
 * });
 * ```
 */
export const euisPlugin: EuisPlugin = (options: EuisPluginOptions = {}): Plugin => {
  const resolvedOptions = { ...options };
  let viteServer: any = null;
  let isProduction = false;

  return {
    name: 'vite-plugin-euis',
    enforce: 'pre',

    configResolved(config) {
      // Detect production mode to apply appropriate optimizations
      isProduction = config.command === 'build' || config.mode === 'production';
    },

    configureServer(server) {
      // Store reference to Vite dev server for HMR error handling
      viteServer = server;
    },

    async transform(code: string, id: string) {
      if (!id.endsWith('.euis')) {
        return null;
      }

      try {
        // Ensure WASM is initialized before first compilation
        const compiler = await initWASM();
        
        // Apply production-specific configuration
        // During production builds, ensure minification and tree-shaking are applied if enabled
        const productionOptions = isProduction ? {
          ...resolvedOptions,
          // Ensure minification is applied in production if enabled
          minify: resolvedOptions.minify ?? false,
          // Ensure tree-shaking is applied in production if enabled
          treeShaking: resolvedOptions.treeShaking ?? false,
        } : resolvedOptions;
        
        const configJson = buildConfigJson(productionOptions);
        const result: CompileResult = compiler.compile(code, configJson);

        // Surface warnings via Vite's warning mechanism
        if (result.warnings && result.warnings.length > 0) {
          this.warn(formatDiagnostics(result.warnings, 'warning'));
        }

        // Surface errors
        if (result.errors && result.errors.length > 0) {
          const errorMessage = `Euis compilation errors:\n${formatDiagnostics(result.errors, 'error')}`;
          
          // During HMR, send error to client without throwing
          // This preserves previous styles and shows error overlay
          if (viteServer) {
            viteServer.ws.send({
              type: 'error',
              err: {
                message: errorMessage,
                stack: '',
                id,
                frame: formatDiagnostics(result.errors, 'error'),
                plugin: 'vite-plugin-euis',
                loc: result.errors[0]?.line ? {
                  file: id,
                  line: result.errors[0].line,
                  column: result.errors[0].column || 0,
                } : undefined,
              },
            });
            
            // Return cached result to preserve previous styles
            const cached = compilationCache.get(id);
            if (cached) {
              return cached;
            }
            
            // If no cache, return empty CSS to avoid breaking the page
            return {
              code: 'export default "";',
              map: null,
            };
          }
          
          // During build, throw error normally
          this.error(errorMessage);
        }

        // Build the exported module code
        let moduleCode = `export default ${JSON.stringify(result.css)};`;

        // If the compiler produced a JS runtime chunk (e.g. Typed OM helpers),
        // append it as a named export so consumers can import it.
        if (result.js) {
          moduleCode += `\nexport const runtime = ${JSON.stringify(result.js)};`;
        }

        // Determine source map to return
        let map: any = null;
        if (resolvedOptions.sourceMaps !== false && result.source_map) {
          // result.source_map is a JSON string or object from the compiler
          map =
            typeof result.source_map === 'string'
              ? JSON.parse(result.source_map)
              : result.source_map;
        }

        const transformResult = {
          code: moduleCode,
          map,
        };

        // Cache successful compilation for HMR error recovery
        compilationCache.set(id, transformResult);

        return transformResult;
      } catch (error: any) {
        const message = error?.message ?? String(error);
        const errorMessage = `Euis compilation failed: ${message}`;
        
        // During HMR, send error to client without throwing
        if (viteServer) {
          viteServer.ws.send({
            type: 'error',
            err: {
              message: errorMessage,
              stack: error?.stack || '',
              id,
              plugin: 'vite-plugin-euis',
            },
          });
          
          // Return cached result to preserve previous styles
          const cached = compilationCache.get(id);
          if (cached) {
            return cached;
          }
          
          // If no cache, return empty CSS to avoid breaking the page
          return {
            code: 'export default "";',
            map: null,
          };
        }
        
        // During build, throw error normally
        this.error({
          message: errorMessage,
          id,
        });
      }
    },

    handleHotUpdate(ctx: HmrContext) {
      if (ctx.file.endsWith('.euis')) {
        // For .euis files, we want to trigger CSS-only HMR without full page reload
        // Return the affected modules so Vite can handle the HMR update
        // This allows styles to update without losing application state
        
        // ctx.modules contains all modules that import this .euis file
        // By returning them, Vite will invalidate and re-transform these modules
        // Since .euis files are transformed to CSS exports, Vite's built-in CSS HMR
        // will apply the style changes without a full page reload
        return ctx.modules;
      }
    },
  };
};

export default euisPlugin;
