<div align="center">
  <a href="https://github.com/ThePrimeagen/htmx-lsp#gh-light-mode-only"><img src="assets/logo.svg#gh-light-mode-only"        width="300px" alt="HTMX-LSP logo"/></a>
  <a href="https://github.com/ThePrimeagen/htmx-lsp#gh-dark-mode-only"><img src="assets/logo.darkmode.svg#gh-dark-mode-only" width="300px" alt="HTMX-LSP logo"/></a>
  <br>
  <a href="https://crates.io/crates/htmx-lsp"><img alt="crates.io" src="https://img.shields.io/crates/v/htmx-lsp.svg?style=for-the-badge&color=bc3f48&logo=rust" height="20"></a>
  <a href="https://github.com/ThePrimeagen/htmx-lsp/actions?query=branch%3Amaster"><img alt="build status" src="https://img.shields.io/github/actions/workflow/status/ThePrimeagen/htmx-lsp/ci.yml?branch=master&style=for-the-badge&logo=github" height="20"></a>
</div>

<h4 align="center">
     its so over
</h4>

## LSP

Right now this is very much so a work in progress and currently provides basic autocomplete for _most_ HTMX attributes. We have reached a point where I could use help! If you want to fill in documentation or help with autocompletes please open an issue/pr!

## Integration

### Neovim

`htmx-lsp` can be installed via Mason. And can be configured with `lspconfig`

```lua
local lspconfig = require('lspconfig')
-- ...
lspconfig.htmx.setup{}
```

Another option is to use [lsp-debug-tools](https://github.com/ThePrimeagen/lsp-debug-tools.nvim)

### VSCode

No published extension yet, but there is a development extension in the [`clients/vscode`](client/vscode/README.md) folder (with setup instructions)

## Development

### General

As of right now the general goal is just to provide completion for any `-`
character received without even looking at the context.

After that, would be to perform some code actions that make sense and allow for
amazing utility around htmx.

```console
htmx-lsp -f /path/to/file --level [OFF | TRACE | DEBUG | INFO | WARN | ERROR]
```

### Build

```console
cargo build

# OR auto-build on file save, requires `cargo-watch`
cargo install cargo-watch
cargo watch -x build
```

## Contributors

<div align="center">
  <a href="https://github.com/ThePrimeagen/htmx-lsp/graphs/contributors">
    <img src="https://contrib.rocks/image?repo=ThePrimeagen/htmx-lsp" height="50px"/>
  </a>
</div>
