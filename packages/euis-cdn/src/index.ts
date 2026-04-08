/**
 * CDN Bundle for Euis
 * Enables using Euis without a build step in the browser
 *
 * Usage:
 * ```html
 * <!DOCTYPE html>
 * <html>
 * <head>
 *   <script src="https://unpkg.com/@euis/cdn@latest/dist/euis.min.js"></script>
 *   <script type="text/euis">
 *     .button {
 *       padding: $spacing.md;
 *       background: $colors.primary;
 *       color: white;
 *     }
 *   </script>
 * </head>
 * <body>
 *   <button class="button">Click me</button>
 * </body>
 * </html>
 * ```
 */

export interface EuisCDNOptions {
  /**
   * Auto-initialize on DOMContentLoaded
   * @default true
   */
  autoInit?: boolean;

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

/** WASM module type from pkg/web */
interface WasmModule {
  default: () => Promise<void>;
  compile: (source: string, config: string) => string;
}

let wasmModule: WasmModule | null = null;
let wasmReady: Promise<void> | null = null;

/**
 * Load and initialize the WASM compiler (browser/web build).
 * Returns a promise that resolves once the WASM binary is ready.
 */
function loadWasm(): Promise<void> {
  if (wasmReady) {
    return wasmReady;
  }

  wasmReady = (async () => {
    const mod: WasmModule = await import(
      /* webpackIgnore: true */
      'pkg/web/euis_wasm.js'
    );
    await mod.default(); // init() – fetches & instantiates the .wasm file
    wasmModule = mod;
  })();

  return wasmReady;
}

/**
 * Convert the user-facing tokens config into the Rust-side format where every
 * leaf value is wrapped as `{ Literal: "value" }`.
 */
function buildRustConfig(tokens: EuisCDNOptions['tokens'] = {}): string {
  const wrapCategory = (
    cat: Record<string, string> | undefined
  ): Record<string, { Literal: string }> => {
    const out: Record<string, { Literal: string }> = {};
    if (cat) {
      for (const [k, v] of Object.entries(cat)) {
        out[k] = { Literal: v };
      }
    }
    return out;
  };

  const config = {
    colors: wrapCategory(tokens.colors),
    spacing: wrapCategory(tokens.spacing),
    typography: wrapCategory(tokens.typography),
    breakpoints: wrapCategory(tokens.breakpoints),
    source_maps: 'Disabled' as 'Disabled' | 'Inline' | 'External',
  };

  return JSON.stringify(config);
}

/**
 * Compile Euis source to CSS via the WASM compiler.
 * The WASM module must already be loaded before calling this.
 */
function compileEuis(
  source: string,
  tokens: EuisCDNOptions['tokens'] = {}
): { css: string; errors: string[] } {
  if (!wasmModule) {
    return { css: '', errors: ['WASM compiler not loaded'] };
  }

  const errors: string[] = [];

  try {
    const css = wasmModule.compile(source, buildRustConfig(tokens));
    return { css, errors };
  } catch (err: any) {
    errors.push(String(err));
    return { css: '', errors };
  }
}

/**
 * Euis CDN runtime
 */
export class EuisCDN {
  private options: EuisCDNOptions;
  private compiledStyles: Map<HTMLScriptElement, HTMLStyleElement> = new Map();
  private wasmLoaded: Promise<void>;

  constructor(options: EuisCDNOptions = {}) {
    this.options = {
      autoInit: true,
      ...options,
    };

    // Start loading WASM immediately
    this.wasmLoaded = loadWasm();

    if (this.options.autoInit) {
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => this.init());
      } else {
        this.init();
      }
    }
  }

  /**
   * Initialize Euis - wait for WASM then find and compile all Euis script tags
   */
  async init(): Promise<void> {
    try {
      await this.wasmLoaded;
    } catch (err) {
      console.error('[Euis] Failed to load WASM compiler:', err);
      return;
    }

    const scripts = document.querySelectorAll('script[type="text/euis"]');

    scripts.forEach((script) => {
      this.compileAndInject(script as HTMLScriptElement);
    });

    console.log(`[Euis] Compiled ${scripts.length} Euis script tag(s)`);
  }

  /**
   * Compile a single Euis script and inject styles
   */
  private compileAndInject(script: HTMLScriptElement): void {
    const source = script.textContent || '';

    if (!source.trim()) {
      return;
    }

    try {
      const result = compileEuis(source, this.options.tokens);

      if (result.errors.length > 0) {
        console.error('[Euis] Compilation errors:', result.errors);
      }

      // Remove previously compiled styles for this script
      const existingStyle = this.compiledStyles.get(script);
      if (existingStyle) {
        existingStyle.remove();
      }

      // Create and inject style element
      const style = document.createElement('style');
      style.textContent = result.css;
      style.setAttribute('data-euis', '');

      // Insert into head
      document.head.appendChild(style);

      // Track the style element
      this.compiledStyles.set(script, style);
    } catch (error) {
      console.error('[Euis] Compilation failed:', error);
    }
  }

  /**
   * Manually compile Euis source to CSS.
   * Must be called after WASM is loaded (await init() first).
   */
  compile(source: string): { css: string; errors: string[] } {
    return compileEuis(source, this.options.tokens);
  }

  /**
   * Recompile all Euis scripts (useful for dynamic content)
   */
  async refresh(): Promise<void> {
    this.compiledStyles.forEach((style) => style.remove());
    this.compiledStyles.clear();
    await this.init();
  }
}

// Auto-initialize if not in module mode
if (typeof window !== 'undefined') {
  (window as any).Euis = EuisCDN;

  // Auto-initialize unless disabled
  if (!(window as any).Euis_DISABLE_AUTO_INIT) {
    new EuisCDN();
  }
}

export default EuisCDN;
