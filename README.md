# HTMX LSP
Right now this is very much so a work in progress.

# WARNING
I will reject all PRs for the time being as i just start get the repo in shape.
But i would totally not reject any example PRs.  Meaning:

- you pr a change on how you would go about doing code actions
- (anything else you wish to show me)

# Integration
There is no vim or vscode specific integration, but!!! i do have an example of
launching servers and listening to them for neovim in
[lsp-debug-tools](https://github.com/ThePrimeagen/lsp-debug-tools.nvim) repo

# Help
Creating the required clients to be used in vscode would be super cool.  If i
could avoid touching vscode that would be awesome

## Build

Simple build
```bash
cargo build
```

Watching
```bash
cargo install cargo-watch
cargo watch -x build
```

## Development

### General
As of right now the general goal is just to provide completion for any `-`
character received without even looking at the context.

After that, would be to perform some code actions that make sense and allow for
amazing utility around htmx.

```
htmx-lsp -f /path/to/file --level OFF | TRACE | DEBUG | INFO | WARN | ERROR
```

### NeoVim
As of now,
[lsp-debug-tools](https://github.com/ThePrimeagen/lsp-debug-tools.nvim) is the
only debugging tool designed for in editor experience.

