import * as vscode from 'vscode';
import * as path from 'path';

/**
 * VS Code extension for WCSS
 *
 * Features:
 * - Syntax highlighting (via TextMate grammar)
 * - Format document command
 * - Format on save
 *
 * TODO:
 * - Language server for autocomplete and error reporting
 * - Go-to-definition for token references
 * - Real-time diagnostics
 */

let outputChannel: vscode.OutputChannel;

/** The WASM compiler singleton */
interface WasmCompiler {
  format: (source: string) => string;
}

let compiler: WasmCompiler | null = null;

/**
 * Load the WASM compiler (Node.js build) as a singleton.
 * Returns the compiler instance or null if loading fails.
 */
function getCompiler(): WasmCompiler | null {
  if (compiler) {
    return compiler;
  }

  const wasmPath = path.resolve(__dirname, '../../../pkg/nodejs/wcss_wasm.js');

  try {
    const wasmModule = require(wasmPath);
    compiler = wasmModule;
    outputChannel.appendLine(`WASM compiler loaded from ${wasmPath}`);
    return compiler;
  } catch (err: any) {
    outputChannel.appendLine(`Failed to load WASM compiler from ${wasmPath}: ${err.message}`);
    outputChannel.show(true);
    return null;
  }
}

/**
 * Format the current document using the WASM compiler
 */
async function formatDocument(document: vscode.TextDocument): Promise<vscode.TextEdit[]> {
  const comp = getCompiler();
  if (!comp) {
    vscode.window.showErrorMessage(
      'WCSS: WASM compiler not available. Check the WCSS output channel for details.'
    );
    return [];
  }

  const fullRange = new vscode.Range(
    document.positionAt(0),
    document.positionAt(document.getText().length)
  );

  try {
    const formatted = comp.format(document.getText());
    return [vscode.TextEdit.replace(fullRange, formatted)];
  } catch (err: any) {
    outputChannel.appendLine(`Format error: ${err.message}`);
    vscode.window.showErrorMessage(`WCSS format failed: ${err.message}`);
    return [];
  }
}

/**
 * Activate the extension
 */
export function activate(context: vscode.ExtensionContext) {
  outputChannel = vscode.window.createOutputChannel('WCSS');
  outputChannel.appendLine('WCSS extension activated');

  // Eagerly attempt to load the compiler so errors surface early
  getCompiler();

  // Register format document command
  const formatCommand = vscode.commands.registerCommand(
    'wcss.format',
    async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
      }

      if (editor.document.languageId !== 'wcss') {
        vscode.window.showErrorMessage('Not a WCSS file');
        return;
      }

      try {
        const edits = await formatDocument(editor.document);
        if (edits.length > 0) {
          const edit = new vscode.WorkspaceEdit();
          edit.set(editor.document.uri, edits);
          await vscode.workspace.applyEdit(edit);
          vscode.window.showInformationMessage('WCSS file formatted');
        }
      } catch (error) {
        vscode.window.showErrorMessage(`Format failed: ${error}`);
      }
    }
  );

  // Register document formatting provider
  const formatProvider = vscode.languages.registerDocumentFormattingEditProvider(
    'wcss',
    {
      provideDocumentFormattingEdits(document: vscode.TextDocument): Thenable<vscode.TextEdit[]> {
        return formatDocument(document);
      }
    }
  );

  // Handle format on save
  const saveHandler = vscode.workspace.onWillSaveTextDocument(async (event) => {
    if (event.document.languageId !== 'wcss') {
      return;
    }

    const config = vscode.workspace.getConfiguration('wcss');
    if (!config.get('formatOnSave', true)) {
      return;
    }

    const edits = formatDocument(event.document);
    event.waitUntil(edits);
  });

  context.subscriptions.push(
    formatCommand,
    formatProvider,
    saveHandler,
    outputChannel
  );

  outputChannel.appendLine('WCSS extension ready');
}

/**
 * Deactivate the extension
 */
export function deactivate() {
  outputChannel?.appendLine('WCSS extension deactivated');
  compiler = null;
}
