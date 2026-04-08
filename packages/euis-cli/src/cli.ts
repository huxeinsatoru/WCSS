#!/usr/bin/env node

import { Command } from 'commander';
import { loadConfig, mergeConfig } from './config';
import { buildCommand } from './commands/build';
import { watchCommand } from './commands/watch';
import { formatCommand } from './commands/format';
import { tokensCommand } from './commands/tokens';
import { CompilerConfig } from './types';

const program = new Command();

program
  .name('euis')
  .description('Euis - Euis CLI')
  .version('0.1.0');

/**
 * Build command
 */
program
  .command('build')
  .description('Compile Euis files to CSS')
  .argument('[input]', 'Input file or glob pattern', '**/*.euis')
  .option('-o, --output <path>', 'Output directory or file')
  .option('-c, --config <path>', 'Path to config file')
  .option('-m, --minify', 'Minify CSS output')
  .option('-s, --source-maps [type]', 'Generate source maps (inline or external)')
  .option('--typed-om', 'Enable Typed OM runtime')
  .option('--tree-shaking', 'Enable tree shaking')
  .action(async (input: string, options: any) => {
    try {
      // Load config file
      const fileConfig = await loadConfig(options.config);

      // Build compiler config from CLI options
      const cliConfig: Partial<CompilerConfig> = {};
      if (options.minify !== undefined) cliConfig.minify = options.minify;
      if (options.sourceMaps !== undefined) {
        cliConfig.sourceMaps = options.sourceMaps === true ? 'external' : options.sourceMaps;
      }
      if (options.typedOm !== undefined) cliConfig.typedOM = options.typedOm;
      if (options.treeShaking !== undefined) cliConfig.treeShaking = options.treeShaking;
      if (options.output !== undefined) cliConfig.outputPath = options.output;

      // Merge configs
      const config = mergeConfig(fileConfig, cliConfig);

      // Run build
      await buildCommand({
        input,
        output: options.output,
        config,
      });
    } catch (error) {
      console.error('Build failed:', error);
      process.exit(1);
    }
  });

/**
 * Watch command
 */
program
  .command('watch')
  .description('Watch Euis files and recompile on changes')
  .argument('[input]', 'Input file or glob pattern', '**/*.euis')
  .option('-o, --output <path>', 'Output directory or file')
  .option('-c, --config <path>', 'Path to config file')
  .option('-m, --minify', 'Minify CSS output')
  .option('-s, --source-maps [type]', 'Generate source maps (inline or external)')
  .option('--typed-om', 'Enable Typed OM runtime')
  .option('--tree-shaking', 'Enable tree shaking')
  .action(async (input: string, options: any) => {
    try {
      // Load config file
      const fileConfig = await loadConfig(options.config);

      // Build compiler config from CLI options
      const cliConfig: Partial<CompilerConfig> = {};
      if (options.minify !== undefined) cliConfig.minify = options.minify;
      if (options.sourceMaps !== undefined) {
        cliConfig.sourceMaps = options.sourceMaps === true ? 'external' : options.sourceMaps;
      }
      if (options.typedOm !== undefined) cliConfig.typedOM = options.typedOm;
      if (options.treeShaking !== undefined) cliConfig.treeShaking = options.treeShaking;
      if (options.output !== undefined) cliConfig.outputPath = options.output;

      // Merge configs
      const config = mergeConfig(fileConfig, cliConfig);

      // Run watch
      await watchCommand({
        input,
        output: options.output,
        config,
      });
    } catch (error) {
      console.error('Watch failed:', error);
      process.exit(1);
    }
  });

/**
 * Format command
 */
program
  .command('format')
  .description('Format Euis files')
  .argument('[input]', 'Input file or glob pattern', '**/*.euis')
  .option('-w, --write', 'Write formatted output to file')
  .action(async (input: string, options: any) => {
    try {
      await formatCommand({
        input,
        write: options.write || false,
      });
    } catch (error) {
      console.error('Format failed:', error);
      process.exit(1);
    }
  });

/**
 * Tokens command - W3C Design Tokens support
 */
program
  .command('tokens')
  .description('Compile W3C Design Tokens to platform-specific code')
  .argument('<input>', 'Input W3C Design Tokens JSON file')
  .option('-o, --output <path>', 'Output directory', '.')
  .option('-p, --platform <target>', 'Target platform (css, ios, android, android-kotlin, flutter, typescript, docs)', 'css')
  .action(async (input: string, options: any) => {
    try {
      await tokensCommand({
        input,
        output: options.output,
        platform: options.platform,
      });
    } catch (error) {
      console.error('Tokens compilation failed:', error);
      process.exit(1);
    }
  });

// Parse arguments
program.parse();
