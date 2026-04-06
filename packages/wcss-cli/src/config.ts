import * as fs from 'fs';
import * as path from 'path';
import { CompilerConfig } from './types';

/**
 * Load WCSS configuration from file system
 * Checks for wcss.config.js and wcss.config.json in order
 */
export async function loadConfig(configPath?: string): Promise<CompilerConfig> {
  // If explicit config path provided, use it
  if (configPath) {
    return loadConfigFromPath(configPath);
  }

  // Check for wcss.config.js
  const jsConfigPath = path.join(process.cwd(), 'wcss.config.js');
  if (fs.existsSync(jsConfigPath)) {
    return loadConfigFromPath(jsConfigPath);
  }

  // Check for wcss.config.json
  const jsonConfigPath = path.join(process.cwd(), 'wcss.config.json');
  if (fs.existsSync(jsonConfigPath)) {
    return loadConfigFromPath(jsonConfigPath);
  }

  // Return defaults
  return getDefaultConfig();
}

/**
 * Load configuration from a specific path
 */
async function loadConfigFromPath(configPath: string): Promise<CompilerConfig> {
  const ext = path.extname(configPath);

  if (ext === '.js') {
    // Load JavaScript config
    const absolutePath = path.resolve(configPath);
    const config = require(absolutePath);
    return config.default || config;
  } else if (ext === '.json') {
    // Load JSON config
    const content = fs.readFileSync(configPath, 'utf-8');
    return JSON.parse(content);
  } else {
    throw new Error(`Unsupported config file format: ${ext}`);
  }
}

/**
 * Get default configuration
 */
export function getDefaultConfig(): CompilerConfig {
  return {
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
}

/**
 * Merge CLI options with config file
 */
export function mergeConfig(
  fileConfig: CompilerConfig,
  cliOptions: Partial<CompilerConfig>
): CompilerConfig {
  return {
    ...fileConfig,
    ...cliOptions,
    tokens: {
      ...fileConfig.tokens,
      ...cliOptions.tokens,
    },
  };
}
