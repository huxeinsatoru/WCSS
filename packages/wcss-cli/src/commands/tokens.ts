import * as fs from 'fs';
import * as path from 'path';
import { CompilerConfig } from '../types';
import { loadCompiler } from '../compiler';
import { displayErrors, displaySuccess, displayInfo } from '../error-display';

export interface TokensOptions {
  input: string;
  output?: string;
  platform: string;
  config?: CompilerConfig;
}

export type PlatformTarget = 
  | 'css' 
  | 'ios' 
  | 'android' 
  | 'android-kotlin' 
  | 'flutter' 
  | 'typescript' 
  | 'docs';

/**
 * Tokens command: compile W3C Design Tokens to platform-specific code
 */
export async function tokensCommand(options: TokensOptions): Promise<void> {
  const { input, output, platform } = options;

  try {
    // Validate input file exists
    if (!fs.existsSync(input)) {
      console.error(`Error: Token file not found: ${input}`);
      process.exit(1);
    }

    // Read token file
    const tokenContent = fs.readFileSync(input, 'utf-8');

    // Validate JSON
    let jsonContent: any;
    try {
      jsonContent = JSON.parse(tokenContent);
    } catch (e) {
      console.error(`Error: Invalid JSON in token file: ${input}`);
      process.exit(1);
    }

    // Validate it's a W3C Design Tokens file
    if (!isW3CTokenFile(jsonContent)) {
      console.warn(`Warning: File doesn't appear to be a valid W3C Design Tokens file.`);
      console.warn(`Expected objects with $value and optionally $type fields.`);
    }

    displayInfo(`Compiling W3C Design Tokens to ${platform}...`);

    // Load compiler and compile tokens
    const compiler = await loadCompiler();
    
    // Map platform string to enum
    const target = mapPlatformToTarget(platform);
    
    // Call the WASM compile_w3c_tokens function
    const result = compiler.compile_w3c_tokens(tokenContent, target);

    if (result.errors && result.errors.length > 0) {
      console.error(`\nErrors in ${input}:`);
      displayErrors(result.errors, tokenContent);
      process.exit(1);
    }

    // Determine output directory
    const outputDir = output || '.';
    fs.mkdirSync(outputDir, { recursive: true });

    // Write output files
    for (const [filename, content] of Object.entries(result.output)) {
      const outputPath = path.join(outputDir, filename);
      fs.writeFileSync(outputPath, content, 'utf-8');
      displayInfo(`${input} → ${outputPath}`);
    }

    displaySuccess(`W3C Design Tokens compiled successfully to ${platform}`);
  } catch (error) {
    console.error('Tokens compilation failed:', error);
    process.exit(1);
  }
}

/**
 * Check if the JSON content appears to be a W3C Design Tokens file
 */
function isW3CTokenFile(content: any): boolean {
  if (typeof content !== 'object' || content === null) {
    return false;
  }

  // Recursively check for $value fields
  function hasTokenValue(obj: any): boolean {
    if (typeof obj !== 'object' || obj === null) {
      return false;
    }

    if ('$value' in obj) {
      return true;
    }

    for (const key in obj) {
      if (hasTokenValue(obj[key])) {
        return true;
      }
    }

    return false;
  }

  return hasTokenValue(content);
}

/**
 * Map platform string to PlatformTarget enum
 */
function mapPlatformToTarget(platform: string): any {
  const platformMap: Record<string, string> = {
    'css': 'CSS',
    'ios': 'IOS',
    'android': 'Android',
    'android-kotlin': 'AndroidKotlin',
    'flutter': 'Flutter',
    'typescript': 'TypeScript',
    'docs': 'Docs',
  };

  const target = platformMap[platform.toLowerCase()];
  if (!target) {
    throw new Error(`Unknown platform: ${platform}. Supported platforms: ${Object.keys(platformMap).join(', ')}`);
  }

  return target;
}
