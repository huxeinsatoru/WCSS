import chalk from 'chalk';
import { CompilerError } from './types';

/**
 * Format and display compiler errors with colors and code snippets
 */
export function displayErrors(errors: CompilerError[], source?: string): void {
  if (errors.length === 0) return;

  console.error(chalk.red.bold(`\n✖ ${errors.length} error${errors.length > 1 ? 's' : ''} found:\n`));

  for (const error of errors) {
    displayError(error, source);
  }
}

/**
 * Display a single error with formatting
 */
function displayError(error: CompilerError, source?: string): void {
  // Error header
  console.error(chalk.red.bold(`Error: ${error.message}`));

  // Location information
  if (error.span && source) {
    const { line, column } = error.span;
    console.error(chalk.cyan(`  ┌─ line ${line}:${column}`));
    console.error(chalk.cyan('  │'));

    // Show code snippet with error highlighting
    const snippet = getCodeSnippet(source, error.span.line, error.span.column, error.span.start, error.span.end);
    console.error(snippet);
    console.error(chalk.cyan('  │'));
  }

  // Suggestion
  if (error.suggestion) {
    console.error(chalk.yellow(`  = help: ${error.suggestion}`));
  }

  console.error('');
}

/**
 * Extract code snippet with error highlighting
 */
function getCodeSnippet(
  source: string,
  line: number,
  column: number,
  start: number,
  end: number
): string {
  const lines = source.split('\n');
  const errorLine = lines[line - 1];

  if (!errorLine) return '';

  const lineNumber = String(line).padStart(2, ' ');
  const errorLength = Math.max(1, end - start);
  const underline = ' '.repeat(column - 1) + chalk.red('^'.repeat(errorLength));

  return (
    chalk.cyan(`${lineNumber}│ `) +
    errorLine +
    '\n' +
    chalk.cyan('  │ ') +
    underline
  );
}

/**
 * Display warnings
 */
export function displayWarnings(warnings: any[]): void {
  if (warnings.length === 0) return;

  console.warn(chalk.yellow.bold(`\n⚠ ${warnings.length} warning${warnings.length > 1 ? 's' : ''}:\n`));

  for (const warning of warnings) {
    console.warn(chalk.yellow(`  • ${warning.message}`));
  }

  console.warn('');
}

/**
 * Display success message
 */
export function displaySuccess(message: string): void {
  console.log(chalk.green.bold(`✓ ${message}`));
}

/**
 * Display info message
 */
export function displayInfo(message: string): void {
  console.log(chalk.blue(`ℹ ${message}`));
}
