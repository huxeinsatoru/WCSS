import type { LoaderContext } from 'webpack';
import * as path from 'path';

export interface EuisLoaderOptions {
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
   * Deduplicate generated CSS rules
   * @default true
   */
  deduplicate?: boolean;

  /**
   * List of classes known to be used (for tree shaking)
   */
  usedClasses?: string[];

  /**
   * Glob paths to scan for class usage (for tree shaking)
   */
  contentPaths?: string[];

  /**
   * Classes that should never be removed by tree shaking
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

// Singleton: WASM compiler instance, loaded once and reused
let compilerInstance: any = null;
let compilerLoadPromise: Promise<any> | null = null;

/**
 * Load the WASM compiler singleton. Returns the cached instance
 * on subsequent calls.
 */
function getCompiler(): Promise<any> {
  if (compilerInstance) {
    return Promise.resolve(compilerInstance);
  }
  if (compilerLoadPromise) {
    return compilerLoadPromise;
  }
  compilerLoadPromise = (async () => {
    const wasmPath = path.resolve(__dirname, '../../../pkg/nodejs/euis_wasm.js');
    const { EuisCompiler } = await import(wasmPath);
    compilerInstance = new EuisCompiler();
    return compilerInstance;
  })();
  return compilerLoadPromise;
}

/**
 * Convert a flat token record (key -> value) into Rust's expected format
 * where each entry is { Literal: "value" }.
 */
function mapTokenCategory(
  record: Record<string, string> | undefined
): Record<string, { Literal: string }> | undefined {
  if (!record) return undefined;
  const mapped: Record<string, { Literal: string }> = {};
  for (const [key, value] of Object.entries(record)) {
    mapped[key] = { Literal: value };
  }
  return mapped;
}

/**
 * Build the Rust CompilerConfig JSON from loader options.
 */
function buildConfigJson(options: EuisLoaderOptions, webpackSourceMap: boolean | undefined): string {
  const useSourceMaps = options.sourceMaps ?? webpackSourceMap ?? true;

  let source_maps: string;
  if (!useSourceMaps) {
    source_maps = 'Disabled';
  } else {
    source_maps = 'External';
  }

  const config: Record<string, unknown> = {
    minify: options.minify ?? false,
    tree_shaking: options.treeShaking ?? false,
    typed_om: options.typedOM ?? false,
    source_maps,
    deduplicate: options.deduplicate ?? true,
    used_classes: options.usedClasses ?? [],
    content_paths: options.contentPaths ?? [],
    safelist: options.safelist ?? [],
  };

  if (options.tokens) {
    const tokens: Record<string, unknown> = {};
    if (options.tokens.colors) tokens.colors = mapTokenCategory(options.tokens.colors);
    if (options.tokens.spacing) tokens.spacing = mapTokenCategory(options.tokens.spacing);
    if (options.tokens.typography) tokens.typography = mapTokenCategory(options.tokens.typography);
    if (options.tokens.breakpoints) tokens.breakpoints = mapTokenCategory(options.tokens.breakpoints);
    config.tokens = tokens;
  }

  return JSON.stringify(config);
}

/**
 * Webpack loader for Euis
 */
export default function euisLoader(
  this: LoaderContext<EuisLoaderOptions>,
  source: string
): void {
  const options = this.getOptions() || {};
  const callback = this.async();

  if (!callback) {
    throw new Error('euis-loader requires async mode');
  }

  getCompiler()
    .then((compiler) => {
      const configJson = buildConfigJson(options, this.sourceMap);
      const result = compiler.compile(source, configJson);

      // Emit warnings through webpack
      if (result.warnings && result.warnings.length > 0) {
        for (const warning of result.warnings) {
          this.emitWarning(new Error(String(warning)));
        }
      }

      // Emit errors through webpack but still fail the build
      if (result.errors && result.errors.length > 0) {
        for (const err of result.errors) {
          this.emitError(new Error(String(err)));
        }
        const errorMessage = `Euis compilation errors:\n${result.errors.map((e: any) => `  - ${e}`).join('\n')}`;
        callback(new Error(errorMessage));
        return;
      }

      // Parse source map if present
      const sourceMapObj = result.sourceMap ? JSON.parse(result.sourceMap) : undefined;

      callback(null, result.css, sourceMapObj);
    })
    .catch((error: unknown) => {
      callback(error instanceof Error ? error : new Error(String(error)));
    });
}
