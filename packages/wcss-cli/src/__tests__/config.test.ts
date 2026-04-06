import { getDefaultConfig, mergeConfig } from '../config';
import { CompilerConfig } from '../types';

describe('Config', () => {
  describe('getDefaultConfig', () => {
    it('should return default configuration', () => {
      const config = getDefaultConfig();

      expect(config).toEqual({
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
      });
    });
  });

  describe('mergeConfig', () => {
    it('should merge CLI options with file config', () => {
      const fileConfig: CompilerConfig = {
        minify: false,
        sourceMaps: false,
        tokens: {
          colors: { primary: '#3b82f6' },
          spacing: {},
          typography: {},
          breakpoints: {},
        },
      };

      const cliOptions: Partial<CompilerConfig> = {
        minify: true,
        sourceMaps: 'external',
      };

      const merged = mergeConfig(fileConfig, cliOptions);

      expect(merged.minify).toBe(true);
      expect(merged.sourceMaps).toBe('external');
      expect(merged.tokens?.colors?.primary).toBe('#3b82f6');
    });

    it('should preserve tokens from file config', () => {
      const fileConfig: CompilerConfig = {
        tokens: {
          colors: { primary: '#3b82f6', secondary: '#8b5cf6' },
          spacing: { md: '1rem' },
          typography: {},
          breakpoints: {},
        },
      };

      const cliOptions: Partial<CompilerConfig> = {
        minify: true,
      };

      const merged = mergeConfig(fileConfig, cliOptions);

      expect(merged.tokens?.colors?.primary).toBe('#3b82f6');
      expect(merged.tokens?.colors?.secondary).toBe('#8b5cf6');
      expect(merged.tokens?.spacing?.md).toBe('1rem');
    });
  });
});
