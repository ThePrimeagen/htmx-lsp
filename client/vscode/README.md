# VSC\*de HTMX LSP

## Vscode Extension Bundle

The vscode extension created through ci will have all the lsp executables bundled with it. ex: `bin/htmx-lsp`
- TODO: Use the bundled version of the lsp with the extension by default.

## Usage

Future `todo!()`

## Development

### Setup your environment

The extension will default to `htmx-lsp` as the executable name. But you can change it in vscode settings with the key `htmx-lsp.intreperterPath`. For example, when developing the lsp you can set it to: `<workspace>/target/debug/htmx-lsp`.

```console
# Build htmx-lsp
cargo build

# Add htmx-lsp executable to $PATH
ln -s "$(pwd)/target/debug/htmx-lsp" "/usr/local/bin/htmx-lsp"

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
