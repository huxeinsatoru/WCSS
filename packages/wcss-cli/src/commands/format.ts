import * as fs from 'fs';
import { glob } from 'glob';
import { format } from '../compiler';
import { displaySuccess, displayInfo, displayErrors } from '../error-display';

export interface FormatOptions {
  input: string;
  write: boolean;
}

/**
 * Format command: format WCSS files
 */
export async function formatCommand(options: FormatOptions): Promise<void> {
  const { input, write } = options;

  try {
    // Resolve input files
    const files = await resolveInputFiles(input);

    if (files.length === 0) {
      console.error(`No WCSS files found matching: ${input}`);
      process.exit(1);
    }

    displayInfo(`Formatting ${files.length} file${files.length > 1 ? 's' : ''}...`);

    let errorCount = 0;

    // Format each file
    for (const file of files) {
      try {
        await formatFile(file, write);
      } catch (error) {
        console.error(`Error formatting ${file}:`, error);
        errorCount++;
      }
    }

    // Display summary
    if (errorCount > 0) {
      console.error(`\nFormatting failed for ${errorCount} file${errorCount > 1 ? 's' : ''}`);
      process.exit(1);
    }

    if (write) {
      displaySuccess(`Formatted ${files.length} file${files.length > 1 ? 's' : ''}`);
    } else {
      displayInfo('Run with --write to save changes');
    }
  } catch (error) {
    console.error('Format failed:', error);
    process.exit(1);
  }
}

/**
 * Resolve input files from glob pattern
 */
async function resolveInputFiles(input: string): Promise<string[]> {
  // If input is a directory, search for .wcss files
  if (fs.existsSync(input) && fs.statSync(input).isDirectory()) {
    input = `${input}/**/*.wcss`;
  }

  // Resolve glob pattern
  const files = await glob(input, {
    ignore: ['**/node_modules/**', '**/dist/**'],
  });

  return files;
}

/**
 * Format a single file
 */
async function formatFile(filePath: string, write: boolean): Promise<void> {
  // Read file
  const source = fs.readFileSync(filePath, 'utf-8');

  // Format
  const formatted = await format(source);

  if (write) {
    // Write formatted output
    fs.writeFileSync(filePath, formatted, 'utf-8');
    displayInfo(`Formatted: ${filePath}`);
  } else {
    // Display formatted output
    console.log(`\n${filePath}:`);
    console.log(formatted);
  }
}
