import type { AstroIntegration } from 'astro';
import { wcssPlugin } from 'vite-plugin-wcss';

export interface WCSSAstroOptions {
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
 * Astro integration for WCSS
 * 
 * Usage:
 * ```js
 * // astro.config.mjs
 * import { defineConfig } from 'astro/config';
 * import wcss from 'astro-wcss';
 *
 * export default defineConfig({
 *   integrations: [wcss({
 *     minify: true,
 *     tokens: {
 *       colors: { primary: '#3b82f6' }
 *     }
 *   })],
 * });
 * ```
 */
export default function wcssAstroIntegration(
  options: WCSSAstroOptions = {}
): AstroIntegration {
  const {
    treeShaking = false,
    minify = false,
    sourceMaps = true,
    typedOM = false,
    tokens = {},
  } = options;

  return {
    name: 'astro-wcss',
    hooks: {
      'astro:config:setup': ({ updateConfig }) => {
        updateConfig({
          vite: {
            plugins: [
              wcssPlugin({
                treeShaking,
                minify,
                sourceMaps,
                typedOM,
                tokens,
              }),
            ],
          },
        });
      },
    },
  };
}

export { wcssAstroIntegration as wcss };
