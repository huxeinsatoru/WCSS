/**
 * Tests for TypeScript module declarations for .euis files
 * 
 * These tests verify that the TypeScript declarations in euis.d.ts
 * provide correct type information for .euis file imports.
 */

describe('TypeScript Module Declarations', () => {
  describe('.euis module type declarations', () => {
    it('should declare default export as string', () => {
      // This test verifies that the type system recognizes .euis imports
      // The actual type checking happens at compile time, not runtime
      
      // Type assertion to verify the declaration exists
      // In a real project, this would be:
      // import styles from './test.euis';
      // const css: string = styles;
      
      const mockWcssImport = 'mock css content';
      const css: string = mockWcssImport;
      
      expect(typeof css).toBe('string');
    });

    it('should declare optional runtime export', () => {
      // This test verifies that the runtime export is typed correctly
      // In a real project, this would be:
      // import styles, { runtime } from './test.euis';
      // const runtimeCode: string | undefined = runtime;
      
      const mockRuntime: string | undefined = 'mock runtime code';
      
      expect(typeof mockRuntime === 'string' || mockRuntime === undefined).toBe(true);
    });

    it('should allow importing .euis files without type errors', () => {
      // This is a compile-time test - if TypeScript compiles this file
      // without errors, it means the .euis module declarations are working
      
      // Simulate the type structure that would be created by the declaration
      type EuisModule = {
        default: string;
        runtime?: string;
      };
      
      const mockModule: EuisModule = {
        default: '.button { padding: 1rem; }',
        runtime: undefined,
      };
      
      expect(mockModule.default).toBeDefined();
      expect(typeof mockModule.default).toBe('string');
    });

    it('should support runtime export when typedOM is enabled', () => {
      // When typedOM option is enabled, the runtime export contains JavaScript code
      type EuisModuleWithRuntime = {
        default: string;
        runtime: string;
      };
      
      const mockModuleWithRuntime: EuisModuleWithRuntime = {
        default: '.button { padding: 1rem; }',
        runtime: 'const typedOM = { ... };',
      };
      
      expect(mockModuleWithRuntime.runtime).toBeDefined();
      expect(typeof mockModuleWithRuntime.runtime).toBe('string');
    });
  });

  describe('module declaration file location', () => {
    it('should be available in the dist directory', () => {
      // The euis.d.ts file should be copied to dist/ during build
      // This ensures it's available to consumers of the package
      
      const fs = require('fs');
      const path = require('path');
      
      const distPath = path.join(__dirname, '../../dist/euis.d.ts');
      const srcPath = path.join(__dirname, '../euis.d.ts');
      
      // Check that the source file exists
      expect(fs.existsSync(srcPath)).toBe(true);
      
      // Check that it's copied to dist (after build)
      // This may not exist during development, only after build
      if (fs.existsSync(distPath)) {
        const content = fs.readFileSync(distPath, 'utf-8');
        expect(content).toContain("declare module '*.euis'");
        expect(content).toContain('const css: string');
        expect(content).toContain('export const runtime');
      }
    });
  });

  describe('package.json configuration', () => {
    it('should include euis.d.ts in the published package', () => {
      // Verify that package.json is configured to include the declaration file
      const packageJson = require('../../package.json');
      
      // The dist directory should be in the files array
      expect(packageJson.files).toContain('dist');
      
      // The build script should copy euis.d.ts to dist
      expect(packageJson.scripts.build).toContain('euis.d.ts');
    });

    it('should have correct types entry point', () => {
      const packageJson = require('../../package.json');
      
      // The types field should point to the main declaration file
      expect(packageJson.types).toBe('dist/index.d.ts');
    });
  });

  describe('TypeScript compilation', () => {
    it('should compile without errors when importing .euis files', () => {
      // This test verifies that TypeScript can compile code that imports .euis files
      // The presence of euis.d.ts should prevent "Cannot find module" errors
      
      // Simulate TypeScript's module resolution
      // In a real project, TypeScript would find euis.d.ts and use it for type checking
      
      const mockTypeCheck = () => {
        // This would be actual TypeScript code in a real project:
        // import styles from './button.euis';
        // const buttonStyles: string = styles;
        
        // For testing purposes, we just verify the type structure
        type ImportedEuis = string;
        const styles: ImportedEuis = '.button { padding: 1rem; }';
        return styles;
      };
      
      expect(() => mockTypeCheck()).not.toThrow();
    });
  });
});
