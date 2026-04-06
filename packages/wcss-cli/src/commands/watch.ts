import * as chokidar from 'chokidar';
import { CompilerConfig } from '../types';
import { buildCommand, BuildOptions } from './build';
import { displayInfo } from '../error-display';

export interface WatchOptions {
  input: string;
  output?: string;
  config: CompilerConfig;
}

/**
 * Watch command: recompile on file changes
 */
export async function watchCommand(options: WatchOptions): Promise<void> {
  const { input, output, config } = options;

  displayInfo(`Watching for changes in: ${input}`);

  // Create file watcher
  const watcher = chokidar.watch(input, {
    ignored: ['**/node_modules/**', '**/dist/**'],
    persistent: true,
    ignoreInitial: false,
  });

  // Debounce timer
  let debounceTimer: NodeJS.Timeout | null = null;
  const DEBOUNCE_MS = 50;

  // Handle file changes
  const handleChange = (path: string) => {
    // Clear existing timer
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    // Debounce recompilation
    debounceTimer = setTimeout(async () => {
      try {
        // Clear console
        console.clear();

        // Show status
        displayInfo(`File changed: ${path}`);
        displayInfo('Recompiling...\n');

        // Recompile
        const buildOptions: BuildOptions = {
          input,
          output,
          config,
        };

        await buildCommand(buildOptions);
      } catch (error) {
        // Display errors but don't exit
        console.error('Compilation error:', error);
      }
    }, DEBOUNCE_MS);
  };

  // Watch for changes
  watcher
    .on('add', handleChange)
    .on('change', handleChange)
    .on('unlink', (path) => {
      displayInfo(`File removed: ${path}`);
    })
    .on('error', (error) => {
      console.error('Watcher error:', error);
    });

  // Keep process alive
  process.on('SIGINT', () => {
    displayInfo('\nStopping watch mode...');
    watcher.close();
    process.exit(0);
  });
}
