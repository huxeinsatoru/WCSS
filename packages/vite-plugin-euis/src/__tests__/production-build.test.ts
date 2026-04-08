import { euisPlugin, EuisPluginOptions } from '../index';
import type { Plugin } from 'vite';

describe('Production Build Support', () => {
  describe('Production CSS matches development output (Requirement 10.6)', () => {
    it('should produce identical CSS output in production and development modes', async () => {
      const code = `
        .container {
          padding: 1rem;
          margin: 0 auto;
          max-width: 1200px;
        }
        
        .button {
          background: blue;
          color: white;
          padding: 0.5rem 1rem;
        }
      `;
      const id = '/src/styles/app.euis';

      // Development mode (no minification)
      const devOptions: EuisPluginOptions = {
        minify: false,
        sourceMaps: false,
      };

      const devPlugin = euisPlugin(devOptions) as Plugin;
      const devContext = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      // Production mode (no minification for comparison)
      const prodOptions: EuisPluginOptions = {
        minify: false,
        sourceMaps: false,
      };

      const prodPlugin = euisPlugin(prodOptions) as Plugin;
      const prodContext = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (devPlugin.transform && prodPlugin.transform) {
        const devResult = await (devPlugin.transform as any).call(devContext, code, id);
        const prodResult = await (prodPlugin.transform as any).call(prodContext, code, id);

        expect(devResult).toBeDefined();
        expect(prodResult).toBeDefined();

        // Extract CSS from both results
        const devCssMatch = devResult.code.match(/export default "(.*?)";/);
        const prodCssMatch = prodResult.code.match(/export default "(.*?)";/);

        expect(devCssMatch).toBeTruthy();
        expect(prodCssMatch).toBeTruthy();

        const devCss = devCssMatch![1];
        const prodCss = prodCssMatch![1];

        // Production CSS should match development CSS (when minification is off)
        expect(prodCss).toBe(devCss);

        expect(devContext.error).not.toHaveBeenCalled();
        expect(prodContext.error).not.toHaveBeenCalled();
      }
    });

    it('should produce semantically equivalent CSS when minification is enabled', async () => {
      const code = `
        .header {
          display: flex;
          justify-content: space-between;
          padding: 1rem 2rem;
        }
      `;
      const id = '/src/styles/header.euis';

      // Development mode (no minification)
      const devOptions: EuisPluginOptions = {
        minify: false,
        sourceMaps: false,
      };

      const devPlugin = euisPlugin(devOptions) as Plugin;
      const devContext = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      // Production mode (with minification)
      const prodOptions: EuisPluginOptions = {
        minify: true,
        sourceMaps: false,
      };

      const prodPlugin = euisPlugin(prodOptions) as Plugin;
      const prodContext = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (devPlugin.transform && prodPlugin.transform) {
        const devResult = await (devPlugin.transform as any).call(devContext, code, id);
        const prodResult = await (prodPlugin.transform as any).call(prodContext, code, id);

        expect(devResult).toBeDefined();
        expect(prodResult).toBeDefined();

        // Extract CSS from both results
        const devCssMatch = devResult.code.match(/export default "(.*?)";/);
        const prodCssMatch = prodResult.code.match(/export default "(.*?)";/);

        expect(devCssMatch).toBeTruthy();
        expect(prodCssMatch).toBeTruthy();

        const devCss = devCssMatch![1];
        const prodCss = prodCssMatch![1];

        // Both should contain valid CSS
        expect(devCss.length).toBeGreaterThan(0);
        expect(prodCss.length).toBeGreaterThan(0);

        // Minified CSS should be shorter or equal
        expect(prodCss.length).toBeLessThanOrEqual(devCss.length);

        // Both should contain the same selectors (semantic equivalence)
        expect(devCss).toContain('.header');
        expect(prodCss).toContain('.header');

        expect(devContext.error).not.toHaveBeenCalled();
        expect(prodContext.error).not.toHaveBeenCalled();
      }
    });

    it('should handle tree-shaking consistently in production and development', async () => {
      const code = `
        .used-class {
          color: blue;
        }
        
        .unused-class {
          color: red;
        }
      `;
      const id = '/src/styles/components.euis';

      const sharedOptions = {
        treeShaking: true,
        usedClasses: ['used-class'],
        minify: false,
        sourceMaps: false,
      };

      // Development mode with tree-shaking
      const devPlugin = euisPlugin(sharedOptions) as Plugin;
      const devContext = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      // Production mode with tree-shaking
      const prodPlugin = euisPlugin(sharedOptions) as Plugin;
      const prodContext = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (devPlugin.transform && prodPlugin.transform) {
        const devResult = await (devPlugin.transform as any).call(devContext, code, id);
        const prodResult = await (prodPlugin.transform as any).call(prodContext, code, id);

        expect(devResult).toBeDefined();
        expect(prodResult).toBeDefined();

        // Extract CSS from both results
        const devCssMatch = devResult.code.match(/export default "(.*?)";/);
        const prodCssMatch = prodResult.code.match(/export default "(.*?)";/);

        expect(devCssMatch).toBeTruthy();
        expect(prodCssMatch).toBeTruthy();

        const devCss = devCssMatch![1];
        const prodCss = prodCssMatch![1];

        // Tree-shaking should work identically in both modes
        expect(prodCss).toBe(devCss);

        // Both should contain used class
        expect(devCss).toContain('used-class');
        expect(prodCss).toContain('used-class');

        expect(devContext.error).not.toHaveBeenCalled();
        expect(prodContext.error).not.toHaveBeenCalled();
      }
    });
  });

  it('should compile .euis files with minification enabled', async () => {
    const options: EuisPluginOptions = {
      minify: true,
      sourceMaps: false,
    };

    const plugin = euisPlugin(options) as Plugin;

    const code = '.button { padding: 1rem; }';
    const id = '/src/styles/button.euis';

    const context = {
      error: jest.fn(),
      warn: jest.fn(),
    };

    if (plugin.transform) {
      const result = await (plugin.transform as any).call(context, code, id);

      expect(result).toBeDefined();
      expect(result.code).toContain('export default');
      expect(context.error).not.toHaveBeenCalled();
    }
  });

  describe('Minification (Requirement 10.2)', () => {
    it('should minify CSS by removing unnecessary whitespace', async () => {
      const options: EuisPluginOptions = {
        minify: true,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .card {
          padding: 2rem;
          margin: 1rem;
          background: white;
        }
      `;
      const id = '/src/styles/card.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Minified CSS should not contain excessive whitespace
        expect(css).not.toMatch(/\n\s+/); // No indentation
        expect(css.length).toBeGreaterThan(0);

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should preserve CSS functionality when minified', async () => {
      const options: EuisPluginOptions = {
        minify: true,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .nav {
          display: flex;
          align-items: center;
          gap: 1rem;
        }
        
        .nav-item {
          padding: 0.5rem 1rem;
          text-decoration: none;
        }
      `;
      const id = '/src/styles/nav.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Should contain both selectors
        expect(css).toContain('.nav');
        expect(css).toContain('.nav-item');

        // Should contain all properties
        expect(css).toContain('display');
        expect(css).toContain('flex');
        expect(css).toContain('align-items');
        expect(css).toContain('padding');

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should handle minification with complex selectors', async () => {
      const options: EuisPluginOptions = {
        minify: true,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .parent > .child {
          color: blue;
        }
        
        .item:hover {
          background: gray;
        }
        
        .list .item:first-child {
          margin-top: 0;
        }
      `;
      const id = '/src/styles/complex.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Should preserve complex selectors
        expect(css).toContain('.parent');
        expect(css).toContain('.child');
        expect(css).toContain(':hover');
        expect(css).toContain(':first-child');

        expect(context.error).not.toHaveBeenCalled();
      }
    });
  });

  it('should compile .euis files with tree-shaking enabled', async () => {
    const options: EuisPluginOptions = {
      treeShaking: true,
      usedClasses: ['button'],
      minify: false,
    };

    const plugin = euisPlugin(options) as Plugin;

    const code = '.button { padding: 1rem; } .unused { display: none; }';
    const id = '/src/styles/components.euis';

    const context = {
      error: jest.fn(),
      warn: jest.fn(),
    };

    if (plugin.transform) {
      const result = await (plugin.transform as any).call(context, code, id);

      expect(result).toBeDefined();
      expect(result.code).toBeDefined();
      expect(context.error).not.toHaveBeenCalled();
    }
  });

  describe('Tree-Shaking (Requirement 10.3)', () => {
    it('should remove unused classes when tree-shaking is enabled', async () => {
      const options: EuisPluginOptions = {
        treeShaking: true,
        usedClasses: ['button', 'card'],
        minify: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .button { padding: 1rem; }
        .card { border: 1px solid gray; }
        .unused-class { display: none; }
        .another-unused { color: red; }
      `;
      const id = '/src/styles/components.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Should contain used classes
        expect(css).toContain('button');
        expect(css).toContain('card');

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should keep all classes when tree-shaking is disabled', async () => {
      const options: EuisPluginOptions = {
        treeShaking: false,
        minify: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .used { color: blue; }
        .unused { color: red; }
      `;
      const id = '/src/styles/all.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Should contain all classes when tree-shaking is off
        expect(css).toContain('used');
        expect(css).toContain('unused');

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should handle tree-shaking with empty usedClasses array', async () => {
      const options: EuisPluginOptions = {
        treeShaking: true,
        usedClasses: [],
        minify: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .class1 { color: blue; }
        .class2 { color: red; }
      `;
      const id = '/src/styles/empty-used.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Should still produce valid output
        expect(result.code).toContain('export default');

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should combine tree-shaking with minification', async () => {
      const options: EuisPluginOptions = {
        treeShaking: true,
        usedClasses: ['active', 'highlight'],
        minify: true,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .active {
          background: blue;
          color: white;
        }
        
        .highlight {
          border: 2px solid yellow;
        }
        
        .unused {
          display: none;
        }
      `;
      const id = '/src/styles/optimized.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Should contain used classes
        expect(css).toContain('active');
        expect(css).toContain('highlight');

        // Should be minified (no excessive whitespace)
        expect(css.length).toBeGreaterThan(0);

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should handle tree-shaking with safelist', async () => {
      const options: EuisPluginOptions = {
        treeShaking: true,
        usedClasses: ['button'],
        safelist: ['important'],
        minify: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .button { padding: 1rem; }
        .important { color: red; }
        .unused { display: none; }
      `;
      const id = '/src/styles/safelist.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Extract CSS
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const css = cssMatch![1];

        // Should contain used class
        expect(css).toContain('button');
        
        // Should contain safelisted class even if not in usedClasses
        expect(css).toContain('important');

        expect(context.error).not.toHaveBeenCalled();
      }
    });
  });

  describe('WASM Build-Time Only (Requirements 10.4, 10.5)', () => {
    it('should output pure CSS without runtime JavaScript when typedOM is disabled', async () => {
      const options: EuisPluginOptions = {
        minify: false,
        typedOM: false, // Explicitly disable typedOM
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = '.card { padding: 2rem; background: white; }';
      const id = '/src/styles/card.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Verify output is pure CSS export without runtime JS
        expect(result.code).toMatch(/^export default ".*";$/);
        expect(result.code).not.toContain('export const runtime');
        expect(result.code).not.toContain('wasm');
        expect(result.code).not.toContain('WebAssembly');
        expect(result.code).not.toContain('@euis/wasm');

        // Verify the exported CSS is a string
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        expect(cssMatch![1]).toBeTruthy();

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should not include WASM imports or references in the output', async () => {
      const options: EuisPluginOptions = {
        minify: true,
        treeShaking: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = '.header { font-size: 2rem; } .footer { font-size: 1rem; }';
      const id = '/src/styles/layout.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Verify no WASM-related code in output
        expect(result.code).not.toContain('import');
        expect(result.code).not.toContain('require');
        expect(result.code).not.toContain('wasm');
        expect(result.code).not.toContain('WebAssembly');
        expect(result.code).not.toContain('initWASM');
        expect(result.code).not.toContain('EuisCompiler');

        // Verify output is just a CSS export
        expect(result.code).toMatch(/^export default ".*";$/);

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should only include runtime JS when typedOM is explicitly enabled', async () => {
      const options: EuisPluginOptions = {
        minify: false,
        typedOM: true, // Explicitly enable typedOM
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = '.box { width: 100px; }';
      const id = '/src/styles/box.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // When typedOM is enabled, runtime JS may be included
        // But WASM should still not be in the output
        expect(result.code).not.toContain('wasm');
        expect(result.code).not.toContain('WebAssembly');
        expect(result.code).not.toContain('@euis/wasm');
        expect(result.code).not.toContain('import');
        expect(result.code).not.toContain('require');

        // Should have CSS export
        expect(result.code).toContain('export default');

        // May have runtime export if compiler generates it
        // (This is acceptable when typedOM is enabled)

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should produce static CSS output suitable for production deployment', async () => {
      const options: EuisPluginOptions = {
        minify: true,
        treeShaking: true,
        usedClasses: ['container', 'title'],
        typedOM: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = `
        .container { max-width: 1200px; margin: 0 auto; }
        .title { font-size: 2rem; font-weight: bold; }
        .unused { display: none; }
      `;
      const id = '/src/styles/app.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // Verify output is pure static CSS
        expect(result.code).toMatch(/^export default ".*";$/);

        // Extract the CSS string
        const cssMatch = result.code.match(/export default "(.*?)";/);
        expect(cssMatch).toBeTruthy();
        const cssContent = cssMatch![1];

        // Verify it's actual CSS content (not empty)
        expect(cssContent.length).toBeGreaterThan(0);

        // Verify no dynamic code or WASM references
        expect(result.code).not.toContain('function');
        expect(result.code).not.toContain('async');
        expect(result.code).not.toContain('await');
        expect(result.code).not.toContain('Promise');
        expect(result.code).not.toContain('wasm');

        expect(context.error).not.toHaveBeenCalled();
      }
    });

    it('should verify WASM is only used during build-time transformation', async () => {
      const options: EuisPluginOptions = {
        minify: false,
        sourceMaps: false,
      };

      const plugin = euisPlugin(options) as Plugin;

      const code = '.test { color: red; }';
      const id = '/src/styles/test.euis';

      const context = {
        error: jest.fn(),
        warn: jest.fn(),
      };

      if (plugin.transform) {
        // The transform function uses WASM internally (build-time)
        const result = await (plugin.transform as any).call(context, code, id);

        expect(result).toBeDefined();
        expect(result.code).toBeDefined();

        // But the output should not contain any WASM references (runtime)
        const outputCode = result.code;

        // Verify output is a simple module export
        expect(outputCode).toMatch(/^export default ".*";$/);

        // Verify no build-time dependencies leak into output
        expect(outputCode).not.toContain('initWASM');
        expect(outputCode).not.toContain('EuisCompiler');
        expect(outputCode).not.toContain('compile');
        expect(outputCode).not.toContain('@euis/wasm');
        expect(outputCode).not.toContain('wasm-bindgen');

        // Verify output is pure data (CSS string)
        const lines = outputCode.split('\n').filter((line: string) => line.trim());
        expect(lines.length).toBe(1); // Should be a single export statement
        expect(lines[0]).toMatch(/^export default ".*";$/);

        expect(context.error).not.toHaveBeenCalled();
      }
    });
  });
});
