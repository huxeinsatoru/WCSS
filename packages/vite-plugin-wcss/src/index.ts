import type { Plugin, HmrContext } from 'vite';
import * as path from 'path';

export interface WCSSPluginOptions {
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
 * Singleton WASM compiler instance
 */
let wasmCompiler: any = null;

/**
 * Load the WASM compiler module once and cache it.
 */
function getCompiler(): any {
  if (wasmCompiler) {
    return wasmCompiler;
  }
  const wasmPath = path.resolve(__dirname, '../../../pkg/nodejs/wcss_wasm.js');
  try {
    wasmCompiler = require(wasmPath);
  } catch (err: any) {
    throw new Error(
      `Failed to load WCSS WASM compiler from ${wasmPath}: ${err.message}`
    );
  }
  return wasmCompiler;
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
function buildConfigJson(options: WCSSPluginOptions): string {
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
  };

  // Only include tokens if at least one category is provided
  const hasTokens =
    tokens.colors || tokens.spacing || tokens.typography || tokens.breakpoints;
  if (hasTokens) {
    config.tokens = {
      colors: toLiteralTokens(tokens.colors) ?? {},
      spacing: toLiteralTokens(tokens.spacing) ?? {},
      typography: toLiteralTokens(tokens.typography) ?? {},
      breakpoints: toLiteralTokens(tokens.breakpoints) ?? {},
    };
  }

  return JSON.stringify(config);
}

/**
 * Format compiler errors/warnings for Vite's error overlay.
 */
function formatDiagnostics(
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
 * Vite plugin for WCSS
 */
export function wcssPlugin(options: WCSSPluginOptions = {}): Plugin {
  const resolvedOptions = { ...options };

  return {
    name: 'vite-plugin-wcss',
    enforce: 'pre',

    async transform(code: string, id: string) {
      if (!id.endsWith('.wcss')) {
        return null;
      }

      try {
        const compiler = getCompiler();
        const configJson = buildConfigJson(resolvedOptions);
        const result = compiler.compile(code, configJson);

        // Surface warnings via Vite's warning mechanism
        if (result.warnings && result.warnings.length > 0) {
          this.warn(formatDiagnostics(result.warnings, 'warning'));
        }

        // Surface errors
        if (result.errors && result.errors.length > 0) {
          this.error(
            `WCSS compilation errors:\n${formatDiagnostics(result.errors, 'error')}`
          );
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
        if (resolvedOptions.sourceMaps !== false && result.sourceMap) {
          // result.sourceMap is a JSON string or object from the compiler
          map =
            typeof result.sourceMap === 'string'
              ? JSON.parse(result.sourceMap)
              : result.sourceMap;
        }

        return {
          code: moduleCode,
          map,
        };
      } catch (error: any) {
        const message = error?.message ?? String(error);
        this.error({
          message: `WCSS compilation failed: ${message}`,
          id,
        });
      }
    },

    handleHotUpdate(ctx: HmrContext) {
      if (ctx.file.endsWith('.wcss')) {
        // Trigger full reload for WCSS files
        ctx.server.ws.send({
          type: 'full-reload',
        });
        return [];
      }
    },
  };
}

export default wcssPlugin;
