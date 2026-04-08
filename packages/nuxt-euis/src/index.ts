import { defineNuxtModule, addWebpackPlugin, addVitePlugin } from '@nuxt/kit';
import { euisPlugin } from 'vite-plugin-euis';

export interface EuisNuxtOptions {
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
 * Nuxt module for Euis
 * 
 * Usage:
 * ```js
 * // nuxt.config.ts
 * export default defineNuxtConfig({
 *   modules: ['nuxt-euis'],
 *   euis: {
 *     minify: true,
 *     tokens: {
 *       colors: { primary: '#3b82f6' }
 *     }
 *   }
 * });
 * ```
 */
export default defineNuxtModule<EuisNuxtOptions>({
  meta: {
    name: 'euis',
    configKey: 'euis',
  },
  
  defaults: {
    treeShaking: false,
    minify: false,
    sourceMaps: true,
    typedOM: false,
    tokens: {},
  },
  
  setup(options, nuxt) {
    // Add Vite plugin
    addVitePlugin(euisPlugin({
      treeShaking: options.treeShaking,
      minify: options.minify,
      sourceMaps: options.sourceMaps,
      typedOM: options.typedOM,
      tokens: options.tokens,
    }));

    // Add Webpack loader for webpack-based builds
    nuxt.hook('webpack:config', (configs) => {
      for (const config of configs) {
        config.module = config.module || {};
        config.module.rules = config.module.rules || [];
        
        config.module.rules.push({
          test: /\.euis$/,
          use: [
            {
              loader: 'euis-loader',
              options: {
                treeShaking: options.treeShaking,
                minify: options.minify,
                sourceMaps: options.sourceMaps,
                typedOM: options.typedOM,
                tokens: options.tokens,
              },
            },
          ],
        });
      }
    });
  },
});

export { EuisNuxtOptions };
