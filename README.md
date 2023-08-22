# HTMX LSP
Right now this is very much so a work in progress.

# GREAT NEWS!
We have reached a point where i could use help!  If you want to fill in documentation or help with autocompletes please make an issue / make a pr!

# BIG TODOS LEFT
* all the autocomplete based on the hx-* attributes
* distribute binary
  
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

