const { LanguageClient } = require("vscode-languageclient/node");
const tmpdir = require("os").tmpdir();

module.exports = {
  activate(context) {
    const serverOptions = {
      run: {
        command: "htmx-lsp",
      },
      debug: {
        command: "htmx-lsp",
        args: ["--file", `${tmpdir}/lsp.log`, "--level", "TRACE"],
      },
    };

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
