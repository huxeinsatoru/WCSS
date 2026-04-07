/**
 * Mock implementation of @wcss/wasm for testing
 */

export class WCSSCompiler {
  compile(source: string, configJson: string) {
    // Parse config to check options
    const config = JSON.parse(configJson);
    
    // Simple mock compilation - just wrap the source in a comment
    let css = `/* Compiled WCSS */\n${source}`;
    
    // Apply minification if enabled (just remove whitespace)
    if (config.minify) {
      css = css.replace(/\s+/g, ' ').trim();
    }
    
    // Generate mock source map
    const sourceMap = config.source_maps !== 'Disabled' ? JSON.stringify({
      version: 3,
      sources: ['input.wcss'],
      names: [],
      mappings: 'AAAA',
      sourcesContent: [source],
    }) : undefined;
    
    return {
      css,
      js: config.typed_om ? 'export const typedOM = {};' : undefined,
      source_map: sourceMap,
      errors: [],
      warnings: [],
      stats: {
        input_size: source.length,
        output_size: css.length,
        compile_time_us: 100,
        rules_processed: 1,
        rules_eliminated: 0,
      },
    };
  }
  
  parse(source: string) {
    // Mock parse - return simple AST
    return {
      rules: [],
      tokens: {},
    };
  }
  
  format(source: string) {
    // Mock format - just return the source
    return source;
  }
  
  validate(source: string, configJson: string) {
    // Mock validate - return no errors
    return [];
  }
}

export default WCSSCompiler;
