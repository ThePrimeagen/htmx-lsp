const { LanguageClient } = require("vscode-languageclient");
const tmpdir = require("os").tmpdir();

module.exports = {
  activate(context) {
    const cmd = "htmx-lsp";

    const serverOptions = {
      run: {
        command: cmd,
      },
      debug: {
        command: cmd,
        args: ["--file", `${tmpdir}/lsp.log`, "--level", "TRACE"],
      },
    };

    const clientOptions = {
      documentSelector: [
        {
          language: "html",
        },
      ],
    };

    const client = new LanguageClient(
      "htmx-lsp",
      "Htmx Language Server",
      serverOptions,
      clientOptions
    );

    context.subscriptions.push(client.start());
  },
};
