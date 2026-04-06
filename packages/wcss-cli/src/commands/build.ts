import * as fs from 'fs';
import * as path from 'path';
import { glob } from 'glob';
import { CompilerConfig } from '../types';
import { compile } from '../compiler';
import { displayErrors, displayWarnings, displaySuccess, displayInfo } from '../error-display';

export interface BuildOptions {
  input: string;
  output?: string;
  config: CompilerConfig;
}

/**
 * Build command: compile WCSS files to CSS
 */
export async function buildCommand(options: BuildOptions): Promise<void> {
  const { input, output, config } = options;

  try {
    // Resolve input files (support glob patterns)
    const files = await resolveInputFiles(input);

    if (files.length === 0) {
      console.error(`No WCSS files found matching: ${input}`);
      process.exit(1);
    }

    displayInfo(`Compiling ${files.length} file${files.length > 1 ? 's' : ''}...`);

    let totalErrors = 0;
    let totalWarnings = 0;

    // Compile each file
    for (const file of files) {
      const result = await compileFile(file, output, config);
      totalErrors += result.errors;
      totalWarnings += result.warnings;
    }

    // Display summary
    if (totalErrors > 0) {
      console.error(`\nCompilation failed with ${totalErrors} error${totalErrors > 1 ? 's' : ''}`);
      process.exit(1);
    }

    if (totalWarnings > 0) {
      console.warn(`\nCompilation completed with ${totalWarnings} warning${totalWarnings > 1 ? 's' : ''}`);
    } else {
      displaySuccess('Compilation completed successfully');
    }
  } catch (error) {
    console.error('Build failed:', error);
    process.exit(1);
  }
}

/**
 * Resolve input files from glob pattern
 */
async function resolveInputFiles(input: string): Promise<string[]> {
  // If input is a directory, search for .wcss files
  if (fs.existsSync(input) && fs.statSync(input).isDirectory()) {
    input = path.join(input, '**/*.wcss');
  }

  // Resolve glob pattern
  const files = await glob(input, {
    ignore: ['**/node_modules/**', '**/dist/**'],
  });

  return files;
}

/**
 * Compile a single file
 */
async function compileFile(
  inputPath: string,
  outputPath: string | undefined,
  config: CompilerConfig
): Promise<{ errors: number; warnings: number }> {
  // Read input file
  const source = fs.readFileSync(inputPath, 'utf-8');

  // Compile
  const result = await compile(source, config);

  // Display errors
  if (result.errors.length > 0) {
    console.error(`\nErrors in ${inputPath}:`);
    displayErrors(result.errors, source);
    return { errors: result.errors.length, warnings: result.warnings.length };
  }

  // Display warnings
  if (result.warnings.length > 0) {
    console.warn(`\nWarnings in ${inputPath}:`);
    displayWarnings(result.warnings);
  }

  // Determine output path
  const cssOutputPath = outputPath || inputPath.replace(/\.wcss$/, '.css');

  // Write CSS output
  fs.mkdirSync(path.dirname(cssOutputPath), { recursive: true });
  fs.writeFileSync(cssOutputPath, result.css, 'utf-8');

  // Write JS output if Typed OM is enabled
  if (result.js) {
    const jsOutputPath = cssOutputPath.replace(/\.css$/, '.js');
    fs.writeFileSync(jsOutputPath, result.js, 'utf-8');
  }

  // Write source map if enabled
  if (result.sourceMap && config.sourceMaps === 'external') {
    const mapOutputPath = cssOutputPath + '.map';
    fs.writeFileSync(mapOutputPath, result.sourceMap, 'utf-8');
  }

  // Display stats
  if (result.stats) {
    const { input_size, output_size, compile_time_us } = result.stats;
    const reduction = ((1 - output_size / input_size) * 100).toFixed(1);
    displayInfo(
      `${inputPath} → ${cssOutputPath} (${compile_time_us}μs, ${reduction}% reduction)`
    );
  }

  return { errors: 0, warnings: result.warnings.length };
}
