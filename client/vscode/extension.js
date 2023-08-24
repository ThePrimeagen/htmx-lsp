// @ts-check
const { LanguageClient } = require("vscode-languageclient/node");
const tmpdir = require("os").tmpdir();

module.exports = {
  /** @param {import("vscode").ExtensionContext} context*/
  activate(context) {
    /** @type {import("vscode-languageclient/node").ServerOptions} */
    const serverOptions = {
      run: {
        command: "htmx-lsp",
      },
      debug: {
        command: "htmx-lsp",
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
