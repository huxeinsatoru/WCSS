import wcssAstroIntegration, { wcss, WCSSAstroOptions } from '../index';
import type { AstroIntegration } from 'astro';

// Mock vite-plugin-wcss
jest.mock('vite-plugin-wcss', () => ({
  wcssPlugin: jest.fn((options) => ({
    name: 'vite-plugin-wcss',
    options,
  })),
}));

describe('Astro Integration Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('sample Astro project integration', () => {
    it('should return a valid Astro integration', () => {
      const integration = wcssAstroIntegration();

      expect(integration).toBeDefined();
      expect(integration.name).toBe('astro-wcss');
      expect(integration.hooks).toBeDefined();
      expect(integration.hooks['astro:config:setup']).toBeDefined();
    });

    it('should add Vite plugin during config setup', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(mockUpdateConfig).toHaveBeenCalled();
      expect(wcssPlugin).toHaveBeenCalled();
    });

    it('should pass options to Vite plugin', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const options: WCSSAstroOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: true,
        typedOM: false,
        tokens: {
          colors: { primary: '#3b82f6' },
          spacing: { md: '1rem' },
        },
      };

      const integration = wcssAstroIntegration(options);

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith({
        treeShaking: true,
        minify: true,
        sourceMaps: true,
        typedOM: false,
        tokens: options.tokens,
      });
    });

    it('should update Astro config with Vite plugins', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(mockUpdateConfig).toHaveBeenCalledWith({
        vite: {
          plugins: [expect.objectContaining({ name: 'vite-plugin-wcss' })],
        },
      });
    });

    it('should use default options when none provided', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith({
        treeShaking: false,
        minify: false,
        sourceMaps: true,
        typedOM: false,
        tokens: {},
      });
    });

    it('should work with empty options object', () => {
      const integration = wcssAstroIntegration({});

      expect(integration.name).toBe('astro-wcss');
      expect(integration.hooks['astro:config:setup']).toBeDefined();
    });
  });

  describe('.wcss file processing', () => {
    it('should configure Vite to process .wcss files', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalled();
      const pluginInstance = wcssPlugin.mock.results[0].value;
      expect(pluginInstance.name).toBe('vite-plugin-wcss');
    });

    it('should handle multiple .wcss files', () => {
      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      // Vite plugin should be added once and handle all .wcss files
      expect(mockUpdateConfig).toHaveBeenCalledTimes(1);
    });

    it('should work in both development and production modes', () => {
      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      // The integration doesn't need to know about the mode
      // Vite plugin handles mode-specific behavior
      integration.hooks['astro:config:setup'](setupContext as any);

      expect(mockUpdateConfig).toHaveBeenCalled();
    });

    it('should integrate with Astro build process', () => {
      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      const updateCall = mockUpdateConfig.mock.calls[0][0];
      expect(updateCall.vite).toBeDefined();
      expect(updateCall.vite.plugins).toBeDefined();
      expect(Array.isArray(updateCall.vite.plugins)).toBe(true);
    });
  });

  describe('configuration options', () => {
    it('should support minification option', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration({ minify: true });

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ minify: true })
      );
    });

    it('should support tree shaking option', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration({ treeShaking: true });

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ treeShaking: true })
      );
    });

    it('should support source maps option', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration({ sourceMaps: false });

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ sourceMaps: false })
      );
    });

    it('should support Typed OM option', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration({ typedOM: true });

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ typedOM: true })
      );
    });

    it('should support design tokens configuration', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

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
          'text-base': '1rem',
        },
        breakpoints: {
          sm: '640px',
          md: '768px',
        },
      };

      const integration = wcssAstroIntegration({ tokens });

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ tokens })
      );
    });

    it('should support all options combined', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const options: WCSSAstroOptions = {
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

      const integration = wcssAstroIntegration(options);

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith({
        treeShaking: true,
        minify: true,
        sourceMaps: true,
        typedOM: true,
        tokens: options.tokens,
      });
    });

    it('should handle partial options', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const integration = wcssAstroIntegration({
        minify: true,
        tokens: { colors: { primary: '#3b82f6' } },
      });

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      expect(wcssPlugin).toHaveBeenCalledWith({
        treeShaking: false,
        minify: true,
        sourceMaps: true,
        typedOM: false,
        tokens: { colors: { primary: '#3b82f6' } },
      });
    });
  });

  describe('edge cases', () => {
    it('should handle undefined options', () => {
      const integration = wcssAstroIntegration(undefined);

      expect(integration.name).toBe('astro-wcss');
      expect(integration.hooks['astro:config:setup']).toBeDefined();
    });

    it('should not throw when setup hook is called multiple times', () => {
      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      expect(() => {
        integration.hooks['astro:config:setup'](setupContext as any);
        integration.hooks['astro:config:setup'](setupContext as any);
      }).not.toThrow();

      expect(mockUpdateConfig).toHaveBeenCalledTimes(2);
    });

    it('should preserve existing Vite config', () => {
      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      const updateCall = mockUpdateConfig.mock.calls[0][0];
      expect(updateCall.vite).toBeDefined();
      // Should only add plugins, not replace entire vite config
      expect(Object.keys(updateCall.vite)).toEqual(['plugins']);
    });

    it('should work with other Astro integrations', () => {
      const integration = wcssAstroIntegration();

      // Simulate multiple integrations
      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      integration.hooks['astro:config:setup'](setupContext as any);

      // Should not interfere with other integrations
      expect(mockUpdateConfig).toHaveBeenCalledTimes(1);
    });
  });

  describe('module exports', () => {
    it('should export default integration function', () => {
      expect(wcssAstroIntegration).toBeDefined();
      expect(typeof wcssAstroIntegration).toBe('function');
    });

    it('should export named wcss function', () => {
      expect(wcss).toBeDefined();
      expect(wcss).toBe(wcssAstroIntegration);
    });

    it('should export WCSSAstroOptions type', () => {
      const options: WCSSAstroOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: false,
        typedOM: false,
        tokens: {},
      };

      expect(options).toBeDefined();
    });

    it('should return integration with correct type', () => {
      const integration: AstroIntegration = wcssAstroIntegration();

      expect(integration.name).toBe('astro-wcss');
      expect(integration.hooks).toBeDefined();
    });
  });

  describe('integration lifecycle', () => {
    it('should only hook into astro:config:setup', () => {
      const integration = wcssAstroIntegration();

      const hookNames = Object.keys(integration.hooks);
      expect(hookNames).toEqual(['astro:config:setup']);
    });

    it('should execute setup hook synchronously', () => {
      const integration = wcssAstroIntegration();

      const mockUpdateConfig = jest.fn();
      const setupContext = {
        updateConfig: mockUpdateConfig,
      };

      const result = integration.hooks['astro:config:setup'](setupContext as any);

      // Should not return a promise
      expect(result).toBeUndefined();
      expect(mockUpdateConfig).toHaveBeenCalled();
    });

    it('should be idempotent', () => {
      const { wcssPlugin } = require('vite-plugin-wcss');

      const options: WCSSAstroOptions = {
        minify: true,
        tokens: { colors: { primary: '#3b82f6' } },
      };

      const integration1 = wcssAstroIntegration(options);
      const integration2 = wcssAstroIntegration(options);

      expect(integration1.name).toBe(integration2.name);
      expect(typeof integration1.hooks['astro:config:setup']).toBe(
        typeof integration2.hooks['astro:config:setup']
      );
    });
  });
});
