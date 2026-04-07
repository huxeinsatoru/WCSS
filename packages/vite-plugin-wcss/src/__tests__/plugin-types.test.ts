/**
 * Tests for plugin TypeScript type definitions
 * 
 * These tests verify that the exported types (WCSSPluginOptions, WCSSPlugin)
 * provide correct type information for plugin consumers.
 */

import type { Plugin } from 'vite';
import { wcssPlugin, type WCSSPluginOptions, type WCSSPlugin } from '../index';

describe('Plugin Type Definitions', () => {
  describe('WCSSPluginOptions interface', () => {
    it('should accept all valid configuration options', () => {
      const options: WCSSPluginOptions = {
        treeShaking: true,
        minify: true,
        sourceMaps: true,
        typedOM: false,
        deduplicate: true,
        usedClasses: ['btn', 'card'],
        contentPaths: ['src/**/*.tsx'],
        safelist: ['active', 'disabled'],
        tokens: {
          colors: { primary: '#007bff' },
          spacing: { sm: '0.5rem' },
          typography: { base: '16px' },
          breakpoints: { md: '768px' },
        },
      };

      expect(options).toBeDefined();
      expect(options.treeShaking).toBe(true);
      expect(options.tokens?.colors?.primary).toBe('#007bff');
    });

    it('should accept partial configuration', () => {
      const options: WCSSPluginOptions = {
        minify: true,
      };

      expect(options).toBeDefined();
      expect(options.minify).toBe(true);
    });

    it('should accept empty configuration', () => {
      const options: WCSSPluginOptions = {};

      expect(options).toBeDefined();
    });

    it('should accept tokens with only some categories', () => {
      const options: WCSSPluginOptions = {
        tokens: {
          colors: { primary: '#007bff' },
          // spacing, typography, breakpoints are optional
        },
      };

      expect(options.tokens?.colors).toBeDefined();
      expect(options.tokens?.spacing).toBeUndefined();
    });
  });

  describe('WCSSPlugin type', () => {
    it('should match the plugin function signature', () => {
      // Verify that wcssPlugin matches the WCSSPlugin type
      const plugin: WCSSPlugin = wcssPlugin;

      expect(plugin).toBe(wcssPlugin);
    });

    it('should accept options parameter', () => {
      const plugin: WCSSPlugin = (options?: WCSSPluginOptions) => {
        return wcssPlugin(options);
      };

      const result = plugin({ minify: true });
      expect(result).toBeDefined();
      expect(result.name).toBe('vite-plugin-wcss');
    });

    it('should return a Vite Plugin', () => {
      const result: Plugin = wcssPlugin();

      expect(result).toBeDefined();
      expect(result.name).toBe('vite-plugin-wcss');
      expect(result.transform).toBeDefined();
      expect(result.handleHotUpdate).toBeDefined();
    });
  });

  describe('wcssPlugin function', () => {
    it('should work without options', () => {
      const plugin = wcssPlugin();

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-wcss');
    });

    it('should work with options', () => {
      const plugin = wcssPlugin({
        minify: true,
        sourceMaps: false,
      });

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-wcss');
    });

    it('should work with full configuration', () => {
      const plugin = wcssPlugin({
        treeShaking: true,
        minify: true,
        sourceMaps: true,
        typedOM: false,
        deduplicate: true,
        usedClasses: ['btn'],
        contentPaths: ['src/**/*.tsx'],
        safelist: ['active'],
        tokens: {
          colors: { primary: '#007bff' },
        },
      });

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-wcss');
    });
  });

  describe('Type safety', () => {
    it('should provide type checking for options', () => {
      // This test verifies that TypeScript catches invalid options at compile time
      // If this file compiles without errors, it means the types are working correctly

      const validOptions: WCSSPluginOptions = {
        minify: true,
        // TypeScript would error if we tried to add an invalid property:
        // invalidOption: true, // Error: Object literal may only specify known properties
      };

      expect(validOptions).toBeDefined();
    });

    it('should enforce correct types for option values', () => {
      const options: WCSSPluginOptions = {
        minify: true, // boolean
        usedClasses: ['btn', 'card'], // string[]
        tokens: {
          colors: { primary: '#007bff' }, // Record<string, string>
        },
        // TypeScript would error if we used wrong types:
        // minify: 'true', // Error: Type 'string' is not assignable to type 'boolean'
        // usedClasses: 'btn', // Error: Type 'string' is not assignable to type 'string[]'
      };

      expect(options).toBeDefined();
    });
  });

  describe('Default export', () => {
    it('should export wcssPlugin as default', async () => {
      // Test that the default export works
      const defaultExport = (await import('../index')).default;

      expect(defaultExport).toBe(wcssPlugin);
    });

    it('should work with default import syntax', async () => {
      // Simulate: import wcss from 'vite-plugin-wcss';
      const wcss = (await import('../index')).default;
      const plugin = wcss({ minify: true });

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-wcss');
    });
  });

  describe('Named exports', () => {
    it('should export WCSSPluginOptions type', () => {
      // This is a compile-time test - if this file compiles, the type is exported
      type Options = WCSSPluginOptions;
      const options: Options = { minify: true };

      expect(options).toBeDefined();
    });

    it('should export WCSSPlugin type', () => {
      // This is a compile-time test - if this file compiles, the type is exported
      type PluginFactory = WCSSPlugin;
      const factory: PluginFactory = wcssPlugin;

      expect(factory).toBeDefined();
    });

    it('should export wcssPlugin function', () => {
      // Verify the named export works
      expect(wcssPlugin).toBeDefined();
      expect(typeof wcssPlugin).toBe('function');
    });
  });
});
