# VSC\*de HTMX LSP

## Usage

Future `todo!()`

## Development

### Setup your environment

```console
# Build htmx-lsp
cargo build

# Add htmx-lsp executable to $PATH
ln -s "$(pwd)/target/debug/htmx-lsp" "/usr/local/bin/htmx-lsp"

# or add it to vscode settings (after starting vscode extension)
"htmx-lsp.intreperterPath": "<path>/htmx-lsp",

# Setup JS
cd client/vscode
npm install
```

### Debugging

In VSC\*ode, go to the `Run & Debug` sidebar (Ctrl + Shft + D) and click the `Debug LSP Extension` button. This will open a new VSC\*de instance.

To get the lsp server logs, run:

```console
tail -f $(echo "console.log(require('os').tmpdir())" | node)/lsp.log
```
