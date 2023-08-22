# VSC\*de HTMX LSP

## Usage

Future `todo!()`

## Development

### Setup your environment

```console
# Build & link htmx-lsp
cargo build
ln -s "$(pwd)/target/debug/htmx-lsp" "/usr/local/bin/htmx-lsp" # Or another location in your $PATH

# Setup JS
cd client/vscode
npm install
```

### Debugging

In VSC\*ode, go to the `Run & Debug` sidebar (Ctrl + Shft + D) and click the `Debug LSP Extension` button. This will open a new VSC\*de instance with the lsp client installed.

To get the lsp server logs, run:

```console
tail -f $(echo "console.log(require('os').tmpdir())" | node)/lsp.log
```
