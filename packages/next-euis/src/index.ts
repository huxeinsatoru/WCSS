import type { NextConfig } from 'next';

export interface EuisNextOptions {
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
 * Next.js plugin for Euis
 * 
 * Usage:
 * ```js
 * // next.config.js
 * const { withEuis } = require('next-euis');
 * 
 * module.exports = withEuis({
 *   // Euis options
 *   euis: {
 *     minify: true,
 *     tokens: {
 *       colors: { primary: '#3b82f6' }
 *     }
 *   }
 * });
 * ```
 */
export function withEuis(
  nextConfig: NextConfig & { euis?: EuisNextOptions } = {}
): NextConfig {
  const { euis: euisOptions = {}, ...restConfig } = nextConfig;

  return {
    ...restConfig,
    webpack(config, options) {
      // Add Euis loader rule
      config.module.rules.push({
        test: /\.euis$/,
        use: [
          {
            loader: 'euis-loader',
            options: {
              treeShaking: euisOptions.treeShaking ?? false,
              minify: euisOptions.minify ?? false,
              sourceMaps: euisOptions.sourceMaps ?? true,
              typedOM: euisOptions.typedOM ?? false,
              tokens: euisOptions.tokens,
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

export default withEuis;
