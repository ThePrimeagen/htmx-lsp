// @ts-check
const { LanguageClient } = require("vscode-languageclient/node");
const vscode = require("vscode");
const tmpdir = require("os").tmpdir();

module.exports = {
  /** @param {import("vscode").ExtensionContext} context*/
  activate(context) {
    /** @type {import("vscode").WorkspaceConfiguration} */
    const config = vscode.workspace.getConfiguration('htmx-lsp');
    const intreperterPath = config.get('intreperterPath');
    /** @type {import("vscode-languageclient/node").ServerOptions} */
    const serverOptions = {
      run: {
        command: intreperterPath,
      },
      debug: {
        command: intreperterPath,
        args: ["--file", `${tmpdir}/lsp.log`, "--level", "TRACE"],
      },
    };

    /** @type {import("vscode-languageclient/node").LanguageClientOptions} */
    const clientOptions = {
      documentSelector: [{ scheme: "file", language: "html" }],
    };

    const client = new LanguageClient(
      "htmx-lsp",
      "Htmx Language Server",
      serverOptions,
      clientOptions
    );

    client.start();
  },
};
