import { formatDiagnostics } from '../index';

describe('formatDiagnostics utility', () => {
  describe('error formatting with line and column information', () => {
    it('should format error with both line and column', () => {
      const errors = [
        {
          message: 'Undefined token reference',
          line: 5,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Undefined token reference (5:12)');
    });

    it('should format error with line only (no column)', () => {
      const errors = [
        {
          message: 'Invalid property value',
          line: 8,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Invalid property value (8)');
    });

    it('should format error without line or column', () => {
      const errors = [
        {
          message: 'General compilation error',
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: General compilation error');
    });

    it('should handle line 0 correctly', () => {
      const errors = [
        {
          message: 'Error at start of file',
          line: 0,
          column: 0,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Error at start of file (0:0)');
    });
  });

  describe('warning formatting', () => {
    it('should format warning with line and column', () => {
      const warnings = [
        {
          message: 'Deprecated syntax',
          line: 10,
          column: 5,
          severity: 'warning',
        },
      ];

      const result = formatDiagnostics(warnings, 'warning');

      expect(result).toBe('  warning: Deprecated syntax (10:5)');
    });

    it('should format warning without location', () => {
      const warnings = [
        {
          message: 'Performance suggestion',
          severity: 'warning',
        },
      ];

      const result = formatDiagnostics(warnings, 'warning');

      expect(result).toBe('  warning: Performance suggestion');
    });
  });

  describe('multi-line output for multiple diagnostics', () => {
    it('should create readable multi-line output for multiple errors', () => {
      const errors = [
        {
          message: 'First error',
          line: 5,
          column: 12,
          severity: 'error',
        },
        {
          message: 'Second error',
          line: 8,
          severity: 'error',
        },
        {
          message: 'Third error',
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe(
        '  error: First error (5:12)\n' +
        '  error: Second error (8)\n' +
        '  error: Third error'
      );
    });

    it('should create multi-line output for multiple warnings', () => {
      const warnings = [
        {
          message: 'Warning one',
          line: 3,
          column: 7,
          severity: 'warning',
        },
        {
          message: 'Warning two',
          line: 15,
          column: 20,
          severity: 'warning',
        },
      ];

      const result = formatDiagnostics(warnings, 'warning');

      expect(result).toBe(
        '  warning: Warning one (3:7)\n' +
        '  warning: Warning two (15:20)'
      );
    });

    it('should handle empty array', () => {
      const result = formatDiagnostics([], 'error');

      expect(result).toBe('');
    });

    it('should handle single item array', () => {
      const errors = [
        {
          message: 'Single error',
          line: 1,
          column: 1,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Single error (1:1)');
    });
  });

  describe('edge cases and error recovery', () => {
    it('should handle missing message field', () => {
      const errors = [
        {
          line: 5,
          column: 12,
          severity: 'error',
        } as any,
      ];

      const result = formatDiagnostics(errors, 'error');

      // Should use the item itself as fallback
      expect(result).toContain('error:');
      expect(result).toContain('(5:12)');
    });

    it('should handle null line value', () => {
      const errors = [
        {
          message: 'Error message',
          line: null as any,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Error message');
    });

    it('should handle undefined line value', () => {
      const errors = [
        {
          message: 'Error message',
          line: undefined,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Error message');
    });

    it('should handle null column value', () => {
      const errors = [
        {
          message: 'Error message',
          line: 5,
          column: null as any,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Error message (5)');
    });

    it('should handle undefined column value', () => {
      const errors = [
        {
          message: 'Error message',
          line: 5,
          column: undefined,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Error message (5)');
    });

    it('should handle empty message string', () => {
      const errors = [
        {
          message: '',
          line: 5,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error:  (5:12)');
    });

    it('should handle message with special characters', () => {
      const errors = [
        {
          message: 'Error: "invalid" value for \'property\'',
          line: 5,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toBe('  error: Error: "invalid" value for \'property\' (5:12)');
    });

    it('should handle message with newlines', () => {
      const errors = [
        {
          message: 'Error on\nmultiple lines',
          line: 5,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toContain('Error on\nmultiple lines');
      expect(result).toContain('(5:12)');
    });
  });

  describe('custom label support', () => {
    it('should support custom label', () => {
      const items = [
        {
          message: 'Custom diagnostic',
          line: 5,
          column: 12,
          severity: 'info',
        },
      ];

      const result = formatDiagnostics(items, 'info');

      expect(result).toBe('  info: Custom diagnostic (5:12)');
    });

    it('should support empty label', () => {
      const items = [
        {
          message: 'Diagnostic message',
          line: 5,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(items, '');

      expect(result).toBe('  : Diagnostic message (5:12)');
    });
  });

  describe('requirements validation', () => {
    it('should meet requirement 8.1: format errors with line and column numbers', () => {
      const errors = [
        {
          message: 'Syntax error',
          line: 10,
          column: 25,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toContain('(10:25)');
      expect(result).toContain('Syntax error');
    });

    it('should meet requirement 8.2: include error message text and severity level', () => {
      const errors = [
        {
          message: 'Validation failed',
          line: 5,
          column: 12,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      expect(result).toContain('error:');
      expect(result).toContain('Validation failed');
    });

    it('should meet requirement 8.4: format for Vite error overlay', () => {
      const errors = [
        {
          message: 'Compilation error',
          line: 15,
          column: 8,
          severity: 'error',
        },
        {
          message: 'Another error',
          line: 20,
          severity: 'error',
        },
      ];

      const result = formatDiagnostics(errors, 'error');

      // Should be readable multi-line format suitable for error overlay
      expect(result).toContain('\n');
      expect(result).toContain('error:');
      expect(result).toContain('(15:8)');
      expect(result).toContain('(20)');
    });
  });
});
