import { defineNuxtModule } from '@nuxt/kit';
import nuxtWcssModule, { EuisNuxtOptions } from '../index';

// Mock @nuxt/kit functions
jest.mock('@nuxt/kit', () => ({
  defineNuxtModule: jest.fn((config) => config),
  addVitePlugin: jest.fn(),
  addWebpackPlugin: jest.fn(),
}));

// Mock vite-plugin-euis
jest.mock('vite-plugin-euis', () => ({
  euisPlugin: jest.fn((options) => ({
    name: 'vite-plugin-euis',
    options,
  })),
}));

describe('Nuxt Module Integration Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('sample Nuxt project integration', () => {
    it('should define a Nuxt module with correct metadata', () => {
      expect(nuxtWcssModule.meta).toBeDefined();
      expect(nuxtWcssModule.meta.name).toBe('euis');
      expect(nuxtWcssModule.meta.configKey).toBe('euis');
    });

    it('should have default options', () => {
      expect(nuxtWcssModule.defaults).toBeDefined();
      expect(nuxtWcssModule.defaults).toEqual({
        treeShaking: false,
        minify: false,
        sourceMaps: true,
        typedOM: false,
        tokens: {},
      });
    });

    it('should add Vite plugin during setup', () => {
      const { addVitePlugin } = require('@nuxt/kit');
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      const options: EuisNuxtOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: false,
        typedOM: false,
        tokens: {
          colors: { primary: '#3b82f6' },
        },
      };

      nuxtWcssModule.setup(options, mockNuxt as any);

      expect(addVitePlugin).toHaveBeenCalled();
      expect(euisPlugin).toHaveBeenCalledWith({
        treeShaking: false,
        minify: true,
        sourceMaps: true,
        typedOM: false,
        tokens: options.tokens,
      });
    });

    it('should add Webpack loader during setup', () => {
      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            // Simulate webpack:config hook
            const configs = [
              {
                module: {
                  rules: [] as any[],
                },
              },
            ];
            callback(configs);
          }
        }),
      };

      const options: EuisNuxtOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: true,
        typedOM: false,
        tokens: {
          colors: { primary: '#3b82f6' },
        },
      };

      nuxtWcssModule.setup(options, mockNuxt as any);

      expect(mockNuxt.hook).toHaveBeenCalledWith('webpack:config', expect.any(Function));
    });

    it('should configure webpack loader with correct options', () => {
      const mockConfigs = [
        {
          module: {
            rules: [] as any[],
          },
        },
      ];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      const options: EuisNuxtOptions = {
        minify: true,
        sourceMaps: false,
        treeShaking: true,
        typedOM: true,
        tokens: {
          colors: { primary: '#3b82f6' },
          spacing: { md: '1rem' },
        },
      };

      nuxtWcssModule.setup(options, mockNuxt as any);

      expect(mockConfigs[0].module.rules).toHaveLength(1);
      const euisRule = mockConfigs[0].module.rules[0];
      expect(euisRule.test).toEqual(/\.euis$/);
      expect(euisRule.use[0].loader).toBe('euis-loader');
      expect(euisRule.use[0].options).toEqual({
        treeShaking: true,
        minify: true,
        sourceMaps: false,
        typedOM: true,
        tokens: options.tokens,
      });
    });

    it('should handle multiple webpack configs', () => {
      const mockConfigs = [
        {
          module: {
            rules: [] as any[],
          },
        },
        {
          module: {
            rules: [] as any[],
          },
        },
      ];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      const options: EuisNuxtOptions = {
        minify: true,
      };

      nuxtWcssModule.setup(options, mockNuxt as any);

      // Both configs should have Euis rule
      expect(mockConfigs[0].module.rules).toHaveLength(1);
      expect(mockConfigs[1].module.rules).toHaveLength(1);
    });
  });

  describe('.euis file processing', () => {
    it('should configure loader to process .euis files', () => {
      const mockConfigs = [
        {
          module: {
            rules: [] as any[],
          },
        },
      ];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      const euisRule = mockConfigs[0].module.rules[0];
      expect(euisRule.test).toEqual(/\.euis$/);
      expect(euisRule.test.test('button.euis')).toBe(true);
      expect(euisRule.test.test('layout.euis')).toBe(true);
      expect(euisRule.test.test('button.css')).toBe(false);
    });

    it('should work with Vite-based Nuxt builds', () => {
      const { addVitePlugin } = require('@nuxt/kit');
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      expect(addVitePlugin).toHaveBeenCalled();
      expect(euisPlugin).toHaveBeenCalled();
    });

    it('should work with Webpack-based Nuxt builds', () => {
      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            const configs = [{ module: { rules: [] } }];
            callback(configs);
          }
        }),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      expect(mockNuxt.hook).toHaveBeenCalledWith('webpack:config', expect.any(Function));
    });

    it('should handle configs without module property', () => {
      const mockConfigs = [
        {} as any,
      ];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      expect(mockConfigs[0].module).toBeDefined();
      expect(mockConfigs[0].module.rules).toBeDefined();
      expect(mockConfigs[0].module.rules).toHaveLength(1);
    });

    it('should handle configs without rules array', () => {
      const mockConfigs = [
        {
          module: {} as any,
        },
      ];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      expect(mockConfigs[0].module.rules).toBeDefined();
      expect(mockConfigs[0].module.rules).toHaveLength(1);
    });
  });

  describe('configuration options', () => {
    it('should support minification option', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({ minify: true }, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ minify: true })
      );
    });

    it('should support tree shaking option', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({ treeShaking: true }, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ treeShaking: true })
      );
    });

    it('should support source maps option', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({ sourceMaps: false }, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ sourceMaps: false })
      );
    });

    it('should support Typed OM option', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({ typedOM: true }, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ typedOM: true })
      );
    });

    it('should support design tokens configuration', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      const tokens = {
        colors: {
          primary: '#3b82f6',
          secondary: '#8b5cf6',
        },
        spacing: {
          md: '1rem',
          lg: '1.5rem',
        },
        typography: {
          'font-sans': 'system-ui',
        },
        breakpoints: {
          md: '768px',
        },
      };

      nuxtWcssModule.setup({ tokens }, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ tokens })
      );
    });

    it('should support all options combined', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      const options: EuisNuxtOptions = {
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

      nuxtWcssModule.setup(options, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(options);
    });

    it('should use default options when none provided', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith({
        treeShaking: undefined,
        minify: undefined,
        sourceMaps: undefined,
        typedOM: undefined,
        tokens: undefined,
      });
    });
  });

  describe('edge cases', () => {
    it('should handle empty options object', () => {
      const mockNuxt = {
        hook: jest.fn(),
      };

      expect(() => {
        nuxtWcssModule.setup({}, mockNuxt as any);
      }).not.toThrow();
    });

    it('should handle partial options', () => {
      const { euisPlugin } = require('vite-plugin-euis');

      const mockNuxt = {
        hook: jest.fn(),
      };

      nuxtWcssModule.setup({ minify: true }, mockNuxt as any);

      expect(euisPlugin).toHaveBeenCalledWith(
        expect.objectContaining({ minify: true })
      );
    });

    it('should handle empty webpack configs array', () => {
      const mockConfigs: any[] = [];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      expect(() => {
        nuxtWcssModule.setup({}, mockNuxt as any);
      }).not.toThrow();

      expect(mockConfigs).toHaveLength(0);
    });

    it('should not modify existing webpack rules', () => {
      const existingRule = {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      };

      const mockConfigs = [
        {
          module: {
            rules: [existingRule],
          },
        },
      ];

      const mockNuxt = {
        hook: jest.fn((hookName, callback) => {
          if (hookName === 'webpack:config') {
            callback(mockConfigs);
          }
        }),
      };

      nuxtWcssModule.setup({}, mockNuxt as any);

      expect(mockConfigs[0].module.rules).toHaveLength(2);
      expect(mockConfigs[0].module.rules[0]).toBe(existingRule);
    });
  });

  describe('module exports', () => {
    it('should export EuisNuxtOptions type', () => {
      const options: EuisNuxtOptions = {
        minify: true,
        sourceMaps: true,
        treeShaking: false,
        typedOM: false,
        tokens: {},
      };

      expect(options).toBeDefined();
    });

    it('should be a valid Nuxt module', () => {
      expect(nuxtWcssModule.meta).toBeDefined();
      expect(nuxtWcssModule.defaults).toBeDefined();
      expect(nuxtWcssModule.setup).toBeDefined();
      expect(typeof nuxtWcssModule.setup).toBe('function');
    });
  });
});
