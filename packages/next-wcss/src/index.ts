import type { NextConfig } from 'next';

export interface WCSSNextOptions {
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
 * Next.js plugin for WCSS
 * 
 * Usage:
 * ```js
 * // next.config.js
 * const { withWCSS } = require('next-wcss');
 * 
 * module.exports = withWCSS({
 *   // WCSS options
 *   wcss: {
 *     minify: true,
 *     tokens: {
 *       colors: { primary: '#3b82f6' }
 *     }
 *   }
 * });
 * ```
 */
export function withWCSS(
  nextConfig: NextConfig & { wcss?: WCSSNextOptions } = {}
): NextConfig {
  const { wcss: wcssOptions = {}, ...restConfig } = nextConfig;

  return {
    ...restConfig,
    webpack(config, options) {
      // Add WCSS loader rule
      config.module.rules.push({
        test: /\.wcss$/,
        use: [
          {
            loader: 'wcss-loader',
            options: {
              treeShaking: wcssOptions.treeShaking ?? false,
              minify: wcssOptions.minify ?? false,
              sourceMaps: wcssOptions.sourceMaps ?? true,
              typedOM: wcssOptions.typedOM ?? false,
              tokens: wcssOptions.tokens,
            },
          },
        ],
      });

      // Chain with existing webpack config if present
      if (typeof restConfig.webpack === 'function') {
        return restConfig.webpack(config, options);
      }

      return config;
    },
  };
}

export default withWCSS;
