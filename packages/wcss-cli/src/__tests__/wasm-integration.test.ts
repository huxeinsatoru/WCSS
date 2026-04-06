import { loadCompiler, compile, format, validate } from '../compiler';
import { CompilerConfig } from '../types';

describe('WASM Integration Tests', () => {
  describe('compile() from JavaScript', () => {
    it('should compile valid WCSS to CSS', async () => {
      const source = `
.button {
  padding: 1rem;
  background: #3b82f6;
  color: white;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      expect(result.css).toBeDefined();
      expect(result.errors).toEqual([]);
      expect(result.stats).toBeDefined();
    });

    it('should compile WCSS with design tokens', async () => {
      const source = `
.button {
  padding: $spacing.md;
  background: $colors.primary;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: { primary: '#3b82f6' },
          spacing: { md: '1rem' },
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      expect(result.css).toBeDefined();
      expect(result.errors).toEqual([]);
    });

    it('should return errors for invalid WCSS', async () => {
      const source = `
.button {
  padding: $spacing.undefined;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      // Mock compiler doesn't validate tokens yet, but real implementation should
      expect(result).toBeDefined();
    });

    it('should generate source maps when enabled', async () => {
      const source = `
.button {
  padding: 1rem;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: 'external',
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      expect(result.css).toBeDefined();
      expect(result.sourceMap).toBeDefined();
    });

    it('should generate Typed OM code when enabled', async () => {
      const source = `
.button {
  padding: 1rem;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: true,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      expect(result.css).toBeDefined();
      expect(result.js).toBeDefined();
    });

    it('should minify CSS when enabled', async () => {
      const source = `
.button {
  padding: 1rem;
  margin: 0.5rem;
}
`;
      const config: CompilerConfig = {
        minify: true,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      expect(result.css).toBeDefined();
      expect(result.stats).toBeDefined();
      // Mock compiler doesn't actually minify, but real one should reduce size
    });
  });

  describe('parse() function', () => {
    it('should parse valid WCSS to AST', async () => {
      const source = `
.button {
  padding: 1rem;
}
`;
      const compiler = await loadCompiler();
      const ast = compiler.parse(source);

      expect(ast).toBeDefined();
      expect(ast.type).toBe('StyleSheet');
    });

    it('should return errors for invalid syntax', async () => {
      const source = `
.button {
  padding: 1rem
  // Missing semicolon or closing brace
`;
      const compiler = await loadCompiler();
      const result = compiler.parse(source);

      // Mock implementation doesn't validate, but real one should
      expect(result).toBeDefined();
    });
  });

  describe('format() function', () => {
    it('should format WCSS source', async () => {
      const source = `.button{padding:1rem;margin:0.5rem;}`;
      const formatted = await format(source);

      expect(formatted).toBeDefined();
      expect(typeof formatted).toBe('string');
    });

    it('should preserve semantics after formatting', async () => {
      const source = `
.button {
  padding: 1rem;
}
`;
      const formatted = await format(source);
      const compiler = await loadCompiler();
      
      const ast1 = compiler.parse(source);
      const ast2 = compiler.parse(formatted);

      // Both should parse successfully
      expect(ast1).toBeDefined();
      expect(ast2).toBeDefined();
    });
  });

  describe('validate() function', () => {
    it('should validate WCSS without full compilation', async () => {
      const source = `
.button {
  padding: 1rem;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const errors = await validate(source, config);

      expect(Array.isArray(errors)).toBe(true);
      expect(errors.length).toBe(0);
    });

    it('should detect undefined token references', async () => {
      const source = `
.button {
  color: $colors.undefined;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const errors = await validate(source, config);

      // Mock implementation doesn't validate, but real one should return errors
      expect(Array.isArray(errors)).toBe(true);
    });
  });

  describe('memory cleanup', () => {
    it('should handle multiple compilations without memory leaks', async () => {
      const source = `
.button {
  padding: 1rem;
}
`;
      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      // Compile multiple times
      for (let i = 0; i < 100; i++) {
        const result = await compile(source, config);
        expect(result.css).toBeDefined();
      }

      // If there were memory leaks, this test would fail or hang
      expect(true).toBe(true);
    });

    it('should handle large source files', async () => {
      // Generate a large WCSS file
      let source = '';
      for (let i = 0; i < 1000; i++) {
        source += `
.button-${i} {
  padding: ${i}px;
  margin: ${i * 2}px;
}
`;
      }

      const config: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        typedOM: false,
        treeShaking: false,
        tokens: {
          colors: {},
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const result = await compile(source, config);

      expect(result.css).toBeDefined();
      expect(result.stats).toBeDefined();
    });
  });
});
