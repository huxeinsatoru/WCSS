import { withWCSS, WCSSNextOptions } from '../index';
import type { NextConfig } from 'next';

describe('Next.js Plugin Integration Tests', () => {
  describe('sample Next.js project integration', () => {
    it('should add WCSS loader to webpack config', () => {
      const nextConfig: NextConfig = {};
      const config = withWCSS(nextConfig);

      expect(config.webpack).toBeDefined();

      // Simulate webpack config
      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);

        expect(result.module.rules).toHaveLength(1);
        expect(result.module.rules[0].test).toEqual(/\.wcss$/);
        expect(result.module.rules[0].use).toBeDefined();
      }
    });

    it('should pass WCSS options to loader', () => {
      const wcssOptions: WCSSNextOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: true,
        typedOM: false,
        tokens: {
          colors: { primary: '#3b82f6' },
          spacing: { md: '1rem' },
        },
      };

      const nextConfig: NextConfig & { wcss?: WCSSNextOptions } = {
        wcss: wcssOptions,
      };

      const config = withWCSS(nextConfig);

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);

        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].loader).toBe('wcss-loader');
        expect(wcssRule.use[0].options).toEqual({
          treeShaking: true,
          minify: true,
          sourceMaps: true,
          typedOM: false,
          tokens: wcssOptions.tokens,
        });
      }
    });

    it('should chain with existing webpack config', () => {
      const existingWebpackFn = jest.fn((config) => {
        config.customProperty = 'test';
        return config;
      });

      const nextConfig: NextConfig = {
        webpack: existingWebpackFn,
      };

      const config = withWCSS(nextConfig);

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);

        // Should have called existing webpack function
        expect(existingWebpackFn).toHaveBeenCalled();
        expect(result.customProperty).toBe('test');
        // Should also have added WCSS rule
        expect(result.module.rules).toHaveLength(1);
      }
    });

    it('should preserve other Next.js config options', () => {
      const nextConfig: NextConfig = {
        reactStrictMode: true,
        swcMinify: true,
        images: {
          domains: ['example.com'],
        },
      };

      const config = withWCSS(nextConfig);

      expect(config.reactStrictMode).toBe(true);
      expect(config.swcMinify).toBe(true);
      expect(config.images).toEqual({ domains: ['example.com'] });
    });

    it('should use default WCSS options when none provided', () => {
      const nextConfig: NextConfig = {};
      const config = withWCSS(nextConfig);

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);

        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options).toEqual({
          treeShaking: false,
          minify: false,
          sourceMaps: true,
          typedOM: false,
          tokens: undefined,
        });
      }
    });
  });

  describe('.wcss file processing', () => {
    it('should configure loader to process .wcss files', () => {
      const config = withWCSS({});

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);

        const wcssRule = result.module.rules.find((rule: any) =>
          rule.test?.toString().includes('wcss')
        );

        expect(wcssRule).toBeDefined();
        expect(wcssRule.test).toEqual(/\.wcss$/);
      }
    });

    it('should handle multiple .wcss files', () => {
      const config = withWCSS({});

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);

        // Test that the rule would match multiple files
        const wcssRule = result.module.rules[0];
        expect(wcssRule.test.test('button.wcss')).toBe(true);
        expect(wcssRule.test.test('layout.wcss')).toBe(true);
        expect(wcssRule.test.test('theme.wcss')).toBe(true);
        expect(wcssRule.test.test('button.css')).toBe(false);
      }
    });

    it('should work in both client and server builds', () => {
      const config = withWCSS({});

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      // Test client build
      const clientOptions = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const clientResult = config.webpack({ ...webpackConfig }, clientOptions as any);
        expect(clientResult.module.rules).toHaveLength(1);

        // Test server build
        const serverOptions = {
          isServer: true,
          dev: true,
        };

        const serverResult = config.webpack({ ...webpackConfig }, serverOptions as any);
        expect(serverResult.module.rules).toHaveLength(1);
      }
    });

    it('should work in both development and production modes', () => {
      const config = withWCSS({});

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      // Test development mode
      const devOptions = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const devResult = config.webpack({ ...webpackConfig }, devOptions as any);
        expect(devResult.module.rules).toHaveLength(1);

        // Test production mode
        const prodOptions = {
          isServer: false,
          dev: false,
        };

        const prodResult = config.webpack({ ...webpackConfig }, prodOptions as any);
        expect(prodResult.module.rules).toHaveLength(1);
      }
    });
  });

  describe('configuration options', () => {
    it('should support minification option', () => {
      const config = withWCSS({
        wcss: { minify: true },
      });

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: false,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options.minify).toBe(true);
      }
    });

    it('should support tree shaking option', () => {
      const config = withWCSS({
        wcss: { treeShaking: true },
      });

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: false,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options.treeShaking).toBe(true);
      }
    });

    it('should support source maps option', () => {
      const config = withWCSS({
        wcss: { sourceMaps: false },
      });

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options.sourceMaps).toBe(false);
      }
    });

    it('should support Typed OM option', () => {
      const config = withWCSS({
        wcss: { typedOM: true },
      });

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options.typedOM).toBe(true);
      }
    });

    it('should support design tokens configuration', () => {
      const tokens = {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6',
          danger: '#ef4444',
        },
        spacing: {
          xs: '0.25rem',
          sm: '0.5rem',
          md: '1rem',
        },
        typography: {
          'font-sans': 'system-ui, sans-serif',
        },
        breakpoints: {
          sm: '640px',
          md: '768px',
        },
      };

      const config = withWCSS({
        wcss: { tokens },
      });

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options.tokens).toEqual(tokens);
      }
    });

    it('should support all options combined', () => {
      const wcssOptions: WCSSNextOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: true,
        typedOM: true,
        tokens: {
          colors: { primary: '#3b82f6' },
          spacing: { md: '1rem' },
          typography: { 'font-sans': 'system-ui' },
          breakpoints: { md: '768px' },
        },
      };

      const config = withWCSS({
        wcss: wcssOptions,
      });

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: false,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        const wcssRule = result.module.rules[0];
        expect(wcssRule.use[0].options).toEqual(wcssOptions);
      }
    });
  });

  describe('edge cases', () => {
    it('should handle empty Next.js config', () => {
      const config = withWCSS();

      expect(config.webpack).toBeDefined();
    });

    it('should handle config with only webpack function', () => {
      const existingWebpack = (config: any) => {
        config.test = true;
        return config;
      };

      const nextConfig: NextConfig = {
        webpack: existingWebpack,
      };

      const config = withWCSS(nextConfig);

      const webpackConfig = {
        module: {
          rules: [] as any[],
        },
      };

      const options = {
        isServer: false,
        dev: true,
      };

      if (config.webpack) {
        const result = config.webpack(webpackConfig, options as any);
        expect(result.test).toBe(true);
        expect(result.module.rules).toHaveLength(1);
      }
    });

    it('should not modify original Next.js config', () => {
      const originalConfig: NextConfig = {
        reactStrictMode: true,
      };

      const config = withWCSS(originalConfig);

      // Original config should not have webpack function
      expect(originalConfig.webpack).toBeUndefined();
      // New config should have webpack function
      expect(config.webpack).toBeDefined();
    });
  });
});
