import * as path from 'path';
import { CompilerConfig, CompileResult, EuisCompiler } from './types';
import { globalCache } from './cache';

let compilerInstance: EuisCompiler | null = null;

// Type for the WASM module
interface WASMModule {
  EuisCompiler: new () => EuisCompiler;
}

/**
 * Load the WASM compiler module.
 * Uses singleton pattern — loads once and reuses.
 */
export async function loadCompiler(): Promise<EuisCompiler> {
  if (compilerInstance) {
    return compilerInstance as EuisCompiler;
  }

  try {
    // Load the real WASM compiler from pkg/nodejs
    const wasmPath = path.resolve(__dirname, '../../../pkg/nodejs/euis_wasm.js');
    const wasm = require(wasmPath);
    compilerInstance = new wasm.EuisCompiler();
    return compilerInstance!;
  } catch (error: any) {
    throw new Error(
      `Failed to load Euis WASM compiler.\n` +
      `This may be due to:\n` +
      `  1. WASM module not built (run: npm run build:wasm from project root)\n` +
      `  2. Missing wasm32 target (run: rustup target add wasm32-unknown-unknown)\n` +
      `  3. Incompatible Node.js version (requires Node.js 16+)\n` +
      `Original error: ${error.message}`
    );
  }
}

/**
 * Map JS config to Rust CompilerConfig JSON format.
 */
function mapConfig(config: CompilerConfig): string {
  const rustConfig: Record<string, any> = {
    minify: config.minify ?? true,
    tree_shaking: config.treeShaking ?? false,
    typed_om: config.typedOM ?? false,
    source_maps: config.sourceMaps === true || config.sourceMaps === 'inline'
      ? 'Inline'
      : config.sourceMaps === 'external'
        ? 'External'
        : 'Disabled',
    deduplicate: true,
    used_classes: [],
    content_paths: [],
    safelist: [],
    tokens: {
      colors: {},
      spacing: {},
      typography: {},
      breakpoints: {},
    },
  };

  if (config.tokens) {
    const { colors, spacing, typography, breakpoints } = config.tokens;
    if (colors) {
      for (const [k, v] of Object.entries(colors)) {
        rustConfig.tokens.colors[k] = { Literal: v };
      }
    }
    if (spacing) {
      for (const [k, v] of Object.entries(spacing)) {
        rustConfig.tokens.spacing[k] = { Literal: v };
      }
    }
    if (typography) {
      for (const [k, v] of Object.entries(typography)) {
        rustConfig.tokens.typography[k] = { Literal: v };
      }
    }
    if (breakpoints) {
      for (const [k, v] of Object.entries(breakpoints)) {
        rustConfig.tokens.breakpoints[k] = { Literal: v };
      }
    }
  }

  return JSON.stringify(rustConfig);
}

/**
 * Compile Euis source to CSS using the real WASM compiler.
 * Results are cached based on source + config hash.
 */
export async function compile(
  source: string,
  config: CompilerConfig
): Promise<CompileResult> {
  const compiler = await loadCompiler();
  const configJson = mapConfig(config);

  // Check cache
  const cacheKey = globalCache.getCacheKey(source, configJson);
  const cached = globalCache.get(cacheKey);
  if (cached) return cached;

  const result = compiler.compile(source, configJson);

  // Only cache successful compilations
  if (result.errors.length === 0) {
    globalCache.set(cacheKey, result);
  }

  return result;
}

/**
 * Format Euis source using the real WASM compiler.
 */
export async function format(source: string): Promise<string> {
  const compiler = await loadCompiler();
  return compiler.format(source);
}

/**
 * Validate Euis source without full compilation.
 */
export async function validate(
  source: string,
  config: CompilerConfig
): Promise<any[]> {
  const compiler = await loadCompiler();
  const configJson = mapConfig(config);
  return compiler.validate(source, configJson);
}
