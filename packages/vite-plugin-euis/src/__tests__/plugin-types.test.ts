/**
 * Tests for plugin TypeScript type definitions
 * 
 * These tests verify that the exported types (EuisPluginOptions, EuisPlugin)
 * provide correct type information for plugin consumers.
 */

import type { Plugin } from 'vite';
import { euisPlugin, type EuisPluginOptions, type EuisPlugin } from '../index';

describe('Plugin Type Definitions', () => {
  describe('EuisPluginOptions interface', () => {
    it('should accept all valid configuration options', () => {
      const options: EuisPluginOptions = {
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
      const options: EuisPluginOptions = {
        minify: true,
      };

      expect(options).toBeDefined();
      expect(options.minify).toBe(true);
    });

    it('should accept empty configuration', () => {
      const options: EuisPluginOptions = {};

      expect(options).toBeDefined();
    });

    it('should accept tokens with only some categories', () => {
      const options: EuisPluginOptions = {
        tokens: {
          colors: { primary: '#007bff' },
          // spacing, typography, breakpoints are optional
        },
      };

      expect(options.tokens?.colors).toBeDefined();
      expect(options.tokens?.spacing).toBeUndefined();
    });
  });

  describe('EuisPlugin type', () => {
    it('should match the plugin function signature', () => {
      // Verify that euisPlugin matches the EuisPlugin type
      const plugin: EuisPlugin = euisPlugin;

      expect(plugin).toBe(euisPlugin);
    });

    it('should accept options parameter', () => {
      const plugin: EuisPlugin = (options?: EuisPluginOptions) => {
        return euisPlugin(options);
      };

      const result = plugin({ minify: true });
      expect(result).toBeDefined();
      expect(result.name).toBe('vite-plugin-euis');
    });

    it('should return a Vite Plugin', () => {
      const result: Plugin = euisPlugin();

      expect(result).toBeDefined();
      expect(result.name).toBe('vite-plugin-euis');
      expect(result.transform).toBeDefined();
      expect(result.handleHotUpdate).toBeDefined();
    });
  });

  describe('euisPlugin function', () => {
    it('should work without options', () => {
      const plugin = euisPlugin();

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-euis');
    });

    it('should work with options', () => {
      const plugin = euisPlugin({
        minify: true,
        sourceMaps: false,
      });

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-euis');
    });

    it('should work with full configuration', () => {
      const plugin = euisPlugin({
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
      expect(plugin.name).toBe('vite-plugin-euis');
    });
  });

  describe('Type safety', () => {
    it('should provide type checking for options', () => {
      // This test verifies that TypeScript catches invalid options at compile time
      // If this file compiles without errors, it means the types are working correctly

      const validOptions: EuisPluginOptions = {
        minify: true,
        // TypeScript would error if we tried to add an invalid property:
        // invalidOption: true, // Error: Object literal may only specify known properties
      };

      expect(validOptions).toBeDefined();
    });

    it('should enforce correct types for option values', () => {
      const options: EuisPluginOptions = {
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
    it('should export euisPlugin as default', async () => {
      // Test that the default export works
      const defaultExport = (await import('../index')).default;

      expect(defaultExport).toBe(euisPlugin);
    });

    it('should work with default import syntax', async () => {
      // Simulate: import euis from 'vite-plugin-euis';
      const euis = (await import('../index')).default;
      const plugin = euis({ minify: true });

      expect(plugin).toBeDefined();
      expect(plugin.name).toBe('vite-plugin-euis');
    });
  });

  describe('Named exports', () => {
    it('should export EuisPluginOptions type', () => {
      // This is a compile-time test - if this file compiles, the type is exported
      type Options = EuisPluginOptions;
      const options: Options = { minify: true };

      expect(options).toBeDefined();
    });

    it('should export EuisPlugin type', () => {
      // This is a compile-time test - if this file compiles, the type is exported
      type PluginFactory = EuisPlugin;
      const factory: PluginFactory = euisPlugin;

      expect(factory).toBeDefined();
    });

    it('should export euisPlugin function', () => {
      // Verify the named export works
      expect(euisPlugin).toBeDefined();
      expect(typeof euisPlugin).toBe('function');
    });
  });
});
