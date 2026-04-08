/**
 * TypeScript type definitions for Euis compiler
 */

export interface CompilerConfig {
  tokens?: DesignTokens;
  minify?: boolean;
  sourceMaps?: boolean | 'inline' | 'external';
  typedOM?: boolean;
  treeShaking?: boolean;
  outputPath?: string;
}

export interface DesignTokens {
  colors?: Record<string, string>;
  spacing?: Record<string, string>;
  typography?: Record<string, string>;
  breakpoints?: Record<string, string>;
}

export interface CompileResult {
  css: string;
  js?: string;
  sourceMap?: string;
  errors: CompilerError[];
  warnings: CompilerWarning[];
  stats?: CompilationStats;
}

export interface CompilerError {
  kind: string;
  message: string;
  span?: Span;
  suggestion?: string;
}

export interface CompilerWarning {
  message: string;
  span?: Span;
}

export interface Span {
  start: number;
  end: number;
  line: number;
  column: number;
}

export interface CompilationStats {
  input_size: number;
  output_size: number;
  compile_time_us: number;
  rules_processed: number;
  rules_eliminated: number;
}

export interface EuisCompiler {
  compile(source: string, configJson: string): CompileResult;
  parse(source: string): any;
  format(source: string): string;
  validate(source: string, configJson: string): CompilerError[];
}

export interface CLIOptions {
  input?: string;
  output?: string;
  config?: string;
  minify?: boolean;
  sourceMaps?: boolean | string;
  watch?: boolean;
  write?: boolean;
}

/**
 * Platform targets for W3C Design Tokens code generation
 */
export type PlatformTarget = 
  | 'CSS'
  | 'IOS'
  | 'Android'
  | 'AndroidKotlin'
  | 'Flutter'
  | 'TypeScript'
  | 'Docs';

/**
 * Result of W3C Design Tokens compilation
 */
export interface W3CTokensResult {
  output: Record<string, string>;
  errors: CompilerError[];
  warnings?: CompilerWarning[];
}

/**
 * Extended compiler interface with W3C tokens support
 */
export interface EuisCompiler {
  compile(source: string, configJson: string): CompileResult;
  parse(source: string): any;
  format(source: string): string;
  validate(source: string, configJson: string): CompilerError[];
  compile_w3c_tokens(jsonContent: string, target: PlatformTarget): W3CTokensResult;
}
